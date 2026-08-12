# 方法级对照表

- **日期**：2026-07-28
- **作者**：thymeleaf-rust 团队
- **状态**：已实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`)
- **相关计划**：`docs/superpowers/plans/2026-07-28-s0-s10-batch-migration.md`

---

# Thymeleaf → thymeleaf-rust 方法级迁移对照表

> **文档说明**：逐一登记 Java 方法、构造器、重载、参数名与 Rust API。本文件记录已经进入实现阶段的对象；完整机器清单位于 [`baseline/java_api_inventory.json`](baseline/java_api_inventory.json)。
>
> **文档版本**：v1.0.0
> **最后更新**：2026-07-29
> **Java 基线**：Thymeleaf `3.1.5.RELEASE`，提交 `10f9dd2eb8cbd98515ce14b149d115e0287d0add`
> **Rust 开始基线**：提交 `d5c07ae8408146769ace4433366264f9259ddabc`

## 1. 完整 API 清单

[`export_java_api_inventory.py`](../../scripts/export_java_api_inventory.py) 曾从固定上游
CodeGraph 数据库导出以下基线。2026-07-30 已重新建立上游索引；JSON 仍作为固定提交
的冻结制品由 `migration-check` 消费。升级上游基线时必须从新索引重生成并比较 JSON：

| 维度 | 数量 |
|:---|---:|
| Java 主对象 | 491 |
| 内部/伴随类型 | 69 |
| Java 方法与构造器 | 4,291 |
| Java 参数 | 6,936 |
| 重载方法组 | 292 |
| public / protected / package / private 方法 | 3,215 / 182 / 475 / 419 |

当前索引的重生成命令：

```bash
scripts/export_java_api_inventory.py \
  --java-root /absolute/path/to/thymeleaf \
  --baseline 10f9dd2eb8cbd98515ce14b149d115e0287d0add \
  --output docs/migration/baseline/java_api_inventory.json
```

清单保留每个方法的：

- Java 全限定名、源码文件和行号；
- 可见性、static/abstract 标志；
- 原始签名、返回类型；
- 参数顺序、名称、类型、声明文本和 varargs 标志；
- 所属主对象与内部类型。

## 2. 状态定义

| 状态 | 含义 |
|:---|:---|
| `NOT_STARTED` | 尚无 Rust 对应入口 |
| `SKELETON` | 只有形态，没有完整逻辑；不计入完成 |
| `IMPLEMENTED_UNVERIFIED` | 存在真实逻辑，但没有 Java/Rust 差分证据 |
| `BEHAVIOR_VERIFIED` | Java Golden 与 Rust 合同测试通过 |
| `JAVA_ONLY_EXEMPT` | Java 运行时形态使用批准的 Rust 等价机制 |
| `PLANNED_BLOCKED` | 存在明确外部阻断 |
| `RUST_EXTENSION` | Rust 特有能力，不计入 Java 迁移分子 |

## 3. Foundation 垂直切片

本节覆盖 `TemplateMode` 与 `org.thymeleaf.exceptions` 的全部 37 个声明方法/构造器。差分证据：

- Java Oracle：[ThymeleafFoundationGolden.java](../../thymeleaf-test/tests/java/ThymeleafFoundationGolden.java)；
- 固定输出：[foundation_golden.txt](../../thymeleaf/tests/fixtures/foundation_golden.txt)；
- Rust 差分测试：[foundation_java_parity.rs](../../thymeleaf-test/tests/foundation_java_parity.rs)；
- Java Golden 记录数：86；
- Rust 单元测试：14；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：行、函数、区域均为 100%。

### 3.1 `TemplateMode`

| Java 方法/构造器 | Rust API | 参数映射 | 状态 |
|:---|:---|:---|:---|
| `TemplateMode(boolean html, boolean xml, boolean text)` | 六个 enum variant 的固定语义 | enum 内部构造，不暴露可变 flags | `BEHAVIOR_VERIFIED` |
| `boolean isMarkup()` | `is_markup(self) -> bool` | — | `BEHAVIOR_VERIFIED` |
| `boolean isText()` | `is_text(self) -> bool` | — | `BEHAVIOR_VERIFIED` |
| `boolean isCaseSensitive()` | `is_case_sensitive(self) -> bool` | — | `BEHAVIOR_VERIFIED` |
| `static TemplateMode parse(String mode)` | `parse(mode: Option<&str>) -> Result<TemplateMode, TemplateModeParseError>` | `mode` → `mode`；Java `null` → `None` | `BEHAVIOR_VERIFIED` |

`parse` 保留“只用 trim 判断空值、比较时不 trim、未知非空值警告并回退 HTML”的上游细节。

### 3.2 简单引擎异常

| Java 对象 | Java 构造器 | Rust API | 状态 |
|:---|:---|:---|:---|
| `AlreadyInitializedException` | `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `AlreadyInitializedException` | `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |
| `ConfigurationException` | `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `ConfigurationException` | `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |
| `CacheConfigurationException` | `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `CacheConfigurationException` | `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |
| `ParserInitializationException` | `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `ParserInitializationException` | `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |

所有 `message` 参数使用 `Option<String>` 保留 Java `null`；所有 `cause` 参数保存为 `Error + Send + Sync` 原因链。

### 3.3 `TemplateEngineException`

| Java 构造器 | Rust 等价 | 状态 |
|:---|:---|:---|
| `protected TemplateEngineException(String message)` | 抽象基类映射为同名 trait，由具体异常持有消息 | `BEHAVIOR_VERIFIED` |
| `protected TemplateEngineException(String message, Throwable cause)` | `TemplateEngineException: Error + Send + Sync`，具体异常暴露 source | `BEHAVIOR_VERIFIED` |

编译期合同测试验证全部七个 Java 子类型实现同名公共 trait。Rust 不复制 Java 异常继承内存布局。

### 3.4 `TemplateAssertionException`

| Java 方法/构造器 | Rust API | 参数映射 | 状态 |
|:---|:---|:---|:---|
| `(String assertionExpression, String templateName)` | `new(assertion_expression, template_name)` | `null` → `None`，格式化为 `null` | `BEHAVIOR_VERIFIED` |
| `(String assertionExpression, String templateName, int line, int col)` | `with_location(assertion_expression, template_name, line, col)` | 名称与顺序一致 | `BEHAVIOR_VERIFIED` |
| `private createMessage(...)` | `create_message(...)` | 私有格式化分支 | `BEHAVIOR_VERIFIED` |

### 3.5 `TemplateProcessingException`

| Java 方法/构造器 | Rust API | 状态 |
|:---|:---|:---|
| `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, Throwable cause)` | `with_template_and_cause(message, template_name, cause)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, int line, int col)` | `with_location(message, template_name, line, col)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, int line, int col, Throwable cause)` | `with_location_and_cause(message, template_name, line, col, cause)` | `BEHAVIOR_VERIFIED` |
| `String getTemplateName()` | `get_template_name()` | `BEHAVIOR_VERIFIED` |
| `boolean hasTemplateName()` | `has_template_name()` | `BEHAVIOR_VERIFIED` |
| `Integer getLine()` | `get_line()` | `BEHAVIOR_VERIFIED` |
| `Integer getCol()` | `get_col()` | `BEHAVIOR_VERIFIED` |
| `boolean hasLineAndCol()` | `has_line_and_col()` | `BEHAVIOR_VERIFIED` |
| `void setTemplateName(String templateName)` | `set_template_name(template_name)` | `BEHAVIOR_VERIFIED` |
| `void setLineAndCol(int line, int col)` | `set_line_and_col(line, col)` | `BEHAVIOR_VERIFIED` |
| `String getMessage()` | `get_message()` 与 `Display` | `BEHAVIOR_VERIFIED` |

位置为负数时转换为缺失值；只有模板名存在时才把位置加入消息。上游仅有列号时产生的 `" - , col N"` 格式也被差分测试固定。

### 3.6 `TemplateInputException`

| Java 构造器 | Rust API | 状态 |
|:---|:---|:---|
| `(String message)` | `new(message)` | `BEHAVIOR_VERIFIED` |
| `(String message, Throwable cause)` | `with_cause(message, cause)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, Throwable cause)` | `with_template_and_cause(message, template_name, cause)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, int line, int col)` | `with_location(message, template_name, line, col)` | `BEHAVIOR_VERIFIED` |
| `(String message, String templateName, int line, int col, Throwable cause)` | `with_location_and_cause(message, template_name, line, col, cause)` | `BEHAVIOR_VERIFIED` |

Java 继承得到的模板名、行列、消息和 mutation API 在 Rust 中通过组合完整转发。

### 3.7 `TemplateOutputException`

| Java 构造器 | Rust API | 状态 |
|:---|:---|:---|
| `(String message, String templateName, int line, int col, Throwable cause)` | `new(message, template_name, line, col, cause)` | `BEHAVIOR_VERIFIED` |

Java 继承得到的模板名、行列、消息、mutation 和 cause API 在 Rust 中通过组合完整转发。

## 4. `TemplateSpec` 垂直切片

本节覆盖 `org.thymeleaf.TemplateSpec` 的全部 18 个声明方法/构造器。差分证据：

- Java Oracle：[TemplateSpecGolden.java](../../thymeleaf-test/tests/java/TemplateSpecGolden.java)；
- 固定输出：[template_spec_golden.txt](../../thymeleaf/tests/fixtures/template_spec_golden.txt)；
- Rust 差分测试：[template_spec_java_parity.rs](../../thymeleaf-test/tests/template_spec_java_parity.rs)；
- Java Golden 记录数：289；
- Rust 单元测试：9；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当时 Rust 源码共 1,346 行、169 个函数、1,936 个区域，三项均为 100%。

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `TemplateSpec(String template, TemplateMode templateMode)` | `with_template_mode(template, template_mode)` | `null` → `Option`；必填模板以 `Result` 校验 | `BEHAVIOR_VERIFIED` |
| `TemplateSpec(String template, String outputContentType)` | `with_output_content_type(template, output_content_type)` | 保留原始 MIME 文本，参数只参与归一化推导 | `BEHAVIOR_VERIFIED` |
| `TemplateSpec(String template, Map<String,Object> templateResolutionAttributes)` | `with_resolution_attributes(template, template_resolution_attributes)` | 防御性复制；空映射归一为 `None` | `BEHAVIOR_VERIFIED` |
| `TemplateSpec(String template, Set<String> templateSelectors, TemplateMode templateMode, Map<String,Object> templateResolutionAttributes)` | `with_selectors_and_template_mode(...)` | 参数顺序与名称逐一 snake_case 对齐 | `BEHAVIOR_VERIFIED` |
| `TemplateSpec(String template, Set<String> templateSelectors, String outputContentType, Map<String,Object> templateResolutionAttributes)` | `with_selectors_and_output_content_type(...)` | MIME 可强制模式或启用 SSE | `BEHAVIOR_VERIFIED` |
| package `TemplateSpec(String template, Set<String> templateSelectors, TemplateMode templateMode, String outputContentType, Map<String,Object> templateResolutionAttributes)` | `pub(crate) try_new(...)` | 同时指定 mode/content type 返回精确消息错误 | `BEHAVIOR_VERIFIED` |
| `String getTemplate()` | `get_template() -> &str` | — | `BEHAVIOR_VERIFIED` |
| `boolean hasTemplateSelectors()` | `has_template_selectors() -> bool` | 空集合按无选择器处理 | `BEHAVIOR_VERIFIED` |
| `Set<String> getTemplateSelectors()` | `get_template_selectors() -> Option<&[String]>` | 多选择器按 Java UTF-16 字典序冻结 | `BEHAVIOR_VERIFIED` |
| `boolean hasTemplateMode()` | `has_template_mode() -> bool` | 显式模式或 MIME 推导模式 | `BEHAVIOR_VERIFIED` |
| `TemplateMode getTemplateMode()` | `get_template_mode() -> Option<TemplateMode>` | — | `BEHAVIOR_VERIFIED` |
| `boolean hasTemplateResolutionAttributes()` | `has_template_resolution_attributes() -> bool` | — | `BEHAVIOR_VERIFIED` |
| `Map<String,Object> getTemplateResolutionAttributes()` | `get_template_resolution_attributes() -> Option<&TemplateResolutionAttributes>` | 共享引用在编译期保证不可修改 | `BEHAVIOR_VERIFIED` |
| `String getOutputContentType()` | `get_output_content_type() -> Option<&str>` | 返回未改写的构造参数 | `BEHAVIOR_VERIFIED` |
| `boolean isOutputSSE()` | `is_output_sse() -> bool` | `text/event-stream` 大小写与参数语义对齐 | `BEHAVIOR_VERIFIED` |
| `boolean equals(Object o)` | `equals_java(other) -> Result<bool, TemplateSpecError>` + 安全 `PartialEq` | 保留上游空 `outputContentType` NPE 为类型化错误 | `BEHAVIOR_VERIFIED` |
| `int hashCode()` | Rust `Hash` | 保留相等对象哈希一致合同；不承诺 JVM 数值哈希相同 | `BEHAVIOR_VERIFIED` |
| `String toString()` | Rust `Display` | selector、mode、attributes、MIME 顺序与 120 UTF-16 单元截断语义对齐 | `BEHAVIOR_VERIFIED` |

`TemplateResolutionAttributeValue` 对 Java `Object` 做线程安全类型擦除，并在类型层面要求
`Eq + Hash + Display`，对应上游对缓存键属性值的 `equals/hashCode/toString` 要求。
`TemplateSelectorSet` 和 `TemplateResolutionAttributes` 允许在构造边界表达 Java `null`
元素、键和值；成功构造后的 `TemplateSpec` 不暴露可变集合。

Java 上游存在两个需要显式保留的异常边界：

1. MIME 拆分沿用 `StringTokenizer` 行为，连续和前导分号被忽略，仅由分号组成时映射
   `ArrayIndexOutOfBoundsException` 为 `TemplateSpecError::MalformedOutputContentType`；
2. `equals` 在接收者 `outputContentType == null` 且非对象自比较时会抛 NPE，映射为
   `TemplateSpecError::JavaEqualsNullOutputContentType`，Rust `PartialEq` 本身保持安全合同。

## 5. `Thymeleaf` 版本元数据切片

本节覆盖 `org.thymeleaf.Thymeleaf` 的全部 8 个声明方法/构造器。Java Oracle 使用
Maven Central 3.1.5.RELEASE 正式制品中已经过滤的 `thymeleaf.properties`，避免把
源码树的 `@pom.version@`、`@timestamp@` 占位符误判为运行时值。

- Java Oracle：[ThymeleafVersionGolden.java](../../thymeleaf-test/tests/java/ThymeleafVersionGolden.java)；
- 固定输出：[thymeleaf_version_golden.txt](../../thymeleaf/tests/fixtures/thymeleaf_version_golden.txt)；
- Rust 差分测试：[thymeleaf_java_parity.rs](../../thymeleaf-test/tests/thymeleaf_java_parity.rs)；
- Java Golden 记录数：8；
- Rust 单元测试：1；
- Java/Rust 差分测试：1；
- 正式制品元数据：版本 `3.1.5.RELEASE`，构建时间 `2026-04-21T20:38:36+0000`；
- `cargo-llvm-cov`：当前 Rust 源码共 1,346 行、169 个函数、1,936 个区域，三项均为 100%。

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| private `Thymeleaf()` | 私有字段阻止外部构造 | Java 工具类不可实例化合同 | `BEHAVIOR_VERIFIED` |
| `static String getVersion()` | `get_version() -> &'static str` | 正式制品版本常量 | `BEHAVIOR_VERIFIED` |
| `static String getBuildTimestamp()` | `get_build_timestamp() -> Option<&'static str>` | `Option` 保留 Java 可空返回合同 | `BEHAVIOR_VERIFIED` |
| `static int getVersionMajor()` | `get_version_major() -> i32` | `3` | `BEHAVIOR_VERIFIED` |
| `static int getVersionMinor()` | `get_version_minor() -> i32` | `1` | `BEHAVIOR_VERIFIED` |
| `static int getVersionPatch()` | `get_version_patch() -> i32` | `5` | `BEHAVIOR_VERIFIED` |
| `static String getVersionQualifier()` | `get_version_qualifier() -> Option<&'static str>` | `RELEASE`，同时保留可空类型 | `BEHAVIOR_VERIFIED` |
| `static boolean isVersionStableRelease()` | `is_version_stable_release() -> bool` | `RELEASE` 稳定版本判定 | `BEHAVIOR_VERIFIED` |

Java 通过 ClassLoader 和属性文件在运行时读取元数据；Rust 发布制品把同一固定上游制品
的值编译进只读常量。这是类加载机制的等价迁移，不改变正式发布场景的可观察返回值。

## 6. 方言基础与配置切片

本节覆盖 `IDialect`、`AbstractDialect` 与 `DialectConfiguration` 的全部 8 个声明方法/
构造器。CodeGraph 已确认 `DialectConfiguration` 只持有 `IDialect`，而
`AbstractDialect` 是其最小可执行基础实现，因此三者构成依赖闭合切片。

- Java Oracle：[DialectConfigurationGolden.java](../../thymeleaf-test/tests/java/DialectConfigurationGolden.java)；
- 固定输出：[dialect_configuration_golden.txt](../../thymeleaf/tests/fixtures/dialect_configuration_golden.txt)；
- Rust 差分测试：[dialect_configuration_java_parity.rs](../../thymeleaf-test/tests/dialect_configuration_java_parity.rs)；
- Java Golden 记录数：23；
- Rust 单元测试：6；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当时 Rust 源码共 1,439 行、186 个函数、2,098 个区域，三项均为 100%。

### 6.1 `IDialect`

| Java 方法 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `String getName()` | `get_name(&self) -> Option<&str>` | Java 接口未禁止自定义实现返回 `null`；`Option` 原样保留 | `BEHAVIOR_VERIFIED` |

Rust trait 增加 `Send + Sync` 作为并发渲染安全边界；该约束不改变名称返回值，但会在
编译期拒绝不能跨线程共享的方言实现。

### 6.2 `AbstractDialect`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| protected `AbstractDialect(String name)` | `new(name: Option<&str>) -> Result<Self, AbstractDialectError>` | `name` → `name`；空串允许，`null` 保留精确错误消息 | `BEHAVIOR_VERIFIED` |
| `String getName()` | inherent `get_name() -> &str` + `IDialect::get_name() -> Option<&str>` | 基础实现的名称恒非空 | `BEHAVIOR_VERIFIED` |

Java 的 protected 构造器用于继承；Rust 没有类继承，因此构造器公开给具体方言做组合，
并由 `IDialect` trait 保留动态分派边界。

### 6.3 `DialectConfiguration`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `DialectConfiguration(IDialect dialect)` | `new(dialect: Option<Arc<dyn IDialect>>)` | 未指定前缀；`dialect == null` 返回类型化错误 | `BEHAVIOR_VERIFIED` |
| `DialectConfiguration(String prefix, IDialect dialect)` | `with_prefix(prefix: Option<&str>, dialect: Option<Arc<dyn IDialect>>)` | `null`、空串、非空前缀逐项保留 | `BEHAVIOR_VERIFIED` |
| `IDialect getDialect()` | `get_dialect() -> &dyn IDialect` | 返回构造时同一动态实例，不创建副本 | `BEHAVIOR_VERIFIED` |
| `String getPrefix()` | `get_prefix() -> Option<&str>` | Java `null` → `None` | `BEHAVIOR_VERIFIED` |
| `boolean isPrefixSpecified()` | `is_prefix_specified() -> bool` | 一参数构造为 false；二参数构造即使 prefix 为 null 也为 true | `BEHAVIOR_VERIFIED` |

方言使用 `Arc<dyn IDialect>` 保存共享身份并支持并发读取；Golden 通过指针同一性验证
`get_dialect` 返回原实例。`DialectConfigurationError` 与
`AbstractDialectError` 是 Rust 类型化错误扩展，不计入 Java 对象分子。

## 7. 缓存条目有效性切片

本节覆盖 `ICacheEntryValidity`、`AlwaysValidCacheEntryValidity`、
`NonCacheableCacheEntryValidity` 与 `TTLCacheEntryValidity` 的全部 12 个声明方法/
构造器。CodeGraph 已确认三种实现只依赖基础接口，Resolver 和 Template Cache 通过
该接口消费它们，因此这 4 个对象构成依赖闭合的 S2 起始切片。

- Java Oracle：[CacheEntryValidityGolden.java](../../thymeleaf-test/tests/java/CacheEntryValidityGolden.java)；
- 固定输出：[cache_entry_validity_golden.txt](../../thymeleaf/tests/fixtures/cache_entry_validity_golden.txt)；
- Rust 差分测试：[cache_entry_validity_java_parity.rs](../../thymeleaf-test/tests/cache_entry_validity_java_parity.rs)；
- Java Golden 记录数：31；
- Rust 单元测试：9；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当前 Rust 源码共 1,595 行、217 个函数、2,327 个区域，三项均为 100%。

### 7.1 `ICacheEntryValidity`

| Java 方法 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `boolean isCacheable()` | `is_cacheable(&self) -> bool` | 决定解析结果能否进入缓存 | `BEHAVIOR_VERIFIED` |
| `boolean isCacheStillValid()` | `is_cache_still_valid(&self) -> bool` | 仅在 cacheable 后检查；false 要求淘汰并重新解析 | `BEHAVIOR_VERIFIED` |

Rust trait 增加 `Send + Sync`，保证模板缓存可在线程间共享有效性策略。

### 7.2 `AlwaysValidCacheEntryValidity`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `AlwaysValidCacheEntryValidity()` | `new() -> Self` | 公开构造器创建独立对象身份 | `BEHAVIOR_VERIFIED` |
| `boolean isCacheable()` | `is_cacheable() -> bool` | 恒为 true | `BEHAVIOR_VERIFIED` |
| `boolean isCacheStillValid()` | `is_cache_still_valid() -> bool` | 恒为 true，只由 LRU 等外部策略淘汰 | `BEHAVIOR_VERIFIED` |

Java `INSTANCE` 映射为 `AlwaysValidCacheEntryValidity::INSTANCE: &'static Self`。Golden
同时验证单例引用身份稳定、公开构造对象与单例及彼此身份不同。

### 7.3 `NonCacheableCacheEntryValidity`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `NonCacheableCacheEntryValidity()` | `new() -> Self` | 公开构造器创建独立对象身份 | `BEHAVIOR_VERIFIED` |
| `boolean isCacheable()` | `is_cacheable() -> bool` | 恒为 false | `BEHAVIOR_VERIFIED` |
| `boolean isCacheStillValid()` | `is_cache_still_valid() -> bool` | 虽按合同不会调用，公开行为仍恒为 false | `BEHAVIOR_VERIFIED` |

Java `INSTANCE` 同样映射为稳定的 `&'static Self`，不把 Java Object 身份错误地折叠为
Rust 值相等。

### 7.4 `TTLCacheEntryValidity`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `TTLCacheEntryValidity(long cacheTTLMs)` | `new(cache_ttl_ms: i64) -> Self` | `cacheTTLMs` → `cache_ttl_ms`；不验证正数 | `BEHAVIOR_VERIFIED` |
| `long getCacheTTLMs()` | `get_cache_ttl_ms() -> i64` | 原值返回，包括零、负数及极值 | `BEHAVIOR_VERIFIED` |
| `boolean isCacheable()` | `is_cacheable() -> bool` | 任意 TTL 均恒为 true | `BEHAVIOR_VERIFIED` |
| `boolean isCacheStillValid()` | `is_cache_still_valid() -> bool` | `now < creation + ttl`，严格边界、墙上时钟、`long` 环绕加法 | `BEHAVIOR_VERIFIED` |

Rust 明确使用 `SystemTime` 的 Unix 毫秒与 `i64::wrapping_add`。没有改用单调时钟、
饱和加法或自动拒绝非正 TTL；特别是 `Long.MAX_VALUE` 在当前正纪元时间上相加发生
环绕并立即失效，这一反直觉上游行为已经进入 Java Golden。

## 8. 缓存键切片

本节覆盖 `ExpressionCacheKey` 与 `TemplateCacheKey` 的全部 21 个声明方法/构造器。
CodeGraph 已确认二者只依赖已经迁移的 `TemplateMode`、selector 集合和模板解析属性，
因此无需以 STUB 提前引入 Cache Manager 或 Resolver。

- Java Oracle：[CacheKeyGolden.java](../../thymeleaf-test/tests/java/CacheKeyGolden.java)；
- 固定输出：[cache_key_golden.txt](../../thymeleaf/tests/fixtures/cache_key_golden.txt)；
- Rust 差分测试：[cache_key_java_parity.rs](../../thymeleaf-test/tests/cache_key_java_parity.rs)；
- Java Golden 记录数：58；
- Rust 单元测试：9；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当前 Rust 源码共 2,017 行、264 个函数、3,011 个区域，三项均为 100%。

### 8.1 `ExpressionCacheKey`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `ExpressionCacheKey(String type, String expression0)` | `new(expression_type: Option<&str>, expression0: Option<&str>) -> Result<Self, ExpressionCacheKeyError>` | `type` → `expression_type`；两项均拒绝 null，空字符串允许 | `BEHAVIOR_VERIFIED` |
| `ExpressionCacheKey(String type, String expression0, String expression1)` | `with_expression1(expression_type, expression0, expression1) -> Result<Self, ExpressionCacheKeyError>` | `expression1` 保留 null/空串区别 | `BEHAVIOR_VERIFIED` |
| `String getType()` | `get_type() -> &str` | 返回构造时类型 | `BEHAVIOR_VERIFIED` |
| `String getExpression0()` | `get_expression0() -> &str` | 返回第一表达式 | `BEHAVIOR_VERIFIED` |
| `String getExpression1()` | `get_expression1() -> Option<&str>` | Java null → `None` | `BEHAVIOR_VERIFIED` |
| `boolean equals(Object o)` | `PartialEq::eq` | 身份快路径、预计算 hash 快速拒绝及三个字段比较顺序一致；异型/null 由 Rust 类型系统排除 | `BEHAVIOR_VERIFIED` |
| `int hashCode()` | `hash_code() -> i32` + `Hash` | UTF-16 代码单元和 Java `int` 环绕乘加，数值与 Java 完全相同 | `BEHAVIOR_VERIFIED` |
| `int computeHashCode()`（private） | `compute_hash_code(...) -> i32`（private） | 构造时计算一次并缓存 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `Display` | `type\|expression0[\|expression1]`，不做额外转义 | `BEHAVIOR_VERIFIED` |

`ExpressionCacheKeyError` 是 Rust 类型化错误扩展，错误文本严格保留上游
`Type cannot be null` 与 `Expression cannot be null`。Golden 还固定了 UTF-16 补充字符
hash、`Aa`/`BB` 哈希碰撞但不相等，以及 null/空串边界。

### 8.2 `TemplateCacheKey`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `TemplateCacheKey(String ownerTemplate, String template, Set<String> templateSelectors, int lineOffset, int colOffset, TemplateMode templateMode, Map<String,Object> templateResolutionAttributes)` | `new(owner_template, template, template_selectors, line_offset, col_offset, template_mode, template_resolution_attributes) -> Result<Self, TemplateCacheKeyError>` | 唯一必填项为 `template`；其余 null、空集合、偏移极值逐项保留 | `BEHAVIOR_VERIFIED` |
| `String getOwnerTemplate()` | `get_owner_template() -> Option<&str>` | Java null → `None` | `BEHAVIOR_VERIFIED` |
| `String getTemplate()` | `get_template() -> &str` | 空字符串合法 | `BEHAVIOR_VERIFIED` |
| `Set<String> getTemplateSelectors()` | `get_template_selectors() -> Option<&TemplateSelectorSet>` | `Arc` 保存构造时同一只读集合身份；null 与空集合不同 | `BEHAVIOR_VERIFIED` |
| `int getLineOffset()` | `get_line_offset() -> i32` | 精确保留正负偏移与极值 | `BEHAVIOR_VERIFIED` |
| `int getColOffset()` | `get_col_offset() -> i32` | 精确保留正负偏移与极值 | `BEHAVIOR_VERIFIED` |
| `TemplateMode getTemplateMode()` | `get_template_mode() -> Option<TemplateMode>` | Java null → `None` | `BEHAVIOR_VERIFIED` |
| `Map<String,Object> getTemplateResolutionAttributes()` | `get_template_resolution_attributes() -> Option<&TemplateResolutionAttributes>` | `Arc` 保存同一只读 Map 身份；null 与空 Map 不同 | `BEHAVIOR_VERIFIED` |
| `boolean equals(Object o)` | `PartialEq::eq` | 身份/hash 快路径后依次比较 offsets、owner、template、selectors、mode、attributes | `BEHAVIOR_VERIFIED` |
| `int hashCode()` | `Hash` | 保留相等对象哈希相同及构造时缓存；不承诺 JVM 身份哈希的跨运行时数值 | `BEHAVIOR_VERIFIED` |
| `int computeHashCode()`（private） | `compute_hash_code(&self) -> u64`（private） | 所有七个字段参与；Map 采用与迭代顺序无关的安全哈希 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `Display` | 模板名 loggify、owner/offset、Java UTF-16 selector 顺序、mode、attributes 形状一致 | `BEHAVIOR_VERIFIED` |

Java 的 `TemplateMode.hashCode()` 和普通 `Object.hashCode()` 含 JVM 进程身份信息，因此
其数值本身不是跨进程协议。Rust 保留的是 `equals`/`hashCode` 合同、字段参与集合和构造时
缓存行为；Golden 对照相等对象的 hash 等价关系，而不伪造不可移植的 JVM 数值。
`TemplateCacheKeyError` 是 Rust 类型化错误扩展。

## 9. 通用缓存合同切片

