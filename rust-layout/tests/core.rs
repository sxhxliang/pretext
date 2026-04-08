use rust_layout::{layout, prepare, SegmentKind, SimpleMeasurer, WhiteSpaceMode};

const LINE_HEIGHT: f32 = 19.0;

fn measurer() -> SimpleMeasurer {
    SimpleMeasurer { font_size_px: 16.0 }
}

fn segment_texts(input: &str, mode: WhiteSpaceMode) -> Vec<String> {
    prepare(input, mode, &measurer())
        .segments
        .into_iter()
        .map(|segment| segment.text)
        .collect()
}

fn segment_kinds(input: &str, mode: WhiteSpaceMode) -> Vec<SegmentKind> {
    prepare(input, mode, &measurer())
        .segments
        .into_iter()
        .map(|segment| segment.kind)
        .collect()
}

#[test]
fn ts_whitespace_only_input_stays_empty() {
    let prepared = prepare("  \t\n  ", WhiteSpaceMode::Normal, &measurer());
    let out = layout(&prepared, 200.0, LINE_HEIGHT);
    assert_eq!(out.line_count, 0);
    assert_eq!(out.height, 0.0);
}

#[test]
fn ts_collapses_ordinary_whitespace_runs_and_trims_edges() {
    assert_eq!(
        segment_texts("  Hello\t \n  World  ", WhiteSpaceMode::Normal),
        vec!["Hello", " ", "World"]
    );
    assert_eq!(
        segment_kinds("  Hello\t \n  World  ", WhiteSpaceMode::Normal),
        vec![
            SegmentKind::Text,
            SegmentKind::CollapsibleSpace,
            SegmentKind::Text
        ]
    );
}

#[test]
fn ts_pre_wrap_keeps_ordinary_spaces_instead_of_collapsing() {
    assert_eq!(
        segment_texts("  Hello   World  ", WhiteSpaceMode::PreWrap),
        vec!["  ", "Hello", "   ", "World", "  "]
    );
    assert_eq!(
        segment_kinds("  Hello   World  ", WhiteSpaceMode::PreWrap),
        vec![
            SegmentKind::PreservedSpace,
            SegmentKind::Text,
            SegmentKind::PreservedSpace,
            SegmentKind::Text,
            SegmentKind::PreservedSpace,
        ]
    );
}

#[test]
fn ts_pre_wrap_keeps_hard_breaks_as_explicit_segments() {
    assert_eq!(
        segment_texts("Hello\nWorld", WhiteSpaceMode::PreWrap),
        vec!["Hello", "\n", "World"]
    );
    assert_eq!(
        segment_kinds("Hello\nWorld", WhiteSpaceMode::PreWrap),
        vec![SegmentKind::Text, SegmentKind::HardBreak, SegmentKind::Text]
    );
}

#[test]
fn ts_pre_wrap_normalizes_crlf_into_a_single_hard_break() {
    assert_eq!(
        segment_texts("Hello\r\nWorld", WhiteSpaceMode::PreWrap),
        vec!["Hello", "\n", "World"]
    );
}

#[test]
fn ts_pre_wrap_keeps_tabs_as_explicit_segments() {
    assert_eq!(
        segment_texts("Hello\tWorld", WhiteSpaceMode::PreWrap),
        vec!["Hello", "\t", "World"]
    );
    assert_eq!(
        segment_kinds("Hello\tWorld", WhiteSpaceMode::PreWrap),
        vec![SegmentKind::Text, SegmentKind::Tab, SegmentKind::Text]
    );
}

#[test]
fn ts_keeps_non_breaking_spaces_as_glue_in_text() {
    assert_eq!(
        segment_texts("Hello\u{00A0}world", WhiteSpaceMode::Normal),
        vec!["Hello\u{00A0}world"]
    );
}

#[test]
fn ts_keeps_standalone_nbsp_as_visible_content() {
    let prepared = prepare("\u{00A0}", WhiteSpaceMode::Normal, &measurer());
    let out = layout(&prepared, 200.0, LINE_HEIGHT);
    assert_eq!(out.line_count, 1);
    assert_eq!(out.height, LINE_HEIGHT);
}

#[test]
fn ts_pre_wrap_keeps_whitespace_only_input_visible() {
    let prepared = prepare("   ", WhiteSpaceMode::PreWrap, &measurer());
    let out = layout(&prepared, 200.0, LINE_HEIGHT);
    assert_eq!(out.line_count, 1);
    assert_eq!(out.height, LINE_HEIGHT);
}

#[test]
fn ts_keeps_narrow_no_break_spaces_as_glue_content() {
    assert_eq!(
        segment_texts("10\u{202F}000", WhiteSpaceMode::Normal),
        vec!["10\u{202F}000"]
    );
}

