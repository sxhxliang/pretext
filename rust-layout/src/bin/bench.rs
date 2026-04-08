use std::time::Instant;

use rust_layout::{layout, prepare, SimpleMeasurer, WhiteSpaceMode};

fn parse_arg<T: std::str::FromStr>(name: &str, default: T) -> T {
    let prefix = format!("--{name}=");
    std::env::args()
        .find_map(|arg| arg.strip_prefix(&prefix).and_then(|v| v.parse::<T>().ok()))
        .unwrap_or(default)
}

fn main() {
    let iterations: usize = parse_arg("iterations", 20_000);
    let width: f32 = parse_arg("width", 320.0);
    let mode = match std::env::args()
        .find_map(|arg| arg.strip_prefix("--mode=").map(str::to_string))
        .as_deref()
    {
        Some("pre-wrap") => WhiteSpaceMode::PreWrap,
        _ => WhiteSpaceMode::Normal,
    };

    let seed =
        "Hello 世界 👋🏽 https://example.com/path?q=alpha&lang=zh 中文段落 mixed متن عربي 12345 ";
    let text = seed.repeat(16);
    let measurer = SimpleMeasurer { font_size_px: 16.0 };

    for _ in 0..500 {
        let prepared = prepare(&text, mode, &measurer);
        let _ = layout(&prepared, width, 19.0);
    }

    let start = Instant::now();
    let mut checksum: usize = 0;
    for _ in 0..iterations {
        let prepared = prepare(&text, mode, &measurer);
        let out = layout(&prepared, width, 19.0);
        checksum = checksum.wrapping_add(out.line_count);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iterations as f64;

    println!(
        "{{\"engine\":\"rust\",\"iterations\":{iterations},\"text_len\":{},\"width\":{width},\"ns_per_iter\":{ns_per_iter:.2},\"checksum\":{checksum}}}",
        text.len()
    );
}
