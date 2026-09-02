# 覆盖率基线（cargo-llvm-cov）

- **日期**：2026-08-16
- **状态**：已实施（覆盖率基线落档；按 rust-java-migration-testing 技能口径——覆盖率为信号，非完成证明）
- **工具**：cargo-llvm-cov 0.8.7（llvm-tools-preview）
- **命令**：`cargo llvm-cov --workspace --summary-only`（default features 档案；
  CI 步骤为 `--all-features` 口径，两档差异后续对齐时重测）

## 1. 总量

| 指标 | 值 |
|------|-----|
| 总行数 | 107,823 |
| 已覆盖 | 85,987（**79.75%**） |
| 区域覆盖 | 75.62%（9,489 中 2,313 未覆盖） |
| 函数覆盖 | 81.00%（80,117 中 15,225 未覆盖） |

## 2. 高覆盖区（抽样佐证测试面质量）

validate / version_utils / utf16_string / set_utils / pattern_utils /
pattern_spec / map_utils / logging_utils / object_utils 等 util 模块
**100%**；text_utils 96.99%（2,594 行大文件）、locale 92.37%。

测试面佐证：2,595 corpus（semantic_all 门禁 2/2 批次通过）+
对象级 java_parity 直锁（如 string_utils_java_parity 45 测试 ↔ Java
StringUtilsTest 31 方法 MAPPED）+ workspace 1,437 测试（--all-features 实测）。

## 3. 缺口清单（按绝对未覆盖行数排序——VALUE_ADD 候选，非门禁）

| 模块 | 行覆盖 | 未覆盖行 | 备注 |
|------|-------:|---------:|------|
| util/processor_configuration_utils.rs | 52.24% | 213 | 配置聚合分支 |
| util/string_utils.rs | 76.25% | 199 | Java 测试亦未触达的边界分支（parity 45 测试在位） |
| web/thymeleaf_renderer.rs | 64.80% | 126 | web 渲染路径（P0 新域） |
| util/number_utils.rs | 76.40% | 105 | |
| util/resource_loader_utils.rs | 49.65% | 71 | |
| web/i_web_request.rs | 44.25% | 63 | |

**处置纪律**（rust-java-migration-testing 技能红线）：上述缺口按 test-value
rubric 逐个核出「对应 Java 测试是否触达同分支」后再补 VALUE_ADD 测试；
禁止为凑数字加琐碎用例/排除难文件。当前无覆盖率阈值门禁（CI 为 report-only），
与技能「覆盖率为信号」口径一致。

## 4. 与门禁的关系

- alpha→beta 门禁不含覆盖率阈值（versioning-governance §条件表）；
- 本基线作为 S11 收口后的存证，后续批次（beta.0 前）复测对比趋势。

## 5. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-08-16 | 初版——default-features 档案全量 summary |