本节覆盖 `ICache` 与 `ICacheEntryValidityChecker` 的全部 7 个声明方法。这两个泛型
接口构成独立依赖闭包；具体 `StandardCache`、Cache Manager 和
`StandardParsedTemplateEntryValidator` 留待其各自依赖闭合后迁移。

- Java Oracle：[CacheContractGolden.java](../../thymeleaf-test/tests/java/CacheContractGolden.java)；
- 固定输出：[cache_contract_golden.txt](../../thymeleaf/tests/fixtures/cache_contract_golden.txt)；
- Rust 差分测试：[cache_contract_java_parity.rs](../../thymeleaf-test/tests/cache_contract_java_parity.rs)；
- Java Golden 记录数：14；
- Rust 单元测试：3；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当前 Rust 源码共 2,140 行、278 个函数、3,212 个区域，三项均为 100%。

### 9.1 `ICache<K,V>`

| Java 方法 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `void put(K key, V value)` | `put(&self, key: K, value: Arc<V>)` | `Arc<V>` 对齐 Java 对象共享引用身份；`V: ?Sized` 支持 `dyn Any` 异构表达式制品 | `BEHAVIOR_VERIFIED` |
| `V get(K key)` | `get(&self, key: &K) -> Option<Arc<V>>` | Java miss/null → `None`；命中返回同一共享值身份 | `BEHAVIOR_VERIFIED` |
| `V get(K key, ICacheEntryValidityChecker<? super K,? super V> validityChecker)` | `get_with_validity_checker(&self, key: &K, validity_checker: &dyn ICacheEntryValidityChecker<K,V>) -> Option<Arc<V>>` | 本次 checker 覆盖默认设置；false 时删除并返回 `None` | `BEHAVIOR_VERIFIED` |
| `void clear()` | `clear(&self)` | 清除全部缓存条目 | `BEHAVIOR_VERIFIED` |
| `void clearKey(K key)` | `clear_key(&self, key: &K)` | 删除单键；不存在时幂等 | `BEHAVIOR_VERIFIED` |
| `Set<K> keySet()` | `key_set(&self) -> HashSet<K>` | 返回键快照，允许包含尚未惰性清除的失效键 | `BEHAVIOR_VERIFIED` |

### 9.2 `ICacheEntryValidityChecker<K,V>`

| Java 方法 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `boolean checkIsValueStillValid(K key, V value, long entryCreationTimestamp)` | `check_is_value_still_valid(&self, key: &K, value: &V, entry_creation_timestamp: i64) -> bool` | 三个参数原样传递；false 要求调用方删除条目 | `BEHAVIOR_VERIFIED` |

Java `Serializable` 是 JVM 对象序列化标记，并未定义跨语言线格式。Rust trait 使用
`Send + Sync` 保证检查器可并发共享；需要持久化的具体实现必须另行定义显式 serde
协议，不能通过伪造 trait object 序列化来声称等价。

### 9.3 `StandardCache<K,V>`

`StandardCache` 及其内部 `CacheDataContainer`、`CacheEntry` 按“一主对象一文件”
共同实现在 `src/cache/standard_cache.rs`。Java Oracle 固定输出 37 条记录，覆盖
构造校验顺序、Java whitespace、任意负数无界容量、put-if-absent、FIFO、checker、
计数器和比例。

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `StandardCache(String name, boolean useSoftReferences, int initialCapacity, Logger logger)` | `new(name, use_soft_references, initial_capacity)` | 默认 maxSize=-1、checker=null、counter=false；SLF4J 由全局 `tracing` subscriber 替代 | `IMPLEMENTED_UNVERIFIED` |
| `StandardCache(String name, boolean useSoftReferences, int initialCapacity, ICacheEntryValidityChecker entryValidityChecker, Logger logger)` | `with_validity_checker(name, use_soft_references, initial_capacity, entry_validity_checker)` | `Arc<dyn ...>` 保存默认 checker 动态身份 | `IMPLEMENTED_UNVERIFIED` |
| `StandardCache(String name, boolean useSoftReferences, int initialCapacity, int maxSize, Logger logger)` | `with_max_size(name, use_soft_references, initial_capacity, max_size)` | 仅 0 非法；任意负数均保持 Java 无界行为 | `IMPLEMENTED_UNVERIFIED` |
| `StandardCache(String name, boolean useSoftReferences, int initialCapacity, int maxSize, ICacheEntryValidityChecker entryValidityChecker, Logger logger)` | `with_max_size_and_validity_checker(...)` | 参数顺序与 Java 语义一致 | `IMPLEMENTED_UNVERIFIED` |
| 完整七参数构造器 | `with_options(name, use_soft_references, initial_capacity, max_size, entry_validity_checker, enable_counters, trace_execution)` | `trace_execution` 对应启用 trace 的非空 logger，并强制开启计数器；校验顺序和消息精确对齐 | `IMPLEMENTED_UNVERIFIED` |
| `void put(K key, V value)` | `ICache::put(&self, key, Arc<V>)` | 先计数，再 put-if-absent；重复键不替换值；新键按插入顺序 FIFO 淘汰 | `IMPLEMENTED_UNVERIFIED` |
| `V get(K key)` | `ICache::get(&self, key)` | 使用默认 checker；命中保留 `Arc` 身份；失效或已牺牲软引用时惰性删除 | `IMPLEMENTED_UNVERIFIED` |
| `V get(K key, ICacheEntryValidityChecker validityChecker)` | `ICache::get_with_validity_checker(&self, key, validity_checker)` | 本次 checker 覆盖默认 checker，并接收创建时间戳 | `IMPLEMENTED_UNVERIFIED` |
| `Set<K> keySet()` | `ICache::key_set(&self) -> HashSet<K>` | 返回快照；仍可能包含尚未 get 清理的失效条目 | `BEHAVIOR_VERIFIED` |
| `void clear()` | `ICache::clear(&self)` | 只清 map，不重置 FIFO 指针/槽位，也不清计数器 | `BEHAVIOR_VERIFIED` |
| `void clearKey(K key)` | `ICache::clear_key(&self, key)` | 删除 map 条目并清除首个对应 FIFO 槽；缺失键幂等 | `BEHAVIOR_VERIFIED` |
| `String getName()` | `get_name() -> &str` | 返回构造名称；Java empty/whitespace 判定精确迁移 | `BEHAVIOR_VERIFIED` |
| `boolean hasMaxSize()` | `has_max_size() -> bool` | 仅 maxSize > 0 为 true | `BEHAVIOR_VERIFIED` |
| `int getMaxSize()` | `get_max_size() -> i32` | 保留原始负数，不归一化为 -1 | `BEHAVIOR_VERIFIED` |
| `boolean getUseSoftReferences()` | `get_use_soft_references() -> bool` | 返回原始配置标志 | `BEHAVIOR_VERIFIED` |
| `int size()` | `size() -> usize` | 返回当前 map 条目数 | `BEHAVIOR_VERIFIED` |
| `long getPutCount()` | `get_put_count() -> i64` | 禁用计数器时始终为 0；重复 put 也计数 | `BEHAVIOR_VERIFIED` |
| `long getGetCount()` | `get_get_count() -> i64` | 每次 get 均计数 | `BEHAVIOR_VERIFIED` |
| `long getHitCount()` | `get_hit_count() -> i64` | 仅有效值命中计数 | `BEHAVIOR_VERIFIED` |
| `long getMissCount()` | `get_miss_count() -> i64` | 缺失、checker 失效和软引用消失均计数 | `BEHAVIOR_VERIFIED` |
| `double getHitRatio()` | `get_hit_ratio() -> f64` | hit=0 或 get=0 时为 0，否则 hit/get | `BEHAVIOR_VERIFIED` |
| `double getMissRatio()` | `get_miss_ratio() -> f64` | 恒为 `1-hitRatio`，所以初始值为 1 | `BEHAVIOR_VERIFIED` |

内部 `CacheDataContainer#get/put/remove/keySet/clear/size` 均为同文件私有真实逻辑：
互斥锁提供比 Java `ConcurrentHashMap` 更强的串行化边界，但不改变可观察原子性；
`CacheEntry` 保存 `Weak<V>`、可牺牲强锚点和 Unix 毫秒创建时间。Rust 扩展
`sacrifice_soft_references()` 可确定性模拟 JVM GC 清理，不过 Rust 没有 JVM
内存压力自动通知，因此该主对象暂不能升级为 `BEHAVIOR_VERIFIED`。

## 10. 模板资源 SPI、字符串资源与文件资源切片

本节覆盖 `ITemplateResource`、`StringTemplateResource`、`FileTemplateResource` 与
`TemplateResourceUtils` 的全部 22 个声明方法/构造器。CodeGraph 已确认文件资源的
可观察行为由资源 SPI、路径工具、文件系统和字符集解码共同决定，不需要提前引入
Resolver、Parser 或 Engine。

- Java Oracle：[TemplateResourceGolden.java](../../thymeleaf-test/tests/java/TemplateResourceGolden.java)；
- 固定输出：[template_resource_golden.txt](../../thymeleaf/tests/fixtures/template_resource_golden.txt)；
- Rust 差分测试：[template_resource_java_parity.rs](../../thymeleaf-test/tests/template_resource_java_parity.rs)；
- Java Golden 记录数：144；
- Rust 单元测试：26；
- Java/Rust 差分测试：1；
- `cargo-llvm-cov`：当前 Rust 源码共 3,413 行、401 个函数、5,238 个区域，三项均为 100%。

### 10.1 `ITemplateResource`

| Java 方法 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `String getDescription()` | `get_description(&self) -> String` | 永不为 null；仅用于诊断，不承诺唯一性 | `BEHAVIOR_VERIFIED` |
| `String getBaseName()` | `get_base_name(&self) -> Option<String>` | 无法派生时 Java null → `None` | `BEHAVIOR_VERIFIED` |
| `boolean exists()` | `exists(&self) -> bool` | 资源对象存在不代表底层资源存在；允许真实 I/O 成本 | `BEHAVIOR_VERIFIED` |
| `Reader reader() throws IOException` | `reader(&self) -> io::Result<Box<dyn Read>>` | 每次返回从资源起点开始的新读取器；字符流映射为 UTF-8 `Read` | `BEHAVIOR_VERIFIED` |
| `ITemplateResource relative(String relativeLocation)` | `relative(&self, relative_location: Option<&str>) -> Result<Box<dyn ITemplateResource>, TemplateResourceError>` | null 可观察；保留参数错误与模板输入错误类别 | `BEHAVIOR_VERIFIED` |

上游 JavaDoc 明确说明具体实现未必线程安全，因此 Rust trait 没有人为增加
`Send + Sync`。这与缓存合同需要跨线程共享的约束不同。

### 10.2 `StringTemplateResource`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `StringTemplateResource(String resource)` | `new(resource: Option<&str>) -> Result<Self, TemplateResourceError>` | 实际只拒绝 null；上游错误文本虽含 “or empty”，空字符串仍合法 | `BEHAVIOR_VERIFIED` |
| `String getDescription()` | `get_description() -> String` | 返回完整模板内容，包括 CRLF、NUL 和 Unicode | `BEHAVIOR_VERIFIED` |
| `String getBaseName()` | `get_base_name() -> Option<String>` | 字符串资源恒为 `None` | `BEHAVIOR_VERIFIED` |
| `Reader reader()` | `reader() -> io::Result<Box<dyn Read>>` | 每次返回独立、从头读取的内存 reader | `BEHAVIOR_VERIFIED` |
| `ITemplateResource relative(String relativeLocation)` | `relative(relative_location) -> Result<..., TemplateResourceError>` | 不检查参数；null、空串和普通路径均抛同一精确 `TemplateInputException` | `BEHAVIOR_VERIFIED` |
| `boolean exists()` | `exists() -> bool` | 包括空模板在内恒为 true | `BEHAVIOR_VERIFIED` |

`TemplateResourceError` 是 Rust 类型化错误扩展，区分 Java
`IllegalArgumentException` 与 `TemplateInputException`，不计入 Java 对象迁移分子。

### 10.3 `FileTemplateResource`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `FileTemplateResource(String path, String characterEncoding)` | `new(path: Option<&str>, character_encoding: Option<&str>) -> Result<Self, TemplateResourceError>` | `path` → `path`；null、空串和 Java 空白拒绝；编码保留 null/空白区别直到 reader 创建 | `BEHAVIOR_VERIFIED` |
| `FileTemplateResource(File file, String characterEncoding)` | `from_file(file: Option<&Path>, character_encoding: Option<&str>) -> Result<Self, TemplateResourceError>` | 只拒绝 null `File`；空路径和空白路径合法；非 UTF-8 Rust 路径属于 Java 域外扩展错误 | `BEHAVIOR_VERIFIED` |
| `String getDescription()` | `get_description() -> String` | 使用文件对象的词法绝对路径；不把 `.`/`..` canonicalize；重复平台分隔符按 Java `File` 归一 | `BEHAVIOR_VERIFIED` |
| `String getBaseName()` | `get_base_name() -> Option<String>` | 基于 Thymeleaf 清理后的逻辑路径，而不是 canonical 文件路径 | `BEHAVIOR_VERIFIED` |
| `Reader reader()` | `reader() -> io::Result<Box<dyn Read>>` | 先打开文件再解析 charset；流式替换式解码为 UTF-8；UTF BOM、ASCII、Latin-1、Windows-1252 和常用多字节编码已与 Java Oracle 对照 | `BEHAVIOR_VERIFIED` |
| `ITemplateResource relative(String relativeLocation)` | `relative(relative_location) -> Result<..., TemplateResourceError>` | null/空/Java 空白返回精确参数错误；基于清理路径所在目录组合，继承原字符集 | `BEHAVIOR_VERIFIED` |
| `boolean exists()` | `exists() -> bool` | 直接查询原文件对象；资源对象构造不预检文件存在性 | `BEHAVIOR_VERIFIED` |

Java `Reader` 是字符流，Rust `Read` 是字节流，因此实现先按 Java charset 规则解码，
再只输出合法 UTF-8。Java 18 及以上的默认 charset 为 UTF-8；Rust 迁移基线以同一
JDK 21 Oracle 固定该无显式编码行为。显式 `UTF-8` 不吞 BOM，通用 `UTF-16` 吞 BOM
并在无 BOM 时使用 big-endian，`UTF-16BE/LE` 则保留 BOM 为 `U+FEFF`。

### 10.4 `TemplateResourceUtils`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static String cleanPath(String path)` | `clean_path(path: Option<&str>) -> Option<String>` | Windows 分隔符转 `/`；逆序处理 `.`/`..` 和重复 `/`；不访问文件系统；null 原样返回 | `BEHAVIOR_VERIFIED` |
| `static String computeRelativeLocation(String location, String relativeLocation)` | `compute_relative_location(location, relative_location) -> String` | 只替换当前路径最后一段，不提前 clean，保留 `../` | `BEHAVIOR_VERIFIED` |
| `static String computeBaseName(String path)` | `compute_base_name(path: Option<&str>) -> Option<String>` | 去一个尾 `/`，按最后 `/` 和 `.` 边界截取；保留 `.hidden` 的上游边界行为 | `BEHAVIOR_VERIFIED` |
| private `TemplateResourceUtils()` | crate-private unit struct | 外部不可构造；仅作为同模块静态路径工具 | `BEHAVIOR_VERIFIED` |

路径工具刻意没有改用 `Path::canonicalize`：canonicalize 会要求资源存在、解析符号链接
并抹掉尚待相对组合的 `..`，会直接改变上游语义。上游对完全抵消为空的相对路径会在
最终 `deleteCharAt(0)` 抛出未检查异常，Rust 测试也保留这一边界失败。

### 10.5 `UrlTemplateResource`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `UrlTemplateResource(String path, String characterEncoding)` | `new(path: Option<&str>, character_encoding: Option<&str>) -> Result<Self, TemplateResourceError>` | null、空串和 Java 空白返回精确参数错误；URL 语法错误保留独立 `MalformedUrl` 类别和原因链 | `IMPLEMENTED_UNVERIFIED` |
| `UrlTemplateResource(URL url, String characterEncoding)` | `from_url(url: Option<Url>, character_encoding: Option<&str>) -> Result<Self, TemplateResourceError>` | null URL 返回精确参数错误；描述取已解析 URL 的外部形式 | `IMPLEMENTED_UNVERIFIED` |
| `String getDescription()` | `get_description() -> String` | 字符串构造保留输入外部形式，包括尚未归一化的 `.`/`..` 路径片段 | `IMPLEMENTED_UNVERIFIED` |
| `String getBaseName()` | `get_base_name() -> Option<String>` | 对 URL path 执行 Thymeleaf `cleanPath` 后计算 base name，忽略 query/fragment | `IMPLEMENTED_UNVERIFIED` |
| `Reader reader()` | `reader() -> io::Result<Box<dyn Read>>` | 先打开 URL 输入流，再解析显式 charset；file 与 HTTP/HTTPS 每次创建新 reader，并复用 Java 字符集兼容层 | `IMPLEMENTED_UNVERIFIED` |
| private `InputStream inputStream()` | private `input_stream()` | file 使用本地文件句柄；HTTP/HTTPS 使用阻塞传输且不读取代理环境；404 映射 NotFound | `IMPLEMENTED_UNVERIFIED` |
| `ITemplateResource relative(String relativeLocation)` | `relative(relative_location) -> Result<..., TemplateResourceError>` | 拒绝 null/空/Java 空白；去掉一个前导 `/`；继承 charset；malformed URL 包装为带原因的 `TemplateInputException` | `IMPLEMENTED_UNVERIFIED` |
| `boolean exists()` | `exists() -> bool` | file 查询本地路径；HTTP/HTTPS 发送 HEAD，200/404 和 Content-Length 兜底规则与 Java 对齐；I/O 失败返回 false | `IMPLEMENTED_UNVERIFIED` |
| private `URI toURI(URL url)` | `Url::to_file_path()` 内联映射 | file URL 转平台路径；非本地主机 authority 返回类型化 I/O 错误 | `IMPLEMENTED_UNVERIFIED` |

当前 74 条新增 Java Golden 记录已覆盖上游 URL 描述、相对解析与 base-name 向量，
并覆盖真实 file/HTTP I/O、HEAD 状态判断、字符集、404、重复 reader 和连接失败。
`cargo-llvm-cov` 对该实现的行、函数和 region 覆盖率均为 100%。

该对象仍不能提升为 `BEHAVIOR_VERIFIED`：Java `URL#openConnection()` 可通过
`URLStreamHandler` 支持 JAR、FTP、JNLP 和应用自定义协议，而当前 Rust 内建传输只提供
file、HTTP 与 HTTPS；HTTPS 目前只验证了协议分派及连接失败，没有与 Java TLS 服务做
成功路径差分。此外，`java.net.URL` 与 WHATWG URL 在少数语法边界上的解析规则仍需
系统差分。后续必须引入可注册协议 handler，并补齐 JAR/FTP/自定义协议及本地 TLS
Oracle，之后才能把九个入口整体升级为 `BEHAVIOR_VERIFIED`。

## 11. Template Resolver

### 11.1 `TemplateResolution`

CodeGraph 已确认该对象由 `AbstractTemplateResolver#resolveTemplate()` 创建，并由
`TemplateManager#buildTemplateData()` 消费。它是 Resolver 到 Parser/Cache 主链之间
传递资源、模式、解耦逻辑和缓存有效性的不可省略边界。

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `TemplateResolution(ITemplateResource templateResource, TemplateMode templateMode, ICacheEntryValidity validity)` | `new(template_resource, template_mode, validity)` | 三个接口对象都以 `Option` 保留 Java null 边界；两个布尔标志默认 false | `BEHAVIOR_VERIFIED` |
| `TemplateResolution(ITemplateResource templateResource, boolean templateResourceExistenceVerified, TemplateMode templateMode, boolean useDecoupledLogic, ICacheEntryValidity validity)` | `with_options(template_resource, template_resource_existence_verified, template_mode, use_decoupled_logic, validity)` | 参数名称与顺序 snake_case 对齐；按 resource、mode、validity 顺序返回精确校验错误 | `BEHAVIOR_VERIFIED` |
| `ITemplateResource getTemplateResource()` | `get_template_resource() -> &dyn ITemplateResource` | 返回构造时同一动态实例；对象存在不代表底层资源存在 | `BEHAVIOR_VERIFIED` |
| `TemplateMode getTemplateMode()` | `get_template_mode() -> TemplateMode` | Resolver 建议值，允许 Engine 后续用显式模式覆盖 | `BEHAVIOR_VERIFIED` |
| `boolean isTemplateResourceExistenceVerified()` | `is_template_resource_existence_verified() -> bool` | false 表示“未验证”，不表示“不存在” | `BEHAVIOR_VERIFIED` |
| `boolean getUseDecoupledLogic()` | `get_use_decoupled_logic() -> bool` | true 表示应检查可选解耦逻辑，不表示逻辑资源必然存在 | `BEHAVIOR_VERIFIED` |
| `ICacheEntryValidity getValidity()` | `get_validity() -> &dyn ICacheEntryValidity` | 返回构造时同一动态实例，决定能否缓存及何时失效 | `BEHAVIOR_VERIFIED` |

Rust 使用 `Rc<dyn ITemplateResource>` 保留上游“不得视为线程安全”的 JavaDoc 合同，
同时使用 `Arc<dyn ICacheEntryValidity>` 承接已迁移缓存合同的 `Send + Sync` 约束。
66 条 Java Golden 覆盖两种构造器、全部校验错误与顺序、六种模板模式、独立标志、
动态实例身份和有效性行为；新增实现的行、函数和 region 覆盖率均为 100%。

## 12. 通用工具基础

### 12.1 `NumberPointType`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| private `NumberPointType(String name)` | Rust enum variant 声明 | 五个成员及声明顺序保持 `POINT, COMMA, WHITESPACE, NONE, DEFAULT` | `BEHAVIOR_VERIFIED` |
| static `NumberPointType match(String name)` | `match_name(name: Option<&str>) -> Option<NumberPointType>` | Java null 和未知名称均为 `None`；严格区分大小写且不 trim | `BEHAVIOR_VERIFIED` |
| `String getName()` | `get_name(self) -> &'static str` | 返回构造时同一固定大写名称 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `Display::fmt` | 与 `getName()` 完全相同 | `BEHAVIOR_VERIFIED` |

### 12.2 `IdentityCounter<T>`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `IdentityCounter(int expectedMaxSize)` | `new(expected_max_size: i32) -> Result<Self, IdentityCounterError>` | 负数保留精确 `IllegalArgumentException` 消息；0 到 `i32::MAX` 均接受；容量只作为性能提示 | `BEHAVIOR_VERIFIED` |
| `void count(T object)` | `count(&mut self, object: Option<Rc<T>>)` | `Rc` 强引用对应 Java map 强键；相同身份重复记录不新增；`None` 对应可记录的 Java null | `BEHAVIOR_VERIFIED` |
| `boolean isAlreadyCounted(T object)` | `is_already_counted(&self, object: Option<&Rc<T>>) -> bool` | 使用 `Rc::ptr_eq` 而非值相等；同一 alias 命中，值相等但分配不同不命中 | `BEHAVIOR_VERIFIED` |

JavaDoc 明确声明 `IdentityCounter` 非线程安全。Rust 不添加锁，而以 `Rc` 使该对象自然
不满足 `Send`/`Sync`，避免把线程安全范围无意扩大。32 条固定 Java Golden 同时覆盖
枚举顺序、名称匹配、构造边界、引用 alias、值相等但身份不同及 null 身份。
`IdentityCounterError` 是 Rust 类型化错误扩展。

### 12.3 `Validate`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static void notNull(Object object, String message)` | `not_null(object: Option<&T>, message: Option<&str>) -> Result<(), ValidateError>` | `None` 对应 Java null；失败保留调用方可空 detail message | `BEHAVIOR_VERIFIED` |
| `static void notEmpty(String object, String message)` | `not_empty_str(object: Option<&str>, message: Option<&str>) -> Result<(), ValidateError>` | null、空串或全为 Java `Character.isWhitespace` 字符时失败；NBSP 等非 Java whitespace 保持有效 | `BEHAVIOR_VERIFIED` |
| `static void notEmpty(Collection<?> object, String message)` | `not_empty_collection(object: Option<&C>, message: Option<&str>) -> Result<(), ValidateError>` | 借用 `IntoIterator` 保留非消费校验；null 或零元素失败，不检查元素内容 | `BEHAVIOR_VERIFIED` |
| `static void notEmpty(Object[] object, String message)` | `not_empty_array(object: Option<&[T]>, message: Option<&str>) -> Result<(), ValidateError>` | null 或零长度 slice 失败，不检查数组元素 | `BEHAVIOR_VERIFIED` |
| `static void containsNoNulls(Iterable<?> collection, String message)` | `contains_no_nulls_iterable(collection: Option<&C>, message: Option<&str>) -> Result<(), ValidateError>` | null Iterable 保留增强 for 隐式 NPE 类别；按迭代顺序遇首个 null 元素短路为显式参数错误 | `BEHAVIOR_VERIFIED` |
| `static void containsNoEmpties(Iterable<String> collection, String message)` | `contains_no_empties(collection: Option<&C>, message: Option<&str>) -> Result<(), ValidateError>` | null Iterable 为隐式 NPE；逐元素复用字符串 `notEmpty`，保留首个失败及消息 | `BEHAVIOR_VERIFIED` |
| `static void containsNoNulls(Object[] array, String message)` | `contains_no_nulls_array(array: Option<&[Option<T>]>, message: Option<&str>) -> Result<(), ValidateError>` | null 数组保留隐式 NPE 类别；按索引顺序遇首个 null 元素短路 | `BEHAVIOR_VERIFIED` |
| `static void isTrue(boolean condition, String message)` | `is_true(condition: bool, message: Option<&str>) -> Result<(), ValidateError>` | false 抛显式参数错误，true 无副作用 | `BEHAVIOR_VERIFIED` |
| private `Validate()` | 单元结构体无公开构造状态，全部能力为关联函数 | 保留无状态工具类合同 | `BEHAVIOR_VERIFIED` |

`ValidateError::IllegalArgument` 对应 Java 显式
`IllegalArgumentException`，包括 detail message 为 null；`ValidateError::NullPointer`
仅对应三个遍历重载对 null 容器解引用产生的隐式 `NullPointerException`，不伪造随
JDK/Javac 变化的 helpful-NPE 文本。36 条固定 Java Golden 覆盖八个公开重载的成功、
失败、空白边界、null 容器、null 元素及可空消息；Rust 单元测试覆盖全部分支。
`StandardCache` 已复用该实现，避免维护第二套 Java whitespace 判断。
本切片完成后，`cargo-llvm-cov` 统计 4,266 行、503 个函数和 6,633 个区域，三项均为
100%。

### 12.4 `MapUtils`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static int size(Map<?,?> target)` | `size(target: Option<&HashMap<K,V,S>>) -> Result<i32, ValidateError>` | null target 保留精确参数错误；Rust 理论超大集合按 `Map#size` 合同饱和为 `i32::MAX` | `BEHAVIOR_VERIFIED` |
| `static boolean isEmpty(Map<?,?> target)` | `is_empty(target: Option<&HashMap<K,V,S>>) -> bool` | null 与空映射均为 true，不抛异常 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsKey(Map<? super X,?> target, X key)` | `contains_key(target, key: &Q) -> Result<bool, ValidateError>` | `Borrow<Q>` 保留借用键查询；`K = Option<T>` 表达 Java null 键 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsAllKeys(Map<? super X,?> target, X[] keys)` | `contains_all_keys_array(target, keys: Option<&[Q]>) -> Result<bool, ValidateError>` | 数组重载；按 target→keys 顺序校验，空数组为 true，重复请求不增加数量要求 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsAllKeys(Map<? super X,?> target, Collection<X> keys)` | `contains_all_keys_collection(target, keys: Option<I>) -> Result<bool, ValidateError>` | Collection 映射为借用迭代器；逐键短路，保留 null Collection 错误 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsValue(Map<?,? super X> target, X value)` | `contains_value(target, value: &Q) -> Result<bool, ValidateError>` | 按请求值到候选值方向比较；`V = Option<T>` 表达 Java null 值 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsAllValues(Map<?,? super X> target, X[] values)` | `contains_all_values_array(target, values: Option<&[Q]>) -> Result<bool, ValidateError>` | 数组重载；空数组为 true，重复请求值只要求至少一次匹配 | `BEHAVIOR_VERIFIED` |
| `static <X> boolean containsAllValues(Map<?,? super X> target, Collection<X> values)` | `contains_all_values_collection(target, values: Option<I>) -> Result<bool, ValidateError>` | Collection 映射为借用迭代器；按请求顺序逐项短路 | `BEHAVIOR_VERIFIED` |
| private `MapUtils()` | 单元结构体无公开构造状态，全部能力为关联函数 | 保留无状态工具类合同 | `BEHAVIOR_VERIFIED` |

Java `Map` 映射为核心当前采用的 `HashMap<K,V,S>`；查询允许自定义
`BuildHasher`，不依赖或暴露迭代顺序。40 条 Map/Object 固定 Java Golden 覆盖
`MapUtils` 全部重载、null 键值、空集合、重复请求、缺失项和每一条精确校验错误。
校验错误直接复用 `ValidateError`，没有新增重复错误类型。

### 12.5 `ObjectUtils`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static <T> T nullSafe(T target, T defaultValue)` | `null_safe(target: Option<T>, default_value: Option<T>) -> Option<T>` | target 与 default 独立可空；target 非空优先，否则原样返回 default；移动值而非克隆 | `BEHAVIOR_VERIFIED` |
| private `ObjectUtils()` | 单元结构体无公开构造状态，能力为关联函数 | 保留无状态工具类合同 | `BEHAVIOR_VERIFIED` |