#[test]
fn ts_keeps_word_joiners_as_glue_content() {
    assert_eq!(
        segment_texts("foo\u{2060}bar", WhiteSpaceMode::Normal),
        vec!["foo\u{2060}bar"]
    );
}

#[test]
fn ts_treats_zero_width_spaces_as_explicit_break_opportunities() {
    assert_eq!(
        segment_texts("alpha\u{200B}beta", WhiteSpaceMode::Normal),
        vec!["alpha", "\u{200B}", "beta"]
    );
    assert_eq!(
        segment_kinds("alpha\u{200B}beta", WhiteSpaceMode::Normal),
        vec![
            SegmentKind::Text,
            SegmentKind::ZeroWidthBreak,
            SegmentKind::Text,
        ]
    );
}

#[test]
fn ts_treats_soft_hyphens_as_discretionary_break_points() {
    let prepared = prepare("trans\u{00AD}atlantic", WhiteSpaceMode::Normal, &measurer());
    let texts: Vec<String> = prepared
        .segments
        .iter()
        .map(|segment| segment.text.clone())
        .collect();
    let kinds: Vec<SegmentKind> = prepared
        .segments
        .iter()
        .map(|segment| segment.kind.clone())
        .collect();
    assert_eq!(texts, vec!["trans", "\u{00AD}", "atlantic"]);
    assert_eq!(
        kinds,
        vec![
            SegmentKind::Text,
            SegmentKind::SoftHyphen,
            SegmentKind::Text
        ]
    );
    assert!(prepared.discretionary_hyphen_width > 0.0);
}

#[test]
fn ts_keeps_url_like_runs_together_as_one_breakable_segment() {
    assert_eq!(
        segment_texts(
            "https://example.com/path/to/file?query=one&two=three",
            WhiteSpaceMode::Normal
        ),
        vec!["https://example.com/path/to/file?query=one&two=three"]
    );
}

#[test]
fn ts_keeps_no_space_ascii_punctuation_chains_together() {
    assert_eq!(
        segment_texts("status!!!??--ok", WhiteSpaceMode::Normal),
        vec!["status!!!??--ok"]
    );
}

#[test]
fn ts_keeps_numeric_time_ranges_together() {
    assert_eq!(
        segment_texts("7:00-9:00", WhiteSpaceMode::Normal),
        vec!["7:00-9:00"]
    );
}

#[test]
fn ts_keeps_unicode_digit_numeric_expressions_together() {
    assert_eq!(segment_texts("२४×७", WhiteSpaceMode::Normal), vec!["२४×७"]);
}

#[test]
fn ts_does_not_attach_opening_punctuation_to_following_whitespace() {
    assert_eq!(
        segment_texts("( hello", WhiteSpaceMode::Normal),
        vec!["(", " ", "hello"]
    );
}

#[test]
fn ts_line_count_grows_monotonically_as_width_shrinks() {
    let prepared = prepare(
        "Monotonic width shrinking should never reduce line count.",
        WhiteSpaceMode::Normal,
        &measurer(),
    );

    let wide = layout(&prepared, 320.0, LINE_HEIGHT).line_count;
    let medium = layout(&prepared, 220.0, LINE_HEIGHT).line_count;
    let narrow = layout(&prepared, 120.0, LINE_HEIGHT).line_count;

    assert!(wide <= medium);
    assert!(medium <= narrow);
}

#[test]
fn ts_pre_wrap_hard_breaks_force_separate_lines() {
    let prepared = prepare("first\nsecond\nthird", WhiteSpaceMode::PreWrap, &measurer());
    let out = layout(&prepared, 10_000.0, LINE_HEIGHT);
    assert_eq!(out.line_count, 3);
}

#[test]
fn ts_soft_hyphen_break_shows_visible_trailing_hyphen_in_line_text() {
    use rust_layout::layout_with_lines;

    let prepared = prepare(
        "foo trans\u{00AD}atlantic",
        WhiteSpaceMode::Normal,
        &measurer(),
    );
    let out = layout_with_lines(&prepared, 95.0, LINE_HEIGHT);
    assert_eq!(out.line_count, 2);
    assert_eq!(out.lines[0].text, "foo trans-");
    assert_eq!(out.lines[1].text, "atlantic");
}

#[test]
fn ts_pre_wrap_does_not_invent_extra_trailing_empty_line() {
    use rust_layout::layout_with_lines;

    let prepared = prepare("Hello\nWorld", WhiteSpaceMode::PreWrap, &measurer());
    let out = layout_with_lines(&prepared, 10_000.0, LINE_HEIGHT);
    let lines: Vec<&str> = out.lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(lines, vec!["Hello", "World"]);
}

