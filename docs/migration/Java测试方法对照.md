# Java 测试方法对照（thymeleaf-tests-core → thymeleaf-test）

本表把上游 `tests/thymeleaf-tests-core` 测试类的 **413 个测试方法**逐方法映射到 thymeleaf-test 中的 Rust 覆盖证据：每个方法一行，记录处置、运行时 case 与证据。

数据源：`docs/migration/baseline/source_parity_inventory.json`（875 个源码入口方法级处置，`MISSING=0`）。
Java 基线：`10f9dd2eb8cbd98515ce14b149d115e0287d0add`（thymeleaf/thymeleaf）。
本表由 `scripts/generate_java_test_method_matrix.py` 生成，修改前请先改脚本。

## 1. 汇总

| 维度 | 数量 |
|:---|---:|
| 核心测试方法（tests/thymeleaf-tests-core） | 413 |
| 运行时 case（核心） | 1154 |
| 集成测试方法（spring5/6/security） | 462 |
| 运行时 case（集成） | 1002 |
| 未处置（missing） | 0 |

核心处置分布：

| 处置 | 方法数 | 含义 |
|:---|---:|:---|
| SPLIT | 156 | 方法级断言拆入对应 Rust 对象合同（`thymeleaf/src/**` `#[cfg(test)]`）+ 共享端到端语料 |
| MERGED | 90 | Java `TestExecutor` 外壳合并到数据驱动语料运行器（`thtest_upstream_plain_batch.rs`），输入/期望/异常直读固定上游 .thtest |
| MAPPED | 166 | Java 测试由同名 Rust 合同测试 + 固定 Java Golden 逐记录验证（`thymeleaf-test/tests/*_java_parity.rs`） |
| NOT_APPLICABLE | 1 | 基准工作负载类，正确性由语料与端到端测试承担 |

集成模块（Spring 方言）全部为 `POLICY_DIFFERENCE`：

| 模块 | 方法数 | 处置 |
|:---|---:|:---|
| `tests/thymeleaf-tests-spring5` | 229 | POLICY_DIFFERENCE |
| `tests/thymeleaf-tests-spring6` | 229 | POLICY_DIFFERENCE |
| `tests/thymeleaf-tests-springsecurity5` | 2 | POLICY_DIFFERENCE |
| `tests/thymeleaf-tests-springsecurity6` | 2 | POLICY_DIFFERENCE |

## 2. 方法级映射（按测试类）

### `BenchmarkTest`（1 方法；NOT_APPLICABLE=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testBenchmark` | NOT_APPLICABLE | testBenchmark | `template_engine_smoke.rs`（替代验证） |

### `StandardCacheTest`（1 方法；MAPPED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testSoftReferenceHandling` | MAPPED | testSoftReferenceHandling | `standard_cache_java_parity.rs` `standard_cache_matches_java_golden_except_documented_soft_gc_boundary` |

### `ContextSequenceTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testContextSequenceNoSpring` | SPLIT | testContextSequenceNoSpring | `thymeleaf/src` `#[cfg(test)]` |

### `LazyContextVariableTest`（10 方法；SPLIT=10）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testLazyContextVariable01` | SPLIT | testLazyContextVariable01 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable02` | SPLIT | testLazyContextVariable02 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable03` | SPLIT | testLazyContextVariable03 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable04` | SPLIT | testLazyContextVariable04 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable05` | SPLIT | testLazyContextVariable05 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable06` | SPLIT | testLazyContextVariable06 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable07` | SPLIT | testLazyContextVariable07 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable08` | SPLIT | testLazyContextVariable08 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable09` | SPLIT | testLazyContextVariable09 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |
| `testLazyContextVariable10` | SPLIT | testLazyContextVariable10 | `thymeleaf/src/context/lazy_context_variable.rs` 对象合同 |

### `DialectOrderingTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testDialectOrder` | SPLIT | testDialectOrder | `thymeleaf/src` `#[cfg(test)]` |

### `DialectProcessWrappingTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testDialectWrapping` | SPLIT | testDialectWrapping | `thymeleaf/src` `#[cfg(test)]` |

### `DialectSetConfigurationTest`（8 方法；MAPPED=8）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testProcessorComputation01` | MAPPED | testProcessorComputation01 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation02` | MAPPED | testProcessorComputation02 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation03` | MAPPED | testProcessorComputation03 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation04` | MAPPED | testProcessorComputation04 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation05` | MAPPED | testProcessorComputation05 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation06` | MAPPED | testProcessorComputation06 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation07` | MAPPED | testProcessorComputation07 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |
| `testProcessorComputation08` | MAPPED | testProcessorComputation08 | `dialect_set_configuration_java_parity.rs` `upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering` |

### `AttributeDefinitionsTest`（4 方法；SPLIT=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/attribute_definitions.rs` 对象合同 |
| `testEmptyPrefix` | SPLIT | testEmptyPrefix | `thymeleaf/src/engine/attribute_definitions.rs` 对象合同 |
| `testNullPrefix` | SPLIT | testNullPrefix | `thymeleaf/src/engine/attribute_definitions.rs` 对象合同 |
| `testWhitespacePrefix` | SPLIT | testWhitespacePrefix | `thymeleaf/src/engine/attribute_definitions.rs` 对象合同 |