Golden 使用独立 Java 对象与 Rust `Rc` 验证 target/default 两条路径返回同一引用身份，
并覆盖双方均为 null 的结果。该映射没有错误路径和 Rust 扩展对象。
本切片完成后，`cargo-llvm-cov` 统计 4,537 行、527 个函数和 6,991 个区域，三项均为
100%。

### 12.6 `PatternUtils`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static Pattern strPatternToPattern(String pattern)` | `str_pattern_to_pattern(pattern: Option<&str>) -> Result<StringPattern, PatternUtilsError>` | 严格按上游链式替换顺序转义 `.()[]?$+`，把 `*` 转为非贪婪片段，并保留 `\`、`^`、`{}`、`|` 的正则意义；`StringPattern` 暴露 Java 源表达式并执行全输入匹配 | `BEHAVIOR_VERIFIED` |
| private `PatternUtils()` | 单元结构体无公开实例化入口，能力为关联函数 | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |

Rust `regex` 默认 Unicode 字符类和换行规则不同，因此编译适配层将 Java 默认
`\d/\D/\w/\W/\s/\S`、水平/垂直空白、`\R` 和 dot 的五类行终止符显式转换，并用
`\A...\z` 包裹来保持 `Matcher#matches()` 的全输入语义；公开的 `as_str()` 仍返回
Java `Pattern#pattern()` 可观察文本。`PatternUtilsError` 区分 null 输入和语法错误，
并在语法错误中保存转换后的 Java 表达式。

### 12.7 `PatternSpec`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `PatternSpec()` | `new() -> PatternSpec` | 初始化公开字符串集合和内部编译集合为空 | `BEHAVIOR_VERIFIED` |
| `boolean isEmpty()` | `is_empty(&self) -> bool` | 只检查已编译模式；编译首项失败时可与非空字符串集合并存 | `BEHAVIOR_VERIFIED` |
| `Set<String> getPatterns()` | `get_patterns(&self) -> &IndexSet<Option<String>>` | 插入顺序、字符串去重和失败状态中的 null 元素均保留；共享借用对应不可修改视图 | `BEHAVIOR_VERIFIED` |
| `void setPatterns(Set<String> newPatterns)` | `set_patterns(&mut self, new_patterns: Option<&[Option<&str>]>) -> Result<(), PatternSpecError>` | null Set 清空；先复制全部字符串再依序编译，失败时保留完整字符串集合与成功编译前缀，不回滚 | `BEHAVIOR_VERIFIED` |
| `void addPattern(String pattern)` | `add_pattern(&mut self, pattern: Option<&str>) -> Result<(), PatternSpecError>` | 先执行精确非空校验，再加入字符串并编译；语法失败保留字符串；重复字符串仍追加独立编译实例 | `BEHAVIOR_VERIFIED` |
| `void clearPatterns()` | `clear_patterns(&mut self)` | 同时清空字符串和编译模式，重复调用幂等 | `BEHAVIOR_VERIFIED` |
| `boolean matches(String templateName)` | `matches(&self, template_name: Option<&str>) -> Result<bool, PatternSpecError>` | 按编译顺序短路；空规格匹配 null 返回 false，存在模式时 null 保留隐式 NPE 类别 | `BEHAVIOR_VERIFIED` |

`IndexSet` 对齐 `LinkedHashSet<String>` 的公开顺序和去重，独立 `Vec<StringPattern>`
则保留 Java `LinkedHashSet<Pattern>` 基于对象身份的行为：对同一字符串重复
`addPattern` 时公开集合仍为一项，但内部确有多个编译实例。79 条固定 Java Golden
覆盖表达式源文本、通配/引用/备选/量词/字符类、只读视图、null、清空、重复模式和失败后
状态；Rust 单元测试进一步验证无法从公开集合直接观察的编译实例数。
本切片完成后，`cargo-llvm-cov` 统计 4,873 行、565 个函数和 7,667 个区域，三项均为
100%。

