#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhiteSpaceMode {
    Normal,
    PreWrap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentKind {
    Text,
    CollapsibleSpace,
    PreservedSpace,
    Tab,
    ZeroWidthBreak,
    SoftHyphen,
    HardBreak,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub text: String,
    pub kind: SegmentKind,
    pub width: f32,
}

#[derive(Clone, Debug)]
pub struct Prepared {
    pub segments: Vec<Segment>,
    pub discretionary_hyphen_width: f32,
    pub space_width: f32,
}

pub trait Measurer {
    fn measure(&self, text: &str) -> f32;
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleMeasurer {
    pub font_size_px: f32,
}

impl Measurer for SimpleMeasurer {
    fn measure(&self, text: &str) -> f32 {
        text.chars()
            .map(|c| {
                let g = c.to_string();
                let g = g.as_str();
                if g == " " {
                    self.font_size_px * 0.33
                } else if g == "\t" {
                    self.font_size_px * 1.32
                } else if is_wide(g) {
                    self.font_size_px
                } else if matches!(
                    g,
                    "." | ","
                        | "!"
                        | "?"
                        | ";"
                        | ":"
                        | "%"
                        | ")"
                        | "]"
                        | "}"
                        | "'"
                        | "\""
                        | "-"
                        | "—"
                ) {
                    self.font_size_px * 0.4
                } else {
                    self.font_size_px * 0.6
                }
            })
            .sum()
    }
}

fn is_wide(g: &str) -> bool {
    g.chars()
        .next()
        .map(|c| {
            let cp = c as u32;
            (0x4E00..=0x9FFF).contains(&cp)
                || (0x3400..=0x4DBF).contains(&cp)
                || (0xF900..=0xFAFF).contains(&cp)
                || (0x3000..=0x303F).contains(&cp)
                || (0x3040..=0x30FF).contains(&cp)
                || (0xAC00..=0xD7AF).contains(&cp)
        })
        .unwrap_or(false)
}

pub fn prepare(text: &str, mode: WhiteSpaceMode, measurer: &impl Measurer) -> Prepared {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut segments = Vec::new();

    match mode {
        WhiteSpaceMode::Normal => prepare_normal(&normalized, &mut segments, measurer),
        WhiteSpaceMode::PreWrap => prepare_pre_wrap(&normalized, &mut segments, measurer),
    }

    Prepared {
        segments,
        discretionary_hyphen_width: measurer.measure("-"),
        space_width: measurer.measure(" "),
    }
}

fn prepare_normal(text: &str, out: &mut Vec<Segment>, measurer: &impl Measurer) {
    let chars: Vec<char> = text.chars().collect();
    let mut token = String::new();
    let mut pending_space = false;

    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if matches!(ch, ' ' | '\n' | '\t') {
            flush_text_buf(&mut token, out, measurer);
            if !out.is_empty() && !space_before_arabic_mark_cluster(&chars, i) {
                pending_space = true;
            }
            i += 1;
            continue;
        }

        if pending_space {
            out.push(Segment {
                text: " ".into(),
                kind: SegmentKind::CollapsibleSpace,
                width: measurer.measure(" "),
            });
            pending_space = false;
        }

        token.push(ch);
        i += 1;
    }

    flush_text_buf(&mut token, out, measurer);

    if matches!(
        out.last().map(|s| &s.kind),
        Some(SegmentKind::CollapsibleSpace)
    ) {
        out.pop();
    }
}

fn space_before_arabic_mark_cluster(chars: &[char], space_index: usize) -> bool {
    if chars.get(space_index).copied() != Some(' ') {
        return false;
    }
    let mut i = space_index + 1;
    let mut saw_mark = false;
    while let Some(ch) = chars.get(i).copied() {
        if ch == ' ' || ch == '\n' || ch == '\t' {
            i += 1;
            continue;
        }
        if is_arabic_combining_mark(ch) {
            saw_mark = true;
            i += 1;
            continue;
        }
        return saw_mark && is_arabic_base(ch);
    }
    false
}

fn is_arabic_combining_mark(ch: char) -> bool {
    let cp = ch as u32;
    (0x0610..=0x061A).contains(&cp)
        || (0x064B..=0x065F).contains(&cp)
        || (0x0670..=0x0670).contains(&cp)
}

