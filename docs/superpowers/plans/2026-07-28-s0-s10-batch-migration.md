# S0-S10 批量语义迁移计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Rust 中完整迁移 Thymeleaf 3.1.5.RELEASE 的核心模板语义，覆盖 S0（基线建立）到 S10（框架适配器）全部生产代码语义域。

**Architecture:** 单一核心 crate `thymeleaf`（569 个 .rs 文件）承载 Engine、Parser、Expression、Standard Dialect、Web 中立合同；15 个 `thymeleaf-{framework}` 适配器 crate + `thymeleaf-vernal` 桥接层。采用"批量语义迁移优先"策略：先按完整调用链和语义域批量实现全部生产对象，S11 统一验证。

**Tech Stack:** Rust 1.88+（MSRV）、html5gum（HTML tokenizer）、cargo-deny/cargo-audit（治理）、cargo-public-api（API 基线）、CodeGraph（调用链分析）

**Related Design Doc:** `docs/superpowers/specs/2026-07-28-architecture-design.md`

---

## 全局约定

- **上游基线**：Thymeleaf 3.1.5.RELEASE，commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`
- **一文件一主对象**：一个 Java 主对象对应一个 `.rs` 文件
- **名称默认一致**：类型保持 Java 简单名称；目录/文件/方法/参数转 snake_case
- **中立内核**：`thymeleaf` 不依赖 Axum、Actix Web、Topcoat、Vernal 或其他宿主
- **批量迁移优先**：S1-S10 期间只运行 `cargo fmt`、`cargo check`、清单审计等快速反馈；Java Oracle、Rust 单元测试、`.thtest`、覆盖率统一在 S11 执行

---

## 实施阶段总览

| Stage | 目标 | 主对象数 | 日期 |
|-------|------|---------|------|
| S0 | 基线、对象表、语义表、名称门禁 | 0 | 2026-07-28 |
| S1 | crate 骨架、根 API、错误、模板模式 | 20 | 2026-07-28 |
| S2 | Resource、Resolver、Cache | 30 | 2026-07-28~29 |
| S3 | Context、Engine、Event、Model、Handler | 128 | 2026-07-28~29 |
| S4 | 六种模板模式 Parser 与解耦逻辑 | 41 | 2026-07-29 |
| S5 | Processor/Dialect/Pre/Post/Inline SPI | 48 | 2026-07-29 |
| S6 | 核心表达式对象、Message、Link、Util | 64 | 2026-07-29 |
| S7 | Standard Expression、Inline、Serializer | 88 | 2026-07-29 |
| S8 | Standard `th:*` Processor | 56 | 2026-07-29 |
| S9 | 中立 Web 合同和 Servlet 语义等价迁移 | 16 | 2026-07-29 |
| S10 | `thymeleaf-{framework}`、`thymeleaf-vernal` | — | 2026-07-29~31 |

---

## Stage S0 — 迁移治理基线

### Task S0.1：建立迁移文档基线

**Files:**
- Create: `docs/migration/迁移路线图.md`
- Create: `docs/migration/对象级对照表.md`
- Create: `docs/migration/语义迁移对照表.md`
- Create: `docs/migration/对象名称一致性检查.md`
- Create: `docs/migration/Thymeleaf-Rust-迁移技术要求.md`

- [x] **Step 1:** 建立四份迁移文档，引用同一上游版本和提交
- [x] **Step 2:** 491 个主对象全部有唯一 Rust 目标
- [x] **Step 3:** 491 个默认目标路径碰撞数为 0
- [x] **Step 4:** 语义域、阶段和对象包之间无未分类项

---

## Stage S1 — 根 API、错误与模板模式（20 主对象）

### Task S1.1：crate 骨架与根 API

**Files:**
- Create: `thymeleaf/Cargo.toml`
- Create: `thymeleaf/src/lib.rs`
- Create: `thymeleaf/src/template_engine.rs`
- Create: `thymeleaf/src/i_template_engine.rs`
- Create: `thymeleaf/src/template_spec.rs`

- [x] **Step 1:** 建立 workspace 和核心 crate 骨架
- [x] **Step 2:** 迁移 `TemplateEngine`、`ITemplateEngine`、`TemplateSpec`
- [x] **Step 3:** 配置状态机（初始化前可配置、初始化后冻结）

### Task S1.2：异常体系（9 主对象）

**Files:**
- Create: `thymeleaf/src/exceptions/` 目录（9 个 .rs 文件）

- [x] **Step 1:** 迁移 `TemplateEngineException`、输入/处理/输出异常
- [x] **Step 2:** 保留源码位置（模板名、行、列和原因链）

### Task S1.3：模板模式（1 主对象）

**Files:**
- Create: `thymeleaf/src/templatemode/template_mode.rs`

- [x] **Step 1:** 迁移 `TemplateMode::{HTML, XML, TEXT, JAVASCRIPT, CSS, RAW}`

---

## Stage S2 — 模板资源、解析和缓存（30 主对象）

### Task S2.1：缓存域（13 主对象）

**Files:**
- Create: `thymeleaf/src/cache/` 目录（13 个 .rs 文件）

- [x] **Step 1:** 迁移 Always/TTL/Non-cacheable validity
- [x] **Step 2:** 迁移 Template、Expression、Specific 缓存隔离
- [x] **Step 3:** 12/13 达到 BEHAVIOR_VERIFIED（StandardCache 保持 IMPLEMENTED_UNVERIFIED）

### Task S2.2：模板解析器域（10 主对象）

**Files:**
- Create: `thymeleaf/src/templateresolver/` 目录（10 个 .rs 文件）

- [x] **Step 1:** 迁移 Resolver 按 order 顺序尝试语义
- [x] **Step 2:** 10/10 达到 BEHAVIOR_VERIFIED

### Task S2.3：模板资源域（7 主对象）

**Files:**
- Create: `thymeleaf/src/templateresource/` 目录（7 个 .rs 文件）

- [x] **Step 1:** 迁移 ClassLoader/URL/String/WebApplication 资源
- [x] **Step 2:** 7/7 达到 BEHAVIOR_VERIFIED

---

## Stage S3 — Context、Engine、Event、Model 与 Handler（128 主对象）

### Task S3.1：Context 域（20 主对象）

**Files:**
- Create: `thymeleaf/src/context/` 目录

- [x] **Step 1:** 迁移 EngineContextManager、selection target、inliner、变量层级
- [x] **Step 2:** 迁移 WebContext、WebExpressionContext

### Task S3.2：Engine 域（88 主对象）

**Files:**
- Create: `thymeleaf/src/engine/` 目录

- [x] **Step 1:** 迁移 TemplateManager.parseAndProcess 缓存命中/未命中双路径
- [x] **Step 2:** 迁移 ITemplateHandler 链和结构处理 Handler

### Task S3.3：Model 域（20 主对象）

**Files:**
- Create: `thymeleaf/src/model/` 目录

- [x] **Step 1:** 迁移不可变 TemplateModel 与可变 Model
- [x] **Step 2:** Model 插入时禁止外部加入 TemplateStart/End

---

## Stage S4 — 六种 Parser 与解耦逻辑（41 主对象）

### Task S4.1：Parser 抽象与 Markup Parser

**Files:**
- Create: `thymeleaf/src/templateparser/` 目录
- Create: `thymeleaf/src/markup/` 目录

- [x] **Step 1:** 迁移 HTML 宽容标记解析、XML 严格解析
- [x] **Step 2:** 迁移 selector、offset、嵌套模板

### Task S4.2：Text/Raw/Reader Parser

**Files:**
- Create: `thymeleaf/src/text/` 目录
- Create: `thymeleaf/src/raw/` 目录
- Create: `thymeleaf/src/reader/` 目录

- [x] **Step 1:** 迁移 TEXT 原型语法和可见控制标记
- [x] **Step 2:** 迁移 RAW 原样输出
- [x] **Step 3:** 5/41 达到 BEHAVIOR_VERIFIED（BlockAwareReader + comment Readers）

### Task S4.3：解耦逻辑

**Files:**
- Create: `thymeleaf/src/decoupled/` 目录

- [x] **Step 1:** 迁移 decoupled template logic

---

## Stage S5 — Processor 与 Dialect SPI（48 主对象）

### Task S5.1：Processor 基础（34 主对象）

**Files:**
- Create: `thymeleaf/src/processor/` 目录

- [x] **Step 1:** 迁移 Processor 按 Dialect/Processor precedence 稳定排序
- [x] **Step 2:** 迁移 StructureHandler（remove、replace、insert、iterate 等）
- [x] **Step 3:** 七类非元素 StructureHandler 28/28 达到 BEHAVIOR_VERIFIED

### Task S5.2：Dialect SPI（8 主对象）

**Files:**
- Create: `thymeleaf/src/dialect/` 目录

- [x] **Step 1:** 迁移 Dialect 可贡献 Processor、ExpressionObject、ExecutionAttribute

### Task S5.3：Pre/Post/Inline SPI（6 主对象）

**Files:**
- Create: `thymeleaf/src/preprocessor/` 目录
- Create: `thymeleaf/src/postprocessor/` 目录
- Create: `thymeleaf/src/inline/` 目录

- [x] **Step 1:** 迁移 PreProcessor、PostProcessor 与 Output Handler 顺序
- [x] **Step 2:** 迁移 IInlinePreProcessorHandler 12 个同步事件回调

---

## Stage S6 — 表达式服务、Message、Link 与通用对象（64 主对象）

### Task S6.1：核心表达式对象（20 主对象）

**Files:**
- Create: `thymeleaf/src/expression/` 目录

- [x] **Step 1:** 迁移 IStandardVariableExpressionEvaluator
- [x] **Step 2:** 迁移 Rust 原生 evaluator（只读、受控的 OGNL 兼容层）

### Task S6.2：Message Resolver（4 主对象）

**Files:**
- Create: `thymeleaf/src/messageresolver/` 目录

- [x] **Step 1:** 迁移 Message 按 resolver order、locale、template stack 回退
- [x] **Step 2:** 4/4 达到 BEHAVIOR_VERIFIED

### Task S6.3：Link Builder（3 主对象）

**Files:**
- Create: `thymeleaf/src/linkbuilder/` 目录

- [x] **Step 1:** 迁移 context-relative、server-relative、absolute、protocol-relative

### Task S6.4：Util/Temporal（37 主对象）

**Files:**
- Create: `thymeleaf/src/util/` 目录
- Create: `thymeleaf/src/temporal/` 目录

- [x] **Step 1:** 迁移值模型（Null、Boolean、Number、String、List、Map、Object）
- [x] **Step 2:** 迁移 Locale、日期、聚合、列表、集合、映射工具
- [x] **Step 3:** 17/64 达到 BEHAVIOR_VERIFIED

---

## Stage S7 — Standard Expression、Inline 与 Serializer（88 主对象）

### Task S7.1：Standard Expression（70 主对象）

**Files:**
- Create: `thymeleaf/src/standard/expression/` 目录

- [x] **Step 1:** 迁移 `${}` variable、`*{}` selection、`#{}` message、`@{}` link、`~{}` fragment
- [x] **Step 2:** 迁移 literal、token、binary operation、conditional、default、assignation、each、expression sequence
- [x] **Step 3:** 迁移 `__...__` preprocessing