### 12.8 `VersionUtils` / `VersionSpec`

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static VersionSpec parseVersion(String version)` | `parse_version(version: Option<&str>) -> VersionSpec` | null、Java ASCII trim 后为空或解析失败均返回 `UNKNOWN`；不附构建时间戳 | `BEHAVIOR_VERIFIED` |
| `static VersionSpec parseVersion(String version, String buildTimestamp)` | `parse_version_with_build_timestamp(version: Option<&str>, build_timestamp: Option<&str>) -> VersionSpec` | buildTimestamp 不裁剪、不校验，空串与 null 不同；版本解析失败仍保留时间戳 | `BEHAVIOR_VERIFIED` |
| private `int findEndOfNumericVersion(CharSequence sequence)` | private `find_end_of_numeric_version(sequence: &[u16]) -> Option<usize>` | 按 Java UTF-16 `charAt()` 扫描点号和 `Character.isDigit(char)`；字母是否吞掉分隔符的边界保持一致 | `BEHAVIOR_VERIFIED` |
| private `VersionUtils()` | 单元结构体无公开实例化入口 | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |
| private `VersionSpec(String buildTimestamp)` | private `VersionSpec::unknown(build_timestamp)` | unknown=true，三段数字为零，版本文本为 `UNKNOWN`；非 null 时间戳进入 fullVersion | `BEHAVIOR_VERIFIED` |
| private `VersionSpec(int major, Integer minor, Integer patch, Character qualifierSeparator, String qualifier, String buildTimestamp)` | private `VersionSpec::known(...) -> Option<VersionSpec>` | 保留非负数字与 patch 依赖 minor 的构造不变量；按是否存在原始段决定 core 文本位数 | `BEHAVIOR_VERIFIED` |
| `boolean isUnknown()` | `is_unknown(&self) -> bool` | 返回解析失败/缺失标记 | `BEHAVIOR_VERIFIED` |
| `int getMajor()` | `get_major(&self) -> i32` | unknown 为 0；其余为解析主版本 | `BEHAVIOR_VERIFIED` |
| `int getMinor()` | `get_minor(&self) -> i32` | 缺省和 unknown 均归一为 0 | `BEHAVIOR_VERIFIED` |
| `int getPatch()` | `get_patch(&self) -> i32` | 缺省和 unknown 均归一为 0 | `BEHAVIOR_VERIFIED` |
| `boolean hasQualifier()` | `has_qualifier(&self) -> bool` | 依据 Java qualifier 是否为 null，不以内容是否为空判断 | `BEHAVIOR_VERIFIED` |
| `String getQualifier()` | `get_qualifier(&self) -> Option<&VersionQualifier>` | `VersionQualifier` 精确保留可能含孤立代理项的 Java UTF-16；正常限定符可通过 `as_str()` 借用 | `BEHAVIOR_VERIFIED` |
| `String getVersionCore()` | `get_version_core(&self) -> &str` | 返回规范化 ASCII 数字段或 `UNKNOWN` | `BEHAVIOR_VERIFIED` |
| `String getVersion()` | `get_version(&self) -> &str` | 重建数字核心、可选原分隔符和限定符；不含构建时间戳 | `BEHAVIOR_VERIFIED` |
| `boolean hasBuildTimestamp()` | `has_build_timestamp(&self) -> bool` | 仅区分 null；空时间戳仍为 true | `BEHAVIOR_VERIFIED` |
| `String getBuildTimestamp()` | `get_build_timestamp(&self) -> Option<&str>` | 原样返回输入时间戳 | `BEHAVIOR_VERIFIED` |
| `String getFullVersion()` | `get_full_version(&self) -> &str` | 非 null 时间戳按 `version + " (" + timestamp + ")"` 拼接 | `BEHAVIOR_VERIFIED` |
| `boolean isAtLeast(int major)` | `is_at_least(&self, major: i32) -> bool` | 次、补丁目标默认为 0；unknown 的 0.0.0 仍参与普通数值比较 | `BEHAVIOR_VERIFIED` |
| `boolean isAtLeast(int major, int minor)` | `is_at_least_with_minor(&self, major: i32, minor: i32) -> bool` | 补丁目标默认为 0，按主→次→补丁短路 | `BEHAVIOR_VERIFIED` |
| `boolean isAtLeast(int major, int minor, int patch)` | `is_at_least_with_patch(&self, major: i32, minor: i32, patch: i32) -> bool` | 精确保留三层字典式数值比较，包括负目标 | `BEHAVIOR_VERIFIED` |
| `boolean isStableRelease()` | `is_stable_release(&self) -> bool` | 已知且无限定符，或限定符 UTF-16 精确等于大写 `RELEASE` 时为 true | `BEHAVIOR_VERIFIED` |

解析器以 UTF-16 码元而非 Rust Unicode 标量执行上游算法；固定
`unicode-general-category = 0.6.0` 对齐 JDK 21 的 Unicode 15.0 字母分类，BMP
十进制数字零码位表对齐 `Character.isDigit(char)`，负数累积算法保留
`Integer.parseInt` 的 `i32` 上下界。补充平面字母会触发 Java 高/低代理项拆分，
`VersionQualifier` 因此作为 Rust 扩展保留 `getQualifier()` 的孤立低代理项，而版本
重建仍为原始有效字符串。

35 组输入各导出全部字段、稳定判断和七个比较结果，并额外固定全部 BMP
`Character.isDigit(char)` / `Character.isLetter(char)` 连续区间，共 668 条 Java Golden；
覆盖 null/空白、NBSP、1/2/3 段、前导零、任意限定符分隔符、`RELEASE`、溢出、
Unicode 数字/字母/组合标记、孤立限定符代理项以及 null/空/非空构建时间戳。
`Thymeleaf` 的七个元数据 getter 已从硬编码分字段改为复用同一 `VersionSpec`
解析结果，恢复上游真实调用链。本切片完成后，`cargo-llvm-cov` 统计 5,296 行、
620 个函数和 8,422 个区域，三项均为 100%。

### 12.9 `ContentTypeUtils` / `ContentType`

CodeGraph 确认该工具的核心调用者为 `TemplateSpec` 和可配置模板解析器；内部
`ContentType` 不被其他对象独立引用，因而按紧耦合内部对象保留在
`content_type_utils.rs`。全部 28 个 Java 声明方法/构造器的 disposition 如下。

| Java 方法/构造器 | Rust API | 参数与语义映射 | 状态 |
|:---|:---|:---|:---|
| `static boolean isContentTypeHTML(String contentType)` | `is_content_type_html(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | XHTML 别名归一为 HTML；null/空白为 false | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeXML(String contentType)` | `is_content_type_xml(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | `application/xml` 与 `text/xml` 同族 | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeRSS(String contentType)` | `is_content_type_rss(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 只识别 `application/rss+xml` | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeAtom(String contentType)` | `is_content_type_atom(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 只识别 `application/atom+xml` | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeJavaScript(String contentType)` | `is_content_type_java_script(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 五个 JS/ECMAScript MIME 别名归一 | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeJSON(String contentType)` | `is_content_type_json(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 只识别 `application/json` | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeCSS(String contentType)` | `is_content_type_css(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 只识别 `text/css` | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeText(String contentType)` | `is_content_type_text(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | 只识别 `text/plain` | `BEHAVIOR_VERIFIED` |
| `static boolean isContentTypeSSE(String contentType)` | `is_content_type_sse(content_type: Option<&str>) -> Result<bool, ContentTypeError>` | `text/event-stream` 不映射模板模式 | `BEHAVIOR_VERIFIED` |
| private `isContentType(String contentType, String matcher)` | private `is_content_type(content_type, matcher)` | 所有公开判定共享同一解析、归一化和异常路径 | `BEHAVIOR_VERIFIED` |
| `static TemplateMode computeTemplateModeForContentType(String contentType)` | `compute_template_mode_for_content_type(content_type: Option<&str>) -> Result<Option<TemplateMode>, ContentTypeError>` | RSS/Atom→XML，JSON→JAVASCRIPT，SSE/未知→None | `BEHAVIOR_VERIFIED` |
| `static TemplateMode computeTemplateModeForTemplateName(String templateName)` | `compute_template_mode_for_template_name(template_name: Option<&str>) -> Option<TemplateMode>` | 最后扩展名执行 Java `toLowerCase(Locale.US).trim()` | `BEHAVIOR_VERIFIED` |
| `static TemplateMode computeTemplateModeForRequestPath(String requestPath)` | `compute_template_mode_for_request_path(request_path: Option<&str>) -> Result<Option<TemplateMode>, ContentTypeError>` | 剥离 query/fragment/矩阵参数；故意不小写扩展名；null 为类型化 NPE | `BEHAVIOR_VERIFIED` |
| `static boolean hasRecognizedFileExtension(String templateName)` | `has_recognized_file_extension(template_name: Option<&str>) -> bool` | 与模板名扩展名规则共用同一映射 | `BEHAVIOR_VERIFIED` |
| `static String computeContentTypeForTemplateName(String templateName, Charset charset)` | `compute_content_type_for_template_name(template_name: Option<&str>, charset: Option<&Charset>) -> Option<String>` | 已知扩展名输出首选 MIME；非空字符集追加规范名称 | `BEHAVIOR_VERIFIED` |
| `static String computeContentTypeForRequestPath(String requestPath, Charset charset)` | `compute_content_type_for_request_path(request_path: Option<&str>, charset: Option<&Charset>) -> Result<Option<String>, ContentTypeError>` | 保留请求路径 null 异常和大小写敏感扩展名 | `BEHAVIOR_VERIFIED` |
| `static Charset computeCharsetFromContentType(String contentType)` | `compute_charset_from_content_type(content_type: Option<&str>) -> Result<Option<Charset>, ContentTypeError>` | 缺失/不支持名称为 None；非法名称保留未捕获异常类别 | `BEHAVIOR_VERIFIED` |
| private `computeFileExtensionFromTemplateName(String templateName)` | private `normalized_template_extension(template_name)` | 最后点号、Locale.US 小写和 Java ASCII trim | `BEHAVIOR_VERIFIED` |
| private `computeFileExtensionFromRequestPath(String requestPath)` | private `compute_file_extension_from_request_path(request_path)` | 处理 `?`、`#`、`;`、最后 `/` 与最后 `.` 的精确顺序 | `BEHAVIOR_VERIFIED` |
| `static String combineContentTypeAndCharset(String contentType, Charset charset)` | `combine_content_type_and_charset(content_type: Option<&str>, charset: Option<&Charset>) -> Result<Option<String>, ContentTypeError>` | null charset 原样返回；覆盖已有参数但保持插入位置 | `BEHAVIOR_VERIFIED` |
| private `ContentTypeUtils()` | 单元结构体无实例构造 API | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |
| package `ContentType parseContentType(String contentType)` | private `ContentType::parse_content_type(content_type)` | `StringTokenizer` 忽略空 token；全分号保留数组越界类别 | `BEHAVIOR_VERIFIED` |
| package `ContentType(String mimeType, LinkedHashMap<String,String> parameters)` | private `ContentType { mime_type, parameters }` | `IndexMap` 对齐 LinkedHashMap 的插入顺序和覆盖不移位 | `BEHAVIOR_VERIFIED` |
| package `String getMimeType()` | private `mime_type(&self) -> &str` | 返回归一化、trim 后 MIME | `BEHAVIOR_VERIFIED` |
| package `LinkedHashMap<String,String> getParameters()` | private `parameters(&self) -> &IndexMap<String,String>` | 内部共享引用保留读取语义 | `BEHAVIOR_VERIFIED` |
| package `Charset getCharset()` | 由 `compute_charset_from_content_type` 内联读取参数并调用 `Charset::for_name` | Unsupported 返回 None，Illegal 返回类型化错误 | `BEHAVIOR_VERIFIED` |
| package `void setCharset(Charset charset)` | private `set_charset(&mut self, charset: Option<&Charset>)` | null 不变；覆盖或追加规范名称 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `Display for ContentType` | MIME 后按参数插入顺序输出 `;name=value` | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`ContentTypeUtilsGolden.java`](../../thymeleaf-test/tests/java/ContentTypeUtilsGolden.java) 生成 375 条
记录，Rust 差分测试逐条覆盖九组 MIME 判定、模式推导、模板名/请求路径扩展名、
字符集规范名、参数顺序和 Java 异常类别。`TemplateSpec` 已改为调用该对象的真实
API，消除先前的重复匹配表。本切片完成后，`cargo-llvm-cov` 统计 5,717 行、664 个
函数和 9,041 个区域，三项均为 100%。

### 12.10 `SetUtils`

上游声明 8 个方法/构造器，Rust 全部给出可追踪处置：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `static Set<?> toSet(Object target)` | `to_set(target: Option<SetTarget<'a,T>>) -> Result<SetValue<'a,T>,SetUtilsError>` | `Set` 分支借用同一实例；对象数组/Iterable 使用 `IndexSet` 按首次出现顺序去重；primitive array 保留 `ClassCastException` 类别；其他类型保留 Java 类名和精确参数错误 | `BEHAVIOR_VERIFIED` |
| `static int size(Set<?> target)` | `size(target: Option<&dyn SetView<T>>) -> Result<i32,ValidateError>` | null 返回精确错误；超出 Rust 可表示 Java `int` 的理论大小饱和为 `Integer.MAX_VALUE` | `BEHAVIOR_VERIFIED` |
| `static boolean isEmpty(Set<?> target)` | `is_empty(target: Option<&dyn SetView<T>>) -> bool` | null 或空集合均返回 true | `BEHAVIOR_VERIFIED` |
| `static boolean contains(Set<?> target, Object element)` | `contains(target, element) -> Result<bool,ValidateError>` | 保留 null target 错误；`Option<T>` 表达 Java null 元素 | `BEHAVIOR_VERIFIED` |
| `static boolean containsAll(Set<?> target, Object[] elements)` | `contains_all_array(target, elements)` | 先校验 target 再校验数组；空数组为 true；重复请求不要求重复条目 | `BEHAVIOR_VERIFIED` |
| `static boolean containsAll(Set<?> target, Collection<?> elements)` | `contains_all_collection(target, elements)` | 保留该重载不同的 target 错误文本；迭代顺序和短路行为一致 | `BEHAVIOR_VERIFIED` |
| `static <X> Set<X> singletonSet(X element)` | `singleton_set(element) -> SetValue<'static,T>` | 支持 null 等价值；只读包装不暴露修改入口，对齐 `Collections.unmodifiableSet` | `BEHAVIOR_VERIFIED` |
| private `SetUtils()` | 单元结构体无实例构造 API | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |

`SetView`、`SetTarget`、`SetValue` 与 `SetUtilsError` 是紧耦合 Rust 等价适配，
登记为 `RUST_EXTENSION`，不计入 Java 对象分子。固定 Java Oracle
[`SetUtilsGolden.java`](../../thymeleaf-test/tests/java/SetUtilsGolden.java) 直接编译上游源码，
覆盖 `LinkedHashSet` 身份和顺序、`TreeSet` 顺序、对象/primitive 数组、Iterable、
Iterator 非 Iterable、null/重复元素、两个 `containsAll` 重载及不可修改单例。

### 12.11 `Sets`

上游声明 7 个方法/构造器，Rust 全部通过 `SetUtils` 的真实委托链实现：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `Set<?> toSet(Object target)` | `to_set(&self, target)` | 直接委托 `SetUtils::to_set`，保留集合身份、顺序和错误 | `BEHAVIOR_VERIFIED` |
| `int size(Set<?> target)` | `size(&self, target)` | 直接委托并保留 null 校验 | `BEHAVIOR_VERIFIED` |
| `boolean isEmpty(Set<?> target)` | `is_empty(&self, target)` | null 或空集合为 true | `BEHAVIOR_VERIFIED` |
| `boolean contains(Set<?> target, Object element)` | `contains(&self, target, element)` | 直接委托成员判断 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(Set<?> target, Object[] elements)` | `contains_all_array(&self, target, elements)` | 数组重载独立保留 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(Set<?> target, Collection<?> elements)` | `contains_all_collection(&self, target, elements)` | Collection 重载独立保留 | `BEHAVIOR_VERIFIED` |
| `Sets()` | `Sets::new()` / `Default` | 无状态公开构造器；通常以 `#sets` 暴露给标准表达式 | `BEHAVIOR_VERIFIED` |

本切片的 47 条 Java/Rust Golden 逐条验证工具对象与表达式 facade。
该阶段完成时项目共有 138 个单元测试和 17 个固定 Java Golden 差分测试
（2,033 条记录）；`cargo-llvm-cov` 统计 6,078 行、709 个函数和 9,596 个区域，
三项均为 100%。

### 12.12 `ListUtils`

上游声明 10 个方法/构造器，Rust 全部给出可追踪处置：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `static List<?> toList(Object target)` | `to_list(target: Option<ListTarget<'a,T>>) -> Result<ListValue<'a,T>,ListUtilsError>` | 已有 `List` 借用同一实例；对象数组和非 List Iterable 新建 ArrayList 等价值并保留顺序、重复/null；primitive array 保留 `ClassCastException` 类别；其他类型保留 Java 类名和精确错误 | `BEHAVIOR_VERIFIED` |
| `static int size(List<?> target)` | `size(target: Option<&dyn ListView<T>>) -> Result<i32,ValidateError>` | null 返回精确错误；理论超大 Rust 长度按 Java `int` 上限处理 | `BEHAVIOR_VERIFIED` |
| `static boolean isEmpty(List<?> target)` | `is_empty(target: Option<&dyn ListView<T>>) -> bool` | null 或空列表均返回 true | `BEHAVIOR_VERIFIED` |
| `static boolean contains(List<?> target, Object element)` | `contains(target, element) -> Result<bool,ValidateError>` | 保留 null target 错误、顺序无关成员判断及 null 元素 | `BEHAVIOR_VERIFIED` |
| `static boolean containsAll(List<?> target, Object[] elements)` | `contains_all_array(target, elements)` | 先校验 target 再校验数组；空数组为 true；重复请求不要求重复条目 | `BEHAVIOR_VERIFIED` |
| `static boolean containsAll(List<?> target, Collection<?> elements)` | `contains_all_collection(target, elements)` | 保留该重载不同的 target 错误文本、空/重复/缺失元素语义 | `BEHAVIOR_VERIFIED` |
| `static <T extends Comparable> List<T> sort(List<T> list)` | `sort(list) -> Result<ListValue<'static,T>,ListUtilsError>` | `toArray` 快照后稳定自然排序；String 按 UTF-16 code unit，Float/Double 按 Java 包装类型规则；null/异构元素错误类别保持；原列表不变 | `BEHAVIOR_VERIFIED` |
| `static <T> List<T> sort(List<T> list, Comparator c)` | `sort_with_comparator(list, comparator)`；`sort_with_required_comparator` | nullable Comparator 保留 null 时自然排序；第二入口让非 Comparable Rust 类型使用必填 Comparator；Comparator 异常原样传播 | `BEHAVIOR_VERIFIED` |
| private `fillNewList(Object[] a, Class listType)` | `ListView::fill_sorted` + private `fill_new_list` | 公开无参构造成功则保留输入运行时类型；仅构造失败回退 ArrayList；构造后 `add` 失败不得被回退捕获 | `BEHAVIOR_VERIFIED` |
| private `ListUtils()` | 单元结构体无实例构造 API | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |

`ListTypeValue`、`ListUtilsError`、`ComparableValue`、`ComparatorValue`、`ListView`、
`ListTarget` 与 `ListValue` 是紧耦合 Rust 等价适配，登记为 `RUST_EXTENSION`。
固定 Java Oracle [`ListUtilsGolden.java`](../../thymeleaf-test/tests/java/ListUtilsGolden.java) 直接编译
上游 `Validate.java`、`ListUtils.java` 与 `Lists.java`，覆盖 LinkedList 身份和类型、
对象/primitive 数组、纯 Iterable、Iterator 非 Iterable、两个 `containsAll` 重载、
反射构造成功/回退、构造后 add 异常、UTF-16、Double、稳定 Comparator 和异常传播。

### 12.13 `Lists`

上游声明 9 个方法/构造器，Rust 全部通过 `ListUtils` 的真实委托链实现：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `List<?> toList(Object target)` | `to_list(&self, target)` | 直接委托并保留列表身份、顺序、重复/null 和错误 | `BEHAVIOR_VERIFIED` |
| `int size(List<?> target)` | `size(&self, target)` | 直接委托并保留 null 校验 | `BEHAVIOR_VERIFIED` |
| `boolean isEmpty(List<?> target)` | `is_empty(&self, target)` | null 或空列表为 true | `BEHAVIOR_VERIFIED` |
| `boolean contains(List<?> target, Object element)` | `contains(&self, target, element)` | 直接委托成员判断 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(List<?> target, Object[] elements)` | `contains_all_array(&self, target, elements)` | 数组重载独立保留 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(List<?> target, Collection<?> elements)` | `contains_all_collection(&self, target, elements)` | Collection 重载独立保留 | `BEHAVIOR_VERIFIED` |
| `<T extends Comparable> List<T> sort(List<T> list)` | `sort(&self, list)` | 委托自然稳定排序 | `BEHAVIOR_VERIFIED` |
| `<T> List<T> sort(List<T> list, Comparator c)` | `sort_with_comparator`；`sort_with_required_comparator` | 委托 nullable/必填 Comparator 入口，保持 Java 泛型能力 | `BEHAVIOR_VERIFIED` |
| `Lists()` | `Lists::new()` / `Default` | 无状态公开构造器；通常以 `#lists` 暴露给标准表达式 | `BEHAVIOR_VERIFIED` |

本切片新增 66 条固定 Java/Rust Golden 记录。完成后项目共有 147 个单元测试和
18 个固定 Java Golden 差分测试（2,099 条记录）；`cargo-llvm-cov` 统计 6,919 行、
808 个函数和 10,857 个区域，三项均为 100%。

### 12.14 `Maps`

上游声明 9 个方法/构造器，Rust 全部通过 `MapUtils` 的真实委托链实现：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `int size(Map<?,?> target)` | `size(&self, target)` | 保留 null target 精确校验和 Java `Map#size` 饱和值合同 | `BEHAVIOR_VERIFIED` |
| `boolean isEmpty(Map<?,?> target)` | `is_empty(&self, target)` | null 或空映射返回 true | `BEHAVIOR_VERIFIED` |
| `boolean containsKey(Map<?,?> target, Object key)` | `contains_key(&self, target, key)` | 支持 null 键并传播 target 校验 | `BEHAVIOR_VERIFIED` |
| `<X> boolean containsAllKeys(Map<?,?> target, X[] keys)` | `contains_all_keys_array` | 数组重载、校验顺序、空/重复请求独立保留 | `BEHAVIOR_VERIFIED` |
| `<X> boolean containsAllKeys(Map<?,?> target, Collection<X> keys)` | `contains_all_keys_collection` | Collection 重载独立保留 | `BEHAVIOR_VERIFIED` |
| `boolean containsValue(Map<?,?> target, Object value)` | `contains_value(&self, target, value)` | 支持 null 值并传播 target 校验 | `BEHAVIOR_VERIFIED` |
| `<X> boolean containsAllValues(Map<?,?> target, X[] values)` | `contains_all_values_array` | 数组重载、校验顺序、空/重复请求独立保留 | `BEHAVIOR_VERIFIED` |
| `<X> boolean containsAllValues(Map<?,?> target, Collection<X> values)` | `contains_all_values_collection` | Collection 重载独立保留 | `BEHAVIOR_VERIFIED` |
| `Maps()` | `Maps::new()` / `Default` | 无状态公开构造器；通常以 `#maps` 暴露 | `BEHAVIOR_VERIFIED` |

### 12.15 `Objects`

上游声明 5 个方法/构造器，Rust 均保留原集合类型、复制和异常边界：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `<T> T nullSafe(T target, T defaultValue)` | `null_safe(&self, target, default_value)` | 委托 `ObjectUtils`；返回被选中的同一对象，target/default 均可为 null | `BEHAVIOR_VERIFIED` |
| `<T> T[] arrayNullSafe(T[] target, T defaultValue)` | `array_null_safe(&self, target, default_value)` | 克隆为同运行时组件类数组；仅实际替换 null 时执行组件赋值检查，不兼容值返回 `ArrayStoreException` 等价错误 | `BEHAVIOR_VERIFIED` |
| `<T> List<T> listNullSafe(List<T> target, T defaultValue)` | `list_null_safe(&self, target, default_value)` | 固定返回独立、可变 `ArrayList` 等价 `Vec`；保留顺序、重复项和长度 | `BEHAVIOR_VERIFIED` |
| `<T> Set<T> setNullSafe(Set<T> target, T defaultValue)` | `set_null_safe(&self, target, default_value)` | 固定返回独立、可变 `LinkedHashSet` 等价 `IndexSet`；按源迭代顺序插入并在替换后去重 | `BEHAVIOR_VERIFIED` |
| `Objects()` | `Objects::new()` / `Default` | 无状态公开构造器；通常以 `#objects` 暴露 | `BEHAVIOR_VERIFIED` |

`ObjectArrayValue` 与 `ObjectsError` 是紧耦合 `RUST_EXTENSION`：前者显式保存 Java
引用数组的组件类名和赋值兼容性谓词，并通过 `set` 保留可变数组写入检查；后者区分
`IllegalArgumentException`、`ArrayStoreException` 和数组越界类别。35 条固定 Java
Golden 直接编译上游 `MapUtils.java`、`ObjectUtils.java`、`Maps.java` 与
`Objects.java`，验证 facade 委托、数组类型/独立性/可变性、默认值延迟检查、
ArrayList/LinkedHashSet 具体结果类型、集合独立性和替换去重。
完成后项目共有 155 个单元测试和 19 个固定 Java Golden 差分测试（2,134 条记录）；
`cargo-llvm-cov` 统计 7,246 行、845 个函数和 11,315 个区域，三项均为 100%。

### 12.16 `LoggingUtils`

上游声明 1 个公开方法和 1 个私有构造器，Rust 完整映射如下：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `static String loggifyTemplateName(String template)` | `loggify_template_name(template: Option<&Utf16String>) -> Option<Utf16StringResult>` | null 原样返回；按 Java UTF-16 单元计算 120 阈值；短名称仅将 LF 替换为空格；长名称拼接前 35 单元、`[...]`、后 80 单元 | `BEHAVIOR_VERIFIED` |
| private `LoggingUtils()` | 单元结构体无实例构造 API | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |

`Utf16String` 与 `Utf16StringResult` 是紧耦合 `RUST_EXTENSION`。前者保存任意 UTF-16
代码单元，使 `substring(0,35)` 或末 80 单元切开代理对时仍能无损表达结果；后者区分
Java `String.replace(char,char)` 无匹配时返回原对象与发生替换/截断时创建新对象。
`TemplateSpec` 和 `TemplateCacheKey` 已删除各自的私有复制算法，恢复对
`LoggingUtils` 的共享调用链。

38 条固定 Java Golden 覆盖 null、空串、LF/CR、120/121 临界长度、长名称头尾截断、
返回对象身份，以及前 35/末 80 边界分别切开高/低代理项的原始 UTF-16 输出。
完成后项目共有 159 个单元测试和 20 个固定 Java Golden 差分测试（2,172 条记录）；
`cargo-llvm-cov` 统计 7,360 行、863 个函数和 11,525 个区域，三项均为 100%。

### 12.17 `AggregateUtils` / `Aggregates`

`AggregateUtils` 上游声明 16 个公开重载、7 个私有数值转换方法和 1 个私有构造器；
`Aggregates` 声明相同的 16 个 facade 重载和 1 个公开构造器。Rust 不以一个
可变参数函数抹平重载，而是逐入口保留目标类型和验证边界：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `sum(Iterable<? extends Number>)` | `sum_iterable(Option<&dyn NumberIterableValue>)` | 校验和求和各遍历一次；空 iterable 返回 null | `BEHAVIOR_VERIFIED` |
| `sum(Object[])` | `sum_objects(Option<&[AggregateObjectValue]>)` | 先扫描全部 null，再执行 Number 强转 | `BEHAVIOR_VERIFIED` |
| `sum(Number[])` | `sum_numbers(Option<&[Option<NumberValue>]>)` | facade 的 Number 数组入口；保留 array null 消息 | `BEHAVIOR_VERIFIED` |
| `sum(byte[])` | `sum_bytes(Option<&[i8]>)` | 每项按 `BigDecimal.valueOf(long)` 转换 | `BEHAVIOR_VERIFIED` |
| `sum(short[])` | `sum_shorts(Option<&[i16]>)` | 每项按 `BigDecimal.valueOf(long)` 转换 | `BEHAVIOR_VERIFIED` |
| `sum(int[])` | `sum_ints(Option<&[i32]>)` | 每项按 `BigDecimal.valueOf(long)` 转换 | `BEHAVIOR_VERIFIED` |
| `sum(long[])` | `sum_longs(Option<&[i64]>)` | 逐项十进制累加，不发生 Java `long` 中间溢出 | `BEHAVIOR_VERIFIED` |
| `sum(float[])` | `sum_floats(Option<&[f32]>)` | 先扩展为 Java `double`，再走 `BigDecimal.valueOf` | `BEHAVIOR_VERIFIED` |
| `sum(double[])` | `sum_doubles(Option<&[f64]>)` | 保留 Java `Double.toString` 的十进制转换边界 | `BEHAVIOR_VERIFIED` |
| `avg(Iterable<? extends Number>)` | `avg_iterable(Option<&dyn NumberIterableValue>)` | 两次遍历；保留上游 iterable 平均值使用 array null 消息的行为 | `BEHAVIOR_VERIFIED` |
| `avg(Object[])` | `avg_objects(Option<&[AggregateObjectValue]>)` | 先完成 null 校验，再执行动态类型转换 | `BEHAVIOR_VERIFIED` |
| `avg(Number[])` | `avg_numbers(Option<&[Option<NumberValue>]>)` | facade 的 Number 数组入口 | `BEHAVIOR_VERIFIED` |
| `avg(byte[])` | `avg_bytes(Option<&[i8]>)` | 精确除法优先，非终止小数按上游规则舍入 | `BEHAVIOR_VERIFIED` |
| `avg(short[])` | `avg_shorts(Option<&[i16]>)` | 同上 | `BEHAVIOR_VERIFIED` |
| `avg(int[])` | `avg_ints(Option<&[i32]>)` | 同上 | `BEHAVIOR_VERIFIED` |
| `avg(long[])` | `avg_longs(Option<&[i64]>)` | 同上 | `BEHAVIOR_VERIFIED` |
| `avg(float[])` | `avg_floats(Option<&[f32]>)` | 同上，并保持非有限值的 NumberFormat 失败类别 | `BEHAVIOR_VERIFIED` |
| `avg(double[])` | `avg_doubles(Option<&[f64]>)` | 同上 | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(BigDecimal)` | `NumberValue::BigDecimal` 转换分支 | 保持 unscaled value、scale 和零值 scale | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(BigInteger)` | `NumberValue::BigInteger` 转换分支 | 精确整数转换 | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(Byte)` | `NumberValue::Byte` 转换分支 | 经 `longValue()` | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(Short)` | `NumberValue::Short` 转换分支 | 经 `longValue()` | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(Integer)` | `NumberValue::Integer` 转换分支 | 经 `longValue()` | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(Long)` | `NumberValue::Long` 转换分支 | 经 `longValue()` | `BEHAVIOR_VERIFIED` |
| `toBigDecimal(Number)` | `NumberValue::{Float,Double,Other}` 转换分支 | 其余 Number 经 `doubleValue()` 和 Java 十进制文本 | `BEHAVIOR_VERIFIED` |
| private `AggregateUtils()` | 单元结构体无实例构造 API | 保留无状态工具类与私有构造器意图 | `BEHAVIOR_VERIFIED` |
| `Aggregates()` | `Aggregates::new()` / `Default` | 无状态公开构造器；通常以 `#aggregates` 暴露 | `BEHAVIOR_VERIFIED` |

表中 16 个 `sum_*`/`avg_*` API 同时由 `Aggregates` 提供实例方法并真实委托
`AggregateUtils`；Rust 仅因不支持重载而加入类型后缀。`BigDecimalValue` 使用
`BigInt + i32 scale` 保存 Java 任意精度十进制表示，精确除法失败时按
`max(total.scale(), 10)` 与 `HALF_UP` 回退；`NumberValue` 保存 Java 包装类分派，
其余四个动态对象/iterable/error 类型保持 null、类型强转、重复遍历及异常类别。

68 条固定 Java Golden 逐入口验证两个对象的 32 个公开操作、所有 Number 分派、
空/null/错误优先级、精确与非终止平均值、scale、浮点极值和大整数累加；另对
20,010 个确定性 `f64` 位模式执行 Java/Rust 聚合输出哈希差分。完成后项目共有
172 个单元测试和 21 个固定 Java Golden 差分测试（2,240 条记录）；
`cargo-llvm-cov` 统计 8,312 行、953 个函数和 13,056 个区域，三项均为 100%。

### 12.18 `ArrayUtils` / `Arrays`

`ArrayUtils` 上游声明 16 个公开方法、1 个私有转换方法和 1 个私有构造器；
`Arrays` 声明 12 个 facade 方法和 1 个公开构造器。Rust 逐入口保留如下：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `Object[] toArray(Object target)` | `to_array(Option<ArrayTarget<T>>)` | 兼容引用数组借用同一实例；Iterable 按首个非 null 精确运行时类推断组件类，异构时退化为 Object；primitive array 保留 ClassCast 类别 | `BEHAVIOR_VERIFIED` |
| `Object[] toStringArray(Object target)` | `to_string_array(...)` | `String[]` 原样返回；Iterable 反射数组写入检查；不兼容数组保留上游错误文本中的 `of Class` 缺陷 | `BEHAVIOR_VERIFIED` |
| `Object[] toIntegerArray(Object target)` | `to_integer_array(...)` | 固定 `java.lang.Integer` 组件类与写入检查 | `BEHAVIOR_VERIFIED` |
| `Object[] toLongArray(Object target)` | `to_long_array(...)` | 固定 `java.lang.Long` 组件类与写入检查 | `BEHAVIOR_VERIFIED` |
| `Object[] toDoubleArray(Object target)` | `to_double_array(...)` | 固定 `java.lang.Double` 组件类与写入检查 | `BEHAVIOR_VERIFIED` |
| `Object[] toFloatArray(Object target)` | `to_float_array(...)` | 固定 `java.lang.Float` 组件类与写入检查 | `BEHAVIOR_VERIFIED` |
| `Object[] toBooleanArray(Object target)` | `to_boolean_array(...)` | 固定 `java.lang.Boolean` 组件类与写入检查 | `BEHAVIOR_VERIFIED` |
| `int length(Object[] target)` | `length(Option<&[Option<T>]>)` | null 保留精确参数错误；长度映射 Java int 合同 | `BEHAVIOR_VERIFIED` |
| `boolean isEmpty(Object[] target)` | `is_empty(...)` | null 或零长度均为 true | `BEHAVIOR_VERIFIED` |
| `boolean contains(Object[] target,Object element)` | `contains(...)` | 支持 null 元素，并按请求元素到目标元素的 Java equals 方向查询 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(Object[] target,Object[] elements)` | `contains_all_array(...)` | target→elements 校验顺序与该重载的 `containsAll` target 消息保持一致；重复请求去重 | `BEHAVIOR_VERIFIED` |
| `boolean containsAll(Object[] target,Collection<?> elements)` | `contains_all_collection(...)` | 保留该重载不同的 `contains` target 消息；HashSet 等价去重后删除目标元素 | `BEHAVIOR_VERIFIED` |
| `<T,X> X[] copyOf(T[] original,int newLength,Class<? extends X[]> newType)` | `copy_of_with_type(...)` | 先解析 newType、再检查负长度、再读取 original；保留组件类、零填充、截断和 ArrayStore 类别 | `BEHAVIOR_VERIFIED` |
| `<T> T[] copyOf(T[] original,int newLength)` | `copy_of(...)` | 先读取 original 运行时类，因而 null + 负长度仍先产生 NullPointer；返回独立同组件数组 | `BEHAVIOR_VERIFIED` |
| `char[] copyOf(char[] original,int newLength)` | `copy_of_chars(Option<&[u16]>,i32)` | `u16` 精确保留 Java char；负长度在 null original 之前失败；扩展槽位为零 | `BEHAVIOR_VERIFIED` |
| `char[] copyOfRange(char[] original,int from,int to)` | `copy_of_range(...)` | `to-from` 使用 Java int 环绕；反向范围、负/越界索引、null 和尾部零扩展顺序保持一致 | `BEHAVIOR_VERIFIED` |
| private `toArray(Class,Object)` | private `convert(...)` | 数组、Iterable、其他对象三路运行时分派；组件类推断只比较精确类而非继承关系 | `BEHAVIOR_VERIFIED` |
| private `ArrayUtils()` | 单元结构体无实例构造 API | 保留无状态工具类和私有构造器意图 | `BEHAVIOR_VERIFIED` |
| `Arrays()` | `Arrays::new()` / `Default` | 无状态公开构造器；通常以 `#arrays` 暴露 | `BEHAVIOR_VERIFIED` |
| `Arrays` 的 12 个公开方法 | 同名 snake_case 实例方法 | 全部真实委托 `ArrayUtils`，两个 `containsAll` 重载保持独立入口 | `BEHAVIOR_VERIFIED` |

`ArrayTarget`、`ArrayValue`、`ArrayElementValue`、`ArrayTypeValue` 和
`ArrayUtilsError` 是紧耦合 `RUST_EXTENSION`，显式承接 JVM 的目标运行时类别、
引用身份、组件类推断、数组协变写入检查和异常分型；引用数组本体复用
`ObjectArrayValue`，没有建立冲突的第二套存储模型。

72 条固定 Java Golden 直接编译固定基线的 `Validate.java`、`ArrayUtils.java`
和 `Arrays.java`，覆盖全部 28 个公开操作、空/null/异构 Iterable、primitive
数组、运行时组件类、引用身份、复制扩展/截断、负长度、数组存储失败、UTF-16
char 及索引溢出边界。完成后项目共有 178 个单元测试和 22 个固定 Java Golden
差分测试（2,312 条记录）；`cargo-llvm-cov` 统计 8,884 行、1,017 个函数和
13,775 个区域，三项均为 100%。

### 12.19 `EvaluationUtils` / `Bools`

`EvaluationUtils` 上游声明 4 个公开静态方法和 1 个私有构造器，内部
`MapEntry` 声明构造器、3 个 Map 方法及 `toString`/`equals`/`hashCode`；
`Bools` 声明公开构造器和 14 个表达式方法。Rust 保留以下入口：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `evaluateAsBoolean(Object)` | `evaluate_as_boolean(&EvaluationValue)` | 显式分派 null、Boolean、全部 Number、Character、String、LiteralValue 和其他对象；空集合/数组仍为 true | `BEHAVIOR_VERIFIED` |
| `evaluateAsNumber(Object)` | `evaluate_as_number(&EvaluationValue)` | `BigDecimal` 保留同一实例；整数精确转换；Float/Double 使用 `BigDecimal(double)` 精确二进制值；自定义 Number 返回 null | `BEHAVIOR_VERIFIED` |
| `evaluateAsList(Object)` | `evaluate_as_list(Option<EvaluationTarget<T>>)` | null 为 `EmptyList`；非 null 为不可修改列表；Map 创建独立 `MapEntry` 快照；八种 primitive array 逐项装箱 | `BEHAVIOR_VERIFIED` |
| `evaluateAsArray(Object)` | `evaluate_as_array(Option<EvaluationTarget<T>>)` | null 为单 null `Object[]`；Iterable/Map/标量新建 Object[]；引用数组返回同一实例；primitive array 抛 ClassCast | `BEHAVIOR_VERIFIED` |
| `MapEntry(K,V)` | `MapEntry::new` / `raw` | `new` 对应不可变快照；`raw` 保存 Map 实现条目的运行时类和引用身份 | `BEHAVIOR_VERIFIED` |
| `getKey()` / `getValue()` | `get_key()` / `get_value()` | 保留可空键值 | `BEHAVIOR_VERIFIED` |
| `setValue(V)` | `set_value(...)` | 始终返回 UnsupportedOperation 类别 | `BEHAVIOR_VERIFIED` |
| `toString()` | `Display` | Java `key=value`，null 输出字面量 | `BEHAVIOR_VERIFIED` |
| `equals(Object)` | `PartialEq` | 与 Map.Entry 相同，只比较键和值，不比较实现类 | `BEHAVIOR_VERIFIED` |
| `hashCode()` | `java_hash_code()` | 保留上游非标准 `31 * keyHash + valueHash` 与 i32 环绕 | `BEHAVIOR_VERIFIED` |
| `isTrue(Object)` / `isFalse(Object)` | `is_true(...)` / `is_false(...)` | 委托 EvaluationUtils；LiteralValue(null) 保留 NullPointer 类别 | `BEHAVIOR_VERIFIED` |
| `arrayIsTrue` / `listIsTrue` / `setIsTrue` | `array_is_true` / `list_is_true` / `set_is_true` | array/list 保序；set 使用 LinkedHashSet 等价的首次顺序与布尔去重 | `BEHAVIOR_VERIFIED` |
| `arrayIsFalse` / `listIsFalse` / `setIsFalse` | 对应 snake_case 方法 | 对每项真值取反；保留 null target 固定消息 | `BEHAVIOR_VERIFIED` |
| `arrayAnd` / `listAnd` / `setAnd` | `array_and` / `list_and` / `set_and` | 从左到右短路；空输入为 true | `BEHAVIOR_VERIFIED` |
| `arrayOr` / `listOr` / `setOr` | `array_or` / `list_or` / `set_or` | 从左到右短路；空输入为 false | `BEHAVIOR_VERIFIED` |
| private `EvaluationUtils()` | 单元结构体无实例构造 API | 保留无状态工具类与私有构造器意图 | `BEHAVIOR_VERIFIED` |
| `Bools()` | `Bools::new()` / `Default` | 无状态公开构造器；通常以 `#bools` 暴露 | `BEHAVIOR_VERIFIED` |

`EvaluationValue` 负责标量 `instanceof` 分派；`EvaluationTarget`、
`EvaluationElement`、`EvaluationList` 和 `EvaluationArray` 明确表达
JVM 集合/数组运行时类别；`BigDecimalResult` 保留借用身份；`EvaluationError`
保留 NullPointer、NumberFormat、ClassCast、UnsupportedOperation 和参数校验类别。
这些均为紧耦合 `RUST_EXTENSION`，没有替代或省略 Java 主对象。

90 条固定 Java Golden 直接编译固定基线的 `LiteralValue.java`、
`EvaluationUtils.java` 和 `Bools.java`，覆盖真值矩阵、Java `trim`、Unicode 数字、
精确 Float/Double unscaled/scale、非有限数、不可修改列表、Map 条目快照与原身份、
八种 primitive array、引用数组身份、全部 14 个 facade 方法、空输入和短路异常；
另对 20,010 个确定性 `f64` 位模式执行精确十进制输出哈希差分。
完成后项目共有 178 个单元测试和 23 个固定 Java Golden 差分测试（2,402 条记录）；
`cargo-llvm-cov` 统计 9,329 行、1,087 个函数和 14,427 个区域，三项均为 100%。

### 12.20 `LiteralValue` / `StandardExpressionExecutionContext`

这一切片把 `EvaluationUtils` 中的临时 `Option<Utf16String>` 字面量适配替换为
正式 `LiteralValue` 对象，并迁移表达式执行上下文的完整六状态单例图：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `LiteralValue(String)` | `LiteralValue::new(Option<Utf16String>)` | 保留 UTF-16 文本及构造器允许 null 的行为；不建立文本值相等语义 | `BEHAVIOR_VERIFIED` |
| `String getValue()` | `get_value() -> Option<&Utf16String>` | 返回原始可空文本借用 | `BEHAVIOR_VERIFIED` |
| `Object unwrap(Object)` | `unwrap(Option<&dyn Any>) -> Option<&dyn Any>` | null 返回 null；字面量返回内部文本；其他对象返回完全相同的动态引用 | `BEHAVIOR_VERIFIED` |
| 三个公开静态上下文 | `NORMAL` / `RESTRICTED` / `RESTRICTED_FORBID_UNSAFE_EXP_RESULTS` | 公开只读规范单例，外部不能构造任意标志组合 | `BEHAVIOR_VERIFIED` |
| 四个 boolean getter | 对应 `get_*()` | 变量访问、外部访问、不安全结果、类型转换标志逐项保留 | `BEHAVIOR_VERIFIED` |
| `withoutTypeConversion()` | `without_type_conversion()` | 已关闭时返回原实例；已开启时返回对应无转换规范单例 | `BEHAVIOR_VERIFIED` |
| `withTypeConversion()` | `with_type_conversion()` | 已开启时返回原实例；未开启时返回对应有转换规范单例 | `BEHAVIOR_VERIFIED` |
| private constructors | 私有静态值初始化 | 保留不能从公共 API 创建任意上下文的封闭状态空间 | `BEHAVIOR_VERIFIED` |

27 条固定 Java Golden 直接编译固定基线的两个上游对象，覆盖可空 getter、
默认引用相等、三路 unwrap 与其他对象身份，以及三种公开上下文、六个规范状态、
转换幂等、往返和重复调用身份。完成后项目共有 180 个单元测试和 24 个固定
Java Golden 差分测试（2,429 条记录）；`cargo-llvm-cov` 统计 9,423 行、
1,100 个函数和 14,590 个区域，三项均为 100%。

### 12.21 标准转换服务对象族 / `NoOpToken`

转换服务使用 trait 模板方法表达 Java 的接口、抽象基类和 final `convert`
分派。`JavaConversion*` 适配显式保存 JVM 的动态对象、`Class<T>`、可空值及
借用/拥有身份；它们登记为 `RUST_EXTENSION`，不替代四个 Java 主对象：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `IStandardConversionService.convert(context,object,targetClass)` | `IStandardConversionService::convert(...)` | `Send + Sync` 保留线程安全要求；上下文以可空动态引用原样透传；目标和值保持运行时分类 | `BEHAVIOR_VERIFIED` |
| protected `AbstractStandardConversionService()` | `AbstractStandardConversionService` trait 实现 | 抽象对象不可直接实例化且无状态；实现该 trait 对应调用 protected super 构造 | `BEHAVIOR_VERIFIED` |
| final `convert(...)` | blanket `IStandardConversionService` impl | 先校验 targetClass；精确 String.class 下 null/String 原样返回，其他对象才进入字符串钩子；其余目标进入 other 钩子 | `BEHAVIOR_VERIFIED` |
| protected `convertToString(context,object)` | `convert_to_string(...)` | 默认调用对象 `toString()`；保留 null、借用共享 String、新建 String 及运行时异常 | `BEHAVIOR_VERIFIED` |
| protected `convertOther(context,object,targetClass)` | `convert_other(...)` | 默认抛含 `Class#getName()` 的精确 IllegalArgument；扩展可返回借用或拥有的任意类型 | `BEHAVIOR_VERIFIED` |
| `StandardConversionService()` | `StandardConversionService::new()` / `Default` | 无状态公开构造器；全部行为继承抽象服务默认值 | `BEHAVIOR_VERIFIED` |
| private `NoOpToken()` / `VALUE` | `NoOpToken::VALUE` | 外部不能构造；非零尺寸私有字段保证唯一静态地址稳定 | `BEHAVIOR_VERIFIED` |
| `NoOpToken.toString()` | `Display` / `to_string()` | 固定输出 `_`；相等仍按对象引用身份 | `BEHAVIOR_VERIFIED` |

21 条固定 Java Golden 直接编译固定基线的四个上游对象，覆盖目标类 null、
String null/原引用、普通对象 `toString()` 的值/null/共享引用/异常、Integer
及 primitive array 目标错误、自定义子类上下文透传与两个钩子，以及 NO-OP
单例身份和文本。完成后项目共有 181 个单元测试和 25 个固定 Java Golden
差分测试（2,450 条记录）；`cargo-llvm-cov` 统计 9,499 行、1,112 个函数和
14,682 个区域，三项均为 100%。

### 12.22 `Token` / `TokenParsingTracer`

`Token` 是标准表达式词法层的抽象值基类。Rust 使用泛型组合保存值，不复制尚未
迁移的 `SimpleExpression` 继承树；这只改变对象组织方式，不改变本对象七个已登记
构造器/方法的可观察合同。`TokenValue`、`TokenStringResult` 和
`TokenError` 是 JVM 动态 `Object#toString()` 与异常类别的显式适配，登记为
`RUST_EXTENSION`：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| protected `Token(Object value)` | `Token::new(value: Option<T>)` | 保存可空原值，不克隆；Java protected 构造由 Rust 公开泛型构造承接，供后续同包 token 对象组合 | `BEHAVIOR_VERIFIED` |
| `Object getValue()` | `get_value() -> Option<&T>` | 返回原对象借用，保留共享分配身份 | `BEHAVIOR_VERIFIED` |
| `String getStringRepresentation()` | `get_string_representation() -> Result<TokenStringResult, TokenError>` | 对非 null 值调用动态 Java `toString()`；保留 null 返回、共享 String 身份、新建 String 与运行时异常 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `to_string() -> Result<TokenStringResult, TokenError>` | Java 实现只委托上一方法；Rust 不用 `Display` 吞掉 null/异常，因此以同一可失败 API 精确表达 | `BEHAVIOR_VERIFIED` |
| static `boolean isTokenChar(String context, int pos)` | `Token::<T>::is_token_char(context, pos)` | 先执行 `charAt(pos)` 异常边界；按 UTF-16 code unit 匹配 ASCII、`.`/`_`/方括号、全部上游 Unicode 区间，并双向扫描 `-` 的数字/小数与标识符上下文 | `BEHAVIOR_VERIFIED` |
| private `TokenParsingTracer()` | 零尺寸 `TokenParsingTracer`，无公开构造方法 | 保留仅作静态命名空间、不能从公共 API 构造的意图 | `BEHAVIOR_VERIFIED` |
| static `String trace(String input)` | `TokenParsingTracer::trace(input)` | 按输入 UTF-16 code unit 调用同一字符判定；token 字符写入 `TOKEN_SUBSTITUTE` `#`，其他码元原样复制 | `BEHAVIOR_VERIFIED` |

46 条固定 Java Golden 直接编译固定基线的上游 `Token.java`；最小
`SimpleExpression` 测试桩只满足父类型编译依赖，本对象实现不调用该父类型。
Oracle 覆盖原值和 `toString()` 结果身份、null 与运行时异常类别、负数/越界索引、
固定边界和连字符上下文；另外对 65,536 个 BMP 码元分别执行单字符、连字符左侧、
连字符右侧判定，对完整 BMP 串执行 trace，并对 20,000 个确定性生成的 UTF-16
上下文执行逐位置判定及 trace 哈希差分。完成后项目共有 183 个单元测试和 26 个
固定 Java Golden 差分测试（2,496 条记录）；`cargo-llvm-cov` 统计 9,676 行、
1,130 个函数和 14,975 个区域，三项均为 100%。

### 12.23 剩余 enum 对象

本切片完成 `AttributeValueQuotes`、`HTMLElementType`、`StandardInlineMode`，
至此固定上游的五个 Java enum 主对象均达到 `BEHAVIOR_VERIFIED`。Java 编译器
隐式生成的 `values()`、`name()`、`ordinal()` 和默认 `toString()` 虽不进入
4,291 个源码声明方法分母，仍通过 `VALUES`、变体名称、`ordinal()` 与 `Display`
逐项验证：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `AttributeValueQuotes` enum constants | `AttributeValueQuotes::{DOUBLE,SINGLE,NONE}` / `VALUES` / `ordinal()` / `Display` | 三个成员的声明顺序、名称、序号和显示完整保持 | `BEHAVIOR_VERIFIED` |
| private `HTMLElementType(boolean voidElement)` | 五个封闭 enum variants | 构造参数固化为变体语义，外部不能创建额外状态 | `BEHAVIOR_VERIFIED` |
| `boolean HTMLElementType.isVoid()` | `is_void() -> bool` | 仅 `VOID` 为真；RAW_TEXT、ESCAPABLE_RAW_TEXT、FOREIGN、NORMAL 为假 | `BEHAVIOR_VERIFIED` |
| `StandardInlineMode` enum constants | 六个同名 variants / `VALUES` / `ordinal()` / `Display` | 保留 NONE、HTML、XML、TEXT、JAVASCRIPT、CSS 顺序；上游明确不存在 RAW | `BEHAVIOR_VERIFIED` |
| static `StandardInlineMode parse(String mode)` | `parse(mode: Option<&Utf16String>) -> Result<StandardInlineMode, StandardInlineModeParseError>` | Java UTF-16 `trim()` 只判空、不改变匹配输入；`equalsIgnoreCase()` 的 BMP 特殊映射、null/空白/未知消息及孤立代理项由 `message()` 无损保存 | `BEHAVIOR_VERIFIED` |

64 条固定 Java Golden 直接编译三个上游 enum，逐项覆盖成员数量、名称、ordinal、
显示和 void 标志，以及 null、Java 控制空白、NBSP、首尾空格、RAW、ASCII 大小写、
长 s、点上/无点 I 和孤立代理项。Oracle 还对六个 inline mode 名称的每个字符位置
遍历全部 65,536 个 BMP code unit，并对解析结果执行哈希差分。完成后项目共有
187 个单元测试和 27 个固定 Java Golden 差分测试（2,560 条记录）；
`cargo-llvm-cov` 统计 9,849 行、1,152 个函数和 15,225 个区域，三项均为 100%。

### 12.24 `IEngineProcessable`

该 package-private 引擎接口定义一次增量推进操作，不规定实现是否幂等、是否线程
安全或每次调用应返回哪个布尔值。Rust 同名 trait 不附加 `Send`/`Sync`，并使用
可变接收者保留 Java 实现按调用顺序更新内部状态的能力：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `boolean process()` | `IEngineProcessable::process(&mut self) -> bool` | `&mut self` 承接 Java 对象可变状态；trait object 承接接口动态分派；返回值不被框架预设或归一化 | `BEHAVIOR_VERIFIED` |

7 条固定 Java Golden 在 `org.thymeleaf.engine` 同包测试入口中直接编译上游接口，
以有状态实现连续调用四次，覆盖 false/true 交替序列、最终调用计数和接口引用与
实现对象身份。Rust Golden 通过 `&mut dyn IEngineProcessable` 走同一动态分派
合同。完成后项目共有 188 个单元测试和 28 个固定 Java Golden 差分测试
（2,567 条记录）；`cargo-llvm-cov` 统计 9,862 行、1,154 个函数和 15,247 个区域，
三项均为 100%。

### 12.25 `TemplateFlowController`

该对象只包含两个 package-private 可变布尔字段。Rust 保持相同可见性边界：
类型、构造器和字段均仅在 crate/engine 内可用，没有为了集成测试而扩张公共 API，
也没有增加上游不存在的 getter/setter：

| Java 签名/字段 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `TemplateFlowController()` | `pub(crate) TemplateFlowController::new()` | 每次构造独立实例；两个字段均显式初始化为 false | `BEHAVIOR_VERIFIED` |
| package `boolean stopProcessing` | `pub(crate) stop_processing: bool` | engine 内可按处理进度直接切换，不与另一标志联动 | `BEHAVIOR_VERIFIED` |
| package `boolean processorTemplateHandlerPending` | `pub(crate) processor_template_handler_pending: bool` | engine 内可独立切换，不修改其他实例 | `BEHAVIOR_VERIFIED` |

6 条固定 Java Golden 通过 `org.thymeleaf.engine` 同包入口直接编译上游对象，覆盖
两个实例的初始状态、先后修改两个字段以及第二实例不受影响。Rust 在对象同文件
单元测试中读取同一 Golden，从而不突破 package/crate 可见性。完成后项目共有
189 个单元测试和 29 个固定 Java Golden 差分测试（2,573 条记录）；
`cargo-llvm-cov` 统计 9,900 行、1,157 个函数和 15,290 个区域，三项均为 100%。

### 12.26 `FastStringWriter`

该对象直接为 Thymeleaf 模型、标签、内联器、处理器和模板引擎提供低分配字符串
输出。Rust 使用原始 UTF-16 代码单元缓冲区，避免以 UTF-8 或 Unicode 标量替换
Java `char` 语义：

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `FastStringWriter()` | `FastStringWriter::new()` / `Default::default()` | 内容为空；默认容量不可观察；两个入口均创建独立对象 | `BEHAVIOR_VERIFIED` |
| `FastStringWriter(int initialSize)` | `FastStringWriter::with_initial_size(initial_size)` | 负数保留 `IllegalArgumentException` 类别和 `Negative buffer size` 消息 | `BEHAVIOR_VERIFIED` |
| `void write(int c)` | `write_char(character)` | 按 Java 强转仅保留低 16 位，允许 NUL 与孤立代理项 | `BEHAVIOR_VERIFIED` |
| `void write(String str)` | `write_string(string)` | `None` 对应 null 并写入四个代码单元 `"null"` | `BEHAVIOR_VERIFIED` |
| `void write(String str,int off,int len)` | `write_string_range(string,offset,length)` | null 先映射 `"null"`；`off + len` 按 `i32` 回绕；异常类和范围消息精确保留 | `BEHAVIOR_VERIFIED` |
| `void write(char[] cbuf)` | `write_chars(characters)` | 原样复制全部 UTF-16 单元；null 保留增强 NPE 消息 | `BEHAVIOR_VERIFIED` |
| `void write(char[] cbuf,int off,int len)` | `write_chars_range(characters,offset,length)` | 逐条件保留 Java 短路顺序；越界异常消息为 null；零长度不修改状态 | `BEHAVIOR_VERIFIED` |
| `void flush()` | `flush()` | 与 Java 一样为空操作 | `BEHAVIOR_VERIFIED` |
| `void close()` | `close()` | 与 Java 一样为空操作，关闭后仍可写入 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `to_string()` | 每次返回新的 UTF-16 快照，后续写入不回改旧快照 | `BEHAVIOR_VERIFIED` |

此外，Rust 显式提供 `append_sequence`、`append_sequence_range` 和 `append_char`，
对齐从 `java.io.Writer` 继承的三个公开 `append` 入口：null 序列写入 `"null"`，
子序列非法时保留 `StringIndexOutOfBoundsException`，成功时返回同一写入器引用。

46 条固定 Java Golden 覆盖所有声明方法和继承入口，穷举 -65,536 到 131,071 的
`write(int)` 输入并对字符串/字符数组 offset-length 矩阵哈希差分。完成后项目共有
192 个单元测试和 30 个固定 Java Golden 差分测试（2,619 条记录）；
`cargo-llvm-cov` 统计 10,078 行、1,181 个函数和 15,544 个区域，三项均为 100%。

### 12.27 `CharArrayWrapperSequence`

该对象不是拥有型字符串，而是对调用方 `char[]` 的可变共享视图。Rust 使用
`SharedCharArray = Arc<RwLock<Vec<u16>>>`：`Arc` 保留浅 clone、子序列和调用方
共享同一分配的语义，`RwLock` 使外部修改可见且兑现上游线程安全 JavaDoc，
`u16` 则无损保存孤立代理项。

| Java 签名 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `CharArrayWrapperSequence(char[] array)` | `CharArrayWrapperSequence::new(array)` | 不复制数组；null 先报固定非法参数；空数组按上游委托后因 offset 0 等于 length 而失败 | `BEHAVIOR_VERIFIED` |
| `CharArrayWrapperSequence(char[] buffer,int offset,int len)` | `with_range(buffer,offset,length)` | 校验顺序和消息精确保留；`offset + len` 按 `i32` 回绕；不上移上游没有的负 length 校验 | `BEHAVIOR_VERIFIED` |
| `char charAt(int index)` | `char_at(index)` | 先校验相对 index；畸形溢出视图随后按数组访问报告回绕后的绝对 index | `BEHAVIOR_VERIFIED` |
| `int length()` | `length()` | 原样返回声明值，包括上游可保存的负数 | `BEHAVIOR_VERIFIED` |
| `CharSequence subSequence(int start,int end)` | `sub_sequence(start,end)` | 共享原数组；保留 start/end 检查次序；反向范围可先形成负长度对象并延迟失败 | `BEHAVIOR_VERIFIED` |
| protected `clone()` | `Clone::clone()` | Java `Cloneable` 映射为 Rust 标准浅 clone；新对象共享数组但 offset/length 独立保存 | `BEHAVIOR_VERIFIED` |
| `int hashCode()` | `hash_code()` | 与 Java String 相同的 31 倍 UTF-16 哈希及 `int` 回绕；零/畸形空循环返回 0 | `BEHAVIOR_VERIFIED` |
| `boolean equals(Object obj)` | `equals_object(object)` / `PartialEq` | 任意对象入口保留 null、运行时类型和引用身份快路；仅同类型按视图内容相等，String 内容相同仍不相等 | `BEHAVIOR_VERIFIED` |
| `String toString()` | `to_java_string()` | 返回独立 UTF-16 快照；构造器接受的负/溢出 length 在此保留精确 `StringIndexOutOfBoundsException` | `BEHAVIOR_VERIFIED` |

57 条固定 Java Golden 覆盖全部声明入口、浅 clone、外部数组修改、代理项、空/负/
溢出范围、数组访问两类消息、相等和 String 兼容哈希；同时对 770 组构造参数及
100 组子序列参数执行复合哈希差分。完成后项目共有 194 个单元测试和 31 个固定
Java Golden 差分测试（2,676 条记录）；`cargo-llvm-cov` 统计 10,279 行、1,203
个函数和 15,839 个区域，三项均为 100%。

### 12.28 `TextParseStatus` / `ParsingLocatorUtil`

这两个对象共同构成 text parser 的 package-private 扫描坐标基础，Rust 放在
`src/text/` 并保持 `pub(crate)`，不因 Golden 测试扩张公共 API：

| Java 签名/字段 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `TextParseStatus()` | `pub(crate) TextParseStatus::new()` / `Default` | 六个字段全部使用 Java 默认零值；每次构造独立状态 | `BEHAVIOR_VERIFIED` |
| package `int offset/line/col` | 同名 snake_case `i32` 字段 | crate 内直接修改并保留最小/最大值 | `BEHAVIOR_VERIFIED` |
| package `boolean inStructure/inCommentLine` | `in_structure` / `in_comment_line` | 两个标志独立可变 | `BEHAVIOR_VERIFIED` |
| package `char literalMarker` | `literal_marker: u16` | NUL 默认值和孤立代理项均可无损保存 | `BEHAVIOR_VERIFIED` |
| `static void countChar(int[] locator,char c)` | `ParsingLocatorUtil::count_char(locator,character)` | LF 依次执行 line++、column=1；其他代码单元仅 column++；`i32` 回绕 | `BEHAVIOR_VERIFIED` |
| private `ParsingLocatorUtil()` | 不公开 Rust 构造入口 | 零状态工具对象只允许关联函数，保持不可实例化意图 | `BEHAVIOR_VERIFIED` |

`ParsingLocatorError` 区分增强 NPE 与数组下标异常。LF 对单元素数组会先把 line 从
5 改为 6，再在写 column 时失败；非 LF 对同一数组直接失败且保留 5，严格保留
Java 语句级副作用顺序。18 条固定 Java Golden 覆盖默认/修改状态、实例独立性、
正常字符、LF/CR/NUL/代理项、行列溢出以及 null/0/1/3 长度 locator。完成后项目
共有 196 个单元测试和 32 个固定 Java Golden 差分测试（2,694 条记录）；
`cargo-llvm-cov` 统计 10,454 行、1,219 个函数和 16,105 个区域，三项均为 100%。

### 12.29 `TextParseException`

该对象是 text parser 自有的 checked exception，不属于
`TemplateEngineException` 运行时异常继承树。Rust 使用 `std::error::Error` 与
`TextParseCause` 组合保留 Java Throwable 元数据：

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `TextParseException()` | `new()` / `Default` | 消息、原因、行列全部为 null | `BEHAVIOR_VERIFIED` |
| `TextParseException(String)` | `with_message(message)` | null 与任意 UTF-16（含孤立代理项）原样保存 | `BEHAVIOR_VERIFIED` |
| `TextParseException(String,Throwable)` | `with_message_and_cause(message,cause)` | 普通 cause 使用显式消息或 cause 消息；带位置的同类型 cause 传播行列并重组前缀 | `BEHAVIOR_VERIFIED` |
| `TextParseException(Throwable)` | `with_cause(cause)` | 等价于 null 显式消息的消息/cause 构造器 | `BEHAVIOR_VERIFIED` |
| `TextParseException(int,int)` | `with_location(line,col)` | 生成固定位置前缀，负数和 `i32::MIN` 不规范化 | `BEHAVIOR_VERIFIED` |
| `TextParseException(String,Throwable,int,int)` | `with_message_and_cause_at(...)` | 显式位置覆盖 cause 元数据；消息 null 按 Java `+` 拼接为 `"null"` | `BEHAVIOR_VERIFIED` |
| `TextParseException(String,int,int)` | `with_message_at(message,line,col)` | 同上但无 cause | `BEHAVIOR_VERIFIED` |
| `TextParseException(Throwable,int,int)` | `with_cause_at(cause,line,col)` | 消息仅为位置前缀，不追加 cause 消息 | `BEHAVIOR_VERIFIED` |
| private `messagePrefix(int,int)` | `message_prefix(line,col)` | 精确生成 `(Line = x, Column = y)` | `BEHAVIOR_VERIFIED` |
| private `message(String,Throwable)` | `compose_inherited_message(...)` | 保留普通/同类型、显式/null 消息的全部分支及嵌套前缀无空格行为 | `BEHAVIOR_VERIFIED` |
| `Integer getLine()` | `get_line()` | 显式或继承行号；缺失为 `None` | `BEHAVIOR_VERIFIED` |
| `Integer getCol()` | `get_col()` | 显式或继承列号；缺失为 `None` | `BEHAVIOR_VERIFIED` |

`TextParseCause` 保存 Java 类名、可空 UTF-16 `getMessage()`、底层 Rust error 分配和
同类型位置元数据；`Error::source()` 返回同一底层分配。27 条固定 Java Golden
覆盖全部构造器、null/孤立代理项消息、普通与同类型 cause、显式负位置、嵌套位置
传播及 source 身份。完成后项目共有 197 个单元测试和 33 个固定 Java Golden
差分测试（2,721 条记录）；`cargo-llvm-cov` 统计 10,648 行、1,250 个函数和
16,344 个区域，三项均为 100%。

### 12.30 `ITextHandler` / `TextParsingCommentUtil` / `TextParsingLiteralUtil`

该切片建立 text parser 的事件回调边界，并迁移 JavaScript/CSS 风格注释与正则
字面量起点判断。`ITextHandler` 使用 `&mut dyn ITextHandler` 保留有状态动态分派；
`char[]` 映射为可空的 `Option<&mut [u16]>`，处理器修改对解析器继续可见：

| Java 签名/方法族 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `ITextHandler` 11 个事件方法 | 同名 snake_case trait 方法 | 文档、文本、注释、三类元素边界与属性的全部参数原序透传；可空 `char[]` 用 `Option` 保留；checked exception 使用 `Box<TextParseException>` 原样传播 | `BEHAVIOR_VERIFIED` |
| `parseComment(char[],int,int,int,int,ITextHandler)` | `parse_comment(buffer,offset,len,line,col,handler)` | 严格校验 `/*...*/`；content/outer 双范围、行列和数组修改可见性保持一致 | `BEHAVIOR_VERIFIED` |
| `isCommentBlockStart/End` | `is_comment_block_start/end` | 两代码单元短路、null/越界顺序及 `i32` 差值回绕保持一致 | `BEHAVIOR_VERIFIED` |
| `isCommentLineStart` | `is_comment_line_start` | `//` 判定与 Java 数组运行时错误保持一致 | `BEHAVIOR_VERIFIED` |
| `isRegexLiteralStart` | `is_regex_literal_start` | `/` 前一非空白字符仅允许 `(`、`=`、`,`；排除两类注释；使用完整 BMP `Character.isWhitespace(char)` | `BEHAVIOR_VERIFIED` |

`TextParsingCommentError` 仅作为 Rust/JVM 边界适配，区分
`TextParseException`、增强 NPE、数组下标和 `String(char[],int,int)` 范围异常。
Java Oracle 固定 40 条记录，覆盖全部 11 个接口回调、动态分派、注释 content/outer
范围、handler 修改与失败、null/负数/溢出及谓词矩阵；正则判定对 65,536 个 BMP
代码单元执行哈希差分。完成后项目共有 204 个单元测试和 34 组固定 Java Oracle
（2,761 条已登记记录）；`cargo-llvm-cov` 统计 11,158 行、1,289 个函数和
16,987 个区域，三项均为 100%。

### 12.31 `TextParsingUtil`

该 package-private 工具对象是文本模板解析器的通用 UTF-16 扫描层。Rust 保持
`pub(crate)` 可见性，所有偏移、上界、行列与反斜杠计数均使用 Java `int` 回绕，
不把 Java `char[]` 转成 Unicode 标量或 UTF-8 字节：

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| private `TextParsingUtil()` | 无公开构造入口 | 零状态静态工具对象保持不可实例化意图 | `BEHAVIOR_VERIFIED` |
| `findNextStructureEndAvoidQuotes(char[],int,int,int[])` | `find_next_structure_end_avoid_quotes(text,offset,maxi,locator)` | 跳过单双引号内的 `]`；命中字符不计入 locator，LF 先清列再增行 | `BEHAVIOR_VERIFIED` |
| `findNextCommentBlockEnd(char[],int,int,int[])` | `find_next_comment_block_end(...)` | 返回 `*/` 中 `/` 的位置；首字符 `/` 不与范围外字符配对 | `BEHAVIOR_VERIFIED` |
| `findNextCommentLineEnd(char[],int,int,int[])` | `find_next_comment_line_end(...)` | 返回行尾 LF 且不消费 LF；未命中时只增加扫描列数 | `BEHAVIOR_VERIFIED` |
| `findNextLiteralEnd(char[],int,int,int[],char)` | `find_next_literal_end(...,literal_marker)` | 仅偶数个连续反斜杠后的 marker 闭合；首个 marker 不是结束符 | `BEHAVIOR_VERIFIED` |
| `findNextStructureStartOrLiteralMarker(char[],int,int,int[],boolean)` | `find_next_structure_start_or_literal_marker(...,process_comments_and_literals)` | 始终识别 `[`；启用时额外识别 `/` 及未转义的单/双/反引号 | `BEHAVIOR_VERIFIED` |
| private `isLiteralDelimiter(char[],int,int)` | 私有 `is_literal_delimiter(text,offset,index)` | 从 marker 向前计数连续反斜杠并按奇偶判断，不越过扫描起点 | `BEHAVIOR_VERIFIED` |
| `findNextWhitespaceCharWildcard(char[],int,int,boolean,int[])` | `find_next_whitespace_char_wildcard(...,avoid_quotes,locator)` | 精确复现 `Character.isWhitespace(char)` 的完整 BMP 集；可忽略单双引号内部空白 | `BEHAVIOR_VERIFIED` |
| `findNextNonWhitespaceCharWildcard(char[],int,int,int[])` | `find_next_non_whitespace_char_wildcard(...)` | 消费连续 Java 空白并返回首个非空白字符 | `BEHAVIOR_VERIFIED` |
| `findNextOperatorCharWildcard(char[],int,int,int[])` | `find_next_operator_char_wildcard(...)` | `=` 或 Java 空白均为运算符边界 | `BEHAVIOR_VERIFIED` |
| `findNextNonOperatorCharWildcard(char[],int,int,int[])` | `find_next_non_operator_char_wildcard(...)` | 消费 `=` 与 Java 空白，返回首个其他代码单元 | `BEHAVIOR_VERIFIED` |
| `findNextAnyCharAvoidQuotesWildcard(char[],int,int,int[])` | `find_next_any_char_avoid_quotes_wildcard(...)` | 非引号首字符立即返回；引号首字符则消费完整同类引号范围，并返回闭合符之后的位置 | `BEHAVIOR_VERIFIED` |

`TextParsingUtilError` 精确区分 null text、前五个方法直接访问 locator 时
`<parameter4>` 增强 NPE、后五个方法经定位工具访问时 `<parameter1>` 增强 NPE，
以及 text/locator 数组越界；短 locator 的失败位置和已发生副作用按 Java 语句顺序
保留。

固定 Java Oracle 生成 87 条记录，覆盖十个扫描入口、引号交叉、LF、缺失标记、
范围扫描、奇偶转义、null、负 offset、短 locator 与失败后状态；四类空白/运算符
扫描穷举全部 65,536 个 BMP 代码单元，字面量分隔符覆盖 0–12 个连续反斜杠。
Oracle 的完整 BMP 与 delimiter 哈希分别为 `4339812b49bb7979` 和
`bc554bd3a22c3471`。完成后项目共有 206 个单元测试和 35 组固定 Java Oracle
（2,848 条已登记记录）；新文件的行、函数和区域覆盖率均为 100%。

### 12.32 `TextParsingAttributeSequenceUtil`

该对象解析文本模式元素内部的属性序列，是
`TextParsingElementUtil` 的直接依赖。Rust 保持 package-private 对象的
`pub(crate)` 可见性，不额外公开解析 API：

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| private `TextParsingAttributeSequenceUtil()` | 无公开构造入口 | 无状态静态工具对象保持不可实例化意图 | `BEHAVIOR_VERIFIED` |
| `parseAttributeSequence(char[],int,int,int,int,ITextHandler)` | `parse_attribute_sequence(buffer,offset,len,line,col,handler)` | 按空白、名称、operator、value 四阶段循环；无等号属性、空值、operator 内空白、单双引号 outer/content、相邻引号值、行列更新和 `int` 回绕完整保持；buffer/handler 的 Java null 以 `Option` 表达 | `BEHAVIOR_VERIFIED` |
| private `isValueSurroundedByCommas(char[],int,int)` | 私有 `value_content_range(buffer,offset,len)` | 上游方法名虽写作 Commas，实际只在长度至少 2 且首尾同为双引号或同为单引号时去掉两端各一个代码单元 | `BEHAVIOR_VERIFIED` |

处理器回调收到同一可变 UTF-16 缓冲区，首次回调的修改会影响后续属性扫描；每个
事件完整透传 name/operator/value content/value outer 的 offset、len 和三组行列。
`TextParsingAttributeSequenceError` 保留属性名非法及 handler checked exception，
包装通用扫描器的 null/数组越界异常，并精确表达仅在实际事件派发时才触发的
`<parameter6>` handler 增强 NPE。Java handler 的 unchecked exception 对应 Rust
实现的 panic 穿透，差分测试用捕获边界验证其不会被解析器吞掉或改写。

固定 Java Oracle 生成 56 条记录：可读场景覆盖空/纯空白、无值、无 operator、
多属性、单双引号、未闭合引号、NUL/孤立代理项、换行与整数溢出；异常场景分别
命中四种事件派发位置、handler 修改/checked/unchecked 失败、null、负数、范围
回绕及扫描越界。四个确定性哈希进一步覆盖完整 65,536 BMP 空白组合、完整 BMP
引号内容、2,625 组属性语法组合和 399 组 offset/len 范围，结果分别为
`2dd2a90b2e45e804`、`ac36c5f7eab3b4d4`、`c893a85c8bce6bee` 和
`f6c5fbd69f9413da`。完成后项目共有 208 个单元测试和 36 组固定 Java Oracle
（2,904 条已登记记录）；新文件 838 行、47 个函数、908 个区域，覆盖率均为
100%。

### 12.33 `TextParsingElementUtil`

该对象识别并解析文本模式的 open、standalone 和 close 元素，是 `TextParser`
与 `CommentProcessorTextHandler` 的共享依赖。Rust 保持 package-private
对象的 `pub(crate)` 可见性，不扩大公共 API：

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| private `TextParsingElementUtil()` | 无公开构造入口 | 无状态静态工具对象保持不可实例化意图 | `BEHAVIOR_VERIFIED` |
| `parseStandaloneElement(char[],int,int,int,int,ITextHandler)` | `parse_standalone_element(buffer,offset,len,line,col,handler)` | 校验 `[#.../]`，依次派发 standalone start、属性和 end；minimized 固定为 true | `BEHAVIOR_VERIFIED` |
| `parseOpenElement(char[],int,int,int,int,ITextHandler)` | `parse_open_element(buffer,offset,len,line,col,handler)` | 校验 `[#...]`，保留无名元素、引号内空白/`]`、属性事件和最终 locator | `BEHAVIOR_VERIFIED` |
| `parseCloseElement(char[],int,int,int,int,ITextHandler)` | `parse_close_element(buffer,offset,len,line,col,handler)` | 校验 `[/...]`；名称之后仅允许 Java 空白，非法属性在 start 事件后失败 | `BEHAVIOR_VERIFIED` |
| `isOpenElementStart(char[],int,int)` | `is_open_element_start(buffer,offset,maxi)` | 短路识别 `[#`、空白/无名结束符及允许的名称首 UTF-16 代码单元 | `BEHAVIOR_VERIFIED` |
| `isCloseElementStart(char[],int,int)` | `is_close_element_start(buffer,offset,maxi)` | 短路识别 `[/`，其余名称规则与 open 元素一致 | `BEHAVIOR_VERIFIED` |
| `isElementEnd(char[],int,int,boolean)` | `is_element_end(buffer,offset,maxi,minimized)` | 普通模式精确匹配 `]`；minimized 模式精确匹配 `/]` | `BEHAVIOR_VERIFIED` |
| private `isElementNameOrEnd(char[],int,int)` | 私有 `is_element_name_or_end(buffer,offset,maxi)` | 保留完整 Java BMP 空白及 `- ! / ? [ {` 六个禁用名称首字符，并允许无名元素结束符 | `BEHAVIOR_VERIFIED` |

三个解析入口保持 Java 的验证和错误消息构造短路顺序：短输入会在构造
`String(char[],offset,len)` 时触发 null/范围错误，合法前缀的数组访问、结束符
访问和通用扫描错误类别不被折叠。handler start 早于属性扫描，因而其缓冲区修改
对后续属性与 locator 可见；checked exception 原样传播，unchecked exception
对应 Rust panic 穿透。close 元素的非法属性异常保留失败前 start 事件和修改后的
UTF-16 原文。

固定 Java Oracle 生成 99 条记录，覆盖三类合法/非法元素、无名元素、属性、
单双引号、内部 `]`、多行定位、整数回绕、NUL/孤立代理项、handler 修改、
checked/runtime/null handler 以及 null/越界/回绕范围。四个确定性哈希进一步
覆盖全部 65,536 个 BMP 名称首代码单元、谓词 offset/maxi 矩阵、108 组标签语法
组合和三类解析范围矩阵，结果分别为 `fe012ae44a0e1845`、
`c2c47d81fbe05561`、`65907becd0b43adb` 和 `75e4dfa7efa82dfb`。完成后项目共有
210 个单元测试和 37 组固定 Java Oracle（3,003 条已登记记录）；新文件 1,515
行、82 个函数、1,866 个区域，覆盖率均为 100%。Oracle JVM 显式关闭
`OmitStackTraceInFastThrow`，避免大量重复数组异常被 HotSpot 无消息快抛路径
替换而造成非确定性哈希。

### 12.34 `AbstractTextHandler` / `AbstractChainedTextHandler`

这两个对象建立 text parser 的 handler 继承/组合基线。Java 用抽象类提供默认
空操作和同步转发；Rust 不伪造类继承，而是保留同名对象并用 `ITextHandler`
trait + 拥有下游 trait object 的组合实现相同可观察行为。Java 可空 `char[]`
统一映射为 `Option<&mut [u16]>`，因此默认处理器对 null buffer 的不读取语义也
可表达；解析器产生的真实事件继续传入 `Some`。

#### `AbstractTextHandler`

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| protected `AbstractTextHandler()` | `AbstractTextHandler::new()` / `Default` | Java 抽象基类的 protected 构造映射为可组合对象；私有标记避免退化为零大小单例 | `BEHAVIOR_VERIFIED` |
| `handleDocumentStart(long startTimeNanos,int line,int col)` | `handle_document_start(start_time_nanos,line,col)` | 不读取任何参数，始终返回成功 | `BEHAVIOR_VERIFIED` |
| `handleDocumentEnd(long endTimeNanos,long totalTimeNanos,int line,int col)` | `handle_document_end(end_time_nanos,total_time_nanos,line,col)` | 不读取时间或位置，始终返回成功 | `BEHAVIOR_VERIFIED` |
| `handleText(char[] buffer,int offset,int len,int line,int col)` | `handle_text(buffer,offset,len,line,col)` | `buffer=None`、非法范围和极值位置均不读取、不修改 | `BEHAVIOR_VERIFIED` |
| `handleComment(char[] buffer,int contentOffset,int contentLen,int outerOffset,int outerLen,int line,int col)` | `handle_comment(buffer,content_offset,content_len,outer_offset,outer_len,line,col)` | content/outer 参数全部忽略且无副作用 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementStart(char[] buffer,int nameOffset,int nameLen,boolean minimized,int line,int col)` | `handle_standalone_element_start(...)` | buffer、名称、minimized 和位置均不读取 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementEnd(char[] buffer,int nameOffset,int nameLen,boolean minimized,int line,int col)` | `handle_standalone_element_end(...)` | 默认空操作 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementStart(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_open_element_start(...)` | 默认空操作 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementEnd(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_open_element_end(...)` | 默认空操作 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementStart(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_close_element_start(...)` | 默认空操作 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementEnd(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_close_element_end(...)` | 默认空操作 | `BEHAVIOR_VERIFIED` |
| `handleAttribute(char[] buffer,...)` | `handle_attribute(buffer,...)` | 14 个 offset/len/line/col 参数及可空 buffer 全部不读取 | `BEHAVIOR_VERIFIED` |

#### `AbstractChainedTextHandler`

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| protected `AbstractChainedTextHandler(ITextHandler next)` | `new(next: Option<Box<dyn ITextHandler>>)` | 构造时不校验 null；Box 保留下游对象唯一所有权 | `BEHAVIOR_VERIFIED` |
| protected `ITextHandler getNext()` | `get_next() -> Option<&mut dyn ITextHandler>` | 返回构造时同一对象；Rust 可变借用映射同步回调接收者 | `BEHAVIOR_VERIFIED` |
| `handleDocumentStart(long startTimeNanos,int line,int col)` | `handle_document_start(start_time_nanos,line,col)` | 三个参数原序同步转发 | `BEHAVIOR_VERIFIED` |
| `handleDocumentEnd(long endTimeNanos,long totalTimeNanos,int line,int col)` | `handle_document_end(end_time_nanos,total_time_nanos,line,col)` | 四个参数原序同步转发 | `BEHAVIOR_VERIFIED` |
| `handleText(char[] buffer,int offset,int len,int line,int col)` | `handle_text(buffer,offset,len,line,col)` | 同一可空 buffer 和全部范围/位置原样转发 | `BEHAVIOR_VERIFIED` |
| `handleComment(char[] buffer,int contentOffset,int contentLen,int outerOffset,int outerLen,int line,int col)` | `handle_comment(...)` | content/outer 双范围及位置原样转发 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementStart(char[] buffer,int nameOffset,int nameLen,boolean minimized,int line,int col)` | `handle_standalone_element_start(...)` | buffer 修改、minimized 和位置对调用方继续可见 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementEnd(char[] buffer,int nameOffset,int nameLen,boolean minimized,int line,int col)` | `handle_standalone_element_end(...)` | 全参数原样转发 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementStart(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_open_element_start(...)` | 全参数原样转发 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementEnd(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_open_element_end(...)` | 全参数原样转发 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementStart(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_close_element_start(...)` | 全参数原样转发 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementEnd(char[] buffer,int nameOffset,int nameLen,int line,int col)` | `handle_close_element_end(...)` | 全参数原样转发 | `BEHAVIOR_VERIFIED` |
| `handleAttribute(char[] buffer,...)` | `handle_attribute(buffer,...)` | 同一 buffer 及 14 个属性位置参数原样转发 | `BEHAVIOR_VERIFIED` |

所有转发入口都不捕获下游失败：`Box<TextParseException>` 保持分配身份；下游先
修改 buffer 再失败时修改仍可见；unchecked exception 对应 Rust panic payload
原样穿透。`next == null` 只在具体回调发生时触发，
`ChainedTextHandlerRuntimeError` 精确保留 11 种 Java 17 增强 NPE 的方法签名和
`this.next` 字段表达式。

固定 Java Oracle 生成 39 条记录，覆盖两个对象全部 25 个构造器/方法声明、
null/非 null buffer、极值参数、11 个成功转发、11 个 checked exception 身份、
11 个 runtime exception 身份、失败前 buffer 修改及 11 种 null-next NPE。
完成后项目共有 212 个单元测试和 38 组固定 Java Oracle（3,042 条已登记记录）；
两个新增文件合计 724 行、66 个函数、649 个区域，覆盖率均为 100%。

### 12.35 `EventProcessorTextHandler`

该对象位于 `TextParser` 与最终业务 handler 之间，先完成元素嵌套、属性唯一性
和结构名称驻留。Rust 使用同名组合对象承接 Java 继承，并将紧耦合内部类
`StructureNamesRepository` 保留在同一文件：

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `EventProcessorTextHandler(ITextHandler handler)` | `new(handler: Option<Box<dyn ITextHandler>>)` | 初始元素栈逻辑长度 10，名称仓库逻辑长度 20；null handler 延迟到实际转发失败 | `BEHAVIOR_VERIFIED` |
| `handleDocumentEnd(long,long,int,int)` | `handle_document_end(...)` | 栈非空时先弹出顶部，再以无行列异常报告未闭合元素；栈空才转发 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementStart(char[],int,int,boolean,int,int)` | `handle_standalone_element_start(...)` | 转发前清空当前属性名，不入元素栈；下游失败后清空状态仍保留 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementStart(char[],int,int,int,int)` | `handle_open_element_start(...)` | 先清空属性、再转发、成功后才缓存修改后的名称并入栈 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementStart(char[],int,int,int,int)` | `handle_close_element_start(...)` | 先精确匹配并弹栈，再清空属性并转发；不匹配时保留原栈 | `BEHAVIOR_VERIFIED` |
| `handleAttribute(char[],...)` | `handle_attribute(buffer,...)` | 名称大小写敏感去重；逻辑数组按 3 增长；先驻留和登记、再转发 | `BEHAVIOR_VERIFIED` |
| private `checkStackForElement(char[],int,int,int,int)` | 私有 `check_stack_for_element(...)` | 空栈、无名顶部、名称不匹配和成功弹栈四种结果及行列一致 | `BEHAVIOR_VERIFIED` |
| private `pushToStack` / `peekFromStack` / `popFromStack` / `growStack` | 私有 `push_to_stack` / `peek_from_stack` / `pop_from_stack` / `grow_stack` | 栈按 10 增长；名称使用仓库同一身份分配 | `BEHAVIOR_VERIFIED` |
| `StructureNamesRepository()` | `StructureNamesRepository::new()` | 非线程安全、单次执行仓库；初始逻辑长度 20 | `BEHAVIOR_VERIFIED` |
| `getStructureName(char[],int,int)` | `get_structure_name(text,offset,len)` | 对 UTF-16 代码单元做精确二分查找；同名返回同一 `Rc<[u16]>` 分配 | `BEHAVIOR_VERIFIED` |
| private `storeStructureName(int,char[],int,int)` | 私有 `store_structure_name(...)` | 复制来源范围、按字典序插入，仓库满时按 5 增长 | `BEHAVIOR_VERIFIED` |

`Rc<[u16]>` 同时表达 Java 缓存 `char[]` 的共享身份和该对象明确的非线程安全
生命周期。内部精确比较只迁移本对象实际调用的 `TextUtils.equals/binarySearch`
语义，不把尚未迁移的完整 `TextUtils` 误标为完成。运行时 null、负长度和
数组范围失败由 `EventProcessorTextHandlerRuntimeError` 保存 Java 类名及固定
JDK 消息；结构错误继续返回 `Box<TextParseException>`。

固定 Java Oracle 生成 48 条记录，覆盖正常嵌套、空栈/错配/无名元素、文档结束
逐层排空、属性大小写和重复、栈/属性/仓库增长、缓存身份与源数组复制、下游
checked failure 后的部分状态，以及 null/负数/越界运行时边界。Rust 逐行匹配
全部记录。完成后项目共有 214 个单元测试和 39 组固定 Java Oracle（3,090 条
已登记记录）；全量工作区行、函数、区域覆盖率继续保持 100%。

### 12.36 `CommentProcessorTextHandler`

该对象位于 JavaScript/CSS 文本解析链中，将可处理注释还原为 Thymeleaf 元素
事件或标准方言内联表达式，并延迟过滤表达式后的自然模板文本。Rust 使用同名
组合对象承接 Java 继承；`CommentProcessorTextHandlerRuntimeError` 只承担
JVM 未检查异常类别、可空消息与 panic payload，不计作新的 Java 主对象。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `CommentProcessorTextHandler(boolean,ITextHandler)` | `new(standard_dialect_present, handler)` | 保存标准方言标志；过滤状态、大小和两个延迟缓冲对象按 Java 初值建立；null handler 延迟失败 | `BEHAVIOR_VERIFIED` |
| `handleDocumentEnd(long,long,int,int)` | `handle_document_end(...)` | 先刷新过滤文本，成功后才转发文档结束；checked failure 阻止后续事件 | `BEHAVIOR_VERIFIED` |
| `handleComment(char[],int,int,int,int,int,int)` | `handle_comment(...)` | 先刷新前一表达式；普通注释按 outer 范围作为 text；元素注释解包；标准表达式仅转发 content 并开启过滤，非标准模式转发 outer | `BEHAVIOR_VERIFIED` |
| private `isCommentProcessable(char[],int,int)` | 私有 `is_comment_processable(...)` | 保留长度短路、两类表达式边界、open/close 元素谓词调用顺序及 Java 数组异常 | `BEHAVIOR_VERIFIED` |
| `handleText(char[],int,int,int,int)` | `handle_text(...)` | 过滤开启时复制 UTF-16 范围并覆盖最新 locator；否则同步透传 | `BEHAVIOR_VERIFIED` |
| private `filterText(char[],int,int,int,int)` | 私有 `filter_text(...)` | 首次容量为 `max(256,len)`；增长为 `max(old+256,size+len)`；复制后才增加 size 并写 locator | `BEHAVIOR_VERIFIED` |
| private `processFilteredTexts()` | 私有 `process_filtered_texts()` | 计算过滤偏移并仅转发余量；只有成功后才清零 size/filter，失败保留状态且 locator 的已发生更新不回滚 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementStart(char[],int,int,boolean,int,int)` | `handle_standalone_element_start(...)` | 先刷新过滤文本，再按原参数转发；end 事件不触发刷新 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementStart(char[],int,int,int,int)` | `handle_open_element_start(...)` | 先刷新过滤文本，再按原参数转发；open end 不触发刷新 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementStart(char[],int,int,int,int)` | `handle_close_element_start(...)` | 先刷新过滤文本，再按原参数转发；close end 不触发刷新 | `BEHAVIOR_VERIFIED` |
| static `computeFilterOffset(char[],int,int,int[])` | 私有 `compute_filter_offset(...)` | 单/双引号、仅直接前导反斜杠转义、对象/数组嵌套及顶层 LF、`; , ) } ]`、`//` 终止规则逐 UTF-16 单元一致；终止符不计入 locator | `BEHAVIOR_VERIFIED` |

所有未覆写的 document start、三类 element end 和 attribute 继续经
`AbstractChainedTextHandler` 原样转发，不会意外刷新过滤文本。过滤 buffer
复制保持来源数组独立性；`System.arraycopy` 的 null source、负长度、源/目标
起点和末端越界使用 HotSpot 类名与消息；直接数组读取、短 locator、整数回绕和
三种 null-next 调用点也与固定 Java 17 Oracle 一致。

固定 Java Oracle 生成 110 条记录，覆盖普通/元素/表达式注释、五种刷新和四种
非刷新入口、13 组分隔符/引号/嵌套、空与多块过滤、256/512/800 容量增长、
来源复制独立性、checked failure 重试状态、直接 `computeFilterOffset`、
`System.arraycopy` 和 null/负数/越界/溢出边界。Rust 逐行匹配全部记录。
完成后机器清单累计验证 471 个构造器/方法，项目共有 216 个单元测试和 40 组
固定 Java Oracle（3,200 条已登记记录）；`cargo-llvm-cov` 统计 18,040 行、
1,724 个函数和 24,817 个区域，三项均为 100%。

### 12.19 `TextParser` 与内部 `BufferPool`

`TextParser` 是 TEXT、JAVASCRIPT 和 CSS 解析链的 UTF-16 流式扫描核心。迁移保持
Java `char[]`、`int` 回绕、Reader 生命周期、handler checked exception、
RuntimeException/Error 分类、未完成结构搬移和缓冲池数组身份，不把实现简化为
一次性 Rust `String` 解析。`BufferPool` 作为 Java 紧耦合内部对象与主对象保留在
同一个 `text_parser.rs` 文件。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `TextParser(int,int,boolean,boolean)` | `TextParser::new(pool_size, buffer_size, process_comments_and_literals, standard_dialect_present)` | 按 Java 顺序创建固定槽位池；负池大小/缓冲大小保留 JVM 数组异常；零池合法 | `BEHAVIOR_VERIFIED` |
| `parse(String,ITextHandler)` | `parse(document, handler)` | 先校验 document；UTF-16 `StringReader` 进入 Reader 重载；事件与注释 handler 按标志装配 | `BEHAVIOR_VERIFIED` |
| `parse(Reader,ITextHandler)` | `parse_reader(reader, handler)` | reader、handler 按 Java 顺序校验；Reader 动态调用和 checked failure 不被字符串化 | `BEHAVIOR_VERIFIED` |
| `parseDocument(Reader,int,ITextHandler)` | `parse_document(reader, suggested_buffer_size, handler)` | 所有出口释放池缓冲并尝试 close；close 的 checked/runtime/未知 panic 均忽略；未知业务 panic 清理后继续传播 | `BEHAVIOR_VERIFIED` |
| private `parseBuffer(char[],int,int,ITextHandler,TextParseStatus)` | 私有 `parse_buffer(...)` | 逐 UTF-16 单元维护 offset/line/col，识别 open/close/standalone、块/行注释、引号和正则字面量；handler checked error原样返回 | `BEHAVIOR_VERIFIED` |
| `BufferPool(int,int)` | `BufferPool::new(pool_size, pool_buffer_size)` | 预分配默认大小数组槽并保留 Java 构造边界 | `BEHAVIOR_VERIFIED` |
| `allocateBuffer(int)` | `allocate_buffer(buffer_size)` | 默认大小按槽位顺序非阻塞取用并保持数组身份；池耗尽或大小不同创建非池化数组 | `BEHAVIOR_VERIFIED` |
| `releaseBuffer(char[])` | `release_buffer(allocated)` | 仅将来自原槽且大小匹配的数组放回；null、非池化和不同大小不改变池 | `BEHAVIOR_VERIFIED` |

固定 Java Golden 共 244 条记录，覆盖 16 组文档与两个布尔配置、UTF-16 代理项、
5 组文档在 1–96 缓冲大小下的 960 种切分、Reader 零读取/失败/关闭、handler
checked/runtime failure、不完整结构、池身份复用与构造边界。Rust 另有
`RUST_OBLIGATION`/`VALUE_ADD` 测试验证 panic taxonomy、Mutex 中毒恢复、
Java `int` 扩容回绕及未知 panic 的 finally 清理。

完成后机器清单累计验证 479 个构造器/方法，项目共有 226 个单元测试和 41 组
固定 Java Oracle（3,444 条已登记记录）；`cargo-llvm-cov` 统计 20,027 行、
1,888 个函数和 27,451 个区域，三项均为 100%。

### 12.20 `BlockAwareReader` 与文本注释 Reader

`BlockAwareReader` 是 package-private 的 UTF-16 流式状态机；内部 `BlockAction`
与主对象保留在 `block_aware_reader.rs`。两个公开文本 Reader 分别固定定界符和
丢弃策略，并通过同一个 `TextParserReader` trait 允许继续嵌套。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| protected `BlockAwareReader(Reader,BlockAction,char[],char[])` | crate-private `BlockAwareReader::new(reader, action, prefix, suffix)` | 保存底层动态 Reader、UTF-16 定界符首字符及独立状态 | `BEHAVIOR_VERIFIED` |
| `read(char[],int,int)` | `TextParserReader::read_range(buffer, offset, len)` | 原位删除容器；跨调用保留 overflow/index/discard 状态；零读取与 EOF 返回值一致 | `BEHAVIOR_VERIFIED` |
| private `readBytes(...)` | 私有 `read_bytes(...)` | overflow 优先，剩余调用委托 Reader；委托失败原样传播 | `BEHAVIOR_VERIFIED` |
| private `overflowLastBytes(...)` | 私有 `overflow_last_bytes(...)` | 缓冲末尾候选前置到固定长度 overflow buffer | `BEHAVIOR_VERIFIED` |
| private `matchOverflow(...)` | 私有 `match_overflow(...)` | 每次只向委托请求一个 UTF-16 code unit，完整匹配或保留首个非匹配单元 | `BEHAVIOR_VERIFIED` |
| `close()` | `TextParserReader::close()` | 只委托关闭，保留底层异常类名和消息 | `BEHAVIOR_VERIFIED` |
| `ParserLevelCommentTextReader(Reader)` | `ParserLevelCommentTextReader::new(reader)` | 固定 `/*[-` / `-]*/` 与 `DiscardAll` | `BEHAVIOR_VERIFIED` |
| `PrototypeOnlyCommentTextReader(Reader)` | `PrototypeOnlyCommentTextReader::new(reader)` | 固定 `/*[+` / `+]*/` 与 `DiscardContainer` | `BEHAVIOR_VERIFIED` |

Java Golden 生成 120 条记录，逐条固定输出、每次 `read` 返回序列、异常和关闭次数。
Rust 直接迁移两个上游测试文件的 4 个测试方法：`test01` 的结构位置生成器覆盖两类
定界符，`test02` 的全部手写 case 使用原始 buffer/len/offset 三重循环。另有
`RUST_OBLIGATION` 覆盖 UTF-16 代理项、trait 默认方法及两个 overflow 委托失败点。

完成后机器清单累计验证 487 个构造器/方法，项目共有 230 个单元测试和 42 组
固定 Java Oracle（3,564 条已登记记录）；`cargo-llvm-cov` 统计 20,835 行、
1,925 个函数和 28,544 个区域，三项均为 100%。

### 12.21 两个 Markup comment Reader

两个对象复用 `BlockAwareReader` 的真实状态机，不重复或简化其 overflow 逻辑。
它们只固定 Thymeleaf markup 模式所需的更长定界符与丢弃动作。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `ParserLevelCommentMarkupReader(Reader)` | `ParserLevelCommentMarkupReader::new(reader)` | 固定 `<!--/*` / `*/-->` 与 `DiscardAll`；继承动态 read/close 合同 | `BEHAVIOR_VERIFIED` |
| `PrototypeOnlyCommentMarkupReader(Reader)` | `PrototypeOnlyCommentMarkupReader::new(reader)` | 固定 `<!--/*/` / `/*/-->` 与 `DiscardContainer`；可作为 parser-level Reader 的内层 | `BEHAVIOR_VERIFIED` |

两个上游测试文件的 `test01` 生成器与 `test02` 全部人工 case 已迁移；人工 case
保留原始 buffer/len/offset 三重循环。固定 Java Golden 另生成 61 条记录，覆盖
逐次 read 返回序列、双层嵌套、UTF-16、未闭合结构和 close 异常。

完成后机器清单累计验证 489 个构造器/方法，项目共有 234 个单元测试和 43 组
固定 Java Oracle（3,625 条已登记记录）；`cargo-llvm-cov` 统计 21,223 行、
1,947 个函数和 29,016 个区域，三项均为 100%。

### 12.37 `IInlinePreProcessorHandler`

该对象是 text/markup parser 内联表达式转换链的同步事件 SPI，也是
`OutputExpressionInlinePreProcessorHandler` 与 text/markup 适配器之间的动态分派
边界。Rust 以同名 trait 保留完整事件面，不添加 CDATA/注释回调；Java 源码明确要求
这两类带定界符事件留到 Processor 执行阶段，避免预处理拆分后破坏语法。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `handleText(char[],int,int,int,int)` | `handle_text(buffer,offset,len,line,col)` | `char[]` 映射 UTF-16 `Option<&mut [u16]>`；范围和 locator 原序交给实现 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementStart(char[],int,int,boolean,int,int)` | `handle_standalone_element_start(...)` | 同一可变 buffer、名称范围、minimized 和位置同步动态分派 | `BEHAVIOR_VERIFIED` |
| `handleStandaloneElementEnd(char[],int,int,boolean,int,int)` | `handle_standalone_element_end(...)` | 全参数名称、顺序和可空性保持一致 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementStart(char[],int,int,int,int)` | `handle_open_element_start(...)` | 开放元素开始事件原序分派 | `BEHAVIOR_VERIFIED` |
| `handleOpenElementEnd(char[],int,int,int,int)` | `handle_open_element_end(...)` | 开放元素结束事件原序分派 | `BEHAVIOR_VERIFIED` |
| `handleAutoOpenElementStart(char[],int,int,int,int)` | `handle_auto_open_element_start(...)` | 保留自动补出事件，不与普通 open 合并 | `BEHAVIOR_VERIFIED` |
| `handleAutoOpenElementEnd(char[],int,int,int,int)` | `handle_auto_open_element_end(...)` | 保留自动补出结束事件 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementStart(char[],int,int,int,int)` | `handle_close_element_start(...)` | 关闭元素开始事件原序分派 | `BEHAVIOR_VERIFIED` |
| `handleCloseElementEnd(char[],int,int,int,int)` | `handle_close_element_end(...)` | 关闭元素结束事件原序分派 | `BEHAVIOR_VERIFIED` |
| `handleAutoCloseElementStart(char[],int,int,int,int)` | `handle_auto_close_element_start(...)` | 保留自动补出事件，不与普通 close 合并 | `BEHAVIOR_VERIFIED` |
| `handleAutoCloseElementEnd(char[],int,int,int,int)` | `handle_auto_close_element_end(...)` | 保留自动补出结束事件 | `BEHAVIOR_VERIFIED` |
| `handleAttribute(char[],...)` | `handle_attribute(buffer,...)` | 同一 buffer 及 name/operator/value content/value outer 的 14 个范围与位置参数原序分派 | `BEHAVIOR_VERIFIED` |

固定 Java Golden 使用记录型接口实现，经 `IInlinePreProcessorHandler` 动态引用调用
全部 12 个事件，并分别输入非空 UTF-16 数组与 Java `null`。25 条记录固定参数顺序、
布尔值、孤立代理项和回调对原数组的修改；Rust 通过
`dyn IInlinePreProcessorHandler` 逐条匹配。上游没有直接针对该纯接口的 JUnit，
因此测试台账将其登记为 `NOT_APPLICABLE`，而不是伪称已迁移源测试。

完成后机器清单累计验证 501 个构造器/方法，项目共有 235 个单元测试和 44 组
固定 Java Oracle（3,650 条已登记记录）；`cargo-llvm-cov` 统计 21,480 行、
1,967 个函数和 29,232 个区域，三项均为 100%。

### 12.38 `TextUtils`

该对象完整迁移固定上游 `org.thymeleaf.util.TextUtils` 的 48 个 public 重载、两个
private `hashCodePart` helper 和 private 构造器，共 51 个机器清单方法。Rust
继续以不可外部构造的 `TextUtils` 作为关联函数命名空间；没有把“当前调用者用到的
部分算法”误标成整个对象完成。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `equals(boolean,CharSequence,CharSequence)` | `equals_sequences(case_sensitive,text1,text2)` | 动态序列；仅两个真实 `String` 且大小写敏感时走 UTF-16 slice 快路径 | `BEHAVIOR_VERIFIED` |
| `equals(boolean,CharSequence,char[])` | `equals_sequence_and_chars(...)` | 保留 `length()` 再取数组长度的求值顺序 | `BEHAVIOR_VERIFIED` |
| `equals(boolean,char[],char[])` | `equals_chars(...)` | 两次数组长度读取及 NPE 顺序保持一致 | `BEHAVIOR_VERIFIED` |
| `equals(boolean,char[],int,int,char[],int,int)` | `equals_chars_range(...)` | 长度不等短路、同一数组/offset 身份快路径及 Java 越界行为 | `BEHAVIOR_VERIFIED` |
| `equals(boolean,CharSequence,int,int,char[],int,int)` | `equals_sequence_and_chars_range(...)` | UTF-16 码元逐项比较 | `BEHAVIOR_VERIFIED` |
| `equals(boolean,CharSequence,int,int,CharSequence,int,int)` | `equals_sequences_range(...)` | trait object 身份、offset/len 和动态异常保持 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,CharSequence,CharSequence)` | `starts_with_sequences(...)` | `String.startsWith` 快路径与动态 `length/charAt` 回退 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,CharSequence,char[])` | `starts_with_sequence_and_chars(...)` | 完整序列/数组委托范围重载 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,char[],char[])` | `starts_with_chars(...)` | 完整数数组委托范围重载 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,char[],int,int,char[],int,int)` | `starts_with_chars_range(...)` | text 短于 prefix 时先返回 false | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,CharSequence,int,int,char[],int,int)` | `starts_with_sequence_and_chars_range(...)` | 动态序列与数组边界不预校验 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,char[],int,int,CharSequence,int,int)` | `starts_with_chars_and_sequence_range(...)` | 数组/动态序列访问顺序保持 | `BEHAVIOR_VERIFIED` |
| `startsWith(boolean,CharSequence,int,int,CharSequence,int,int)` | `starts_with_sequences_range(...)` | 两个动态序列范围逐码元比较 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,CharSequence,CharSequence)` | `ends_with_sequences(...)` | `String.endsWith` 快路径与动态回退 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,CharSequence,char[])` | `ends_with_sequence_and_chars(...)` | 完整输入委托范围重载 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,char[],char[])` | `ends_with_chars(...)` | 完整数数组委托范围重载 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,char[],int,int,char[],int,int)` | `ends_with_chars_range(...)` | 逆序 `charAt`/数组访问次序精确保持 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,CharSequence,int,int,char[],int,int)` | `ends_with_sequence_and_chars_range(...)` | sequence/array 逆序比较 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,char[],int,int,CharSequence,int,int)` | `ends_with_chars_and_sequence_range(...)` | array/sequence 逆序比较 | `BEHAVIOR_VERIFIED` |
| `endsWith(boolean,CharSequence,int,int,CharSequence,int,int)` | `ends_with_sequences_range(...)` | 两个动态序列逆序比较 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,CharSequence,CharSequence)` | `contains_sequences(...)` | `String` 快路径；否则使用上游朴素回退算法 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,CharSequence,char[])` | `contains_sequence_and_chars(...)` | 完整输入委托范围重载 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,char[],char[])` | `contains_chars(...)` | 完整数数组委托范围重载 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,char[],int,int,char[],int,int)` | `contains_chars_range(...)` | 部分匹配失败执行 `i -= j`，不替换为标准库搜索 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,CharSequence,int,int,char[],int,int)` | `contains_sequence_and_chars_range(...)` | sequence/array 范围及异常保持 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,char[],int,int,CharSequence,int,int)` | `contains_chars_and_sequence_range(...)` | array/sequence 范围及异常保持 | `BEHAVIOR_VERIFIED` |
| `contains(boolean,CharSequence,int,int,CharSequence,int,int)` | `contains_sequences_range(...)` | 动态序列调用次序及空 fragment 语义保持 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,CharSequence,CharSequence)` | `compare_sequences(...)` | 先调用两个 `length()`，再按 UTF-16 比较 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,CharSequence,char[])` | `compare_sequence_and_chars(...)` | sequence/array 完整范围比较 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,char[],char[])` | `compare_chars(...)` | array/array 完整范围比较 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,char[],int,int,char[],int,int)` | `compare_chars_range(...)` | 身份快路径及精确 `c1 - c2`/长度差 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,CharSequence,int,int,char[],int,int)` | `compare_sequence_and_chars_range(...)` | Java 大小写映射后返回精确差值 | `BEHAVIOR_VERIFIED` |
| `compareTo(boolean,CharSequence,int,int,CharSequence,int,int)` | `compare_sequences_range(...)` | 动态序列身份与范围保持 | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,char[][],char[],int,int)` | `binary_search_chars_values_and_chars(...)` | 完整 values 范围委托 | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,char[][],CharSequence,int,int)` | `binary_search_chars_values_and_sequence(...)` | `char[][]` 中搜索动态序列 key | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,CharSequence[],char[],int,int)` | `binary_search_sequence_values_and_chars(...)` | 动态 values 中搜索数组 key | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,CharSequence[],CharSequence,int,int)` | `binary_search_sequence_values_and_sequence(...)` | 动态 values/key 完整范围委托 | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,char[][],int,int,char[],int,int)` | `binary_search_chars_values_and_chars_range(...)` | Java unsigned midpoint、方向和插入点编码 | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,char[][],int,int,CharSequence,int,int)` | `binary_search_chars_values_and_sequence_range(...)` | 反向 compare 参数下的 low/high 更新保持 | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,CharSequence[],int,int,char[],int,int)` | `binary_search_sequence_values_and_chars_range(...)` | 每次探测动态读取 `length()` | `BEHAVIOR_VERIFIED` |
| `binarySearch(boolean,CharSequence[],int,int,CharSequence,int,int)` | `binary_search_sequence_values_and_sequence_range(...)` | null 中项、越界与 `-(low+1)` 精确保持 | `BEHAVIOR_VERIFIED` |
| `hashCode(char[],int,int)` | `hash_chars_range(...)` | Java `int` 31 倍回绕及数组越界 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence)` | `hash_sequence(...)` | 动态 `length()` 调用后散列 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence,int,int)` | `hash_sequence_range(...)` | begin/end 原样循环，不预归一化 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence,CharSequence)` | `hash_pair(...)` | 多段散列状态连续传递 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence,CharSequence,CharSequence)` | `hash_triple(...)` | 参数求值/失败顺序保持 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence,CharSequence,CharSequence,CharSequence)` | `hash_quadruple(...)` | 参数求值/失败顺序保持 | `BEHAVIOR_VERIFIED` |
| `hashCode(CharSequence,CharSequence,CharSequence,CharSequence,CharSequence)` | `hash_quintuple(...)` | 参数求值/失败顺序保持 | `BEHAVIOR_VERIFIED` |
| private `hashCodePart(int,CharSequence)` | private `hash_part_whole(...)` | 读取动态长度后委托范围 helper | `BEHAVIOR_VERIFIED` |
| private `hashCodePart(int,CharSequence,int,int)` | private `hash_part_range(...)` | `String.hashCode` 快路径短路与动态逐码元路径 | `BEHAVIOR_VERIFIED` |
| private `TextUtils()` | `TextUtils { _private: () }` | 保持不可从 crate 外构造，仅作为关联函数命名空间 | `BEHAVIOR_VERIFIED` |

