# thymeleaf/test —— Java `lib/testing` 参考镜像

本目录是上游 Thymeleaf **3.1.5.RELEASE**（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`）
`lib/testing/` 模块的 **1:1 字节镜像**（`diff -r` 验证，95/95 文件一致）：

- `org/thymeleaf/testing/templateengine/` ← `lib/testing/thymeleaf-testing/src/main/java`
  （.thtest 测试引擎：`TestExecutor`、`StandardTestBuilder`、`StandardTestEvaluator`、
  `TestContext`、`TestMessages`、`TestReporters`、资源与工具类）
- `org/thymeleaf/testing/templateengine/spring5/` ← `lib/testing/thymeleaf-testing-spring5`
- `org/thymeleaf/testing/templateengine/spring6/` ← `lib/testing/thymeleaf-testing-spring6`

## 与 Rust 移植的对应关系

| Java lib/testing 角色 | Rust 对应 |
|---|---|
| `TestExecutor` / `StandardTestBuilder` / `StandardTestEvaluator`（.thtest 解析、评估、比较） | `thymeleaf-test/tests/thtest_upstream_plain_batch.rs` + `tests/support/thtest_harness.rs`（2608 例语料数据驱动运行器） |
| `TestEngineMessageResolver`（%MESSAGES + MessageFormat 语义） | `thymeleaf-test/tests/support/test_engine_message_resolver.rs` |
| `TestContext` / `WebProcessingContextBuilder`（请求/会话/应用作用域） | `thymeleaf-test/tests/support/corpus_web_*.rs`（`CorpusWebExchange` 等） |
| `TestResource` / `ClassPathFileTestResource`（语料资源读取） | `CorpusStringTemplateResolver` + `thymeleaf-test/assets/thymeleaf-tests/` 资产镜像 |
| `TestLinkBuilder` | `thymeleaf-test/tests/support/test_link_builder.rs` |
| 测试引擎本身（`ITest`/`ITestResult`/`TestSequence` 等 JUnit 外壳） | 数据驱动运行器直接遍历清单，无逐类 JUnit 外壳（同 freemarker-test 模式） |

## 用途

- **参考**：实现或审计 Rust 语料机制时对照 Java 原语义
- **溯源**：`thymeleaf-test` 的差分测试在注释中按类名引用本镜像（如
  `TestEngineMessageResolver`、`WebProcessingContextBuilder`）
- 本目录不参与编译；它随主 crate 发布（与 `tests/fixtures` 等资产一致）
