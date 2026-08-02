# Java 测试类对照（thymeleaf-tests-core → thymeleaf-test）

本表把上游 `tests/thymeleaf-tests-core` 的 **83 个测试类**（84 个 `*Test.java` 中的
83 个在 SOURCE_PARITY 台账有方法级条目；`TemplateEngineTest` 的类名与 Java 包
根一致）逐类映射到 thymeleaf-test 中的 Rust 覆盖证据。数据源：
`docs/migration/baseline/source_parity_inventory.json`（875 个源码入口方法级
处置，`MISSING=0`）。

| 测试类 | 处置 | 方法 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|---|
| `DialectSetConfigurationTest` | MAPPED | 8 | 8 | dialect_set_configuration_java_parity.rs |
| `TemplateEngineTest` | MAPPED | 10 | 10 | template_engine_smoke.rs |
| `BenchmarkTest` | NOT_APPLICABLE | 1 | 1 | template_engine_smoke.rs |
| `StandardCacheTest` | MAPPED | 1 | 1 | standard_cache_java_parity.rs |
| `ContextSequenceTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `LazyContextVariableTest` | SPLIT | 10 | 10 | src/context/lazy_context_variable.rs；语料运行器 |
| `DialectOrderingTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `DialectProcessWrappingTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `AttributeDefinitionsTest` | SPLIT | 4 | 4 | src/engine/attribute_definitions.rs；语料运行器 |
| `AttributeNamesTest` | SPLIT | 6 | 6 | src/engine/attribute_names.rs；语料运行器 |
| `BareHtmlEngineTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `CDATASectionTest` | SPLIT | 3 | 3 | src/engine/cdata_section.rs；语料运行器 |
| `CommentTest` | SPLIT | 3 | 3 | src/engine/comment.rs；语料运行器 |
| `DocTypeTest` | SPLIT | 1 | 1 | src/engine/doc_type.rs；语料运行器 |
| `ElementAttributesTest` | SPLIT | 4 | 4 | src；语料运行器 |
| `ElementDefinitionsTest` | SPLIT | 4 | 4 | src/engine/element_definitions.rs；语料运行器 |
| `ElementNamesTest` | SPLIT | 6 | 6 | src/engine/element_names.rs；语料运行器 |
| `ElementProcessorIteratorTest` | SPLIT | 14 | 14 | src/engine/element_processor_iterator.rs；语料运行器 |
| `EngineContextTest` | SPLIT | 10 | 10 | src/context/engine_context.rs；语料运行器 |
| `OpenElementTagTest` | SPLIT | 2 | 2 | src/engine/open_element_tag.rs；语料运行器 |
| `ProcessingInstructionTest` | SPLIT | 1 | 1 | src/engine/processing_instruction.rs；语料运行器 |
| `SSEThrottledTemplateWriterTest` | SPLIT | 1 | 1 | src/engine/sse_throttled_template_writer.rs；语料运行器 |
| `StandaloneElementTagTest` | SPLIT | 4 | 4 | src/engine/standalone_element_tag.rs；语料运行器 |
| `TextTest` | SPLIT | 3 | 3 | src/engine/text.rs；语料运行器 |
| `WebEngineContextTest` | SPLIT | 14 | 14 | src/context/web_engine_context.rs；web_engine_context_java_parity.rs（共享标识守卫）；语料运行器 |
| `XmlDeclarationTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `ScriptInlineTest` | MERGED | 4 | 4 | 语料运行器 |
| `LinkBuilderTest` | MAPPED | 2 | 2 | link_builder_java_parity.rs |
| `OfflineTest` | MERGED | 1 | 1 | 语料运行器 |
| `Parsing01Test` | MERGED | 1 | 1 | 语料运行器 |
| `Parsing02Test` | MERGED | 1 | 1 | 语料运行器 |
| `Parsing03Test` | MERGED | 1 | 1 | 语料运行器 |
| `ExpressionTest` | SPLIT | 1 | 1 | src/expression/expression.rs；语料运行器 |
| `FragmentExpressionTest` | SPLIT | 1 | 1 | src/expression/fragment_expression.rs；语料运行器 |
| `FragmentSignatureTest` | SPLIT | 1 | 1 | src/expression/fragment_signature.rs；语料运行器 |
| `LiteralSubstitutionUtilTest` | SPLIT | 1 | 1 | src/expression/literal_substitution_util.rs；语料运行器 |
| `TemporalsArrayTest` | MAPPED | 17 | 17 | temporal_utils_java_parity.rs |
| `TemporalsClassesFormattingTest` | MAPPED | 10 | 10 | temporal_objects_java_parity.rs |
| `TemporalsCreationTest` | MAPPED | 12 | 12 | temporal_utils_java_parity.rs |
| `TemporalsFormattingTest` | MAPPED | 44 | 44 | temporal_objects_java_parity.rs |
| `TemporalsListTest` | MAPPED | 17 | 17 | temporal_utils_java_parity.rs |
| `TemporalsSetTest` | MAPPED | 17 | 17 | temporal_utils_java_parity.rs |
| `FragmentInsertionExpressionTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `StandardJavaScriptSerializerTest` | MAPPED | 12 | 12 | standard_java_script_serializer_java_parity.rs |
| `AggregationTest` | MERGED | 1 | 14 | 语料运行器 |
| `AttrProcessorsTest` | MERGED | 19 | 266 | 语料运行器 |
| `ConditionalCommentsTest` | MERGED | 1 | 14 | 语料运行器 |
| `ContextTest` | MERGED | 2 | 28 | 语料运行器 |
| `Conversion1Test` | MERGED | 1 | 1 | 语料运行器 |
| `Conversion4Test` | MERGED | 1 | 1 | 语料运行器 |
| `DataPrefixAttrProcessorsTest` | MERGED | 15 | 15 | 语料运行器 |
| `DataPrefixElementProcessorsTest` | MERGED | 1 | 1 | 语料运行器 |
| `DataPrefixFeaturesTest` | MERGED | 6 | 6 | 语料运行器 |
| `DOMSelectorTest` | MERGED | 1 | 1 | 语料运行器 |
| `ElementProcessorsTest` | MERGED | 8 | 112 | 语料运行器 |
| `FeaturesTest` | MERGED | 16 | 224 | 语料运行器 |
| `GTVGTest` | MERGED | 1 | 14 | 语料运行器 |
| `ParsingTest` | MERGED | 1 | 14 | 语料运行器 |
| `PrePostProcessorsTest` | MERGED | 1 | 14 | 语料运行器 |
| `ProcessorsTest` | MERGED | 5 | 70 | 语料运行器 |
| `TemplateBoundariesTest` | MERGED | 2 | 28 | 语料运行器 |
| `XmlnsTest` | MERGED | 1 | 1 | 语料运行器 |
| `DecoupledGTVGTest` | SPLIT | 2 | 2 | src；语料运行器 |
| `HtmlBlockSelectorMarkupHandlerTest` | SPLIT | 1 | 1 | src；语料运行器 |
| `ParsingDecoupled01Test` | SPLIT | 1 | 1 | src；语料运行器 |
| `TemplateFragmentMarkupReferenceResolverTest` | SPLIT | 2 | 2 | src/markup/template_fragment_markup_reference_resolver.rs；语料运行器 |
| `ParserLevelCommentMarkupReaderTest` | MAPPED | 2 | 2 | markup_comment_reader_java_parity.rs |
| `ParserLevelCommentTextReaderTest` | MAPPED | 2 | 2 | src/reader/block_aware_reader.rs |
| `PrototypeOnlyCommentMarkupReaderTest` | MAPPED | 2 | 2 | markup_comment_reader_java_parity.rs |
| `PrototypeOnlyCommentTextReaderTest` | MAPPED | 2 | 2 | src/reader/block_aware_reader.rs |
| `TextParserTest` | MAPPED | 1 | 1 | src/text/text_parser.rs |
| `TemplateResolverAttributesTest` | MAPPED | 3 | 3 | template_resolution_java_parity.rs |
| `TemplateResourceTest` | MAPPED | 5 | 5 | template_resource_java_parity.rs；host_template_resource_java_parity.rs |
| `AggregateCharSequenceTest` | SPLIT | 1 | 1 | src/util/aggregate_char_sequence.rs；语料运行器 |
| `DateUtilsTest` | SPLIT | 1 | 1 | src/util/date_utils.rs；语料运行器 |
| `EvaluationUtilsTest` | MAPPED | 4 | 4 | evaluation_utils_java_parity.rs |
| `ExpressionUtilsTest` | SPLIT | 4 | 4 | src/util/expression_utils.rs；语料运行器 |
| `ListUtilsTest` | MAPPED | 2 | 2 | list_utils_java_parity.rs |
| `NumberUtilsTest` | SPLIT | 1 | 1 | src/util/number_utils.rs；语料运行器 |
| `StandardExpressionUtilsTest` | SPLIT | 1 | 1 | src/util/standard_expression_utils.rs；语料运行器 |
| `StringUtilsTest` | MAPPED | 31 | 31 | string_utils_java_parity.rs（31 方法全部 1:1，含 capitalizeWords/substring/pack 边界） |
| `TextUtilsTest` | MAPPED | 1 | 1 | text_utils_java_parity.rs |
| `VersionUtilsTest` | MAPPED | 2 | 2 | version_utils_java_parity.rs |