### `AttributeNamesTest`（6 方法；SPLIT=6）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHTMLBuffer` | SPLIT | testHTMLBuffer | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |
| `testHTMLString` | SPLIT | testHTMLString | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |
| `testXMLBuffer` | SPLIT | testXMLBuffer | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |
| `testXMLString` | SPLIT | testXMLString | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |
| `testTextBuffer` | SPLIT | testTextBuffer | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |
| `testTextString` | SPLIT | testTextString | `thymeleaf/src/engine/attribute_names.rs` 对象合同 |

### `BareHtmlEngineTest`（1 方法；MAPPED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | MAPPED | test | `bare_html_engine_java_parity.rs` `bare_html_engine_matches_java_26_cases` |

### `CDATASectionTest`（3 方法；SPLIT=3）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/cdata_section.rs` 对象合同 |
| `testSubsection` | SPLIT | testSubsection | `thymeleaf/src/engine/cdata_section.rs` 对象合同 |
| `testContentFlags` | SPLIT | testContentFlags | `thymeleaf/src/engine/cdata_section.rs` 对象合同 |

### `CommentTest`（3 方法；SPLIT=3）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/comment.rs` 对象合同 |
| `testSubsection` | SPLIT | testSubsection | `thymeleaf/src/engine/comment.rs` 对象合同 |
| `testContentFlags` | SPLIT | testContentFlags | `thymeleaf/src/engine/comment.rs` 对象合同 |

### `DocTypeTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/doc_type.rs` 对象合同 |

### `ElementAttributesTest`（4 方法；SPLIT=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHtmlElementAttributesAttrManagement` | SPLIT | testHtmlElementAttributesAttrManagement | `thymeleaf/src` `#[cfg(test)]` |
| `testXmlElementAttributesAttrManagement` | SPLIT | testXmlElementAttributesAttrManagement | `thymeleaf/src` `#[cfg(test)]` |
| `testHtmlElementAttributesAttrObtention` | SPLIT | testHtmlElementAttributesAttrObtention | `thymeleaf/src` `#[cfg(test)]` |
| `testXmlElementAttributesAttrObtention` | SPLIT | testXmlElementAttributesAttrObtention | `thymeleaf/src` `#[cfg(test)]` |

### `ElementDefinitionsTest`（4 方法；SPLIT=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/element_definitions.rs` 对象合同 |
| `testEmptyPrefix` | SPLIT | testEmptyPrefix | `thymeleaf/src/engine/element_definitions.rs` 对象合同 |
| `testNullPrefix` | SPLIT | testNullPrefix | `thymeleaf/src/engine/element_definitions.rs` 对象合同 |
| `testWhitespacePrefix` | SPLIT | testWhitespacePrefix | `thymeleaf/src/engine/element_definitions.rs` 对象合同 |

### `ElementNamesTest`（6 方法；SPLIT=6）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHTMLBuffer` | SPLIT | testHTMLBuffer | `thymeleaf/src/engine/element_names.rs` 对象合同 |
| `testHTMLString` | SPLIT | testHTMLString | `thymeleaf/src/engine/element_names.rs` 对象合同 |
| `testXMLBuffer` | SPLIT | testXMLBuffer | `thymeleaf/src/engine/element_names.rs` 对象合同 |
| `testXMLString` | SPLIT | testXMLString | `thymeleaf/src/engine/element_names.rs` 对象合同 |
| `testTextBuffer` | SPLIT | testTextBuffer | `thymeleaf/src/engine/element_names.rs` 对象合同 |
| `testTextString` | SPLIT | testTextString | `thymeleaf/src/engine/element_names.rs` 对象合同 |

### `ElementProcessorIteratorTest`（14 方法；SPLIT=14）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testProcessorIteration01` | SPLIT | testProcessorIteration01 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration02` | SPLIT | testProcessorIteration02 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration03` | SPLIT | testProcessorIteration03 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration04` | SPLIT | testProcessorIteration04 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration05` | SPLIT | testProcessorIteration05 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration06` | SPLIT | testProcessorIteration06 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration07` | SPLIT | testProcessorIteration07 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration08` | SPLIT | testProcessorIteration08 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration09` | SPLIT | testProcessorIteration09 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration10` | SPLIT | testProcessorIteration10 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration11` | SPLIT | testProcessorIteration11 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration12` | SPLIT | testProcessorIteration12 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration13` | SPLIT | testProcessorIteration13 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |
| `testProcessorIteration14` | SPLIT | testProcessorIteration14 | `thymeleaf/src/engine/element_processor_iterator.rs` 对象合同 |

### `EngineContextTest`（10 方法；SPLIT=10）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | SPLIT | test01 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test02` | SPLIT | test02 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test03` | SPLIT | test03 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test04` | SPLIT | test04 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test05` | SPLIT | test05 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test06` | SPLIT | test06 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test07` | SPLIT | test07 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test08` | SPLIT | test08 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test09` | SPLIT | test09 | `thymeleaf/src/context/engine_context.rs` 对象合同 |
| `test10` | SPLIT | test10 | `thymeleaf/src/context/engine_context.rs` 对象合同 |

### `OpenElementTagTest`（2 方法；SPLIT=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHtmlOpenElementAttrManagement` | SPLIT | testHtmlOpenElementAttrManagement | `thymeleaf/src/engine/open_element_tag.rs` 对象合同 |
| `testXmlOpenElementAttrManagement` | SPLIT | testXmlOpenElementAttrManagement | `thymeleaf/src/engine/open_element_tag.rs` 对象合同 |

### `ProcessingInstructionTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/processing_instruction.rs` 对象合同 |

### `SSEThrottledTemplateWriterTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testSSE01` | SPLIT | testSSE01 | `thymeleaf/src/engine/sse_throttled_template_writer.rs` 对象合同 |

### `StandaloneElementTagTest`（4 方法；SPLIT=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHtmlStandaloneElementAttrManagement` | SPLIT | testHtmlStandaloneElementAttrManagement | `thymeleaf/src/engine/standalone_element_tag.rs` 对象合同 |
| `testXmlStandaloneElementAttrManagement` | SPLIT | testXmlStandaloneElementAttrManagement | `thymeleaf/src/engine/standalone_element_tag.rs` 对象合同 |
| `testHtmlStandaloneElementPropertyManagement` | SPLIT | testHtmlStandaloneElementPropertyManagement | `thymeleaf/src/engine/standalone_element_tag.rs` 对象合同 |
| `testXmlStandaloneElementPropertyManagement` | SPLIT | testXmlStandaloneElementPropertyManagement | `thymeleaf/src/engine/standalone_element_tag.rs` 对象合同 |

### `TextTest`（3 方法；SPLIT=3）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src/engine/text.rs` 对象合同 |
| `testSubsection` | SPLIT | testSubsection | `thymeleaf/src/engine/text.rs` 对象合同 |
| `testContentFlags` | SPLIT | testContentFlags | `thymeleaf/src/engine/text.rs` 对象合同 |

### `WebEngineContextTest`（14 方法；SPLIT=14）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | SPLIT | test01 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test02` | SPLIT | test02 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test03` | SPLIT | test03 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test04` | SPLIT | test04 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test05` | SPLIT | test05 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test06` | SPLIT | test06 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test07` | SPLIT | test07 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test08` | SPLIT | test08 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test09` | SPLIT | test09 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test10` | SPLIT | test10 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test11` | SPLIT | test11 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test12` | SPLIT | test12 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test13` | SPLIT | test13 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |
| `test14` | SPLIT | test14 | `thymeleaf/src/context/web_engine_context.rs` 对象合同 |

### `XmlDeclarationTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src` `#[cfg(test)]` |

### `ScriptInlineTest`（4 方法；MERGED=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testDateInline` | MERGED | testDateInline | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testObjectInline` | MERGED | testObjectInline | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testArrayInline` | MERGED | testArrayInline | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testCollectionInline` | MERGED | testCollectionInline | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `LinkBuilderTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testLinkBuilder01` | MAPPED | testLinkBuilder01 | `link_builder_java_parity.rs` `standard_link_builder_matches_java_golden` |
| `testLinkBuilderWithECFactory01` | MAPPED | testLinkBuilderWithECFactory01 | `link_builder_java_parity.rs` `standard_link_builder_matches_java_golden` |

### `OfflineTest`（1 方法；MAPPED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testOffline01` | MAPPED | testOffline01 | `offline_java_parity.rs` `offline01_matches_java` |

### `Parsing01Test`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testParsing` | MERGED | testParsing | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `Parsing02Test`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testParsing` | MERGED | testParsing | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `Parsing03Test`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testParsing` | MERGED | testParsing | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ExpressionTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testExpression` | SPLIT | testExpression | `thymeleaf/src/expression/expression.rs` 对象合同 |

### `FragmentExpressionTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testFragmentSelection` | SPLIT | testFragmentSelection | `thymeleaf/src/expression/fragment_expression.rs` 对象合同 |

### `FragmentSignatureTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testFragmentSignature` | SPLIT | testFragmentSignature | `thymeleaf/src/expression/fragment_signature.rs` 对象合同 |

### `LiteralSubstitutionUtilTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testLiteralSubstitution` | SPLIT | testLiteralSubstitution | `thymeleaf/src/expression/literal_substitution_util.rs` 对象合同 |

### `TemporalsArrayTest`（17 方法；MAPPED=17）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testArrayFormat` | MAPPED | testArrayFormat | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayFormatWithLocale` | MAPPED | testArrayFormatWithLocale | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayFormatWithPattern` | MAPPED | testArrayFormatWithPattern | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayFormatWithPatternAndLocale` | MAPPED | testArrayFormatWithPatternAndLocale | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayDay` | MAPPED | testArrayDay | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayMonth` | MAPPED | testArrayMonth | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayMonthName` | MAPPED | testArrayMonthName | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayMonthNameShort` | MAPPED | testArrayMonthNameShort | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayYear` | MAPPED | testArrayYear | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayDayOfWeek` | MAPPED | testArrayDayOfWeek | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayDayOfWeekName` | MAPPED | testArrayDayOfWeekName | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayDayOfWeekNameShort` | MAPPED | testArrayDayOfWeekNameShort | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayHour` | MAPPED | testArrayHour | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayMinute` | MAPPED | testArrayMinute | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArraySecond` | MAPPED | testArraySecond | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testArrayNanosecond` | MAPPED | testArrayNanosecond | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |
| `testFormatISO` | MAPPED | testFormatISO | `temporal_utils_java_parity.rs` `temporal_array_utils_matches_java` |

