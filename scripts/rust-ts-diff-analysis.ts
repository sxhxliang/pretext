import { execFileSync, execSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'

import { clearCache, layout, prepare, setLocale } from '../src/layout.js'

const emojiPresentationRe = /\p{Emoji_Presentation}/u
const punctuationRe = /[.,!?;:%)\]}'"”’»›…—-]/u

function parseFontSize(font: string): number {
  const match = font.match(/(\d+(?:\.\d+)?)\s*px/)
  return match ? Number.parseFloat(match[1]!) : 16
}

function isWideCharacter(ch: string): boolean {
  const code = ch.codePointAt(0)!
  return (
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0x3400 && code <= 0x4dbf) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0x2f800 && code <= 0x2fa1f) ||
    (code >= 0x20000 && code <= 0x2a6df) ||
    (code >= 0x2a700 && code <= 0x2b73f) ||
    (code >= 0x2b740 && code <= 0x2b81f) ||
    (code >= 0x2b820 && code <= 0x2ceaf) ||
    (code >= 0x2ceb0 && code <= 0x2ebef) ||
    (code >= 0x30000 && code <= 0x3134f) ||
    (code >= 0x3000 && code <= 0x303f) ||
    (code >= 0x3040 && code <= 0x309f) ||
    (code >= 0x30a0 && code <= 0x30ff) ||
    (code >= 0xac00 && code <= 0xd7af) ||
    (code >= 0xff00 && code <= 0xffef)
  )
}

function measureWidth(text: string, font: string): number {
  const fontSize = parseFontSize(font)
  let width = 0
  for (const ch of text) {
    if (ch === ' ') {
      width += fontSize * 0.33
    } else if (ch === '\t') {
      width += fontSize * 1.32
    } else if (emojiPresentationRe.test(ch) || ch === '\uFE0F') {
      width += fontSize
    } else if (isWideCharacter(ch)) {
      width += fontSize
    } else if (punctuationRe.test(ch)) {
      width += fontSize * 0.4
    } else {
      width += fontSize * 0.6
    }
  }
  return width
}

class TestCanvasRenderingContext2D {
  font = ''
  measureText(text: string): { width: number } {
    return { width: measureWidth(text, this.font) }
  }
}

class TestOffscreenCanvas {
  constructor(_width: number, _height: number) {}
  getContext(_kind: string): TestCanvasRenderingContext2D {
    return new TestCanvasRenderingContext2D()
  }
}

Reflect.set(globalThis, 'OffscreenCanvas', TestOffscreenCanvas)

const widths = [80, 100, 120, 140, 160, 180, 200, 220, 240, 260, 300, 320]
const font = '16px Inter'
const lineHeight = 19

const cases = [
  { name: 'mixed-app', mode: 'normal', text: 'Hello 世界 👋🏽 https://example.com/path?q=alpha&lang=zh 中文段落 mixed متن عربي 12345' },
  { name: 'arabic-punct', mode: 'normal', text: 'قال،وقال؟ثم—تابع' },
  { name: 'arabic-mark', mode: 'normal', text: 'قال ّبكم عن الفقرة' },
  { name: 'myanmar', mode: 'normal', text: 'မြန်မာစာ၊အမှတ်အသားနှင့်စမ်းသပ်မှု' },
  { name: 'cjk-kinsoku', mode: 'normal', text: '漢字漢字、漢字漢字。日本語テキスト（テスト）' },
  { name: 'emoji-zwj', mode: 'normal', text: 'Family 👨‍👩‍👧‍👦 and flags 🇺🇸🇨🇳 with text wrap test' },
  { name: 'soft-hyphen', mode: 'normal', text: 'foo trans\u00ADatlantic integration check' },
  { name: 'pre-wrap-spaces', mode: 'pre-wrap', text: '  Hello   world  \nnext  line  ' },
  { name: 'pre-wrap-tabs', mode: 'pre-wrap', text: 'A\tB\tC\n\tD' },
] as const

setLocale(undefined)
clearCache()

execSync('cargo build -p rust-layout --release --bin compare', { stdio: 'inherit' })

const rows: Array<{
  case: string
  mode: 'normal' | 'pre-wrap'
  width: number
  tsLineCount: number
  rustLineCount: number
  delta: number
}> = []

for (const row of cases) {
  for (const width of widths) {
    const options = row.mode === 'pre-wrap' ? { whiteSpace: 'pre-wrap' as const } : undefined
    const tsPrepared = prepare(row.text, font, options)
    const tsOut = layout(tsPrepared, width, lineHeight)

    const rustRaw = execFileSync('target/release/compare', [`--width=${width}`, `--mode=${row.mode}`], {
      input: row.text,
      encoding: 'utf8',
    })
    const rustOut = JSON.parse(rustRaw) as { line_count: number }

    rows.push({
      case: row.name,
      mode: row.mode,
      width,
      tsLineCount: tsOut.lineCount,
      rustLineCount: rustOut.line_count,
      delta: rustOut.line_count - tsOut.lineCount,
    })
  }
}

const mismatchRows = rows.filter(row => row.delta !== 0)
const report = {
  generatedAt: new Date().toISOString(),
  totalRows: rows.length,
  mismatches: mismatchRows.length,
  mismatchRate: mismatchRows.length / rows.length,
  rows,
}

writeFileSync('benchmarks/rust-ts-diff-analysis.json', `${JSON.stringify(report, null, 2)}\n`, 'utf8')

const byCase = new Map<string, { total: number; mismatch: number; maxDelta: number }>()
for (const row of rows) {
  const key = `${row.case} (${row.mode})`
  const entry = byCase.get(key) ?? { total: 0, mismatch: 0, maxDelta: 0 }
  entry.total += 1
  if (row.delta !== 0) entry.mismatch += 1
  entry.maxDelta = Math.max(entry.maxDelta, Math.abs(row.delta))
  byCase.set(key, entry)
}

const caseLines = [...byCase.entries()]
  .map(([name, stat]) => `- ${name}: mismatch ${stat.mismatch}/${stat.total}, max |delta|=${stat.maxDelta}`)
  .join('\n')

const md = `# Rust vs TS diff analysis\n\n- rows: ${rows.length}\n- mismatches: ${mismatchRows.length}\n- mismatch rate: ${(report.mismatchRate * 100).toFixed(2)}%\n\n## By case\n${caseLines}\n\n## Top mismatches\n${mismatchRows
  .slice(0, 10)
  .map(row => `- ${row.case} @ ${row.width}px: delta=${row.delta}`)
  .join('\n') || '- none'}\n`

writeFileSync('benchmarks/rust-ts-diff-analysis.md', md, 'utf8')
console.log(md)
