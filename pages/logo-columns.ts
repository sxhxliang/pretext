import { layoutNextLine, prepareWithSegments, type LayoutCursor, type PreparedTextWithSegments } from '../src/layout.ts'

const BODY_FONT = '16px "Helvetica Neue", Helvetica, Arial, sans-serif'
const BODY_LINE_HEIGHT = 25
const MOBILE_BODY_FONT = '14.5px "Helvetica Neue", Helvetica, Arial, sans-serif'
const MOBILE_BODY_LINE_HEIGHT = 22

const LEFT_COPY = `
You can often see the future first in San Francisco. The conversation changes before the institutions do. One year people are still speaking cautiously about large training runs; the next year the working assumptions have moved to ten-billion-dollar clusters, then to a hundred billion, and then beyond that again. Each planning cycle adds another zero. What sounded extravagant six months ago becomes the conservative baseline for the next round of internal discussion.

From the outside, this can sound like hype. From the inside, it looks more like convergence. Labs want more compute because compute still buys capability. Governments are waking up because capability is starting to look strategic. Capital keeps arriving because the prize is no longer framed as another software category but as leverage over the rest of the economy. These forces do not line up perfectly, but they do push in the same direction.
`.trim().replace(/\s+/gu, ' ') + ' ' + `
If that trajectory holds, the decisive variable is not a single model release. It is industrialization: power, chips, datacenters, supply chains, teams willing to spend enormous amounts of money because they think the next system over the horizon will repay the cost many times over. The question stops being whether frontier systems will improve and starts becoming how quickly the surrounding world can absorb what the labs are already trying to build.

That is what makes the next decade feel strange. Progress looks both incremental and discontinuous at the same time: one more scaling law paper, one more procurement round, one more cluster plan, and then suddenly a threshold is crossed and the surrounding institutions realize they were preparing for a different world.
`.trim().replace(/\s+/gu, ' ')

const RIGHT_COPY = `
The practical implication is not that every forecasted timeline will arrive on schedule. It is that the frontier is now expensive enough, concentrated enough, and geopolitically entangled enough that we should think about it as infrastructure. Training runs, chip allocations, grid capacity, export controls, and security posture begin to matter in the same sentence. Once that happens, the old habit of treating AI progress as a purely academic curve starts to break down.

The people closest to this transition do not seem especially relaxed by it. Their confidence that systems will keep improving is often matched by uncertainty about who will govern the deployment environment, how much slack exists in the supply chain, and whether political systems are capable of responding at the speed that technical systems and capital markets are currently moving.
`.trim().replace(/\s+/gu, ' ') + ' ' + `
So the point of situational awareness is not prediction as performance. It is to notice the shape of the field while it is still possible to act. If compute, talent, state interest, and model capability are all compounding together, then the relevant question is not whether the curve is real in the abstract. The relevant question is what institutions, norms, and technical practices have to exist before that curve becomes impossible to manage gracefully.

That is also why this text lands differently in a typographic setting than it does in a feed. On a page, the claims feel infrastructural rather than merely rhetorical. They sit there as objects to move around, compare, and return to. Even stripped to raw text, the argument keeps its essential mood: the world ahead may arrive through a long sequence of ordinary decisions, but the resulting change will not feel ordinary once it is here.
`.trim().replace(/\s+/gu, ' ')

type Rect = {
  x: number
  y: number
  width: number
  height: number
}

type Interval = {
  left: number
  right: number
}

type MaskRow = {
  left: number
  right: number
}

type ImageMask = {
  width: number
  height: number
  rows: Array<MaskRow | null>
}

const stage = document.getElementById('stage') as HTMLDivElement
const headline = document.getElementById('headline') as HTMLHeadingElement
const credit = document.getElementById('credit') as HTMLParagraphElement
const openaiLogo = document.getElementById('openai-logo') as HTMLImageElement
const claudeLogo = document.getElementById('claude-logo') as HTMLImageElement

const preparedByKey = new Map<string, PreparedTextWithSegments>()
const scheduled = { value: false }

function getTypography(): { font: string, lineHeight: number } {
  if (window.innerWidth <= 900) {
    return { font: MOBILE_BODY_FONT, lineHeight: MOBILE_BODY_LINE_HEIGHT }
  }
  return { font: BODY_FONT, lineHeight: BODY_LINE_HEIGHT }
}

