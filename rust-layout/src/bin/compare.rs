use std::io::{self, Read};

use rust_layout::{layout_with_lines, prepare, SimpleMeasurer, WhiteSpaceMode};

fn parse_arg<T: std::str::FromStr>(name: &str, default: T) -> T {
    let prefix = format!("--{name}=");
    std::env::args()
        .find_map(|arg| arg.strip_prefix(&prefix).and_then(|v| v.parse::<T>().ok()))
        .unwrap_or(default)
}

fn main() {
    let width: f32 = parse_arg("width", 320.0);
    let mode = match std::env::args()
        .find_map(|arg| arg.strip_prefix("--mode=").map(str::to_string))
        .as_deref()
    {
        Some("pre-wrap") => WhiteSpaceMode::PreWrap,
        _ => WhiteSpaceMode::Normal,
    };

    let mut text = String::new();
    io::stdin()
        .read_to_string(&mut text)
        .expect("read stdin text");

    let measurer = SimpleMeasurer { font_size_px: 16.0 };
    let prepared = prepare(&text, mode, &measurer);
    let out = layout_with_lines(&prepared, width, 19.0);

    let lines_json = out
        .lines
        .iter()
        .map(|line| {
            format!(
                "{{\"text\":\"{}\",\"width\":{},\"start\":{{\"segmentIndex\":{},\"graphemeIndex\":0}},\"end\":{{\"segmentIndex\":{},\"graphemeIndex\":0}}}}",
                escape_json(&line.text),
                line.width,
                line.start,
                line.end
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    println!(
        "{{\"line_count\":{},\"lines\":[{}]}}",
        out.line_count, lines_json
    );
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
}
