import { writeFileSync } from 'node:fs'
import { execSync } from 'node:child_process'
import { performance } from 'node:perf_hooks'

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

const iterations = Number.parseInt(process.env.BENCH_ITERATIONS ?? '20000', 10)
const width = Number.parseFloat(process.env.BENCH_WIDTH ?? '320')
const mode = process.env.BENCH_MODE === 'pre-wrap' ? 'pre-wrap' : 'normal'
const options = mode === 'pre-wrap' ? { whiteSpace: 'pre-wrap' as const } : undefined

const seed = 'Hello 世界 👋🏽 https://example.com/path?q=alpha&lang=zh 中文段落 mixed متن عربي 12345 '
const text = seed.repeat(16)
const font = '16px Inter'
const lineHeight = 19

setLocale(undefined)
clearCache()

for (let i = 0; i < 500; i += 1) {
  const prepared = prepare(text, font, options)
  layout(prepared, width, lineHeight)
}

const start = performance.now()
let checksum = 0
for (let i = 0; i < iterations; i += 1) {
  const prepared = prepare(text, font, options)
  checksum += layout(prepared, width, lineHeight).lineCount
}
const elapsedMs = performance.now() - start
const tsNsPerIter = (elapsedMs * 1e6) / iterations

const rustRaw = execSync(
  `cargo run -p rust-layout --release --bin bench -- --iterations=${iterations} --width=${width} --mode=${mode}`,
  { encoding: 'utf8' },
)
const rust = JSON.parse(rustRaw.trim()) as {
  ns_per_iter: number
  checksum: number
  text_len: number
}

const report = {
  generatedAt: new Date().toISOString(),
  mode,
  iterations,
  width,
  textLength: text.length,
  ts: {
    engine: 'typescript',
    nsPerIter: tsNsPerIter,
    checksum,
  },
  rust: {
    engine: 'rust',
    nsPerIter: rust.ns_per_iter,
    checksum: rust.checksum,
  },
  speedup: tsNsPerIter / rust.ns_per_iter,
}

writeFileSync('benchmarks/rust-vs-ts.json', `${JSON.stringify(report, null, 2)}\n`, 'utf8')

console.log(JSON.stringify(report, null, 2))
