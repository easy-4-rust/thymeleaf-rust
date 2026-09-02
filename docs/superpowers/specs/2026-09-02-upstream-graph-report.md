# Java 上游 3.1.5 深度图谱报告（迁移对照证据）

- **日期**：2026-09-02
- **状态**：已落档（证据型 spec——非计划非规范，供移植对照与审计引用）
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`）
- **数据来源**：code-review-graph 全量重建（13,760 节点 / 90,024 边 / 1,570 执行流 / 18 社区；与 thymeleaf-rust 侧图谱 14,677 / 125,756 并读）
- **关联**：`2026-07-28-object-level-mapping.md`（对象级对照）、`2026-07-28-method-level-mapping.md`（方法级对照）、`2026-08-15-webmvc-view-integration-notes.md`（vernal-webmvc 对接）

## 一、全图扇出之王与核心热点（图视角"无直接测试直连"清单）

| 热点 | 度数 | 位置 | 移植对照 |
|---|---|---|---|
| `Validate.notNull` | **418** | util/Validate.java | 全图最高扇出——参数校验贯穿引擎每层；Rust 侧 `Validate` 镜像已移植 |
| `StandardDialect.createStandardProcessorsSet` | 255 | standard/StandardDialect.java | 方言处理器注册大工厂——Rust `standard_dialect` 对应，2608 corpus 处理器面聚合点 |
| `TemplateProcessingException` | 200 | exceptions/ | 异常枢纽；Rust 同位类型 + 错误消息基线已锁定 |
| `DialectSetConfiguration.build` | 186 | DialectSetConfiguration.java | 方言合并点——Rust 侧同名（139 度 untested hotspot 的 1:1 上游） |
| `StandardTestBuilder.buildTest` | 150 | lib/testing/ | **corpus 引擎心脏**——2609 个 .thtest 全经它构造 |
| `ElementDefinitions.forHTMLName` / `Strings` / `StringUtils.append` | 136/135/131 | engine/ expression/ util/ | 定义查找 + 表达式工具（均已对象级 parity 锁定） |

**测试策略交叉结论**：以上 20 个图视角"无直接测试覆盖"热点，在 thymeleaf-rust 侧已用对象级 golden parity 直锁（Strings/StringUtils/Validate/ElementDefinitions 的 java_parity 全部移植）。Java 侧靠 corpus 间接覆盖，Rust 侧加对象级直锁——**两侧测试策略互补，Rust 测试面 ≥ 上游**。

## 二、引擎核心分派点：`ProcessorTemplateHandler.handleOpenElement`（38 callees）

渲染引擎心脏分派（每个开标签执行处理器链），五个关键协作方：

1. `queueEvent`（自身）——事件缓冲，保证处理器对模型的修改按序生效
2. `TemplateModelController.shouldProcessOpenElement`——gathering model 判定（`th:each`/局部变量作用域下是否抑制处理）
3. `asEngineOpenElementTag`——用户 tag → 引擎内部 tag 视图转换
4. `obtainCurrentGatheringModel`——收集中的模型（deferred 执行）
5. `IEngineContext.setElementTag`——上下文栈回写

**Rust 对照**：`processor_template_handler.rs::handle_open_element_state`（Rust 图 141 度）1:1 镜像，可逐函数核对分派语义。

## 三、examples 层双胞胎镜像与孤立节点成因

- 50 个孤立节点 100% 来自 GTVG 示例 POJO（`Comment`/`Customer`/`Order` 的 getter/setter）——图内无调用方是**反射调用**特征（OGNL 经 getter 访问，静态图不可见）。
- `ProductRepository`（185 度）在 jakarta/javax 两套 examples 精确双生，与 lib 层 spring5/spring6 双胞胎社区呼应——上游用**双份代码**管理 Servlet API 世代交替。
- 迁移含义：spring5/6、webflux、springsecurity、examples 共 ~37% 节点是生态镜像，Rust 侧只移植 core 是正确范围决策。

## 四、图谱查询操作要点

- 裸名 `TemplateEngine` 匹配 2,573 节点（examples 每个配置类都有 `templateEngine` bean）——`query_graph` 必须用全限定路径（`TemplateEngine.java::TemplateEngine.process`）。
- 增量更新不清陈旧节点（改名后旧符号残留，见 thymeleaf-rust 侧 `java_class_name` 案例）——关键决策前必须磁盘 grep 交叉验证；重要分析用全量重建。