function getPrepared(text: string, font: string): PreparedTextWithSegments {
  const key = `${font}::${text}`
  const cached = preparedByKey.get(key)
  if (cached !== undefined) return cached
  const prepared = prepareWithSegments(text, font)
  preparedByKey.set(key, prepared)
  return prepared
}

async function makeImageMask(src: string, width: number, height: number): Promise<ImageMask> {
  const image = new Image()
  image.src = src
  await image.decode()

  const canvas = new OffscreenCanvas(width, height)
  const ctx = canvas.getContext('2d')
  if (ctx === null) throw new Error('2d context unavailable')

  ctx.clearRect(0, 0, width, height)
  ctx.drawImage(image, 0, 0, width, height)

  const { data } = ctx.getImageData(0, 0, width, height)
  const rows: Array<MaskRow | null> = new Array(height)

  for (let y = 0; y < height; y++) {
    let left = width
    let right = -1
    for (let x = 0; x < width; x++) {
      const alpha = data[(y * width + x) * 4 + 3]!
      if (alpha < 12) continue
      if (x < left) left = x
      if (x > right) right = x
    }
    rows[y] = right >= left ? { left, right: right + 1 } : null
  }

  return { width, height, rows }
}

function getMaskIntervalForBand(
  mask: ImageMask,
  rect: Rect,
  bandTop: number,
  bandBottom: number,
  horizontalPadding: number,
  verticalPadding: number,
): Interval | null {
  if (bandBottom <= rect.y || bandTop >= rect.y + rect.height) return null

  const startRow = Math.max(0, Math.floor(bandTop - rect.y - verticalPadding))
  const endRow = Math.min(mask.height - 1, Math.ceil(bandBottom - rect.y + verticalPadding))

  let left = mask.width
  let right = -1

  for (let rowIndex = startRow; rowIndex <= endRow; rowIndex++) {
    const row = mask.rows[rowIndex]
    if (row === null || row === undefined) continue
    if (row.left < left) left = row.left
    if (row.right > right) right = row.right
  }

  if (right < left) return null

  return {
    left: rect.x + left - horizontalPadding,
    right: rect.x + right + horizontalPadding,
  }
}

function subtractIntervals(base: Interval, intervals: Interval[]): Interval[] {
  let slots: Interval[] = [base]

  for (const interval of intervals) {
    const next: Interval[] = []
    for (const slot of slots) {
      if (interval.right <= slot.left || interval.left >= slot.right) {
        next.push(slot)
        continue
      }
      if (interval.left > slot.left) {
        next.push({ left: slot.left, right: interval.left })
      }
      if (interval.right < slot.right) {
        next.push({ left: interval.right, right: slot.right })
      }
    }
    slots = next
  }

  return slots.filter(slot => slot.right - slot.left >= 24)
}

function renderColumn(
  prepared: PreparedTextWithSegments,
  region: Rect,
  font: string,
  lineHeight: number,
  maskRect: Rect,
  mask: ImageMask,
  maskPadding: { horizontal: number, vertical: number },
  lineClassName: string,
  side: 'left' | 'right',
): void {
  let cursor: LayoutCursor = { segmentIndex: 0, graphemeIndex: 0 }
  let lineTop = region.y

  while (true) {
    if (lineTop + lineHeight > region.y + region.height) break

    const bandTop = lineTop
    const bandBottom = lineTop + lineHeight
    const blocked: Interval[] = []
    const maskInterval = getMaskIntervalForBand(
      mask,
      maskRect,
      bandTop,
      bandBottom,
      maskPadding.horizontal,
      maskPadding.vertical,
    )
    if (maskInterval !== null) blocked.push(maskInterval)

    const slots = subtractIntervals(
      { left: region.x, right: region.x + region.width },
      blocked,
    )
    if (slots.length === 0) {
      lineTop += lineHeight
      continue
    }

    const slot = side === 'left'
      ? slots[slots.length - 1]!
      : slots[0]!
    const width = slot.right - slot.left
    const line = layoutNextLine(prepared, cursor, width)
    if (line === null) break

    const el = document.createElement('div')
    el.className = lineClassName
    el.textContent = line.text
    el.style.left = `${Math.round(slot.left)}px`
    el.style.top = `${Math.round(lineTop)}px`
    el.style.font = font
    el.style.lineHeight = `${lineHeight}px`
    stage.appendChild(el)

    cursor = line.end
    lineTop += lineHeight
  }
}