## 处置说明

- **MAPPED**：Java 测试由同名 Rust 合同测试 + 固定 Java Golden 逐记录验证
  （对象级差分测试文件位于 `thymeleaf-test/tests/*_java_parity.rs`）。
- **MERGED**：Java `TestExecutor` 外壳合并到数据驱动语料运行器
  （`thymeleaf-test/tests/thtest_upstream_plain_batch.rs`），输入、期望输出与
  异常直接读取固定上游资源（2608 例 .thtest）。
- **SPLIT**：方法级断言拆入对应 Rust 对象合同（`thymeleaf/src/**` 内
  `#[cfg(test)]` 局部测试）+ 共享端到端语料。
- **NOT_APPLICABLE**：基准工作负载类，正确性由语料与端到端测试承担。

## 测试脚本镜像与测试逻辑 1:1

- **测试脚本**：上游 `tests/` 五模块的 **.thtest 语料（3493 文件）** 1:1 字节镜像到
  `thymeleaf-test/assets/thymeleaf-tests/`（`diff -r` 验证），由
  `tests/acceptance.rs` 按 SHA-256 逐文件固定；`source-test-parity.json`
  的 `test_case`（2608 可执行 .thtest）与 `test_asset`（3570 条目）双门禁。
  Java 源码与其余资源不镜像 —— 测试用例以 Rust 1:1 复刻。