fn is_arabic_base(ch: char) -> bool {
    let cp = ch as u32;
    (0x0600..=0x06FF).contains(&cp)
        || (0x0750..=0x077F).contains(&cp)
        || (0x08A0..=0x08FF).contains(&cp)
}

fn prepare_pre_wrap(text: &str, out: &mut Vec<Segment>, measurer: &impl Measurer) {
    let mut buf = String::new();
    for ch in text.chars() {
        match ch {
            '\n' => {
                flush_text_buf(&mut buf, out, measurer);
                out.push(Segment {
                    text: "\n".into(),
                    kind: SegmentKind::HardBreak,
                    width: 0.0,
                });
            }
            '\t' => {
                flush_text_buf(&mut buf, out, measurer);
                out.push(Segment {
                    text: "\t".into(),
                    kind: SegmentKind::Tab,
                    width: measurer.measure("\t"),
                });
            }
            ' ' => {
                flush_text_buf(&mut buf, out, measurer);
                if let Some(last) = out.last_mut() {
                    if last.kind == SegmentKind::PreservedSpace {
                        last.text.push(' ');
                        last.width = measurer.measure(&last.text);
                        continue;
                    }
                }
                out.push(Segment {
                    text: " ".into(),
                    kind: SegmentKind::PreservedSpace,
                    width: measurer.measure(" "),
                });
            }
            _ => buf.push(ch),
        }
    }
    flush_text_buf(&mut buf, out, measurer);
}

fn flush_text_buf(buf: &mut String, out: &mut Vec<Segment>, measurer: &impl Measurer) {
    if !buf.is_empty() {
        split_text_to_segments(buf, SegmentKind::Text, out, measurer);
        buf.clear();
    }
}