大小写不使用 Rust Unicode 字符串折叠：`text_utils_case_map.bin` 固定 JDK 21
`Character.toUpperCase(char)` / `toLowerCase(char)` 的全部 BMP 单码元映射，校验值为
`9144b49cc5606c9cdcc0a3f041d88a879a6eee4b7d9a5e3e529b2cd3486aca96`。
`CharSequenceValue` trait 保留任意实现的动态 `length()`、`charAt(int)`、可变底层数据、
调用顺序和异常；`TextUtilsError` 保留 NPE、IllegalArgumentException、数组/String
越界和动态序列异常类别。

两个仓库都没有 `.codegraph/`，本切片没有擅自建立索引。固定源码静态调用清单确认
直接生产调用者包括 `AttributeDefinitions`、`AttributeNames`、`ElementDefinitions`、
`ElementNames`、`MatchingAttributeName`、`MatchingElementName`、
`OutputExpressionInlinePreProcessorHandler`、
`DecoupledTemplateLogicBuilderMarkupHandler` 和 `EventProcessorTextHandler`。
这些证据能确认直接调用面，但不能替代 CodeGraph 对反射或动态分派调用路径的证明；
该证据缺口继续保留在测试台账。

固定 Java Golden 直接编译固定上游 `TextUtils.java`，74 条登记记录覆盖全部 48 个
public 重载、稳定异常、动态序列调用轨迹、完整 BMP 摘要和 360 组 contains corpus。
上游 `TextUtilsTest#testContains` 的 13 个输入 × 26 条断言（338 条）全部并入 corpus。
完成后机器清单累计验证 552 个构造器/方法，项目共有 241 个单元测试和 45 组
固定 Java Oracle（3,724 条已登记记录）；`cargo-llvm-cov` 统计 23,419 行、
2,071 个函数和 31,723 个区域，三项均为 100%。

