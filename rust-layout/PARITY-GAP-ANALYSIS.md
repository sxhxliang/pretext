# Rust ↔ TypeScript 未完全对齐项深度分析

> 本文补充 `ALIGNMENT.md` 中“尚未完全对齐”条目，说明根因、风险、下一步实现路径与验收标准。

## 1) 阿拉伯语 / 缅甸语 / CJK 标点粘连与脚本特定预处理

### 现状
- Rust 版本主要依赖 `apply_targeted_parity_patches()` 做样本导向补偿，仍不是通用脚本规则引擎。
- 当前策略会根据宽度和文本特征做后处理，能过现有 sweep，但可迁移性和可解释性不足。

### 根因
- TS 在 `analysis.ts` 中有更细粒度预处理策略（标点簇、脚本上下文、组合附标等），Rust 尚未等价移植。
- Rust 当前“先粗分段、再按补丁修正”的方式，会把脚本规则与断行策略耦合在后处理阶段。

### 风险
- 一旦语料变化（例如新缅甸语标点组合），现有补丁可能失效。
- 依赖 case-specific patch 会让后续维护复杂度上升。

### 下一步
1. 把脚本规则前移到 `prepare` 阶段，建立“脚本感知预处理层”，替代后置补丁。
2. 为 Arabic / Myanmar / CJK 分别建立“最小规则集 + 可回归样本集”。
3. 在 diff 脚本中引入脚本专属 case 组并单独统计 mismatch。

### 验收标准
- 不依赖 `apply_targeted_parity_patches()` 的 case-specific 宽度补丁，也能保持当前矩阵对齐。
- 新增脚本 case 不需要临时补丁即可通过。

---

## 2) `layoutNextLine` / `walkLineRanges` rich 元数据与高阶几何能力

### 现状
- Rust 已提供 API 形态，但核心实现仍是基于 `layout_with_lines()` 的再包装。
- 缺少 TS rich path 中的一些高阶几何与游标语义（如更细粒度状态推进、无物化批量几何信息）。

### 根因
- 当前 Rust 实现优先保障 lineCount 与基础文本输出一致，未完全建模 TS rich path 的状态机。

### 风险
- 在复杂编辑器布局或连续流式排版场景中，Rust API 可能出现可用但语义不完全对齐的问题。

### 下一步
1. 引入独立 line-stepper 内核（非 `layout_with_lines` 包装）。
2. 补齐游标状态对象，覆盖 start/end/grapheme 游标推进语义。
3. 新增 rich API 专项对齐测试（不仅比较 lineCount，还比较逐步推进轨迹）。

### 验收标准
- `layoutNextLine` / `walkLineRanges` 与 TS 对齐测试通过（包含游标轨迹比对）。

---

## 3) 与 TS 完全一致的断行策略（Rust 仍偏简化）

### 现状
- Rust 使用 segment+greedy 断行，辅以补丁策略。
- TS 断行核心有更复杂的语义（含脚本策略、软连字符策略、宽度容差、浏览器行为近似细节）。

### 根因
- Rust 未完整移植 TS 的 line-break 状态机、宽度容差和脚本策略组合。

### 风险
- 即使当前 sweep 对齐，也可能在新宽度、新语料或长段文本累积时出现偏差回归。

### 下一步
1. 抽离 Rust line-break 核心，与 TS line-break 结构逐段映射。
2. 在 diff 分析中加入“随机宽度采样 + 长文本累计 case”。
3. 引入“按段状态快照”对比（不只最终行数）。

### 验收标准
- 扩展 sweep（更多宽度与更长语料）保持稳定对齐。
- 减少/移除基于固定宽度区间的补丁。

---

## 4) Grapheme 级断词与复杂 emoji / CJK 禁则细节

### 现状
- Rust 目前在该层面依然偏近似实现（组合附标与 ZWJ 有基础处理，但未完整 grapheme cluster 语义）。
- emoji ZWJ、区域旗帜、复杂 CJK 禁则仍存在潜在边缘风险。

### 根因
- 缺少完整 grapheme 分段与脚本专属禁则表驱动策略。

### 风险
- 复杂 emoji 序列、罕见 CJK 标点组合在极窄宽度下可能回归。

### 下一步
1. 引入 grapheme 级分段层（与 TS 使用的行为保持一致）。
2. 为 emoji ZWJ / RI flags / CJK 禁则建立边界测试集合。
3. 在 diff 脚本里新增“line text + cursor”双维比对（不仅 lineCount）。

### 验收标准
- emoji / CJK 边界集在扩展矩阵下稳定 0 mismatch。

---

## 推荐推进顺序
1. 先做脚本规则前移（替代后置补丁）
2. 再做 rich API 状态机对齐
3. 然后补 grapheme/emoji 细节
4. 最后扩大 sweep 与随机回归，验证无补丁化稳定性

---

## 最新深度对比（2026-03-30）

- 基于 `scripts/rust-ts-deep-analysis.ts` 的 9 类样本 × 12 宽度对比（共 108 行）：
  - `lineCount` mismatch: `0/108`
  - `line.text` mismatch: `55/108`
  - `cursor(start/end)` mismatch: `101/108`
- 结论：当前 Rust 版本已做到 **lineCount 对齐**，但尚未达到 TS rich path 的 line 文本与游标语义一致；后续应优先推进第 2/3 项（rich 状态机 + grapheme 细节）。

### 已确认的主导 mismatch 形态（用于下一步修复）
- URL/query 相关：TS 会在 `.../path?` 与 query 之间形成更细粒度断点，Rust 当前更偏向把 query 连续吞进同一行（例如 `"/path?"` vs `"/path?q=alpha&lang="`）。
- mixed-app 首行边界：TS 与 Rust 对 `世界 👋🏽` 附近的断点不同（例如 `ts="界 👋🏽 "` vs `rust="世界 👋🏽 "`），提示 grapheme/segment 边界推进语义仍不一致。
- cursor 维度：TS rich cursor 是 `(segmentIndex, graphemeIndex)`，Rust 当前输出仍接近“segment 边界近似”，在 URL/mixed-app 上尤为明显。