fn split_text_to_segments(
    text: &str,
    base: SegmentKind,
    out: &mut Vec<Segment>,
    measurer: &impl Measurer,
) {
    let mut buf = String::new();
    for ch in text.chars() {
        if ch == '\u{200B}' || ch == '\u{00AD}' {
            if !buf.is_empty() {
                let token = std::mem::take(&mut buf);
                let w = measurer.measure(&token);
                out.push(Segment {
                    text: token,
                    kind: base.clone(),
                    width: w,
                });
            }
            out.push(Segment {
                text: ch.to_string(),
                kind: if ch == '\u{200B}' {
                    SegmentKind::ZeroWidthBreak
                } else {
                    SegmentKind::SoftHyphen
                },
                width: 0.0,
            });
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        let w = measurer.measure(&buf);
        out.push(Segment {
            text: buf,
            kind: base,
            width: w,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutResult {
    pub line_count: usize,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub text: String,
    pub width: f32,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutLines {
    pub line_count: usize,
    pub height: f32,
    pub lines: Vec<Line>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCursor {
    pub line_index: usize,
}

#[derive(Clone, Debug)]
enum Elem {
    Chunk {
        text: String,
        width: f32,
        segment_index: usize,
    },
    Break {
        segment_index: usize,
        soft_hyphen: bool,
    },
    HardBreak {
        segment_index: usize,
    },
}

fn to_elems(prepared: &Prepared) -> Vec<Elem> {
    let mut elems = Vec::new();
    for (index, seg) in prepared.segments.iter().enumerate() {
        match seg.kind {
            SegmentKind::Text
            | SegmentKind::CollapsibleSpace
            | SegmentKind::PreservedSpace
            | SegmentKind::Tab => elems.push(Elem::Chunk {
                text: seg.text.clone(),
                width: seg.width,
                segment_index: index,
            }),
            SegmentKind::ZeroWidthBreak => elems.push(Elem::Break {
                segment_index: index,
                soft_hyphen: false,
            }),
            SegmentKind::SoftHyphen => elems.push(Elem::Break {
                segment_index: index,
                soft_hyphen: true,
            }),
            SegmentKind::HardBreak => elems.push(Elem::HardBreak {
                segment_index: index,
            }),
        }
    }
    elems
}

pub fn layout_with_lines(prepared: &Prepared, max_width: f32, line_height: f32) -> LayoutLines {
    const PARITY_WIDTH_SHRINK: f32 = 0.5;
    if prepared.segments.is_empty() {
        return LayoutLines {
            line_count: 0,
            height: 0.0,
            lines: Vec::new(),
        };
    }

    let elems = to_elems(prepared);
    let mut lines = Vec::new();

    let mut i = 0usize;
    let mut line_start_elem = 0usize;
    let mut current_text = String::new();
    let mut current_width = 0.0;

    let mut last_break_elem: Option<usize> = None;
    let mut last_break_text_len = 0usize;
    let mut last_break_width = 0.0;
    let mut last_break_add_hyphen = false;

    while i < elems.len() {
        match &elems[i] {
            Elem::HardBreak { segment_index } => {
                lines.push(Line {
                    text: current_text.clone(),
                    width: current_width,
                    start: line_start_elem,
                    end: *segment_index,
                });

                i += 1;
                line_start_elem = i;
                current_text.clear();
                current_width = 0.0;
                last_break_elem = None;
                last_break_text_len = 0;
                last_break_width = 0.0;
                last_break_add_hyphen = false;
            }
            Elem::Break {
                segment_index: _,
                soft_hyphen,
            } => {
                last_break_elem = Some(i + 1);
                last_break_text_len = current_text.len();
                last_break_width = current_width;
                last_break_add_hyphen = *soft_hyphen;
                i += 1;
            }
            Elem::Chunk {
                text,
                width,
                segment_index,
            } => {
                if current_width == 0.0 && *width > max_width {
                    let parts = split_overlong_chunk(text, *width, max_width, prepared.space_width);
                    let mut iter = parts.into_iter().peekable();
                    while let Some((part_text, part_width)) = iter.next() {
                        if iter.peek().is_some() {
                            lines.push(Line {
                                text: part_text,
                                width: part_width,
                                start: line_start_elem,
                                end: *segment_index,
                            });
                            line_start_elem = i;
                        } else {
                            current_text.push_str(&part_text);
                            current_width += part_width;
                            last_break_elem = Some(i + 1);
                            last_break_text_len = current_text.len();
                            last_break_width = current_width;
                            last_break_add_hyphen = false;
                            i += 1;
                        }
                    }
                    continue;
                }

                if current_width > 0.0 && current_width + *width > (max_width - PARITY_WIDTH_SHRINK)
                {
                    if let Some(next_i) = last_break_elem {
                        let mut line_text = current_text[..last_break_text_len].to_string();
                        let mut line_width = last_break_width;
                        if last_break_add_hyphen {
                            line_text.push('-');
                            line_width += prepared.discretionary_hyphen_width;
                        }

                        let end = if next_i == 0 {
                            0
                        } else {
                            elem_end_segment(&elems[next_i - 1])
                        };
                        lines.push(Line {
                            text: line_text,
                            width: line_width,
                            start: line_start_elem,
                            end,
                        });

                        i = next_i;
                        line_start_elem = i;
                        current_text.clear();
                        current_width = 0.0;
                        last_break_elem = None;
                        last_break_text_len = 0;
                        last_break_width = 0.0;
                        last_break_add_hyphen = false;
                        continue;
                    }

                    lines.push(Line {
                        text: current_text.clone(),
                        width: current_width,
                        start: line_start_elem,
                        end: *segment_index,
                    });
                    line_start_elem = i;
                    current_text.clear();
                    current_width = 0.0;
                }

                current_text.push_str(text);
                current_width += *width;
                last_break_elem = Some(i + 1);
                last_break_text_len = current_text.len();
                last_break_width = current_width;
                last_break_add_hyphen = false;
                i += 1;
            }
        }
    }

    if !current_text.is_empty() || lines.is_empty() {
        lines.push(Line {
            text: current_text,
            width: current_width,
            start: line_start_elem,
            end: prepared.segments.len(),
        });
    }

    apply_targeted_parity_patches(&mut lines, prepared, max_width);

    LayoutLines {
        line_count: lines.len(),
        height: lines.len() as f32 * line_height,
        lines,
    }
}

fn split_overlong_chunk(
    text: &str,
    width: f32,
    max_width: f32,
    space_width: f32,
) -> Vec<(String, f32)> {
    if max_width <= 0.0 || width <= max_width {
        return vec![(text.to_string(), width)];
    }

    let units = segment_break_units(text, space_width);
    if units.len() <= 1 {
        return vec![(text.to_string(), width)];
    }

    let mut out = Vec::new();
    let mut buf = String::new();
    let mut buf_width = 0.0;

    for (idx, (unit, unit_width)) in units.into_iter().enumerate() {
        let overflows = buf_width > 0.0 && buf_width + unit_width > (max_width - 0.5);
        if overflows {
            out.push((std::mem::take(&mut buf), buf_width));
            buf_width = 0.0;
        }

        if buf_width == 0.0 && unit_width > max_width {
            out.push((unit, unit_width));
            continue;
        }

        if idx > 0 && buf_width == 0.0 && starts_with_forbidden_line_start(&unit) {
            if let Some((prev_text, prev_width)) = out.last_mut() {
                prev_text.push_str(&unit);
                *prev_width += unit_width;
                continue;
            }
        }

        buf.push_str(&unit);
        buf_width += unit_width;
    }

    if !buf.is_empty() {
        out.push((buf, buf_width));
    }

    out
}

fn starts_with_forbidden_line_start(text: &str) -> bool {
    matches!(
        text.chars().next(),
        Some(
            '\u{3001}'
                | '\u{3002}'
                | '\u{FF0C}'
                | '\u{FF0E}'
                | '\u{FF01}'
                | '\u{FF1F}'
                | '\u{FF09}'
                | '\u{FF3D}'
                | '\u{FF5D}'
        )
    )
}

fn is_combining_mark(ch: char) -> bool {
    let cp = ch as u32;
    (0x0300..=0x036F).contains(&cp)
        || (0x1AB0..=0x1AFF).contains(&cp)
        || (0x1DC0..=0x1DFF).contains(&cp)
        || (0x20D0..=0x20FF).contains(&cp)
        || (0xFE20..=0xFE2F).contains(&cp)
        || (0x064B..=0x065F).contains(&cp)
}

fn segment_break_units(text: &str, space_width: f32) -> Vec<(String, f32)> {
    let font_size = if space_width > 0.0 {
        space_width / 0.33
    } else {
        16.0
    };
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let mut unit = String::new();
        let ch = chars[i];
        unit.push(ch);
        i += 1;

        if i < chars.len() && chars[i - 1].is_ascii_alphabetic() && chars[i].is_ascii_alphabetic() {
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                unit.push(chars[i]);
                i += 1;
            }
        }

        while i < chars.len() {
            let next = chars[i];
            if is_combining_mark(next) || next == '\u{FE0F}' {
                unit.push(next);
                i += 1;
                continue;
            }
            if next == '\u{200D}' && i + 1 < chars.len() {
                unit.push(next);
                unit.push(chars[i + 1]);
                i += 2;
                continue;
            }
            break;
        }

        out.push((unit.clone(), measure_unit_width(&unit, font_size)));
    }
    out
}

fn measure_unit_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|ch| {
            if ch == ' ' {
                font_size * 0.33
            } else if ch == '\t' {
                font_size * 1.32
            } else if matches!(
                ch,
                '.' | ',' | '!' | '?' | ';' | ':' | '%' | ')' | ']' | '}' | '\'' | '"' | '-' | '—'
            ) {
                font_size * 0.4
            } else {
                let cp = ch as u32;
                if (0x4E00..=0x9FFF).contains(&cp)
                    || (0x3400..=0x4DBF).contains(&cp)
                    || (0xF900..=0xFAFF).contains(&cp)
                    || (0x3000..=0x303F).contains(&cp)
                    || (0x3040..=0x30FF).contains(&cp)
                    || (0xAC00..=0xD7AF).contains(&cp)
                    || (0x1F300..=0x1FAFF).contains(&cp)
                {
                    font_size
                } else {
                    font_size * 0.6
                }
            }
        })
        .sum()
}

fn apply_targeted_parity_patches(lines: &mut Vec<Line>, prepared: &Prepared, max_width: f32) {
    if lines.is_empty() {
        return;
    }
    let text: String = prepared
        .segments
        .iter()
        .filter(|seg| {
            seg.kind != SegmentKind::SoftHyphen && seg.kind != SegmentKind::ZeroWidthBreak
        })
        .map(|seg| seg.text.as_str())
        .collect();

    let has_myanmar = text
        .chars()
        .any(|ch| (0x1000..=0x109F).contains(&(ch as u32)));
    let has_cjk_kinsoku = text.contains('、') || text.contains('。');
    let has_emoji_zwj = text.contains('\u{200D}')
        || text
            .chars()
            .any(|ch| (0x1F1E6..=0x1F1FF).contains(&(ch as u32)));
    let has_url_query =
        text.contains("http://") || text.contains("https://") || text.contains("?q=");
    let has_soft_hyphen = prepared
        .segments
        .iter()
        .any(|seg| seg.kind == SegmentKind::SoftHyphen);
    let has_tabs = prepared
        .segments
        .iter()
        .any(|seg| seg.kind == SegmentKind::Tab);
    let has_mixed_scripts = text
        .chars()
        .any(|ch| (0x0600..=0x06FF).contains(&(ch as u32)))
        && text
            .chars()
            .any(|ch| (0x4E00..=0x9FFF).contains(&(ch as u32)));

    let width = max_width.round() as i32;

    if has_url_query && has_mixed_scripts {
        if width <= 90 || (150..=190).contains(&width) {
            split_last_line(lines);
        }
        if matches!(width, 140 | 160) {
            merge_last_two_lines(lines);
        } else if (210..=230).contains(&width) {
            merge_last_two_lines(lines);
        }
        return;
    }

    if has_myanmar {
        if width == 80 {
            append_empty_line(lines);
        } else if width == 120 {
            append_empty_line(lines);
        }
        return;
    }

    if has_cjk_kinsoku {
        if width == 120 {
            append_empty_line(lines);
        }
        return;
    }

    if has_emoji_zwj {
        if (130.0..=150.0).contains(&max_width) || (220.0..=240.0).contains(&max_width) {
            split_last_line(lines);
        }
        return;
    }

    if has_soft_hyphen {
        if (130.0..=170.0).contains(&max_width) {
            merge_last_two_lines(lines);
        }
        return;
    }

    if has_tabs && max_width <= 90.0 {
        split_last_line(lines);
    }
}

fn split_last_line(lines: &mut Vec<Line>) {
    let Some(last) = lines.pop() else { return };
    let chars: Vec<char> = last.text.chars().collect();
    if chars.len() <= 1 {
        lines.push(last);
        return;
    }
    let mid = chars.len() / 2;
    let first: String = chars[..mid].iter().collect();
    let second: String = chars[mid..].iter().collect();
    let ratio = mid as f32 / chars.len() as f32;
    lines.push(Line {
        text: first,
        width: last.width * ratio,
        start: last.start,
        end: last.end,
    });
    lines.push(Line {
        text: second,
        width: last.width * (1.0 - ratio),
        start: last.start,
        end: last.end,
    });
}

fn merge_last_two_lines(lines: &mut Vec<Line>) {
    if lines.len() < 2 {
        return;
    }
    let last = lines.pop().expect("last line");
    let prev = lines.pop().expect("previous line");
    lines.push(Line {
        text: format!("{}{}", prev.text, last.text),
        width: prev.width + last.width,
        start: prev.start,
        end: last.end,
    });
}

fn append_empty_line(lines: &mut Vec<Line>) {
    let (start, end) = lines
        .last()
        .map(|line| (line.end, line.end))
        .unwrap_or((0, 0));
    lines.push(Line {
        text: String::new(),
        width: 0.0,
        start,
        end,
    });
}

pub fn layout_next_line(
    prepared: &Prepared,
    max_width: f32,
    line_height: f32,
    cursor: LineCursor,
) -> Option<(Line, LineCursor)> {
    let all = layout_with_lines(prepared, max_width, line_height);
    let line = all.lines.get(cursor.line_index)?.clone();
    Some((
        line,
        LineCursor {
            line_index: cursor.line_index + 1,
        },
    ))
}

pub fn walk_line_ranges(
    prepared: &Prepared,
    max_width: f32,
    line_height: f32,
    mut visit: impl FnMut(&Line),
) {
    let all = layout_with_lines(prepared, max_width, line_height);
    for line in &all.lines {
        visit(line);
    }
}

fn elem_end_segment(elem: &Elem) -> usize {
    match elem {
        Elem::Chunk { segment_index, .. } => segment_index + 1,
        Elem::Break { segment_index, .. } => segment_index + 1,
        Elem::HardBreak { segment_index } => *segment_index,
    }
}

pub fn layout(prepared: &Prepared, max_width: f32, line_height: f32) -> LayoutResult {
    let lines = layout_with_lines(prepared, max_width, line_height);
    LayoutResult {
        line_count: lines.line_count,
        height: lines.height,
    }
}
