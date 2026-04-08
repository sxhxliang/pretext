# TypeScript → Rust 行为对齐清单

> 本清单对应 `src/layout.test.ts` 的可迁移“耐久不变量”子集，逐项在 `rust-layout/tests/core.rs` 中落地。

## 已对齐（本次累计）

1. whitespace-only input stays empty
2. collapses ordinary whitespace runs and trims the edges
3. pre-wrap mode keeps ordinary spaces instead of collapsing them
4. pre-wrap mode keeps hard breaks as explicit segments
5. pre-wrap mode normalizes CRLF into a single hard break
6. pre-wrap mode keeps tabs as explicit segments
7. keeps non-breaking spaces as glue instead of collapsing them away
8. keeps standalone non-breaking spaces as visible glue content
9. pre-wrap mode keeps whitespace-only input visible
10. keeps narrow no-break spaces as glue content
11. keeps word joiners as glue content
12. treats zero-width spaces as explicit break opportunities
13. treats soft hyphens as discretionary break points（分段层）
14. soft-hyphen break shows visible trailing `-` in line text
15. keeps URL-like runs together as one breakable segment（无空格 token 层）
16. keeps no-space ascii punctuation chains together as one breakable segment（token 层）
17. keeps numeric time ranges together（token 层）
18. keeps unicode-digit numeric expressions together（token 层）
19. does not attach opening punctuation to following whitespace
20. line count grows monotonically as width shrinks
21. pre-wrap mode treats hard breaks as forced line boundaries
22. pre-wrap keeps empty lines from consecutive hard breaks
23. pre-wrap does not invent an extra trailing empty line
24. `layoutNextLine` basic parity（按行迭代文本与 `layoutWithLines` 一致）
25. `walkLineRanges` basic parity（遍历输出与 `layoutWithLines` 一致）
26. overlong CJK token split avoids leading forbidden punctuation（基础禁则）
27. Arabic `" " + combining mark + base` 基础粘连预处理
28. TS 对齐 canary 矩阵（6 类高风险 case × 12 宽度）固定回归

## 尚未完全对齐（后续）

- 阿拉伯语/缅甸语/CJK 相关标点粘连与脚本特定预处理。
- `layoutNextLine` / `walkLineRanges` rich 元数据与高阶几何能力仍未对齐。
- 与 TS 完全一致的断行策略（当前 Rust 仍是简化 greedy + segment break 模型）。
- Grapheme 级别断词与复杂 emoji/CJK 禁则细节尚未接入。

> 详见：`rust-layout/PARITY-GAP-ANALYSIS.md`（根因、风险、实施步骤与验收标准）。


## Benchmark note

- `benchmark:rust-ts` 当前默认场景（mode=normal, width=320）下，Rust/TS checksum 已对齐（`980000`）。
- 扩展到 9 类样本 × 12 宽度后，`benchmark:rust-ts-diff` 最新结果为 `0/108` mismatch（0.00%）。
- 本轮 sweep 下 `normal` 与 `pre-wrap` 的当前样本集已全部 lineCount 对齐；后续仍需扩大语料与宽度矩阵持续验证。
