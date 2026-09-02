# 覆盖率缺口归因（A1）——分支级清单与处置分类

- **日期**：2026-09-03
- **状态**：已实施（归因基线；处置按批推进）
- **工具**：cargo-llvm-cov --all-features --show-missing-lines
- **口径**：TOTAL 79.89% 行 / 75.77% 区域 / 81.11% 函数（107,783 行，21,671 未覆盖）
- **姊妹文档**：`2026-08-16-coverage-baseline.md`（首轮 default-features 基线）

---

## 1. 模块分布（337 个文件存在未覆盖行）

| 模块 | 文件数 | | 模块 | 文件数 |
|------|-------:|---|------|-------:|
| engine | 77 | | model | 8 |
| expression | 76 | | element / temporal | 7 / 7 |
| processor | 59 | | decoupled / inline | 6 / 6 |
| util | 19 | | cache / markup | 5 / 5 |
| context | 13 | | web | 3 |
| templateresolver | 10 | | 其余 14 模块 | 各 1-4 |

## 2. 缺口 Top 12（按绝对未覆盖行数）

| 文件 | 未覆盖行 | 行覆盖 | 归因分类 | 处置 |
|------|--------:|-------:|---------|------|
| `expression/native_variable_expression_evaluator.rs` | **2,628** | 47.47% | ②+①混合：OGNL 兼容求值器（宿主运行时注入面 + 表达式边界族） | **进行中**——并发会话正以 V3 golden 差分推进（6d84363/ognl_evaluation_golden），勿重复铺测试；协调点 |
| `engine/processor_template_handler.rs` | 1,127 | 56.62% | ①/③混合：引擎分派心脏（gathering model / deferred 分支） | corpus 已覆盖主干；缺口分支逐个对照 Java surefire 后补 |
| `expression/standard_expression_object_invoker.rs` | 672 | 36.84% | ①：表达式对象调用族 | VALUE_ADD 批（对照 Java ExpressionObjects 测试） |
| `expression/template_value.rs` | 395 | 22.85% | ①：值类型转换臂 | VALUE_ADD（类型转换往返矩阵，低成本高确定性） |
| `util/date_utils.rs` | 350 | 66.57% | ①：时间格式化分支 | VALUE_ADD 批 |
| `expression/strings.rs` | 349 | 60.70% | ✅ 部分关闭 | value_add_string_utils（23 测试）已落，剩余为更深层边界 |
| `context/web_engine_context.rs` | 343 | 66.31% | ②：Web 上下文（宿主面） | V5 宿主级（vernal 桥） |
| `markup/abstract_markup_template_parser.rs` | 335 | 81.12% | ③混合 | 逐分支核对后定 |
| `messageresolver/standard_message_resolution_utils.rs` | 282 | 81.31% | ①：消息解析回退族 | VALUE_ADD 批 |
| `engine/iterated_gathering_model_processable.rs` | 282 | 60.28% | ③：迭代收集模型 | 逐分支核对 |
| `util/escaped_attribute_utils.rs` | 280 | 60.51% | ①：属性转义分支 | VALUE_ADD 批 |
| `inline/abstract_standard_inliner.rs` | 273 | 52.27% | ①：内联处理模式族 | VALUE_ADD 批 |

## 3. 已完成批（本轮 VALUE_ADD，97 测试）

6 模块直锁（commit `6c4cf2f`）：processor_configuration_utils（方言装饰契约 11）、
number_utils（数字格式边界 26）、string_utils（escape/turkic/pack 23）、
resource_loader_utils（类路径工具 12）、thymeleaf_renderer（模式 MIME/分块 9）、
i_web_request（URL/参数契约 16）。归因结论：**Java 侧均无独立测试类**——
全部为合规 VALUE_ADD，非 parity 缺口。

## 4. 处置优先级（下一批次排期）

| 批 | 对象 | 理由 | 预估 |
|----|------|------|------|
| 1 | template_value.rs 转换矩阵 | 22.85% 最低覆盖 + 纯函数易测 + 类型错误是运行时高发缺陷 | 0.5 天 |
| 2 | standard_expression_object_invoker | 36.84% + 表达式对象是用户直接触面 | 1 天 |
| 3 | date_utils + escaped_attribute_utils + standard_message_resolution_utils | util 族边界分支 | 1 天 |
| 4 | native_variable_expression_evaluator | **先与并发会话的 golden 差分线协调**，避免重复 | 待协调 |
| 5 | processor_template_handler 深分支 | 需 surefire 覆盖数据逐分支比对 | 1-2 天 |

## 5. 纪律重申（rust-java-migration-testing 红线）

- 覆盖率为信号，非完成证明；无阈值门禁（CI report-only）；
- 每个新测试命名防御的具体缺陷/风险；
- 禁止：凑数用例、排除难文件、为数字加琐碎 getter 测试。

## 6. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-09-03 | 初版——337 文件归因 + Top12 分类 + 优先级排期；批 0（6 模块 97 测试）已于 2026-08-16 完成 |