function clearRenderedLines(): void {
  const lines = stage.querySelectorAll('.line')
  lines.forEach(line => {
    line.remove()
  })
}

async function render(): Promise<void> {
  const { font, lineHeight } = getTypography()
  const pageWidth = window.innerWidth
  const pageHeight = Math.max(window.innerHeight, 980)

  stage.style.minHeight = `${pageHeight}px`

  const gutter = Math.round(Math.max(52, pageWidth * 0.048))
  const centerGap = Math.round(Math.max(34, pageWidth * 0.038))
  const headlineTop = Math.round(Math.max(42, pageHeight * 0.065))
  const headlineWidth = Math.round(Math.min(pageWidth - gutter * 2, pageWidth * 0.62))
  const copyTop = headlineTop + Math.round(Math.max(142, pageWidth * 0.122))
  const columnWidth = Math.round((pageWidth - gutter * 2 - centerGap) / 2)
  const columnHeight = pageHeight - copyTop - gutter

  const leftRegion: Rect = {
    x: gutter,
    y: copyTop,
    width: columnWidth,
    height: columnHeight,
  }

  const rightRegion: Rect = {
    x: gutter + columnWidth + centerGap,
    y: copyTop,
    width: columnWidth,
    height: columnHeight,
  }

  const openaiSize = Math.round(Math.max(260, Math.min(390, pageWidth * 0.25)))
  const openaiRect: Rect = {
    x: leftRegion.x - Math.round(openaiSize * 0.06),
    y: pageHeight - gutter - openaiSize + Math.round(openaiSize * 0.03),
    width: openaiSize,
    height: openaiSize,
  }

  const claudeSize = Math.round(Math.max(220, Math.min(340, pageWidth * 0.21)))
  const claudeRect: Rect = {
    x: rightRegion.x + rightRegion.width - Math.round(claudeSize * 0.61),
    y: Math.round(Math.max(36, headlineTop - 4)),
    width: claudeSize,
    height: claudeSize,
  }

  headline.style.left = `${gutter}px`
  headline.style.top = `${headlineTop}px`
  headline.style.width = `${headlineWidth}px`

  credit.style.left = `${gutter + 4}px`
  credit.style.top = `${copyTop - Math.round(Math.max(34, lineHeight * 1.8))}px`
  credit.style.width = `${Math.round(Math.min(headlineWidth, pageWidth * 0.36))}px`

  openaiLogo.style.left = `${openaiRect.x}px`
  openaiLogo.style.top = `${openaiRect.y}px`
  openaiLogo.style.width = `${openaiRect.width}px`
  openaiLogo.style.height = `${openaiRect.height}px`

  claudeLogo.style.left = `${claudeRect.x}px`
  claudeLogo.style.top = `${claudeRect.y}px`
  claudeLogo.style.width = `${claudeRect.width}px`
  claudeLogo.style.height = `${claudeRect.height}px`

  clearRenderedLines()

  const [openaiMask, claudeMask] = await Promise.all([
    makeImageMask(openaiLogo.src, openaiRect.width, openaiRect.height),
    makeImageMask(claudeLogo.src, claudeRect.width, claudeRect.height),
  ])

  renderColumn(
    getPrepared(LEFT_COPY, font),
    leftRegion,
    font,
    lineHeight,
    openaiRect,
    openaiMask,
    { horizontal: Math.round(lineHeight * 1.15), vertical: Math.round(lineHeight * 0.45) },
    'line line--left',
    'left',
  )

  renderColumn(
    getPrepared(RIGHT_COPY, font),
    rightRegion,
    font,
    lineHeight,
    claudeRect,
    claudeMask,
    { horizontal: Math.round(lineHeight * 1.05), vertical: Math.round(lineHeight * 0.42) },
    'line line--right',
    'right',
  )
}

function scheduleRender(): void {
  if (scheduled.value) return
  scheduled.value = true
  requestAnimationFrame(() => {
    scheduled.value = false
    void render()
  })
}

window.addEventListener('resize', scheduleRender)
void document.fonts.ready.then(() => {
  scheduleRender()
})
scheduleRender()