### `TemporalsClassesFormattingTest`（10 方法；MAPPED=10）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `localDate` | MAPPED | localDate | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `localDateTime` | MAPPED | localDateTime | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `zonedDateTime` | MAPPED | zonedDateTime | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `instant` | MAPPED | instant | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `localTime` | MAPPED | localTime | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `offsetTime` | MAPPED | offsetTime | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `offsetDateTime` | MAPPED | offsetDateTime | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `year` | MAPPED | year | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `yearMonth` | MAPPED | yearMonth | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |
| `yearMonthForYMDLocales` | MAPPED | yearMonthForYMDLocales | `temporal_objects_java_parity.rs` `temporals_classes_no_pattern_matches_java` |

### `TemporalsCreationTest`（12 方法；MAPPED=12）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testCreateLocalDate` | MAPPED | testCreateLocalDate | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateLocalDateTime` | MAPPED | testCreateLocalDateTime | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateLocalDateTimeWithSeconds` | MAPPED | testCreateLocalDateTimeWithSeconds | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateLocalDateTimeWithMilliseconds` | MAPPED | testCreateLocalDateTimeWithMilliseconds | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateNow` | MAPPED | testCreateNow | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateNowForTimeZone` | MAPPED | testCreateNowForTimeZone | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateToday` | MAPPED | testCreateToday | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateTodayForTimeZone` | MAPPED | testCreateTodayForTimeZone | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateDate` | MAPPED | testCreateDate | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateDateTime` | MAPPED | testCreateDateTime | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateDateWithPattern` | MAPPED | testCreateDateWithPattern | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |
| `testCreateDateTimeWithPattern` | MAPPED | testCreateDateTimeWithPattern | `temporal_utils_java_parity.rs` `temporal_creation_utils_matches_java` |

### `TemporalsFormattingTest`（44 方法；MAPPED=44）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testFormat` | MAPPED | testFormat | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithNullTemporal` | MAPPED | testFormatWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithLocale` | MAPPED | testFormatWithLocale | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithLocaleAndNullTemporal` | MAPPED | testFormatWithLocaleAndNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithPattern` | MAPPED | testFormatWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithPatternAndZone` | MAPPED | testFormatWithPatternAndZone | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatStandardPatternDateTime` | MAPPED | testFormatStandardPatternDateTime | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatStandardPatternDate` | MAPPED | testFormatStandardPatternDate | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatStandardPatternTime` | MAPPED | testFormatStandardPatternTime | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithPatternAndNullTemporal` | MAPPED | testFormatWithPatternAndNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithPatternAndLocale` | MAPPED | testFormatWithPatternAndLocale | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatWithPatternAndLocaleAndNullTemporal` | MAPPED | testFormatWithPatternAndLocaleAndNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `localTimeWithPattern` | MAPPED | localTimeWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `offsetDateTimeWithPattern` | MAPPED | offsetDateTimeWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `offsetTimeWithPattern` | MAPPED | offsetTimeWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `yearWithPattern` | MAPPED | yearWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `yearMonthWithPattern` | MAPPED | yearMonthWithPattern | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDay` | MAPPED | testDay | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayWithNullTemporal` | MAPPED | testDayWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonth` | MAPPED | testMonth | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonthWithNullTemporal` | MAPPED | testMonthWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonthName` | MAPPED | testMonthName | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonthNameWithNullTemporal` | MAPPED | testMonthNameWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonthNameShort` | MAPPED | testMonthNameShort | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMonthNameShortWithNullTemporal` | MAPPED | testMonthNameShortWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testYear` | MAPPED | testYear | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testYearWithNullTemporal` | MAPPED | testYearWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeek` | MAPPED | testDayOfWeek | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeekWithNullTemporal` | MAPPED | testDayOfWeekWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeekName` | MAPPED | testDayOfWeekName | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeekNameWithNullTemporal` | MAPPED | testDayOfWeekNameWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeekNameShort` | MAPPED | testDayOfWeekNameShort | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testDayOfWeekNameShortWithNullTemporal` | MAPPED | testDayOfWeekNameShortWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testHour` | MAPPED | testHour | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testHourWithNullTemporal` | MAPPED | testHourWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMinute` | MAPPED | testMinute | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testMinuteWithNullTemporal` | MAPPED | testMinuteWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testSecond` | MAPPED | testSecond | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testSecondWithNullTemporal` | MAPPED | testSecondWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testNanosecond` | MAPPED | testNanosecond | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testNanosecondWithNullTemporal` | MAPPED | testNanosecondWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatISO` | MAPPED | testFormatISO | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testFormatISOWithNullTemporal` | MAPPED | testFormatISOWithNullTemporal | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |
| `testIssue17` | MAPPED | testIssue17 | `temporal_objects_java_parity.rs` `temporals_format_matches_java` |

### `TemporalsListTest`（17 方法；MAPPED=17）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testListFormat` | MAPPED | testListFormat | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListFormatWithLocale` | MAPPED | testListFormatWithLocale | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListFormatWithPattern` | MAPPED | testListFormatWithPattern | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListFormatWithPatternAndLocale` | MAPPED | testListFormatWithPatternAndLocale | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListDay` | MAPPED | testListDay | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListMonth` | MAPPED | testListMonth | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListMonthName` | MAPPED | testListMonthName | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListMonthNameShort` | MAPPED | testListMonthNameShort | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListYear` | MAPPED | testListYear | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListDayOfWeek` | MAPPED | testListDayOfWeek | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListDayOfWeekName` | MAPPED | testListDayOfWeekName | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListDayOfWeekNameShort` | MAPPED | testListDayOfWeekNameShort | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListHour` | MAPPED | testListHour | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListMinute` | MAPPED | testListMinute | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListSecond` | MAPPED | testListSecond | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListNanosecond` | MAPPED | testListNanosecond | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |
| `testListFormatISO` | MAPPED | testListFormatISO | `temporal_utils_java_parity.rs` `temporal_list_utils_matches_java` |