#[test]
fn ts_pre_wrap_keeps_empty_lines_from_consecutive_hard_breaks() {
    use rust_layout::layout_with_lines;

    let prepared = prepare("A\n\nB", WhiteSpaceMode::PreWrap, &measurer());
    let out = layout_with_lines(&prepared, 10_000.0, LINE_HEIGHT);
    let lines: Vec<&str> = out.lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(lines, vec!["A", "", "B"]);
}

#[test]
fn ts_arabic_space_plus_mark_cluster_keeps_together() {
    let prepared = prepare(
        "قال \u{0651}\u{0628}كم",
        WhiteSpaceMode::Normal,
        &measurer(),
    );
    let texts: Vec<String> = prepared
        .segments
        .into_iter()
        .map(|segment| segment.text)
        .collect();
    assert_eq!(texts, vec!["قال", "\u{0651}\u{0628}كم"]);
}

#[test]
fn ts_layout_next_line_matches_layout_with_lines() {
    use rust_layout::{layout_next_line, layout_with_lines, LineCursor};

    let prepared = prepare(
        "one two three four five six seven",
        WhiteSpaceMode::Normal,
        &measurer(),
    );
    let expected = layout_with_lines(&prepared, 80.0, LINE_HEIGHT);
    let mut cursor = LineCursor { line_index: 0 };
    let mut got = Vec::new();
    while let Some((line, next)) = layout_next_line(&prepared, 80.0, LINE_HEIGHT, cursor) {
        got.push(line.text);
        cursor = next;
    }
    let expected_texts: Vec<String> = expected
        .lines
        .iter()
        .map(|line| line.text.clone())
        .collect();
    assert_eq!(got, expected_texts);
}

#[test]
fn ts_walk_line_ranges_matches_layout_with_lines() {
    use rust_layout::{layout_with_lines, walk_line_ranges};

    let prepared = prepare(
        "walk line ranges should align with line materialization",
        WhiteSpaceMode::Normal,
        &measurer(),
    );
    let expected = layout_with_lines(&prepared, 90.0, LINE_HEIGHT);
    let mut walked = Vec::new();
    walk_line_ranges(&prepared, 90.0, LINE_HEIGHT, |line| {
        walked.push(line.text.clone())
    });
    let expected_texts: Vec<String> = expected
        .lines
        .iter()
        .map(|line| line.text.clone())
        .collect();
    assert_eq!(walked, expected_texts);
}

#[test]
fn ts_cjk_forbidden_start_punctuation_stays_with_previous_piece_when_split() {
    use rust_layout::layout_with_lines;

    let prepared = prepare("漢字漢字、漢字漢字", WhiteSpaceMode::Normal, &measurer());
    let out = layout_with_lines(&prepared, 45.0, LINE_HEIGHT);
    assert!(out.line_count >= 2);
    if let Some(second) = out.lines.get(1) {
        assert!(!second.text.starts_with('、'));
    }
}

#[test]
fn ts_alignment_matrix_canaries_match_expected_counts() {
    let widths = [80.0, 100.0, 120.0, 140.0, 160.0, 180.0, 200.0, 220.0, 240.0, 260.0, 300.0, 320.0];

    let cases: [(&str, WhiteSpaceMode, [usize; 12]); 6] = [
        (
            "Hello 世界 👋🏽 https://example.com/path?q=alpha&lang=zh 中文段落 mixed متن عربي 12345",
            WhiteSpaceMode::Normal,
            [12, 10, 8, 6, 6, 6, 5, 4, 4, 4, 4, 4],
        ),
        (
            "မြန်မာစာ၊အမှတ်အသားနှင့်စမ်းသပ်မှု",
            WhiteSpaceMode::Normal,
            [6, 4, 4, 3, 3, 2, 2, 2, 2, 2, 2, 1],
        ),
        (
            "漢字漢字、漢字漢字。日本語テキスト（テスト）",
            WhiteSpaceMode::Normal,
            [5, 4, 4, 3, 3, 2, 2, 2, 2, 2, 2, 2],
        ),
        (
            "Family 👨‍👩‍👧‍👦 and flags 🇺🇸🇨🇳 with text wrap test",
            WhiteSpaceMode::Normal,
            [9, 6, 5, 5, 4, 3, 3, 3, 3, 2, 2, 2],
        ),
        (
            "foo trans\u{00AD}atlantic integration check",
            WhiteSpaceMode::Normal,
            [6, 4, 4, 3, 2, 2, 2, 2, 2, 2, 2, 2],
        ),
        ("A\tB\tC\n\tD", WhiteSpaceMode::PreWrap, [3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]),
    ];

    for (text, mode, expected) in cases {
        let prepared = prepare(text, mode, &measurer());
        for (idx, width) in widths.iter().enumerate() {
            let out = layout(&prepared, *width, LINE_HEIGHT);
            assert_eq!(out.line_count, expected[idx], "text={text}, width={width}");
        }
    }
}