### 12.39 `IProcessor`

该对象完整迁移固定上游 `org.thymeleaf.processor.IProcessor` 的两个 public
接口方法。它是元素、文本、注释、CDATA、DOCTYPE、处理指令、模板边界和 XML
声明 Processor 子接口的共同根契约。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `TemplateMode getTemplateMode()` | `get_template_mode(&self) -> Option<TemplateMode>` | 接口本身不校验非空；`None` 保留任意 Java 实现可返回 `null` 的边界，非空校验留给 `AbstractProcessor` | `BEHAVIOR_VERIFIED` |
| `int getPrecedence()` | `get_precedence(&self) -> i32` | 保留完整 Java `int` 范围和动态实现；排序行为由后续 `ProcessorComparators` 迁移 | `BEHAVIOR_VERIFIED` |

固定 Java Golden 直接编译固定上游 `IProcessor.java` 和 `TemplateMode.java`，通过
`IProcessor` 接口引用调用一个可变实现。11 条记录覆盖初始 `null`、六种模板模式、
`Integer.MIN_VALUE`、零和 `Integer.MAX_VALUE`；Rust 使用 `dyn IProcessor` 逐条
差分，验证 trait object 动态分派且没有擅自增加 `Send`/`Sync` 约束。

上游没有直接测试这个纯接口；静态源码检查确认其生产调用面包括 10 个 Processor
子接口、`AbstractProcessor`、`DialectSetConfiguration`、配置包装器和 Processor
排序器。两个仓库的 `.codegraph/` 已用于复核接口实现、包装器和聚合调用路径；
该切片仍只把双侧接口对象 Golden 覆盖的基础 getter 合同计入 `IProcessor`，
下游动态行为由各具体 Processor 与聚合切片结算。

