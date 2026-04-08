# Rust vs TypeScript benchmark (prepare + layout)

- Command: `bun run benchmark:rust-ts`
- Scenario: mixed-script string (`textLength=1296`), width `320`, `20,000` iterations, `white-space: normal`.
- Date: 2026-03-30.

## Result

- TypeScript: `1,629,846.14 ns/iter`
- Rust: `198,638.78 ns/iter`
- Speedup (TS / Rust): `8.21x`
- Checksum parity: `980000 == 980000` ✅

## Notes

1. 该数字反映当前默认样本下的吞吐，主要用于回归趋势对比。
2. 与本次 diff sweep 一致，当前样本矩阵下 lineCount 已对齐（`0/108` mismatch）。
