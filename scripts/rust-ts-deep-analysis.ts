import { execFileSync, execSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'

import { clearCache, layoutWithLines, prepareWithSegments, setLocale } from '../src/layout.js'

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

type WhiteSpaceMode = 'normal' | 'pre-wrap'

type CompareCursor = { segmentIndex: number; graphemeIndex: number }

type CompareLine = { text: string; width: number; start: CompareCursor; end: CompareCursor }

type CompareOut = { line_count: number; lines: CompareLine[] }

const widths = [80, 100, 120, 140, 160, 180, 200, 220, 240, 260, 300, 320]
const font = '16px Inter'
const lineHeight = 19

const cases: Array<{ name: string; mode: WhiteSpaceMode; text: string }> = [
  { name: 'mixed-app', mode: 'normal', text: 'Hello 世界 👋🏽 https://example.com/path?q=alpha&lang=zh 中文段落 mixed متن عربي 12345' },
  { name: 'arabic-punct', mode: 'normal', text: 'قال،وقال؟ثم—تابع' },
  { name: 'arabic-mark', mode: 'normal', text: 'قال ّبكم عن الفقرة' },
  { name: 'myanmar', mode: 'normal', text: 'မြန်မာစာ၊အမှတ်အသားနှင့်စမ်းသပ်မှု' },
  { name: 'cjk-kinsoku', mode: 'normal', text: '漢字漢字、漢字漢字。日本語テキスト（テスト）' },
  { name: 'emoji-zwj', mode: 'normal', text: 'Family 👨‍👩‍👧‍👦 and flags 🇺🇸🇨🇳 with text wrap test' },
  { name: 'soft-hyphen', mode: 'normal', text: 'foo trans\u00ADatlantic integration check' },
  { name: 'pre-wrap-spaces', mode: 'pre-wrap', text: '  Hello   world  \nnext  line  ' },
  { name: 'pre-wrap-tabs', mode: 'pre-wrap', text: 'A\tB\tC\n\tD' },
]

setLocale(undefined)
clearCache()
execSync('cargo build -p rust-layout --release --bin compare', { stdio: 'inherit' })

const rows: Array<{
  case: string
  mode: WhiteSpaceMode
  width: number
  lineCountDelta: number | null
  lineTextMismatchCount: number
  cursorMismatchCount: number
  firstLineTextDiff?: string
  firstCursorDiff?: string
  tsError?: string
  rustError?: string
}> = []

for (const row of cases) {
  const options = row.mode === 'pre-wrap' ? { whiteSpace: 'pre-wrap' as const } : undefined
  const tsPrepared = prepareWithSegments(row.text, font, options)
  for (const width of widths) {
    let tsOut: ReturnType<typeof layoutWithLines> | null = null
    let rustOut: CompareOut | null = null
    let tsError: string | undefined
    let rustError: string | undefined

    try {
      tsOut = layoutWithLines(tsPrepared, width, lineHeight)
    } catch (error) {
      tsError = String(error)
    }
    try {
      rustOut = JSON.parse(
        execFileSync('target/release/compare', [`--width=${width}`, `--mode=${row.mode}`], {
          input: row.text,
          encoding: 'utf8',
        }),
      ) as CompareOut
    } catch (error) {
      rustError = String(error)
    }

    if (!tsOut || !rustOut) {
      rows.push({
        case: row.name,
        mode: row.mode,
        width,
        lineCountDelta: null,
        lineTextMismatchCount: 0,
        cursorMismatchCount: 0,
        tsError,
        rustError,
      })
      continue
    }

    const maxLines = Math.max(tsOut.lines.length, rustOut.lines.length)
    let lineTextMismatchCount = 0
    let cursorMismatchCount = 0
    let firstLineTextDiff: string | undefined
    let firstCursorDiff: string | undefined

    for (let i = 0; i < maxLines; i += 1) {
      const tsLine = tsOut.lines[i]
      const rustLine = rustOut.lines[i]
      if (!tsLine || !rustLine) {
        lineTextMismatchCount += 1
        cursorMismatchCount += 1
        continue
      }
      if (tsLine.text !== rustLine.text) {
        lineTextMismatchCount += 1
        if (!firstLineTextDiff) {
          firstLineTextDiff = `i=${i}, ts=${JSON.stringify(tsLine.text)}, rust=${JSON.stringify(rustLine.text)}`
        }
      }
      if (
        tsLine.start.segmentIndex !== rustLine.start.segmentIndex ||
        tsLine.start.graphemeIndex !== rustLine.start.graphemeIndex ||
        tsLine.end.segmentIndex !== rustLine.end.segmentIndex ||
        tsLine.end.graphemeIndex !== rustLine.end.graphemeIndex
      ) {
        cursorMismatchCount += 1
        if (!firstCursorDiff) {
          firstCursorDiff =
            `i=${i}, ts=(${tsLine.start.segmentIndex},${tsLine.start.graphemeIndex})->(${tsLine.end.segmentIndex},${tsLine.end.graphemeIndex}),` +
            ` rust=(${rustLine.start.segmentIndex},${rustLine.start.graphemeIndex})->(${rustLine.end.segmentIndex},${rustLine.end.graphemeIndex})`
        }
      }
    }

    rows.push({
      case: row.name,
      mode: row.mode,
      width,
      lineCountDelta: rustOut.line_count - tsOut.lineCount,
      lineTextMismatchCount,
      cursorMismatchCount,
      firstLineTextDiff,
      firstCursorDiff,
      tsError,
      rustError,
    })
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  totalRows: rows.length,
  lineCountMismatchRows: rows.filter(row => row.lineCountDelta !== null && row.lineCountDelta !== 0).length,
  lineTextMismatchRows: rows.filter(row => row.lineTextMismatchCount > 0).length,
  cursorMismatchRows: rows.filter(row => row.cursorMismatchCount > 0).length,
  executionErrorRows: rows.filter(row => row.tsError || row.rustError).length,
  patternSummary: {
    urlQueryBoundary: rows.filter(row => row.firstLineTextDiff?.includes('/path?') || row.firstLineTextDiff?.includes('https://')).length,
    cjkEmojiBoundary: rows.filter(row => row.firstLineTextDiff?.includes('世界') || row.firstLineTextDiff?.includes('👋🏽')).length,
    cursorStartSegmentDrift: rows.filter(row => row.firstCursorDiff?.includes('ts=(')).length,
  },
  rows,
}

writeFileSync('benchmarks/rust-ts-deep-analysis.json', `${JSON.stringify(report, null, 2)}\n`, 'utf8')

const byCase = new Map<string, { total: number; lineCountMismatch: number; lineTextMismatch: number; cursorMismatch: number }>()
for (const row of rows) {
  const key = `${row.case} (${row.mode})`
  const entry = byCase.get(key) ?? { total: 0, lineCountMismatch: 0, lineTextMismatch: 0, cursorMismatch: 0, executionError: 0 }
  entry.total += 1
  if (row.lineCountDelta !== null && row.lineCountDelta !== 0) entry.lineCountMismatch += 1
  if (row.lineTextMismatchCount > 0) entry.lineTextMismatch += 1
  if (row.cursorMismatchCount > 0) entry.cursorMismatch += 1
  if (row.tsError || row.rustError) entry.executionError += 1
  byCase.set(key, entry)
}

const caseLines = [...byCase.entries()]
  .map(
    ([name, stat]) =>
      `- ${name}: lineCountMismatch ${stat.lineCountMismatch}/${stat.total}, lineTextMismatch ${stat.lineTextMismatch}/${stat.total}, cursorMismatch ${stat.cursorMismatch}/${stat.total}, executionError ${stat.executionError}/${stat.total}`,
  )
  .join('\n')

const topRows = rows
  .filter(
    row =>
      row.lineCountDelta === null ||
      row.lineCountDelta !== 0 ||
      row.lineTextMismatchCount > 0 ||
      row.cursorMismatchCount > 0 ||
      row.tsError !== undefined ||
      row.rustError !== undefined,
  )
  .slice(0, 12)

const md = `# Rust vs TS deep analysis\n\n- rows: ${report.totalRows}\n- lineCount mismatch rows: ${report.lineCountMismatchRows}\n- line.text mismatch rows: ${report.lineTextMismatchRows}\n- cursor(start/end) mismatch rows: ${report.cursorMismatchRows}\n- execution error rows: ${report.executionErrorRows}\n\n## Dominant patterns\n- URL/query boundary related rows: ${report.patternSummary.urlQueryBoundary}\n- CJK/emoji boundary related rows: ${report.patternSummary.cjkEmojiBoundary}\n- cursor segment-start drift rows: ${report.patternSummary.cursorStartSegmentDrift}\n\n## By case\n${caseLines}\n\n## Top mismatching rows\n${topRows
  .map(
    row =>
      `- ${row.case} @ ${row.width}px: lineCountDelta=${row.lineCountDelta}, lineTextMismatchCount=${row.lineTextMismatchCount}, cursorMismatchCount=${row.cursorMismatchCount}${
        row.tsError ? `, tsError=${row.tsError}` : ''
      }${row.rustError ? `, rustError=${row.rustError}` : ''}${
        row.firstLineTextDiff ? `, firstLineTextDiff=${row.firstLineTextDiff}` : ''
      }${row.firstCursorDiff ? `, firstCursorDiff=${row.firstCursorDiff}` : ''}`,
  )
  .join('\n') || '- none'}\n`

writeFileSync('benchmarks/rust-ts-deep-analysis.md', md, 'utf8')
console.log(md)