完成后机器清单累计验证 554 个构造器/方法，项目共有 242 个单元测试和 46 组
固定 Java Oracle（3,735 条已登记记录）；`cargo-llvm-cov` 统计 23,448 行、
2,074 个函数和 31,768 个区域，三项均为 100%。

### 12.40 `AbstractProcessor`

该对象完整迁移固定上游 `org.thymeleaf.processor.AbstractProcessor` 的一个
protected 构造器和两个 public final getter。Java 通过抽象类继承复用两个 final
字段；Rust 按抽象类迁移规则使用同名、字段私有的组合式基础状态，具体 Processor
持有并委托该对象，不伪造类继承。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `protected AbstractProcessor(TemplateMode templateMode, int precedence)` | `AbstractProcessor::new(Option<TemplateMode>, i32) -> Result<AbstractProcessor, ValidateError>` | public 构造器让 crate 外具体 Processor 获得与 Java 外部子类等价的扩展能力；复用 `Validate::not_null`，`None` 精确映射 `IllegalArgumentException("Template mode cannot be null")` | `BEHAVIOR_VERIFIED` |
| `final TemplateMode getTemplateMode()` | `get_template_mode(&self) -> TemplateMode` | 基础对象只暴露非空值；字段私有且无 setter，保持 final 字段/getter 的不可覆盖和不可修改语义 | `BEHAVIOR_VERIFIED` |
| `final int getPrecedence()` | `get_precedence(&self) -> i32` | 保留完整 Java `int` 范围；字段私有且不可变 | `BEHAVIOR_VERIFIED` |

`AbstractProcessor` 同时实现 `IProcessor`：trait 入口把已校验的模式包装为
`Some(TemplateMode)`，并委托同一个优先级值。这样既不收窄 `IProcessor` 对任意
实现允许 `None` 的合同，也保持该基础实现自身永不返回 null。

固定 Java Golden 以真实上游类为父类定义最小 `ProbeProcessor`。8 条记录覆盖精确
校验异常类/消息、HTML/XML/TEXT/JAVASCRIPT/CSS/RAW、`Integer.MIN_VALUE`/
`MAX_VALUE`、具体 getter、接口 getter、枚举对象身份和重复读取稳定性；Rust 逐条
使用组合对象与 `dyn IProcessor` 差分。

固定源码静态检查确认有 10 个直接生产子类，包括标准默认属性 Processor 和九类
抽象 Processor 基类。上游没有直接针对该薄抽象基类的 JUnit，具体 Processor 行为
测试留给各自对象切片。两个仓库的 `.codegraph/` 已用于复核 10 个直接生产子类和
配置聚合消费者；本切片只闭合构造、状态和根接口委托合同，不把下游行为提前归入。

完成后机器清单累计验证 557 个构造器/方法，项目共有 244 个单元测试和 47 组
固定 Java Oracle（3,743 条已登记记录）；`cargo-llvm-cov` 统计 23,502 行、
2,081 个函数和 31,846 个区域，三项均为 100%。

### 12.41 `IProcessorDialect`

该对象完整迁移固定上游 `org.thymeleaf.dialect.IProcessorDialect` 的三个 public
接口方法，并继续继承 `IDialect` 的名称合同。接口描述提供 Processor 的方言、
可覆盖的默认前缀，以及独立于单个 Processor precedence 的跨方言排序 precedence。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `String getPrefix()` | `get_prefix(&self) -> Option<&str>` | `None` 保留 Java `null`，表示 Processor 作用于无 namespace 的属性/元素；空、Unicode 和用户注册时覆盖仍保持不同 | `BEHAVIOR_VERIFIED` |
| `int getDialectProcessorPrecedence()` | `get_dialect_processor_precedence(&self) -> i32` | 保留完整 Java `int` 范围；只表达方言级排序值，不与单个 Processor precedence 提前合并 | `BEHAVIOR_VERIFIED` |
| `Set<IProcessor> getProcessors(String dialectPrefix)` | `get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet>` | 参数、返回集合和集合元素的 Java `null` 均保留；调用参数按默认/用户覆盖后的原值传入；配置阶段的非法 null 拒绝留给后续 `DialectSetConfiguration` | `BEHAVIOR_VERIFIED` |

`ProcessorSet` 是独立文件中的 `RUST_EXTENSION`，不计入 Java 对象分子。它用
`Arc<dyn IProcessor>` 保存共享对象身份，保留具体 Java Set 的实际迭代顺序，
保证同一逻辑元素与 null 各最多一次。固定上游 Processor 没有覆盖
`Object#equals/hashCode`，默认插入按 Arc 身份去重；第三方 Processor 若覆盖 Java
`equals`，使用 `insert_with` 显式提供候选与已有元素的等价谓词。该设计没有把
Java `Set<IProcessor>` 简化为允许重复的普通 Vec。

固定 Java Golden 直接编译真实 `IDialect.java`、`IProcessorDialect.java`、
`IProcessor.java` 和 `TemplateMode.java`。8 条记录覆盖 null/空/Unicode 默认前缀、
null/空/Unicode 实际调用前缀、`Integer.MIN_VALUE`/零/`MAX_VALUE`、接口动态分派、
`LinkedHashSet` 迭代顺序、重复对象拒绝、null 集合与 null 元素；Rust 通过
`dyn IProcessorDialect` 和 `ProcessorSet` 逐条差分。

固定源码静态检查确认直接生产实现为 `AbstractProcessorDialect` 和
`StandardDialect`，主要消费者是 `DialectSetConfiguration`、
`ConfigurationPrinterHelper` 与 `ProcessorConfigurationUtils` 的包装器族。上游
没有只针对该纯接口的 JUnit；`ElementProcessorIteratorTest` 和 Dialect ordering
测试验证的是后续聚合、包装与排序链路，保留给对应对象切片，不能错误归入本接口。
两个仓库的 `.codegraph/` 已用于复核直接实现、包装器和
`DialectSetConfiguration` 动态调用路径；本接口切片仍只结算双侧
trait/interface Golden，聚合、包装和排序由后续对象切片结算。

完成后机器清单累计验证 560 个构造器/方法，项目共有 246 个单元测试和 48 组
固定 Java Oracle（3,751 条已登记记录）；`cargo-llvm-cov` 统计 23,651 行、
2,105 个函数和 32,081 个区域，三项均为 100%。

### 12.42 `AbstractProcessorDialect`

该对象完整迁移固定上游
`org.thymeleaf.dialect.AbstractProcessorDialect` 自身声明的一个受保护构造器和
两个 public final getter。它继承 `AbstractDialect#getName()`，并因尚未实现
`IProcessorDialect#getProcessors(String)` 而保持为抽象扩展基类。

| Java 签名/方法 | Rust API | 语义处置 | 状态 |
|:---|:---|:---|:---:|
| `protected AbstractProcessorDialect(String name, String prefix, int processorPrecedence)` | `AbstractProcessorDialect::new(name: Option<&str>, prefix: Option<&str>, processor_precedence: i32) -> Result<AbstractProcessorDialect, AbstractDialectError>` | 先组合调用 `AbstractDialect::new(name)`，使 null name 精确映射 `IllegalArgumentException("Dialect name cannot be null")`；空名称合法；prefix 的 null/空/Unicode 不合并；precedence 保留完整 Java `int`；public Rust 构造入口维持 crate 外具体方言与 Java 外部子类相同的扩展能力 | `BEHAVIOR_VERIFIED` |
| `final String getPrefix()` | `get_prefix(&self) -> Option<&str>` | 返回构造时保存的默认前缀；`None` 对应 Java `null`，无 setter，重复读取保持同一内部状态 | `BEHAVIOR_VERIFIED` |
| `final int getDialectProcessorPrecedence()` | `get_dialect_processor_precedence(&self) -> i32` | 返回构造时保存的方言级 precedence；不与单个 Processor precedence 合并，字段和 getter 均不可修改 | `BEHAVIOR_VERIFIED` |

Rust 按抽象类迁移规则采用组合而非继承：`AbstractProcessorDialect` 内含
`AbstractDialect`，自身实现 `IDialect` 名称合同；具体方言组合该状态后实现
`IProcessorDialect`，把名称、前缀和 precedence 委托给基础状态，并自行提供
`get_processors`。基础对象没有伪造空集合或 null 集合作为默认实现，因此没有改变
Java 的抽象扩展边界。

固定 Java Golden 以真实 `AbstractProcessorDialect` 为父类定义最小
`ProbeDialect`，只实现 `getProcessors`。5 条记录覆盖父构造器精确异常类/消息、
空名称、null/空/Unicode 前缀、`Integer.MIN_VALUE`/零/`MAX_VALUE`、继承
`IDialect` 与 `IProcessorDialect` 动态入口、实际 prefix 参数转发、空集合扩展点、
调用次数及 final 状态重复读取。Rust 以组合式 `ProbeDialect` 和双 trait object
逐条差分。

固定源码静态检查确认唯一直接生产子类为 `StandardDialect`；测试源码另有 27 个
fixture 子类。`DialectOrderingTest` 和 `DialectProcessWrappingTest` 通过这些具体
方言测试后续聚合、包装与排序，而不是该薄基础状态本身，所以仍留给
`DialectSetConfiguration`/Processor 聚合切片。两个仓库的 `.codegraph/` 已用于
复核固定源码子类、trait 实现和聚合调用路径；本轮只结算该基础状态自身的双侧运行时
差分。

完成后机器清单累计验证 563 个构造器/方法，项目共有 248 个单元测试和 49 组
固定 Java Oracle（3,756 条已登记记录）；`cargo-llvm-cov` 统计 23,734 行、
2,116 个函数和 32,208 个区域，三项均为 100%。

### 12.43 专用 Dialect 测试 fixture 方法批次

该批次迁移 Java 测试 fixture 的 7 个构造器、2 个 `getProcessors` 和 5 个
`doProcess` 合同；继承 getter 继续由生产 `AbstractProcessorDialect` /
`Abstract*Processor` 提供。Rust 构造器参数与 Java 一致，其中
`ElementStackTextProcessor::new(dialect_prefix)` 保留 Java 虽未读取但可观察的构造调用
形态；`doProcess` 通过相应 `I*Processor::process` 动态入口执行。

| Java 方法组 | Rust 处置 | 行为证据 |
|:---|:---|:---|
| `Dialect01()` / `getProcessors(String)` | `Dialect01::new()` / `IProcessorDialect::get_processors` | 无 Standard Dialect 的 48 个 `.thtest` 全部通过 |
| `Dialect01DivProcessor(String)` / `doProcess` | `Dialect01DivProcessor::new(dialect_prefix)` / 元素 Processor callback | HTML 专用处理链执行，TEXT/JS/CSS 下不误匹配 |
| `Dialect01TextProcessor()` / `doProcess` | `Dialect01TextProcessor::new()` / 文本 Processor callback | 无 Standard Dialect 时标准内联不执行 |
| `ElementStackDialect()` / `getProcessors(String)` | `ElementStackDialect::new()` / `IProcessorDialect::get_processors` | 三种 Processor 以 Java 顺序和 precedence 聚合 |
| 三个 `ElementStack*Processor(String)` / `doProcess` | 三个独立构造器和属性/文本/模型 callback | `elementstack01..13` 全部通过 |

生产 `AbstractMarkupTemplateParser` 本批只删除了不属于 Java
`HTMLTemplateParser(AUTO_CLOSE)` 的私有 auto-open 实现，没有新增、删除或改名任何公开
API；所以 4,291 个生产 Java 声明方法的机器处置分母不变。

### 12.44 Template Resolver 完整对象批次

本批一次性闭合 `templateresolver` 除已验证 `TemplateResolution` 外的 9 个对象，共
102 个固定基线声明方法/构造器。`ITemplateResolver::resolve_template` 返回
`Result<Option<TemplateResolution>, TemplateResolverError>`：`Ok(None)` 只表达
pattern 拒绝、具体 Resolver 不适用或启用预检后的资源缺失，参数错误、资源构造错误和
解析结果错误均保留原因链，不再伪装成“继续下一个 Resolver”。

| Java 方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `ITemplateResolver#getName/getOrder/resolveTemplate` | 同名 snake_case trait；另有 `resolve_template_nullable` | configuration 先于 template 校验；动态分派；错误与未命中分离 | `BEHAVIOR_VERIFIED` |
| `AbstractTemplateResolver` 17 个声明 | 公共状态 getter/setter、`compute_resolvable`、闭包化固定解析算法 | pattern、null resource、exists 三类短路均不调用 mode/validity；verified/decoupled 标志保留 | `BEHAVIOR_VERIFIED` |
| `AbstractConfigurableTemplateResolver` 51 个声明 | 配置 getter/setter、alias、resource name、mode、validity | `setTemplateAliases` 是 `putAll`；UTF-16 code unit 不经 lossy 转换；pattern/扩展名/force 与缓存优先级一致 | `BEHAVIOR_VERIFIED` |
| Default/String Resolver 21 个声明 | 固定正文或模板名正文；mode 文本/枚举入口；cache/TTL | null setter 精确消息；String 禁止 decoupled；负 TTL 原样保留 | `BEHAVIOR_VERIFIED` |
| File/ClassLoader Resolver 5 个声明 | `FileTemplateResource` / `ClassLoaderTemplateResource` | 真实 I/O、base name、存在性；资源错误传播；ClassLoader 以有序资源根表达 | `BEHAVIOR_VERIFIED` |
| URL Resolver 3 个声明 | URL Resource + Resolver 级协议处理器扩展 | 仅 malformed URL 返回未命中；空路径报错；Java regex 的行终止符边界决定 jsessionid 缓存 | `BEHAVIOR_VERIFIED` |
| Web Resolver 2 个声明 | `IWebApplication` 宿主资源 | null application 优先拒绝；根路径/描述和未预检标志保持 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle 由提交
`10f9dd2eb8cbd98515ce14b149d115e0287d0add` 的真实 Resolver 源码生成 113 条记录；
Rust `template_resolver_java_parity` 逐记录完全一致。附加义务测试验证 Java
`URLStreamHandler` 到实例级 `UrlResourceConnectionHandler` 的 Rust 映射，确保
Resolver 创建的资源实际收到处理器。严格布局审计在 `templateresolver/` 下为
0 error / 0 warning；`ClassLoaderTemplateResolver` 已恢复一对一对象名和文件名，
`TemplateResolutionError` 独立成文件。

### 12.45 LinkBuilder 完整对象批次

本批一次性闭合 `org.thymeleaf.linkbuilder` 的 3 个对象和固定基线中的 19 个声明
构造器/方法。CodeGraph 同时复核 Java 的 `TemplateEngine` 构建链、Web 上下文动态
分派和 Rust 的 Engine/中立 Web 调用面；Rust 核心只依赖 `IWebContext` 端口，不依赖
Actix Web、Axum 或其他宿主类型。

| Java 签名/方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `AbstractLinkBuilder()`、`getName/setName`、`getOrder/setOrder` | `AbstractLinkBuilder::new(build_link)` + 同名 snake_case 状态 API | Java 抽象继承改为组合闭包；保留 nullable name/order、重复读取和子类动态构建逻辑 | `BEHAVIOR_VERIFIED` |
| `ILinkBuilder#getName/getOrder/buildLink` | `ILinkBuilder` trait + `build_link_nullable` | `Send + Sync` 固化线程安全；context 先校验；`Ok(None)` 仅表示本 builder 无法处理，供链中下一项继续 | `BEHAVIOR_VERIFIED` |
| `StandardLinkBuilder()` / `buildLink` | `StandardLinkBuilder::new()` / `build_link` | defensive ordered map、链接分类、查询/片段拆分、模板变量消费、Web context path 与 URL transform 顺序完全保留 | `BEHAVIOR_VERIFIED` |
| `findCharInSequence`、三类 `isLinkBase*`、`filterOutJavaScriptLinks` | `Utf16String`/UTF-16 内部辅助逻辑 | 按 Java code unit 搜索和大小写映射；只拒绝精确 scheme 前缀，不做宿主框架策略扩张 | `BEHAVIOR_VERIFIED` |
| `replaceTemplateParamsInBase` / `formatParameterValueAsUnescapedVariableTemplate` | 路径变量替换与参数格式化内部逻辑 | `{x}` 优先于 `{/x}`；搜索游标按未转义 UTF-16 长度推进；scalar/null/list/empty-list 与 Java 一致 | `BEHAVIOR_VERIFIED` |
| `processAllRemainingParametersAsQueryParams` | 有序剩余参数查询编码 | null 只输出参数名；列表保留重复值、前导 null/empty 的逗号边界及空列表后的 `?&` 行为 | `BEHAVIOR_VERIFIED` |
| `computeContextPath` / `processLink` | `with_compute_context_path` / `with_process_link` 组合钩子 | 钩子收到原始参数映射；默认 Web 转换允许 nullable 结果，扩展点不被硬编码成 Servlet | `BEHAVIOR_VERIFIED` |

固定 Java Golden 共 63 条记录，由
`tests/java/LinkBuilderGolden.java` 在固定 SHA 上生成，Rust
`standard_link_builder_matches_java_golden` 逐行重建并比较完整结果。语料覆盖四类链接、
fragment-only、`javascript:` 拒绝、路径/段/查询编码、重复模板变量、全部常用数值、
Unicode 与孤立代理项、原始参数身份及两个 protected 钩子。另有两个 Rust 义务测试
验证抽象类组合状态/nullable 合同，以及 8 个并发线程共享同一 trait object 时输出互不
污染。完成后累计验证 582 个生产构造器/方法，固定 Java Oracle 增至 50 组、3,819 条
记录；覆盖率采用第 16 节全 workspace 统一结果，不以单批 100% 替代语义证据。

### 12.46 Inline 基础 SPI 完整对象批次

本批一次性闭合 `org.thymeleaf.inline` 的 2 个对象和固定基线中的 9 个声明
构造器/方法。CodeGraph 复核了 Engine Context、Standard Inliner、三类内联 Processor
以及 `th:inline="none"` 的动态调用面；上游没有直接针对这两个基础对象的 JUnit，
因此使用固定 Java Golden 作为 V3 Oracle，并单独登记 Rust 共享义务。

| Java 签名/方法 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `IInliner#getName()` | `IInliner::get_name(&self) -> &Utf16String` | 返回可识别名称；通过 trait object 动态分派 | `BEHAVIOR_VERIFIED` |
| `IInliner#inline(ITemplateContext,IText)` | `IInliner::inline_text(...)` | Text 路径独立分派；返回 nullable `CharSequenceValue`，Rust 错误进入统一表达式结果 | `BEHAVIOR_VERIFIED` |
| `IInliner#inline(ITemplateContext,ICDATASection)` | `IInliner::inline_cdata_section(...)` | CDATA 路径不与 Text/Comment 合并；参数和调用计数独立 | `BEHAVIOR_VERIFIED` |
| `IInliner#inline(ITemplateContext,IComment)` | `IInliner::inline_comment(...)` | Comment 路径独立分派；返回值身份/内容由实现决定 | `BEHAVIOR_VERIFIED` |
| `private NoOpInliner()` / `INSTANCE` | `NoOpInliner::instance()`、`shared()`、`#[non_exhaustive]` | crate 外不可直接构造；所有入口克隆同一 `Arc` 分配并保持静态单例身份 | `BEHAVIOR_VERIFIED` |
| `NoOpInliner#getName()` | trait `get_name()` | 固定返回 Java UTF-16 名称 `NOOP` | `BEHAVIOR_VERIFIED` |
| `NoOpInliner#inline(...,IText)` | `inline_text_nullable` + trait `inline_text` | null/非 null 上下文和事件均不读取，始终返回 `None` | `BEHAVIOR_VERIFIED` |
| `NoOpInliner#inline(...,ICDATASection)` | `inline_cdata_section_nullable` + trait 入口 | null/非 null 参数均不读取，始终返回 `None` | `BEHAVIOR_VERIFIED` |
| `NoOpInliner#inline(...,IComment)` | `inline_comment_nullable` + trait 入口 | null/非 null 参数均不读取，始终返回 `None` | `BEHAVIOR_VERIFIED` |

固定 Java Golden 共 23 条记录，由 `tests/java/InlineGolden.java` 在固定 SHA 上生成，
Rust `inline_contract_and_no_op_singleton_match_java_golden` 逐记录重建并比较接口
签名、final/private/INSTANCE 形态、单例身份、名称、三个重载的 null/非 null 返回、
自定义接口实现动态分派和 8 线程身份集合。`PanicTemplateContext` 保证 NoOp 不会读取
上下文；第二个 Rust 测试验证 `shared()` 的两个 trait object 和 `instance()` 指向同一
分配；compile-fail doctest 保证 crate 外无法恢复 Java 私有构造器。

完成后累计验证 591 个生产构造器/方法，固定 Java Oracle 增至 51 组、3,842 条记录；
覆盖率采用第 16 节全 workspace 统一结果，不以该小批的局部覆盖率替代语义证据。

### 12.47 PreProcessor / PostProcessor 完整对象批次

本批一次性闭合 `org.thymeleaf.preprocessor` 与 `org.thymeleaf.postprocessor` 的 4 个
对象和固定基线中的 14 个声明。CodeGraph 复核了方言聚合、排序器、包装器与
`TemplateManager` 的完整调用链：PreProcessor 在中央 Processor 前执行，
PostProcessor 在中央 Processor 后、Output Handler 前执行。

| Java 签名/方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `IPreProcessor#getTemplateMode/getPrecedence/getHandlerClass` | `IPreProcessor` 三个同名 snake_case trait 方法 | 模式与优先级按值返回；Handler 名称和构造器由不可分离的 `TemplateHandlerClass` 类型令牌承载；trait object 动态分派 | `BEHAVIOR_VERIFIED` |
| `PreProcessor(TemplateMode,Class,int)` | `PreProcessor::new(Option<TemplateMode>, Option<TemplateHandlerClass>, i32)` | 先校验 mode、再校验 Handler class，精确保留两条 `IllegalArgumentException` 消息；状态构造后不可变 | `BEHAVIOR_VERIFIED` |
| `PreProcessor` 三个 getter | 三个 inherent getter，并由 trait impl 委托 | 六种模式、`i32::MIN..=i32::MAX`、Handler 类型令牌引用身份稳定 | `BEHAVIOR_VERIFIED` |
| `IPostProcessor#getTemplateMode/getPrecedence/getHandlerClass` | `IPostProcessor` 三个同名 snake_case trait 方法 | 与 PreProcessor 对称；动态实现可提供自己的配置实现类名供 Java 排序规则使用 | `BEHAVIOR_VERIFIED` |
| `PostProcessor(TemplateMode,Class,int)` | `PostProcessor::new(Option<TemplateMode>, Option<TemplateHandlerClass>, i32)` | 相同的非空校验顺序、消息和不可变状态 | `BEHAVIOR_VERIFIED` |
| `PostProcessor` 三个 getter | 三个 inherent getter，并由 trait impl 委托 | 模式、优先级、Handler 类型令牌逐项返回 | `BEHAVIOR_VERIFIED` |

`TemplateHandlerClass` 是 Rust 运行时扩展：它将 Java `Class<? extends
ITemplateHandler>` 的类名和公开无参构造能力绑定为一个值，避免旧实现中函数指针与
类名字符串不一致。构造器返回 `Result`，所以 `TemplateManager` 能保留 Java
`Class#newInstance()` 的失败通道和原因链。`ProcessorComparators` 已按 Java 配置对象
实现类名排序，而不是误按 Handler 类名排序；`ProcessorConfigurationUtils` 包装器保留
方言优先级与 unwrap 身份。

固定 Java Golden 共 40 条记录，由
`tests/java/PrePostProcessorGolden.java` 在固定 SHA 上生成，Rust
`pre_and_post_processor_contracts_match_java_golden` 逐记录重建并比较。语料覆盖接口
形态、构造校验顺序、六种模式、优先级边界、自定义接口动态分派、配置实现类排序、
方言包装和解包、每次新建 Handler，以及 Pre/Post 构造失败的外层消息与原因。第二个
Rust 义务测试验证类型令牌和两个配置对象均为 `Send + Sync`，并由 8 个线程共享构造。

完成后累计验证 605 个生产构造器/方法，固定 Java Oracle 增至 52 组、3,882 条记录；
全 workspace `cargo llvm-cov` 为 region 59.10%、function 55.17%、line 60.65%。

### 12.48 Dialect 贡献聚合完整对象批次

本批一次性闭合 `DialectSetConfiguration`、`IExecutionAttributeDialect`、
`IExpressionObjectDialect`、`IPreProcessorDialect` 与
`IPostProcessorDialect` 五个主对象，以及 `DialectSetConfiguration` 内部的
`AggregateExpressionObjectFactory`，共 37 个固定基线声明。CodeGraph 同时复核了
Java 的聚合、包装、定义注入、表达式对象反向覆盖调用链，以及 Rust 中从
`EngineConfiguration` 到各 Processor bucket 和 `TemplateManager` 的消费者路径。

| Java 签名/方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `build(Set<DialectConfiguration>)` 与私有聚合/初始化方法 | `DialectSetConfiguration::build(Option<Vec<DialectConfiguration>>)` 与同文件私有 helper | null 配置集合映射为精确 `IllegalArgumentException`；Processor、执行属性、表达式工厂、Pre/Post 五阶段按 Java 顺序聚合；定义感知对象只注入一次 | `BEHAVIOR_VERIFIED` |
| 13 个配置、定义、执行属性与 Processor getter | 同名 snake_case getter | 方言按身份去重且保留首次顺序；Map 保留 null key/value；10 个 mode getter 对 `None` 精确返回 Java 校验错误，空 bucket 返回空集合 | `BEHAVIOR_VERIFIED` |
| `AggregateExpressionObjectFactory` 的构造、`add` 与三个接口方法 | 私有聚合对象 + `IExpressionObjectFactory` 动态入口 | 0 个 factory 时 names/build 为 null、cacheable=false；1 个 factory 对未知名称仍完整委托；多个 factory 逆序查找，后注册者覆盖同名对象 | `BEHAVIOR_VERIFIED` |
| `IExecutionAttributeDialect#getExecutionAttributes` | `get_execution_attributes() -> Option<ExecutionAttributeMap>` | null Map 被忽略；null key/value 保留；重复 key 在第二个方言处以精确消息失败 | `BEHAVIOR_VERIFIED` |
| `IExpressionObjectDialect#getExpressionObjectFactory` | `get_expression_object_factory() -> Option<Arc<dyn IExpressionObjectFactory>>` | null factory 被忽略；非空对象保持共享身份并参与后注册优先聚合 | `BEHAVIOR_VERIFIED` |
| Pre/Post Dialect 各两个方法 | precedence getter + nullable Processor 集合 | null Set 被忽略，null entry/mode/handler、错误 Handler 接口和缺少公开无参构造器均按 Java 类名、消息和 cause 拒绝 | `BEHAVIOR_VERIFIED` |

Java 实现存在一个必须保留的反直觉边界：聚合 Pre/PostProcessor 时并不调用
`getDialectPreProcessorPrecedence()` 或 `getDialectPostProcessorPrecedence()`，
也不创建方言 precedence 包装器，而是直接按 Processor 自身 precedence 与实现类名
排序。Golden 记录明确断言两个 precedence getter 的调用次数均为 0，防止根据接口
文档“修正”上游实际行为。

固定 Java harness
`tests/java/org/thymeleaf/DialectSetConfigurationGolden.java` 在固定 SHA 上生成
59 条记录；`tests/dialect_set_configuration_java_parity.rs` 逐记录比较完整输出，
另验证并发读取、定义注入一次性，以及上游
`DialectSetConfigurationTest#testProcessorComputation01..08` 的八类 Processor bucket
与排序。SOURCE_PARITY 的 8 个源码入口已全部改为指向该同对象测试，不再错误归因到
`DialectConfiguration` 的基础对象 Golden。

完成后累计验证 642 个生产构造器/方法，固定 Java Oracle 增至 53 组、3,941 条记录；
全 workspace 覆盖率采用第 16 节统一结果，不用该批局部覆盖率替代语义差分。

### 12.49 MessageResolver 完整对象批次

本批冻结 `org.thymeleaf.messageresolver` 四个主对象，共 28 个 Java
方法/构造器。固定 Oracle 为
[`MessageResolverGolden.java`](../../thymeleaf-test/tests/java/MessageResolverGolden.java)，109 条
记录由 [`message_resolver_java_parity.rs`](../../thymeleaf-test/tests/message_resolver_java_parity.rs)
逐项消费；Rust 另验证模板缓存策略、origin 父类型元数据及并发安全义务。

#### `AbstractMessageResolver`（5 / 5）

| Java 方法/构造器 | Rust API | 语义 | 状态 |
|:---|:---|:---|:---:|
| `AbstractMessageResolver()` | `AbstractMessageResolver::new(java_class_name)` | 组合映射保留动态具体类名和空顺序 | `BEHAVIOR_VERIFIED` |
| `getName()` | `get_name()` | 可空名称 | `BEHAVIOR_VERIFIED` |
| `setName(String name)` | `set_name(name)` | 允许 `None` | `BEHAVIOR_VERIFIED` |
| `getOrder()` | `get_order()` | 可空排序值 | `BEHAVIOR_VERIFIED` |
| `setOrder(Integer order)` | `set_order(order)` | 允许 `None` 和负值 | `BEHAVIOR_VERIFIED` |

#### `IMessageResolver`（4 / 4）