- **测试逻辑**：每个 Java 测试类的断言逻辑在 `thymeleaf-test/tests/*_java_parity.rs`
  （或对象合同所在 `thymeleaf/src/**` 的 `#[cfg(test)]` 单测）中逐方法 1:1 复刻，
  与 Java 使用相同输入与期望。代表性缺口补齐：
  - `BareHtmlEngineTest`（26 个裸 HTML 解析边界用例）→
    `tests/bare_html_engine_java_parity.rs`
  - `OfflineTest`（offline01.html 渲染对比）→ `tests/offline_java_parity.rs`
  - `ElementProcessorIteratorTest` 05/10-14（迭代器动态 set/remove）→
    `thymeleaf/src/engine/element_processor_iterator.rs` 单测扩展
  - `TemporalsFormattingTest`（44 方法）+ `TemporalsClassesFormattingTest`（10 方法，
    无 pattern 逐类型 formatterFor）→ `tests/temporal_objects_java_parity.rs`
    （期望值以 Java 21 实测：JDK 9+ CLDR en_US，LONG/FULL 无 "at"、UTC 时区名 "Z"、
    Offset 类型 GMT 偏移段、OffsetTime 的 formatISO 走默认 ZoneId）
  - `TemporalsArrayTest` / `TemporalsListTest` / `TemporalsSetTest` /
    `TemporalsCreationTest` → `tests/temporal_utils_java_parity.rs`（utils 层 1:1）
  - `StringUtilsTest`（31 方法全部，capitalize/unCapitalize/capitalizeWords/
    substring/pack 边界与 null/空串/空白变体）→ `tests/string_utils_java_parity.rs`
  - `StandardJavaScriptSerializerTest`（12 方法 enum/字符串/record）→
    `tests/standard_java_script_serializer_java_parity.rs`；抽查修复 JS 日期序列化
    UTC 偏移 `Z` → `+00:00`（Java `ZZZ`+`insert(26,':')`）与 `#dates` 'Z' pattern
    零偏移 `+0000`
  - `WebEngineContextTest`：新增共享标识守卫测试（WebContext 变量 ↔ exchange
    属性双向可见，`tests/web_engine_context_java_parity.rs`）；多层嵌套遮蔽、
    toString 精确串、template stack 等 7 项仍为 SPLIT（实现语义已核验一致，
    断言缺口如实登记）
## Spring 集成模块（thymeleaf-tests-spring5/6/springsecurity5/6）

| 模块 | 测试类 | 方法 | 运行时 case | 处置 |
|---|---|---|---|---|
| `tests/thymeleaf-tests-spring5` | 36 | 229 | 499 | `POLICY_DIFFERENCE`（全部） |
| `tests/thymeleaf-tests-spring6` | 36 | 229 | 499 | `POLICY_DIFFERENCE`（全部） |
| `tests/thymeleaf-tests-springsecurity5` | 2 | 2 | 2 | `POLICY_DIFFERENCE`（全部） |
| `tests/thymeleaf-tests-springsecurity6` | 2 | 2 | 2 | `POLICY_DIFFERENCE`（全部） |

Spring MVC/WebFlux/SpEL/BeanFactory/ViewResolver 与 Spring Security 方言属于宿主
集成，不迁入中立 crate；资产树已完整镜像，等价能力由中立 Web 合同测试
（`web_renderer_source_parity.rs`）、独立 `thymeleaf-*` 适配器与共享语料承担
（每类证据见 `source_parity_inventory.json` 对应条目）。