### `TemporalsSetTest`（17 方法；MAPPED=17）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testSetFormat` | MAPPED | testSetFormat | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetFormatWithLocale` | MAPPED | testSetFormatWithLocale | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetFormatWithPattern` | MAPPED | testSetFormatWithPattern | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetFormatWithPatternAndLocale` | MAPPED | testSetFormatWithPatternAndLocale | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetDay` | MAPPED | testSetDay | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetMonth` | MAPPED | testSetMonth | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetMonthName` | MAPPED | testSetMonthName | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetMonthNameShort` | MAPPED | testSetMonthNameShort | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetYear` | MAPPED | testSetYear | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetDayOfWeek` | MAPPED | testSetDayOfWeek | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetDayOfWeekName` | MAPPED | testSetDayOfWeekName | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetDayOfWeekNameShort` | MAPPED | testSetDayOfWeekNameShort | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetHour` | MAPPED | testSetHour | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetMinute` | MAPPED | testSetMinute | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetSecond` | MAPPED | testSetSecond | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetNanosecond` | MAPPED | testSetNanosecond | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |
| `testSetFormatISO` | MAPPED | testSetFormatISO | `temporal_utils_java_parity.rs` `temporal_set_utils_matches_java` |

### `FragmentInsertionExpressionTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testFragmentExpressionSelection` | SPLIT | testFragmentExpressionSelection | `thymeleaf/src` `#[cfg(test)]` |

### `StandardJavaScriptSerializerTest`（12 方法；SPLIT=12）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testPrintTestEnumDefaultJS01` | SPLIT | testPrintTestEnumDefaultJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintTestEnumJacksonJS01` | SPLIT | testPrintTestEnumJacksonJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintAnonymousEnumDefaultJS01` | SPLIT | testPrintAnonymousEnumDefaultJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintAnonymousEnumJacksonJS01` | SPLIT | testPrintAnonymousEnumJacksonJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintTestEnumDefaultJS02` | SPLIT | testPrintTestEnumDefaultJS02 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintTestEnumJacksonJS02` | SPLIT | testPrintTestEnumJacksonJS02 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintAnonymousEnumDefaultJS02` | SPLIT | testPrintAnonymousEnumDefaultJS02 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintAnonymousEnumJacksonJS02` | SPLIT | testPrintAnonymousEnumJacksonJS02 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintRecordDefaultJS01` | SPLIT | testPrintRecordDefaultJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintRecordJacksonJS01` | SPLIT | testPrintRecordJacksonJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintRecordWithSpecialCharsDefaultJS01` | SPLIT | testPrintRecordWithSpecialCharsDefaultJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |
| `testPrintRecordWithSpecialCharsJacksonJS01` | SPLIT | testPrintRecordWithSpecialCharsJacksonJS01 | `thymeleaf/src/serializer/standard_java_script_serializer.rs` 对象合同 |