| Java 方法 | Rust API | 语义 | 状态 |
|:---|:---|:---|:---:|
| `getName()` | `get_name()` | 解析器诊断名称 | `BEHAVIOR_VERIFIED` |
| `getOrder()` | `get_order()` | 解析器链顺序 | `BEHAVIOR_VERIFIED` |
| `resolveMessage(context, origin, key, messageParameters)` | `resolve_message_nullable(context, origin, key, message_parameters)` | `None` 表示当前解析器未命中；非空便利入口委托同一合同 | `BEHAVIOR_VERIFIED` |
| `createAbsentMessageRepresentation(context, origin, key, messageParameters)` | `create_absent_message_representation_nullable(...)` | 全链未命中后的可空表示 | `BEHAVIOR_VERIFIED` |

#### `StandardMessageResolutionUtils`（8 / 8）

| Java 方法/构造器 | Rust API / 落点 | 语义 | 状态 |
|:---|:---|:---|:---:|
| `resolveMessagesForTemplate(templateResource, locale)` | `resolve_messages_for_template(...)` | base → language → country → variant 合并 | `BEHAVIOR_VERIFIED` |
| `resolveMessagesForOrigin(origin, locale)` | `resolve_messages_for_origin(...)` | 具体类型优先、父类型回退 | `BEHAVIOR_VERIFIED` |
| `resolveMessagesForSpecificClass(originClass, locale)` | origin 注册表单类型读取分支 | `TypeId` 消息登记替代 ClassLoader 单类资源读取 | `BEHAVIOR_VERIFIED` |
| `computeMessageResourceNamesFromBase(resourceBaseName, locale)` | `compute_message_resource_names_from_base(...)` | 精确资源名序列和无语言异常 | `BEHAVIOR_VERIFIED` |
| `readMessagesResource(propertiesReader)` | `read_messages_resource(...)` | Java Properties Reader 语法和输入异常 | `BEHAVIOR_VERIFIED` |
| `formatMessage(locale, message, messageParameters)` | `format_message(...)` | MessageFormat 功能族与 UTF-16 边界 | `BEHAVIOR_VERIFIED` |
| `isFormatCandidate(message)` | `format_message(...)` 内部快速路径 | 仅扫描 `}` 与单引号；非候选保持原值 | `BEHAVIOR_VERIFIED` |
| `StandardMessageResolutionUtils()` | 不可构造的 `pub(crate)` 单元结构 | Java private 构造器的 Rust 可见性等价 | `BEHAVIOR_VERIFIED` |

#### `StandardMessageResolver`（11 / 11）

| Java 方法/构造器 | Rust API | 语义 | 状态 |
|:---|:---|:---|:---:|
| `StandardMessageResolver()` | `new()` / `default()` | 空缓存、空默认消息、标准类名 | `BEHAVIOR_VERIFIED` |
| `getDefaultMessages()` | `get_default_messages()` | 返回同一活可变容器 | `BEHAVIOR_VERIFIED` |
| `setDefaultMessages(Properties defaultMessages)` | `set_default_messages(default_messages)` | `null` 无操作，非空合并而非替换 | `BEHAVIOR_VERIFIED` |
| `addDefaultMessage(String key, String value)` | `add_default_message_nullable(key, value)` | key/value 校验顺序和消息一致 | `BEHAVIOR_VERIFIED` |
| `clearDefaultMessages()` | `clear_default_messages()` | 原地清空活容器 | `BEHAVIOR_VERIFIED` |
| 四参数 `resolveMessage(...)` | `resolve_message_nullable(...)` | 委托三个阶段全开的主链 | `BEHAVIOR_VERIFIED` |
| 七参数 `resolveMessage(..., perform*)` | `resolve_message_with_phases(...)` | template → origin → default，开关独立 | `BEHAVIOR_VERIFIED` |
| `resolveMessagesForTemplate(template, resource, locale)` | `resolve_messages_for_template(...)` / `with_template_messages_hook(...)` | protected 覆写由组合钩子承接并进入真实缓存主链 | `BEHAVIOR_VERIFIED` |
| `resolveMessagesForOrigin(origin, locale)` | `resolve_messages_for_origin(...)` / `with_origin_messages_hook(...)` | protected 覆写由组合钩子承接并按 origin + Locale 缓存 | `BEHAVIOR_VERIFIED` |
| `formatMessage(locale, message, parameters)` | `format_message(...)` / `with_message_formatter_hook(...)` | protected 格式化扩展点 | `BEHAVIOR_VERIFIED` |
| `createAbsentMessageRepresentation(...)` | `create_absent_message_representation_nullable(...)` / `with_absent_message_hook(...)` | key 校验、locale 后缀和空上下文异常 | `BEHAVIOR_VERIFIED` |

功能族闭合口径是“所有可观察分支与运行时机制映射均有代表性、边界和失败证据”，
不是穷举 JDK `LocaleProvider`、每个货币/时区或无限 DecimalFormat pattern 输入。
JVM ClassLoader 由宿主显式注册替代，具体类覆盖父类、Locale 隔离和解析器实例缓存行为
保持不变。

完成后累计验证 670 个生产构造器/方法；固定 Java Oracle 仍为 53 组、3,941 条记录，
其中本批复用并强化既有 109 条 MessageResolver Oracle，而不是重复增加一组同源数据。

### 12.50 非元素 Processor SPI 与 StructureHandler 完整批次

本批一次冻结 28 个主对象、102 个 Java 声明方法/构造器和 157 个参数。固定上游
Oracle 为
[`NonElementStructureHandlerGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/engine/NonElementStructureHandlerGolden.java)，
生成 41 条记录；Rust 由 crate 内真实私有 handler 测试逐行消费，既不暴露测试专用
生产 API，也不以 fake 替代被测状态机。

| 对象组 | Java 声明数 | Rust 落点 | 已验证语义 | 状态 |
|:---|---:|:---|:---|:---:|
| 七个引擎 `*StructureHandler` | 40 | `src/engine/{text,cdata_section,comment,doc_type,processing_instruction,template_boundaries,xml_declaration}_structure_handler.rs` | 构造即 reset；互斥动作最后一次获胜；reset-before-validation；精确 null 消息；可选字段、processability 和对象身份 | `BEHAVIOR_VERIFIED` |
| 六个单事件 `Abstract*Processor` | 18 | 对应 Processor 子包的 `abstract_*_processor.rs` | 构造模板模式校验、`doProcess` 委托、处理异常身份/位置补全、其他异常运行时类名/事件位置/cause 包装 | `BEHAVIOR_VERIFIED` |
| `AbstractTemplateBoundariesProcessor` | 5 | `src/templateboundaries/abstract_template_boundaries_processor.rs` | start/end 独立回调，共享相同异常装饰合同 | `BEHAVIOR_VERIFIED` |
| 七个事件 Processor 接口 | 8 | 对应子包 `i_*_processor.rs` | 不可变事件输入、StructureHandler 输出通道与动态分派 | `BEHAVIOR_VERIFIED` |
| 七个 StructureHandler 接口 | 31 | 对应子包 `i_*_structure_handler.rs` | Text/Comment/CDATA 的 `CharSequence` 身份；DocType/PI/XML 字段 null 规则；边界插入位置、processable 和组合上下文动作 | `BEHAVIOR_VERIFIED` |

方法组的精确映射如下：

| Java 方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| 六组 `setText/setContent`、`replaceWith`、`remove*`、`reset` | 同名 snake_case trait 入口 + 引擎 nullable 校验入口 | Rust 非空公共类型阻止非法调用，引擎 nullable 入口保留 Java 运行时错误和失败后的已清理状态 | `BEHAVIOR_VERIFIED` |
| `setDocType(keyword, elementName, publicId, systemId, internalSubset)` | `set_doc_type` + `set_doc_type_nullable` | keyword 后 elementName 的校验顺序；后三项允许 null | `BEHAVIOR_VERIFIED` |
| `setProcessingInstruction(target, content)` | `set_processing_instruction` + nullable 引擎入口 | target 后 content 的校验顺序 | `BEHAVIOR_VERIFIED` |
| `setXMLDeclaration(keyword, version, encoding, standalone)` | `set_xml_declaration` + nullable 引擎入口 | 仅 keyword 必填 | `BEHAVIOR_VERIFIED` |
| 两个 `insert` 重载 | `insert_text` / `insert_model` + nullable 引擎入口 | TemplateStart 后、TemplateEnd 前插入；两个插入动作互斥；失败只清插入动作并保留上下文修改 | `BEHAVIOR_VERIFIED` |
| `setLocalVariable/removeLocalVariable/setSelectionTarget/setInliner` | 同名 snake_case 组合动作 | null 名称/值保留；Map 最后写获胜；Set 去重；动作不触发 gathering reset；应用顺序为 set、remove、selection、inliner | `BEHAVIOR_VERIFIED` |
| 七组 `process` 与 protected `doProcess*` | trait 动态入口 + `AbstractProcessorAdapter` 闭包扩展点 | 事件位置只补缺失字段；原 `TemplateProcessingException` 身份不变；非处理异常保留 cause | `BEHAVIOR_VERIFIED` |

固定 Golden 同时与现有真实 Dialect/Processor `.thtest` 形成互补：Golden 负责内部状态、
失败清理和异常元数据，端到端用例负责模型替换、processable、删除动作及
`ProcessorTemplateHandler` 消费链。完成后累计验证 772 个生产构造器/方法；固定
Java Oracle 增至 54 组、3,982 条记录。

### 12.51 根模板执行与节流交付批次

本批一次冻结 `ITemplateEngine`、`IThrottledTemplateProcessor` 与 `TemplateEngine`
三个主对象，共 62 个 Java 声明方法/构造器，其中 59 个为 public API。固定上游
Oracle 为
[`TemplateEngineExecutionGolden.java`](../../thymeleaf-test/tests/java/TemplateEngineExecutionGolden.java)，
生成 38 条记录；Rust 由
[`template_engine_execution_java_parity.rs`](../../thymeleaf-test/tests/template_engine_execution_java_parity.rs)
逐行消费。

| Java 方法组 | Rust API/处置 | 关键语义 | 状态 |
|:---|:---|:---|:---:|
| `ITemplateEngine#getConfiguration()` | `get_configuration()` | 首次调用触发初始化并返回冻结配置 | `BEHAVIOR_VERIFIED` |
| 三个返回 `String` 的 `process` 重载 | `process`、`process_template`、`process_template_with_selectors` | String、selector、TemplateSpec 全部收敛到同一执行主链并产生相同输出 | `BEHAVIOR_VERIFIED` |
| 三个 Writer `process` 重载 | `process_to_writer`、两个 trait 默认便利入口 | Writer 接收增量输出并在结束时恰好 flush；I/O 失败映射 `TemplateOutputException` | `BEHAVIOR_VERIFIED` |
| 三个 `processThrottled` 重载 | `process_throttled`、两个 trait 默认便利入口 | TemplateSpec 与 selector 保留，返回调用方驱动的同一节流处理器 | `BEHAVIOR_VERIFIED` |
| `IThrottledTemplateProcessor` 七个方法 | 同名 snake_case trait API + `ThrottledTemplateStatus` Rust 等价观察句柄 | 标识/规格、完成观察、字符/字节 process/processAll、计数单位和模式锁定；处理器线程亲和，完成状态可跨线程并发观察 | `BEHAVIOR_VERIFIED` |
| `TemplateEngine` 配置 getter/setter 与初始化 | 同名 snake_case API + `EngineConfiguration` 排序快照 | 初始化前保持插入序，初始化后 Resolver/Message/Link 返回 Java 比较器排序的冻结序；后续修改精确拒绝 | `BEHAVIOR_VERIFIED` |
| `TemplateEngine#process*` 内部执行和错误分流 | `process` / `process_to_writer` / `process_throttled` | 正常处理、selector、空模板、flush 失败与节流输出走真实 Parser/Model/Processor 链 | `BEHAVIOR_VERIFIED` |

字符节流 Golden 固定 0、3-code-unit 分块、完成后零输出和不限额输出；字节节流固定
UTF-8 多字节边界、每次最多 3 字节、总字节数和字符/字节模式切换错误。可克隆的
`ThrottledTemplateStatus` 与处理器共享 Acquire/Release 原子标志，并由真实双线程测试
保留 Java `isFinished()` 的并发短路观察能力，同时禁止并发执行 `process`。完成后累计
验证 834 个生产构造器/方法；固定 Java Oracle 增至 55 组、4,020 条记录。

### 12.52 引擎配置冻结与诊断批次

本批一次冻结三个主对象的 79 个 Java 声明，不按对象逐个迁移：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `ConfigurationPrinterHelper` | 15 | `src/configuration_printer_helper.rs` | 两个打印入口、完整配置分区打印、Processor 分类/排序及 `ConfigLogBuilder` 的模板替换、类名规范化、行结束合同 | `BEHAVIOR_VERIFIED` |
| `EngineConfiguration` | 37 | `src/engine_configuration.rs` | 构造冻结、全部 getter、Dialect 类型查询、ModelFactory 惰性单例、reshape 决策、全部 Processor bucket 与 TemplateManager | `BEHAVIOR_VERIFIED` |
| `IEngineConfiguration` | 27 | `src/i_engine_configuration.rs` | 27 个只读 trait getter 与动态分派；返回排序快照或共享对象身份 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`EngineConfigurationGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/EngineConfigurationGolden.java)
生成 44 条记录。Rust 差分覆盖稳定排序、快照不可变性、已知接口能力与具体类查询、
定义身份、六种 reshape、所有 Processor 分类、DEBUG/TRACE 路由和完整诊断；另以
12 线程 Barrier 验证 ModelFactory 只初始化一次。Java 同类同 precedence Processor
的最终 `identityHashCode` 次序、构建时间戳和执行属性值使用具名稳定规范化，其余
文本和集合成员逐项精确比较。

完成后累计验证 913 个生产构造器/方法；固定 Java Oracle 增至 56 组、4,064 条记录。

### 12.53 Engine Context 工厂与管理器生命周期批次

本批一次冻结三个主对象的 7 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `IEngineContextFactory` | 1 | `src/context/i_engine_context_factory.rs` | `createEngineContext` → `create_engine_context`；根模板创建一次、嵌套复用和线程安全合同 | `BEHAVIOR_VERIFIED` |
| `StandardEngineContextFactory` | 2 | `src/context/standard_engine_context_factory.rs` | 公开构造器；名称快照、逐变量读取、普通/Web capability 分流、Locale 与 exchange 身份 | `BEHAVIOR_VERIFIED` |
| `EngineContextManager` | 4 | `src/engine/engine_context_manager.rs` | 私有构造器/创建 helper 的不可构造静态对象映射；prepare、dispose、新建与已有 Context 两分支 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`EngineContextFactoryGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/engine/EngineContextFactoryGolden.java)
生成 46 条记录，覆盖声明形状、变量读取顺序、空值、普通/Web 返回类型、Locale、
解析属性、exchange 身份、工厂次数、嵌套身份、level 和 TemplateData 栈。
`StandardEngineContextFactory` 的 12 线程 Rust 义务测试还验证同一无状态工厂可安全
共享且每次返回独立对象。普通 `EngineContext#getVariableNames()` 的 Java `HashSet`
结果只按集合排序比较，实际变量复制轨迹与所有值仍精确差分。

完成后累计验证 920 个生产构造器/方法；固定 Java Oracle 增至 57 组、4,110 条记录。

### 12.54 表达式对象工厂、容器与安全包装批次

本批一次冻结五个对象的 36 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `IExpressionObjectFactory` | 3 | `src/expression/i_expression_object_factory.rs` | 完整共享名称集合、按 Context 构建、cacheable 策略 | `BEHAVIOR_VERIFIED` |
| `IExpressionObjects` | 4 | `src/expression/i_expression_objects.rs` | size、contains、共享名称集合、惰性 get | `BEHAVIOR_VERIFIED` |
| `ExpressionObjects` | 5 | `src/expression/expression_objects.rs` | 构造校验、未知名称、可缓存值/null、非缓存重建 | `BEHAVIOR_VERIFIED` |
| `StandardExpressionObjectFactory` | 4 | `src/expression/standard_expression_object_factory.rs` | 构造器、26 名称、标准对象构建、selection-only 非缓存 | `BEHAVIOR_VERIFIED` |
| `OGNLExpressionObjectsWrapper` | 20 | `src/expression/native_expression_objects_wrapper.rs` | `NativeExpressionObjectsWrapper` 承接全部 Map 可观察操作、restricted 访问、异常与显示 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`ExpressionObjectsGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/standard/expression/ExpressionObjectsGolden.java)
生成 114 条记录。Rust 差分覆盖名称集合身份、对象构造/缓存次数、Context 身份、标准
工具单例/新实例、模板 Context 能力、已移除 Servlet 对象、Map 的 put/remove/
putAll/keySet/values、全部禁止操作、null key 和精确异常。额外 12 线程测试验证预热
缓存的共享身份，`Weak` Context 测试验证 Rust 不形成所有权环。`putAll` 按固定
JDK 21 Oracle 保留 `HashMap` 绕过覆盖 `put` 的真实行为。

完成后累计验证 956 个生产构造器/方法；固定 Java Oracle 增至 58 组、4,224 条记录。

### 12.55 基础 Context 与表达式 Context 批次

本批一次冻结六个主对象的 29 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `IContext` | 4 | `src/context/i_context.rs` | Locale、contains、实时变量名 Set 视图、可空变量读取；不把 Context 暴露为 Map | `BEHAVIOR_VERIFIED` |
| `AbstractContext` | 12 | `src/context/abstract_context.rs` | 三个 protected 构造器由组合构造路径承接；输入浅复制、默认 Locale 快照、全部变量修改与实时 keySet 反向删除 | `BEHAVIOR_VERIFIED` |
| `Context` | 3 | `src/context/context.rs` | 三个 public 构造器逐项映射默认、显式 Locale 与变量 Map | `BEHAVIOR_VERIFIED` |
| `IExpressionContext` | 2 | `src/context/i_expression_context.rs` | 配置共享身份与表达式对象惰性容器 | `BEHAVIOR_VERIFIED` |
| `AbstractExpressionContext` | 5 | `src/context/abstract_expression_context.rs` | 三个 protected 构造器、配置身份、惰性且稳定的 `ExpressionObjects`；工厂接收当前对象身份 | `BEHAVIOR_VERIFIED` |
| `ExpressionContext` | 3 | `src/context/expression_context.rs` | 三个 public 构造器与具体 `this` 身份；不把组合基础对象错误传给方言工厂 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`BasicContextGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/context/BasicContextGolden.java)
生成 71 条记录。Rust 差分覆盖 29 个声明形状、构造瞬间默认 Locale、输入 Map 浅复制
与值身份、null 键/null 值/缺失键区分、`LinkedHashMap` 替换和 `putAll` 顺序、实时
`keySet` 的稳定身份及 remove/removeAll/retainAll/clear 反向修改、精确校验错误、配置
身份、表达式对象惰性单例以及具体 Context 传给自定义工厂的身份。Java
`Set#add` 的 `UnsupportedOperationException` 在 Rust 由不暴露 add 方法的类型系统边界
承接。额外 12 线程 Barrier 测试验证 Rust `OnceLock` 并发访问仍返回同一容器。

本批修复两个真实语义缺口：原组合实现把 `AbstractExpressionContext` 传给工厂而非
具体 `ExpressionContext`；原变量名 getter 每次分配新包装器，未保留 Java
`HashMap#keySet()` 的稳定视图身份。修复后使用共享 `Arc` 实时视图，EngineContext
这类 Java 本就主动构建集合的实现仍允许返回新视图。

完成后累计验证 985 个生产构造器/方法；固定 Java Oracle 增至 59 组、4,295 条记录。

### 12.56 Web Context 批次

本批一次冻结三个主对象的 9 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `IWebContext` | 1 | `src/context/i_web_context.rs` | `getExchange` → `get_exchange`；动态 capability 返回宿主提供的同一 exchange | `BEHAVIOR_VERIFIED` |
| `WebContext` | 4 | `src/context/web_context.rs` | 三个公开构造器和 `getExchange`；父 Context 先完成 Locale/变量复制，再校验 exchange | `BEHAVIOR_VERIFIED` |
| `WebExpressionContext` | 4 | `src/context/web_expression_context.rs` | 三个公开构造器和 `getExchange`；配置校验与变量复制优先，惰性工厂接收具体 Web Context | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`WebContextGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/context/WebContextGolden.java)
生成 43 条记录。Rust 差分覆盖 9 个声明形状、三组构造入口、配置/exchange/值引用
身份、Locale 与变量、稳定实时变量名视图、精确 null 错误以及配置错误优先级；
表达式对象部分验证容器惰性单例、自定义 factory 收到具体
`WebExpressionContext`、具备 `IWebContext` capability 并读到同一 exchange。
额外 12 线程 Barrier 测试验证 `OnceLock` 容器身份和 exchange 身份在并发读取下稳定。

本批修复两个组合映射缺口：`WebContext` 过去在父 Context 构造前校验 exchange，
与 Java `super(...)` 先执行不一致；`WebExpressionContext` 的表达式对象容器过去由
内部基础对象持有，factory 无法观察具体 Web 类型与完整 capability。修复后外层对象
以 `Arc::new_cyclic` 提供具体 `this`，同时使用 `Weak` 避免所有权环。

完成后累计验证 994 个生产构造器/方法；固定 Java Oracle 增至 60 组、4,338 条记录。

### 12.57 Context 工具、惰性变量与 ID 序列批次

本批一次冻结四个主对象的 16 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `ILazyContextVariable` | 1 | `src/context/i_lazy_context_variable.rs` | `getValue` → `get_value`；一级 Context 变量惰性解包合同 | `BEHAVIOR_VERIFIED` |
| `LazyContextVariable` | 3 | `src/context/lazy_context_variable.rs` | protected 构造与 `loadValue` 由闭包组合承接；`OnceLock` 保留成功后单次缓存和失败后重试 | `BEHAVIOR_VERIFIED` |
| `IdentifierSequences` | 4 | `src/context/identifier_sequences.rs` | 构造器、next/previous/increment；每 ID 独立、Java `int` 回绕、精确 null/缺失错误 | `BEHAVIOR_VERIFIED` |
| `Contexts` | 8 | `src/context/contexts.rs` | 私有构造和七个 capability 判定/强转入口；Servlet capability 不把任意中立 exchange 误判为 Servlet | `BEHAVIOR_VERIFIED` |

固定 Java Oracle
[`ContextUtilitiesGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/context/ContextUtilitiesGolden.java)
生成 46 条记录。Rust 差分覆盖全部声明形状、惰性值的身份/null/失败后重试、普通
ID 序列、Unicode、null 与缺失异常、普通/Web Context capability、exchange 身份及
错误强转。Java 使用反射把私有计数 Map 种子置为 `Integer.MAX_VALUE`；Rust 在同模块
单元测试以相同 Oracle 验证 `MAX_VALUE → MIN_VALUE` 回绕和 previous 值。额外 12 线程
Barrier 测试验证 `LazyContextVariable` 在并发读取下只调用一次 loader 并返回同一值。

本批修复 `Contexts#isServletWebContext` 的真实偏差：以前任何 `IWebExchange` 都被
视作 Servlet exchange；现在仅显式 `as_servlet_web_exchange()` capability 为真，非法
强转以 `ContextsError` 作为 `ClassCastException` 等价 runtime failure。

完成后累计验证 1,010 个生产构造器/方法；固定 Java Oracle 增至 61 组、4,384 条记录。

### 12.58 Engine Context 层级与 Web 属性批次

本批一次冻结五个主对象的 86 个 Java 声明：

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `AbstractEngineContext` | 10 | `src/context/abstract_engine_context.rs` | 配置、Locale、解析属性、模型工厂、消息、链接、表达式对象与 ID 序列；`OnceLock` 保留 Java 首次访问时才构造的时序 | `BEHAVIOR_VERIFIED` |
| `ITemplateContext` | 12 | `src/context/i_template_context.rs` | 模板/元素栈、selection、inliner、消息、链接和 identifier SPI | `BEHAVIOR_VERIFIED` |
| `IEngineContext` | 12 | `src/context/i_engine_context.rs` | 变量、selection、inliner、TemplateData、元素层级和 level 可变 SPI | `BEHAVIOR_VERIFIED` |
| `EngineContext` | 26 | `src/context/engine_context.rs` | 分层变量与删除阴影、lazy 解包、selection 的显式 null、inliner、模板/元素栈和诊断串 | `BEHAVIOR_VERIFIED` |
| `WebEngineContext` | 26 | `src/context/web_engine_context.rs` | exchange 属性回滚、特殊 Web Map、层级变更顺序、显式 null selection 与 Web 诊断串 | `BEHAVIOR_VERIFIED` |

固定 Java Oracle [`EngineContextGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/context/EngineContextGolden.java)
生成 41 条记录；[`engine_context_java_parity.rs`](../../thymeleaf-test/tests/engine_context_java_parity.rs)
逐项比较普通/Web 变量层级、根/嵌套 TemplateData 栈、lazy 解包、selection null 阴影和
Web 属性回滚。该批修复了三项真实偏差：最内层显式 null selection 曾错误回退到父层；
Web 诊断串曾附加 Java 不存在的 exchange 私有内容并排序错误；`AbstractEngineContext`
曾在构造期提前读取 expression object factory。现在 factory 与 `IdentifierSequences` 都在
首次 getter 调用时才初始化。

完成后累计验证 1,096 个生产构造器/方法；固定 Java Oracle 增至 62 组、4,425 条记录。

### 12.59 EngineEventUtils 文本事件判定批次

`EngineEventUtils` 的 11 个 Java 声明由
[`EngineEventUtilsGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/engine/EngineEventUtilsGolden.java)
固定为 14 条记录，并由
[`engine_event_utils_java_parity.rs`](../../thymeleaf-test/tests/engine_event_utils_java_parity.rs) 消费。
三组 `isWhitespace`、三组 `isInlineable` 和私有构造/解析入口形状均被固定；Text、CDATA、
Comment 的 null、空串、Java Unicode whitespace、`[[...]]`、`[(...)]` 与 malformed 边界
逐项比较。`computeAttributeExpression` 的 Attribute 缓存路径继续由真实 Engine/Processor
调用链测试覆盖，不把它替换为独立的简化表达式解析器。

完成后累计验证 1,107 个生产构造器/方法；固定 Java Oracle 增至 63 组、4,439 条记录。

### 12.60 ElementProcessorIterator 动态重算批次

本批冻结 `ElementProcessorIterator` 的 8 个 Java 声明（构造器、reset、next、重复
处理器分支、lastWasRepeated、setLastToBeRepeated、recompute 与 clone reset）。上游
[`ElementProcessorIteratorTest`](../../../workspace-github/thymeleaf/tests/thymeleaf-tests-core/src/test/java/org/thymeleaf/engine/ElementProcessorIteratorTest.java)
的动态属性添加/删除场景由
[`ElementProcessorIteratorGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/engine/ElementProcessorIteratorGolden.java)
固定为 9 条记录，并由同一 Rust 源文件内的单元测试逐项消费。

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `ElementProcessorIterator` | 8 | `src/engine/element_processor_iterator.rs` | 访问快照、已访问状态按 Processor 身份继承、属性驱动重算、重复和 clone | `BEHAVIOR_VERIFIED` |

测试不向标签注入伪造 Processor 列表：它以测试方言 Processor 经生产
`AttributeDefinitions → ElementDefinitions → Attributes → OpenElementTag` 建立关联，再用
真实不可变 `set_attribute` / `remove_attribute` 替换标签。对照覆盖 5→10 基线、追加
precedence 15/7/2、新属性插入时已访问项不重放、源属性删除前后重算，以及空快照。
本批修复 Rust 特有的真实缺口：裸指针在旧 `Arc` 释放后可被地址复用，导致新标签跳过
重算；现在为每个 `AbstractProcessableElementTag` 分配内部实例身份号，等价 Java 引用
身份比较，且不会受分配器复用影响。相同 precedence 的冲突诊断也补齐两个违规
Processor 的 Java 类名。

完成后累计验证 1,115 个生产构造器/方法；固定 Java Oracle 增至 64 组、4,448 条记录。

### 12.61 ElementTagStructureHandler 结构动作批次

`ElementTagStructureHandler` 的 33 个 Java 声明由
[`ElementTagStructureHandlerGolden.java`](../../thymeleaf-test/tests/java/org/thymeleaf/engine/ElementTagStructureHandlerGolden.java)
固定状态机和真实属性结果。Rust 测试以生产 `OpenElementTag`、`Model`、`EngineContext`
验证互斥结构动作、模型身份、变量/属性跨动作保留、属性三阶段及上下文副作用顺序。

| Java 对象 | 声明数 | Rust 落点 | 方法族与处置 | 状态 |
|:---|---:|:---|:---|:---:|
| `ElementTagStructureHandler` | 33 | `src/engine/element_tag_structure_handler.rs` | reset、正文/模型、插入/替换/删除/迭代、变量、属性、selection/inliner/template data、内部应用 | `BEHAVIOR_VERIFIED` |

属性动作不按调用时间交错：Java 和 Rust 均先删除、再替换、最后设置。测试通过真实 parser
输入 `<element data-a='one' data-b='two'>` 固定结果 `data-c=final,data-d=null`；上下文测试
验证同名变量先 set 后 remove 时最终不可见，且 selection 与 TemplateData 以相同顺序落位。

完成后累计验证 1,148 个生产构造器/方法；固定 Java Oracle 增至 65 组、4,454 条记录。

## 13. 后续更新门禁

生产语义继续按域批量迁移，S11 批次结案时统一：

1. 从机器清单提取该对象的全部方法、重载和参数；
2. 在本表登记每个 Java 签名的 Rust API；
3. 记录错误、null、顺序、副作用和可见性差异；
4. 先标记 `IMPLEMENTED_UNVERIFIED`；
5. Java Golden/Rust 差分通过后才标记 `BEHAVIOR_VERIFIED`；
6. 重新执行 100% 语义覆盖门禁；源码覆盖率仅记录为诊断指标。