### Task S7.2：Standard Inline（9 主对象）

**Files:**
- Create: `thymeleaf/src/standard/` 目录

- [x] **Step 1:** 迁移内联表达式处理

### Task S7.3：Serializer（5 主对象）

**Files:**
- Create: `thymeleaf/src/serializer/` 目录

- [x] **Step 1:** 迁移 HTML/XML/TEXT 序列化器

---

## Stage S8 — Standard `th:*` Processor（56 主对象）

### Task S8.1：Standard Dialect Processor

**Files:**
- Create: `thymeleaf/src/standard/` 目录下各 processor 文件

- [x] **Step 1:** 迁移 `th:text`、`th:utext`、`th:if`、`th:unless`、`th:each` 等
- [x] **Step 2:** 迁移 `th:with`、`th:attr`、`th:attrprepend`、`th:attrappend`
- [x] **Step 3:** 迁移 `th:switch`、`th:case`、`th:fragment`、`th:replace`、`th:insert`

---

## Stage S9 — 中立 Web 合同（16 主对象）

### Task S9.1：Web 中立合同

**Files:**
- Create: `thymeleaf/src/web/` 目录

- [x] **Step 1:** 迁移 IWebContext、WebContext、WebExpressionContext
- [x] **Step 2:** 迁移完整/流式 Body 与 Host 合同

---

## Stage S10 — 框架适配器

### Task S10.1：独立框架适配器

**Files:**
- Create: `thymeleaf-topcoat/`、`thymeleaf-axum/`、`thymeleaf-actix-web/` 等 13 个适配器 crate

- [x] **Step 1:** 迁移各框架的请求/响应适配
- [x] **Step 2:** 28 个适配器/Hyper 宿主合同测试通过

### Task S10.2：Vernal 桥接层

**Files:**
- Create: `thymeleaf-vernal/`

- [x] **Step 1:** 迁移 Vernal Web/View 宿主适配
- [x] **Step 2:** 不用 SpEL 语义替换 OGNL