### `AggregationTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testContext` | MERGED | testContext(TestExecutor)[10], testContext(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `AttrProcessorsTest`（19 方法；MERGED=19）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testRemove` | MERGED | testRemove(TestExecutor)[10], testRemove(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testIf` | MERGED | testIf(TestExecutor)[10], testIf(TestExecutor)[11], testI... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testUnless` | MERGED | testUnless(TestExecutor)[10], testUnless(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInline` | MERGED | testInline(TestExecutor)[10], testInline(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInclude` | MERGED | testInclude(TestExecutor)[10], testInclude(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInsert` | MERGED | testInsert(TestExecutor)[10], testInsert(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testReplace` | MERGED | testReplace(TestExecutor)[10], testReplace(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testEach` | MERGED | testEach(TestExecutor)[10], testEach(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testObject` | MERGED | testObject(TestExecutor)[10], testObject(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAttr` | MERGED | testAttr(TestExecutor)[10], testAttr(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSimpleValue` | MERGED | testSimpleValue(TestExecutor)[10], testSimpleValue(TestEx... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDoubleValue` | MERGED | testDoubleValue(TestExecutor)[10], testDoubleValue(TestEx... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAppendPrepend` | MERGED | testAppendPrepend(TestExecutor)[10], testAppendPrepend(Te... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testFixedValue` | MERGED | testFixedValue(TestExecutor)[10], testFixedValue(TestExec... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSwitch` | MERGED | testSwitch(TestExecutor)[10], testSwitch(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testWith` | MERGED | testWith(TestExecutor)[10], testWith(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDOMEvent` | MERGED | testDOMEvent(TestExecutor)[10], testDOMEvent(TestExecutor... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAssert` | MERGED | testAssert(TestExecutor)[10], testAssert(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDefault` | MERGED | testDefault(TestExecutor)[10], testDefault(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ConditionalCommentsTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testConditionalComments` | MERGED | testConditionalComments(TestExecutor)[10], testConditiona... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ContextTest`（2 方法；MERGED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testContextBase` | MERGED | testContextBase(TestExecutor)[10], testContextBase(TestEx... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testContextVarTest` | MERGED | testContextVarTest(TestExecutor)[10], testContextVarTest(... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `Conversion1Test`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testConversion` | MERGED | testConversion | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `Conversion4Test`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testConversion` | MERGED | testConversion | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `DataPrefixAttrProcessorsTest`（15 方法；MERGED=15）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testRemove` | MERGED | testRemove | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testIf` | MERGED | testIf | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testUnless` | MERGED | testUnless | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInline` | MERGED | testInline | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInclude` | MERGED | testInclude | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testReplace` | MERGED | testReplace | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testEach` | MERGED | testEach | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testObject` | MERGED | testObject | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAttr` | MERGED | testAttr | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSimpleValue` | MERGED | testSimpleValue | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDoubleValue` | MERGED | testDoubleValue | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAppendPrepend` | MERGED | testAppendPrepend | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testFixedValue` | MERGED | testFixedValue | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSwitch` | MERGED | testSwitch | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testWith` | MERGED | testWith | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `DataPrefixElementProcessorsTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testBlock` | MERGED | testBlock | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `DataPrefixFeaturesTest`（6 方法；MERGED=6）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testText` | MERGED | testText | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testLink` | MERGED | testLink | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testUtil` | MERGED | testUtil | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testExpression` | MERGED | testExpression | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testMessages` | MERGED | testMessages | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testNormalization` | MERGED | testNormalization | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `DOMSelectorTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testDOMSelector` | MERGED | testDOMSelector | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ElementProcessorsTest`（8 方法；MERGED=8）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testBlock` | MERGED | testBlock(TestExecutor)[10], testBlock(TestExecutor)[11],... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testElementMarkupProcessors` | MERGED | testElementMarkupProcessors(TestExecutor)[10], testElemen... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceModelBefore` | MERGED | testDialectPrecedenceModelBefore(TestExecutor)[10], testD... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceModelSame` | MERGED | testDialectPrecedenceModelSame(TestExecutor)[10], testDia... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceModelAfter` | MERGED | testDialectPrecedenceModelAfter(TestExecutor)[10], testDi... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceTagBefore` | MERGED | testDialectPrecedenceTagBefore(TestExecutor)[10], testDia... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceTagSame` | MERGED | testDialectPrecedenceTagSame(TestExecutor)[10], testDiale... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testDialectPrecedenceTagAfter` | MERGED | testDialectPrecedenceTagAfter(TestExecutor)[10], testDial... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `FeaturesTest`（16 方法；MERGED=16）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testText` | MERGED | testText(TestExecutor)[10], testText(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testLink` | MERGED | testLink(TestExecutor)[10], testLink(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testUtil` | MERGED | testUtil(TestExecutor)[10], testUtil(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testExpression` | MERGED | testExpression(TestExecutor)[10], testExpression(TestExec... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testMessages` | MERGED | testMessages(TestExecutor)[10], testMessages(TestExecutor... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testServletContext` | MERGED | testServletContext(TestExecutor)[10], testServletContext(... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSession` | MERGED | testSession(TestExecutor)[10], testSession(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testNormalization` | MERGED | testNormalization(TestExecutor)[10], testNormalization(Te... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testExecInfo` | MERGED | testExecInfo(TestExecutor)[10], testExecInfo(TestExecutor... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testAccessRestrictions` | MERGED | testAccessRestrictions(TestExecutor)[10], testAccessRestr... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInstanceStaticRestrictions` | MERGED | testInstanceStaticRestrictions(TestExecutor)[10], testIns... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInliningStandard` | MERGED | testInliningStandard(TestExecutor)[10], testInliningStand... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInliningNoStandard` | MERGED | testInliningNoStandard(TestExecutor)[10], testInliningNoS... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testInliningInteraction` | MERGED | testInliningInteraction(TestExecutor)[10], testInliningIn... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testLazy` | MERGED | testLazy(TestExecutor)[10], testLazy(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testElementStack` | MERGED | testElementStack(TestExecutor)[10], testElementStack(Test... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `GTVGTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testGTVG` | MERGED | testGTVG(TestExecutor)[10], testGTVG(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ParsingTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testParsing` | MERGED | testParsing(TestExecutor)[10], testParsing(TestExecutor)[... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `PrePostProcessorsTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testPrePostProcessors` | MERGED | testPrePostProcessors(TestExecutor)[10], testPrePostProce... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `ProcessorsTest`（5 方法；MERGED=5）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testReplaceWithProcessable` | MERGED | testReplaceWithProcessable(TestExecutor)[10], testReplace... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testReplaceWithNonProcessable` | MERGED | testReplaceWithNonProcessable(TestExecutor)[10], testRepl... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testRemove` | MERGED | testRemove(TestExecutor)[10], testRemove(TestExecutor)[11... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testNoOp` | MERGED | testNoOp(TestExecutor)[10], testNoOp(TestExecutor)[11], t... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testSurround` | MERGED | testSurround(TestExecutor)[10], testSurround(TestExecutor... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `TemplateBoundariesTest`（2 方法；MERGED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testAttrProcessor` | MERGED | testAttrProcessor(TestExecutor)[10], testAttrProcessor(Te... | `thtest_upstream_plain_batch.rs`（语料运行器） |
| `testConditionalComments` | MERGED | testConditionalComments(TestExecutor)[10], testConditiona... | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `XmlnsTest`（1 方法；MERGED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testXmlns` | MERGED | testXmlns | `thtest_upstream_plain_batch.rs`（语料运行器） |

### `TemplateEngineTest`（10 方法；MAPPED=10）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testTemplateResolverConfiguration01` | MAPPED | testTemplateResolverConfiguration01 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testTemplateResolverConfiguration03` | MAPPED | testTemplateResolverConfiguration03 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testTemplateResolverConfiguration04` | MAPPED | testTemplateResolverConfiguration04 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testTemplateResolverConfiguration05` | MAPPED | testTemplateResolverConfiguration05 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testTemplateResolverConfiguration07` | MAPPED | testTemplateResolverConfiguration07 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testTemplateResolverConfiguration09` | MAPPED | testTemplateResolverConfiguration09 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testStringTemplate` | MAPPED | testStringTemplate | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testDefaultTemplateResolver01` | MAPPED | testDefaultTemplateResolver01 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testDefaultTemplateResolver03` | MAPPED | testDefaultTemplateResolver03 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |
| `testDefaultTemplateResolver05` | MAPPED | testDefaultTemplateResolver05 | `template_engine_smoke.rs` `parsing001_runs_through_the_complete_html_engine_chain` |

### `DecoupledGTVGTest`（2 方法；SPLIT=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testGTVGHome` | SPLIT | testGTVGHome | `thymeleaf/src` `#[cfg(test)]` |
| `testGTVGSubscribe` | SPLIT | testGTVGSubscribe | `thymeleaf/src` `#[cfg(test)]` |

### `HtmlBlockSelectorMarkupHandlerTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | SPLIT | test | `thymeleaf/src` `#[cfg(test)]` |

### `ParsingDecoupled01Test`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testParsingDecoupled` | SPLIT | testParsingDecoupled | `thymeleaf/src` `#[cfg(test)]` |

### `TemplateFragmentMarkupReferenceResolverTest`（2 方法；SPLIT=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testHtml` | SPLIT | testHtml | `thymeleaf/src/markup/template_fragment_markup_reference_resolver.rs` 对象合同 |
| `testXml` | SPLIT | testXml | `thymeleaf/src/markup/template_fragment_markup_reference_resolver.rs` 对象合同 |

### `ParserLevelCommentMarkupReaderTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | MAPPED | test01 | `markup_comment_reader_java_parity.rs` `markup_comment_readers_match_java_golden` |
| `test02` | MAPPED | test02 | `markup_comment_reader_java_parity.rs` `markup_comment_readers_match_java_golden` |

### `ParserLevelCommentTextReaderTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | MAPPED | test01 | `block_aware_reader.rs` `java_golden_matches_text_comment_reader_streaming_contract` |
| `test02` | MAPPED | test02 | `block_aware_reader.rs` `java_golden_matches_text_comment_reader_streaming_contract` |

### `PrototypeOnlyCommentMarkupReaderTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | MAPPED | test01 | `markup_comment_reader_java_parity.rs` `markup_comment_readers_match_java_golden` |
| `test02` | MAPPED | test02 | `markup_comment_reader_java_parity.rs` `markup_comment_readers_match_java_golden` |

### `PrototypeOnlyCommentTextReaderTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test01` | MAPPED | test01 | `block_aware_reader.rs` `java_golden_matches_text_comment_reader_streaming_contract` |
| `test02` | MAPPED | test02 | `block_aware_reader.rs` `java_golden_matches_text_comment_reader_streaming_contract` |

### `TextParserTest`（1 方法；MAPPED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `test` | MAPPED | test | `text_parser.rs` `java_golden_matches_streaming_parser_pool_and_failure_semantics` |

### `TemplateResolverAttributesTest`（3 方法；MAPPED=3）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testTemplateResolutionAttributes01` | MAPPED | testTemplateResolutionAttributes01 | `template_resolution_java_parity.rs` `template_resolution_matches_java_golden` |
| `testTemplateResolutionAttributes02` | MAPPED | testTemplateResolutionAttributes02 | `template_resolution_java_parity.rs` `template_resolution_matches_java_golden` |
| `testTemplateResolutionAttributes03` | MAPPED | testTemplateResolutionAttributes03 | `template_resolution_java_parity.rs` `template_resolution_matches_java_golden` |

### `TemplateResourceTest`（5 方法；MAPPED=5）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testTemplateResourceUtils` | MAPPED | testTemplateResourceUtils | `template_resource_java_parity.rs` `template_resource_objects_match_java_golden`; `host_template_resource_java_parity.rs` `host_template_resources_match_java_golden` |
| `testServletContextResource` | MAPPED | testServletContextResource | `template_resource_java_parity.rs` `template_resource_objects_match_java_golden`; `host_template_resource_java_parity.rs` `host_template_resources_match_java_golden` |
| `testClassLoaderResource` | MAPPED | testClassLoaderResource | `template_resource_java_parity.rs` `template_resource_objects_match_java_golden`; `host_template_resource_java_parity.rs` `host_template_resources_match_java_golden` |
| `testFileResource` | MAPPED | testFileResource | `template_resource_java_parity.rs` `template_resource_objects_match_java_golden`; `host_template_resource_java_parity.rs` `host_template_resources_match_java_golden` |
| `testURLResource` | MAPPED | testURLResource | `template_resource_java_parity.rs` `template_resource_objects_match_java_golden`; `host_template_resource_java_parity.rs` `host_template_resources_match_java_golden` |

### `AggregateCharSequenceTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testAggregateString` | SPLIT | testAggregateString | `thymeleaf/src/util/aggregate_char_sequence.rs` 对象合同 |

### `DateUtilsTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testCreateToday` | SPLIT | testCreateToday | `thymeleaf/src/util/date_utils.rs` 对象合同 |

### `EvaluationUtilsTest`（4 方法；MAPPED=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `convertToBooleanTest` | MAPPED | convertToBooleanTest | `evaluation_utils_java_parity.rs` `evaluation_utils_and_bools_match_java_golden` |
| `convertToNumberTest` | MAPPED | convertToNumberTest | `evaluation_utils_java_parity.rs` `evaluation_utils_and_bools_match_java_golden` |
| `convertToListTest` | MAPPED | convertToListTest | `evaluation_utils_java_parity.rs` `evaluation_utils_and_bools_match_java_golden` |
| `convertToArrayTest` | MAPPED | convertToArrayTest | `evaluation_utils_java_parity.rs` `evaluation_utils_and_bools_match_java_golden` |

### `ExpressionUtilsTest`（4 方法；SPLIT=4）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `typeBlockedForAllPurposesTest` | SPLIT | typeBlockedForAllPurposesTest | `thymeleaf/src/util/expression_utils.rs` 对象合同 |
| `typeBlockedForTypeReferenceTest` | SPLIT | typeBlockedForTypeReferenceTest | `thymeleaf/src/util/expression_utils.rs` 对象合同 |
| `typeAllowedTest` | SPLIT | typeAllowedTest | `thymeleaf/src/util/expression_utils.rs` 对象合同 |
| `memberAllowedForTypeTest` | SPLIT | memberAllowedForTypeTest | `thymeleaf/src/util/expression_utils.rs` 对象合同 |

### `ListUtilsTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testSortListOfT` | MAPPED | testSortListOfT | `list_utils_java_parity.rs` `list_utils_and_expression_facade_match_java_golden` |
| `testSortListOfTComparatorOfQsuperT` | MAPPED | testSortListOfTComparatorOfQsuperT | `list_utils_java_parity.rs` `list_utils_and_expression_facade_match_java_golden` |

### `NumberUtilsTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testSequence` | SPLIT | testSequence | `thymeleaf/src/util/number_utils.rs` 对象合同 |

### `StandardExpressionUtilsTest`（1 方法；SPLIT=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testcontainsExternalAccess` | SPLIT | testcontainsExternalAccess | `thymeleaf/src/util/standard_expression_utils.rs` 对象合同 |

### `StringUtilsTest`（31 方法；SPLIT=31）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testCapitalize1` | SPLIT | testCapitalize1 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalize2` | SPLIT | testCapitalize2 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalize3` | SPLIT | testCapitalize3 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalize4` | SPLIT | testCapitalize4 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalize5` | SPLIT | testCapitalize5 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalize6` | SPLIT | testCapitalize6 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize1` | SPLIT | testUnCapitalize1 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize2` | SPLIT | testUnCapitalize2 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize3` | SPLIT | testUnCapitalize3 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize4` | SPLIT | testUnCapitalize4 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize5` | SPLIT | testUnCapitalize5 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testUnCapitalize6` | SPLIT | testUnCapitalize6 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords1` | SPLIT | testCapitalizeWords1 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords2` | SPLIT | testCapitalizeWords2 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords3` | SPLIT | testCapitalizeWords3 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords4` | SPLIT | testCapitalizeWords4 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords5` | SPLIT | testCapitalizeWords5 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords6` | SPLIT | testCapitalizeWords6 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords7` | SPLIT | testCapitalizeWords7 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords8` | SPLIT | testCapitalizeWords8 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords9` | SPLIT | testCapitalizeWords9 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords10` | SPLIT | testCapitalizeWords10 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords11` | SPLIT | testCapitalizeWords11 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords12` | SPLIT | testCapitalizeWords12 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testCapitalizeWords13` | SPLIT | testCapitalizeWords13 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testSubstring1` | SPLIT | testSubstring1 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testSubstring2` | SPLIT | testSubstring2 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testSubstring3` | SPLIT | testSubstring3 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testSubstring4` | SPLIT | testSubstring4 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testSubstring5` | SPLIT | testSubstring5 | `thymeleaf/src/util/string_utils.rs` 对象合同 |
| `testPack` | SPLIT | testPack | `thymeleaf/src/util/string_utils.rs` 对象合同 |

### `TextUtilsTest`（1 方法；MAPPED=1）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testContains` | MAPPED | testContains | `text_utils_java_parity.rs` `text_utils_matches_all_java_overloads_and_utf16_corpora` |

### `VersionUtilsTest`（2 方法；MAPPED=2）

| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |
|---|---|---|---|
| `testVersionMatches` | MAPPED | testVersionMatches | `version_utils_java_parity.rs` `version_utils_and_spec_match_java_golden` |
| `testIsAtLeast` | MAPPED | testIsAtLeast | `version_utils_java_parity.rs` `version_utils_and_spec_match_java_golden` |

## 3. 证据图例

- **RUST_TEST**：同名 parity 测试文件 `thymeleaf-test/tests/*_java_parity.rs` 中的 marker 测试（1:1 复刻 Java 断言）。
- **RUST_LIB_CONTRACTS**：`thymeleaf/src/**` 内对象合同的 `#[cfg(test)]` 单测。
- **UPSTREAM_THTEST**：`thymeleaf-test/tests/thtest_upstream_plain_batch.rs` 数据驱动运行器（2608 例 .thtest，`THYMELEAF_SCOPE=semantic_all`）。
- **REPLACEMENT_TEST**：替代验证测试（工作负载类）。
