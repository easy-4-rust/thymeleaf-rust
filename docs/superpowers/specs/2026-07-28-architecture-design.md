# Thymeleaf-Rust 可行性与架构设计

- **日期**：2026-07-28
- **作者**：thymeleaf-rust 团队
- **状态**：已实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`)
- **相关计划**：`docs/superpowers/plans/2026-07-28-s0-s10-batch-migration.md`

---

# Thymeleaf-Rust 可行性与架构设计

> 文档状态：实现支撑的架构基线
> 版本：v1.1
> 创建日期：2026-07-28
> 最后更新：2026-07-31
> 项目/仓库名称：`thymeleaf-rust`
> 对外发布主 crate：`thymeleaf`
> 核心定位：Web 框架中立、Vernal 中立
> 独立集成：`thymeleaf-{framework}`
> Vernal 集成：`thymeleaf-vernal`

## 1. 文档目的

本文记录在 Rust 中迁移 Thymeleaf 核心模板语义的架构决策、当前实现和后续生产化
边界。项目向 Topcoat、Actix Web、Axum、Gotham、Hyper、Ntex、Poem、Rocket、
Salvo、Tide、Warp、Tower、Tonic 等框架提供统一的动态内容渲染能力。

本文重点回答：

1. 当前 Rust 内核是否保留 Thymeleaf 的关键运行时拓扑；
2. Resolver、Parser、TemplateModel、Processor、Expression 和 Writer 如何协作；
3. 如何保持框架无关的 Rust 渲染内核；
4. 完整渲染、节流渲染和数据驱动渲染如何穿过中立 Web 边界；
5. 独立框架适配器与 `thymeleaf-vernal` 如何保持平级；
6. 当前实现、明确策略差异和生产化风险分别是什么；
7. 哪些证据支撑兼容性结论。

本评估基于以下代码与设计资料：

- [Thymeleaf 上游源码](https://github.com/thymeleaf/thymeleaf)
- Vernal Framework 的 Spring 组件替换约定与 Web 组件设计资料
- Vernal HTTP、Web、Expression、Cache、Tower 及各 Web 框架适配器源码
- 两个本地仓库的 CodeGraph 索引、符号关系和三层影响分析
- [`migration/对象级对照表.md`](migration/对象级对照表.md)、
  [`migration/方法级对照表.md`](migration/方法级对照表.md)和
  [`migration/迁移测试对照表.md`](migration/迁移测试对照表.md)

## 2. 核心结论

### 2.1 可行性判断

Rust 原生、兼容 Thymeleaf 核心模板语义的 SSR 引擎已经证明可行。当前实现首先是
独立、中立的 `thymeleaf`，然后由薄适配器选择性接入各 Web 框架和 Vernal。

当前状态必须分层描述：

| 范围 | 当前状态 | 证据与边界 |
|:---|:---:|:---|
| Engine、Parser、TemplateModel、Processor、Fragment、Dialect、缓存 | 已实现 | 491 个主对象、69 个内部对象和 4,291 个方法均已处置 |
| Thymeleaf 核心可比较模板行为 | 已验证 | 2,595 / 2,595 个固定上游 `.thtest` 行为一致 |
| Java SOURCE_PARITY | 已闭合 | 875 / 875 源码入口、2,156 / 2,156 运行时 case、0 `MISSING` |
| Spring MVC、WebFlux、SpEL、Spring Security 类型 | 明确不进入核心 | 以 `POLICY_DIFFERENCE` 和中立宿主合同登记 |
| 独立框架适配器 | 已实现薄层 | 13 个框架 crate 与 `thymeleaf-vernal` 可构建；28 个适配器/Hyper 宿主合同测试通过，真实服务器生命周期测试待加强 |
| crates.io 发布 | 未完成 | 发布前仍需能力矩阵、安全审计和适配器验收 |

推荐的产品定位是：

> Thymeleaf-Rust 是一个迁移 Thymeleaf 3.1.5.RELEASE 核心模板语义、采用 Rust 原生
> 并发与类型模型、通过中立渲染协议接入多个 Web 框架的运行时动态内容渲染引擎。
> 它既可以独立集成 Web 框架，也可以作为 Vernal 的可选渲染框架。

不建议采用以下方案：

- 不采用逐行语法翻译；对象和方法必须可追踪，但逻辑按 Rust 类型与 trait 语义落位；
- 不复制 Java 的继承层次和任意反射模型；
- 不建议让渲染内核依赖 Axum、Actix Web、Topcoat、Vernal 或其他宿主框架；
- 不建议在不同适配器中复制模板解析和渲染逻辑；
- 不把 Spring 专用 API 策略差异或尚未完成的宿主测试描述成核心行为一致。

### 2.2 强制中立性原则

以下原则属于项目架构红线：

1. 核心 crate `thymeleaf` 不依赖任何 Web 框架；
2. 核心 crate `thymeleaf` 不依赖 Vernal；
3. `thymeleaf::web` 内部模块只依赖 `http`、`http-body`、`bytes`、`mime` 等中立协议 crate；
4. Topcoat、Actix Web、Axum、Gotham、Hyper、Ntex、Poem、Rocket、Salvo、Tide、Warp、Tower、Tonic 均通过独立可选适配器直接接入；
5. `thymeleaf-vernal` 是消费者和桥接层，不是 Thymeleaf-Rust 核心的上游依赖；
6. 所有独立适配器和 Vernal 适配器必须复用同一个 TemplateEngine、TemplateModel、Processor 和渲染协议；
7. 任何框架专属类型都不得进入 `thymeleaf` 的 Engine、Parser、Expression、Standard Dialect 等公共 API。

两种集成模式必须同时成立：

```mermaid
flowchart LR
    ENGINE["thymeleaf<br/>中立渲染引擎"]

    ENGINE --> DIRECT["独立集成"]
    DIRECT --> TOP["topcoat"]
    DIRECT --> ACTIX["actix-web"]
    DIRECT --> AXUM["axum"]
    DIRECT --> OTHER["gotham / hyper / ntex / poem / rocket / salvo / tide / warp / tower / tonic"]

    ENGINE --> VERNAL["thymeleaf-vernal"]
    VERNAL --> VTOP["vernal-topcoat"]
    VERNAL --> VACTIX["vernal-actix-web"]
    VERNAL --> VAXUM["vernal-axum"]
    VERNAL --> VOTHER["其他 vernal-* Web 场景"]
```

### 2.3 推荐的中立渲染边界

Thymeleaf-Rust 自身的公共结果应为中立类型：

```rust
RenderedTemplate
```

它可以表达：

```text
RenderedTemplate
├── Full(Bytes)
└── Stream(Stream<Frame<Bytes>>)
```

核心 crate 的 `thymeleaf::web` 模块可以进一步提供中立的：

```rust
http::Response<RenderedTemplateBody>
```

随后形成两条彼此独立的转换路径：

- 独立框架适配器：转换为各框架的 `Response`、`Responder`、`Reply`、`Body` 或 `Service`；
- Vernal 适配器：转换为 `vernal_http::HttpBody` 和 Vernal `ViewEngine`。

Vernal 已有的 `HttpBody` 支持：

- 空 Body；
- 完整 `Bytes` Body；
- 流式 `Frame<Bytes>` Body；
- HTTP Trailer；
- 下游背压；
- 客户端取消；
- 显式限制大小的 Body 收集。

这使其非常适合作为 Vernal 场景的渲染输出协议，但它不是 Thymeleaf-Rust 独立适配器的强制依赖。

### 2.4 命名与发布合同

项目名称、发布名称和 Rust 模块名称必须严格区分：

| 层级 | 名称 | 说明 |
|---|---|---|
| Git 仓库/项目目录 | `thymeleaf-rust` | 仅用于代码仓库和项目识别 |
| crates.io 核心 crate | `thymeleaf` | 用户默认依赖；包含 Engine、Parser、Expression、Standard Dialect 与中立 Web 输出 |
| 框架整合 crate | `thymeleaf-axum`、`thymeleaf-actix-web` 等 | 统一使用 `thymeleaf-{framework}` |
| Rust 根路径 | `thymeleaf::...` | 业务代码面向的稳定 API |
| Rust 内部模块 | `engine`、`context`、`parser`、`expression`、`standard`、`web` 等 | 不带项目名重复前缀 |
| Vernal 整合 crate | `thymeleaf-vernal` | 由 Thymeleaf 侧提供、面向 Vernal 的可选整合；命名与 `thymeleaf-spring` 一致 |

禁止发布或创建：

- `thymeleaf-rust-core`；
- `thymeleaf-rust-web`；
- `thymeleaf-rust-axum`；
- `thymeleaf_rust` Rust 根模块；
- 其他任何 `thymeleaf-rust-*` 子 crate 或模块。

推荐的用户 API：

```rust
use thymeleaf::{Context, TemplateEngine};
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
```

使用 Axum 等独立适配器时，依赖名称为 `thymeleaf-axum`。该整合 crate 通过扩展 trait 暴露能力，但面向业务的核心类型仍来自 `thymeleaf::...`。

## 3. Java 与 Rust 核心架构

Thymeleaf 不是 HTML 字符串替换器，而是 Resolver、Parser、事件模型、Processor 链和
输出 Handler 组合的执行引擎。CodeGraph 证明 Rust 版保留了相同的运行时拓扑，而不是
把模板交给第三方模板引擎。

### 3.1 CodeGraph 结构证据

| 仓库 | 索引文件 | 符号节点 | 关系边 | 主要语言 |
|:---|---:|---:|---:|:---|
| Thymeleaf Java | 1,783 | 29,289 | 75,439 | Java 1,543 个文件 |
| thymeleaf-rust | 824 | 18,238 | 68,085 | Rust 751、Java Golden 69 |

Java `TemplateManager` 的三层影响面包含 92 个符号，横跨核心 Engine、Fragment、
Inliner、Spring MVC、WebFlux 和示例应用；Rust `TemplateManager` 的三层影响面为
35 个符号，集中在 Engine、Configuration 和 Manager SPI。数量差异主要来自 Spring、
Servlet 和示例应用被移出 Rust 核心，不能单独用来推断功能缺失。

### 3.2 Java 上游调用链

```mermaid
flowchart LR
    API["TemplateEngine.process / processThrottled"] --> INIT["惰性 initialize"]
    INIT --> CONFIG["EngineConfiguration<br/>冻结 Resolver / Dialect / Cache"]
    API --> MANAGER["TemplateManager.parseAndProcess"]
    MANAGER --> CACHE{"Template Cache"}
    CACHE -->|命中| MODEL["TemplateModel"]
    CACHE -->|未命中| RESOLVER["TemplateResolver 链"]
    RESOLVER --> MODE{"TemplateMode"}
    MODE --> MARKUP["HTML / XML Parser"]
    MODE --> TEXT["TEXT / JavaScript / CSS Parser"]
    MODE --> RAW["RAW Parser"]
    MARKUP --> EVENTS["Template Event Stream"]
    TEXT --> EVENTS
    RAW --> EVENTS
    EVENTS --> MODEL
    MODEL --> PRE["PreProcessor"]
    PRE --> PROCESSOR["ProcessorTemplateHandler"]
    PROCESSOR --> EXPR["Standard Expression + OGNL"]
    EXPR --> POST["PostProcessor"]
    POST --> OUTPUT["OutputTemplateHandler"]
    OUTPUT --> WRITER["Writer / Throttled Writer"]
```

### 3.3 thymeleaf-rust 当前核心调用链

```mermaid
flowchart TB
    INPUT["TemplateSpec + IContext"] --> ENGINE["TemplateEngine<br/>OnceLock + Mutex 冻结配置"]
    ENGINE --> CONFIG["EngineConfiguration<br/>应用级不可变运行时快照"]
    DIALECT["DialectSetConfiguration<br/>聚合 Processor / Pre / Post<br/>ExecutionAttribute / ExpressionObjectFactory"] --> CONFIG
    RESOLVERS["TemplateResolver · MessageResolver<br/>LinkBuilder · CacheManager"] --> CONFIG
    ENGINE --> MANAGER["TemplateManager<br/>parse_and_process / parse_and_process_throttled"]
    CONFIG --> MANAGER
    MANAGER --> CACHE{"Template Cache 命中？"}
    CACHE -->|命中| MODEL["Arc&lt;TemplateModel&gt;<br/>不可变、可重放"]
    CACHE -->|未命中| RESOLVER["按 order 执行 ITemplateResolver trait 链"]
    RESOLVER --> RESOURCE["TemplateResolution + TemplateResource"]
    RESOURCE --> MODE{"TemplateMode"}
    MODE --> MARKUP["HTMLTemplateParser / XMLTemplateParser"]
    MODE --> TEXT["Text / JavaScript / CSS Parser"]
    MODE --> RAW["RawTemplateParser"]
    MARKUP --> EVENTS["ITemplateEvent 流"]
    TEXT --> EVENTS
    RAW --> EVENTS
    EVENTS --> BUILDER["ModelBuilderTemplateHandler"]
    BUILDER --> MODEL
    MODEL -. "仅 Validity 允许时写入" .-> STORE["Template Cache"]
    MODEL --> CHAIN["PreProcessor → ProcessorTemplateHandler → PostProcessor"]
    CHAIN --> FULL["OutputTemplateHandler → TemplateWriter<br/>完整 UTF-16 输出"]
    CHAIN --> THROTTLED["ThrottledTemplateProcessor<br/>FlowController 驱动分块输出"]

    CONFIG -. "固定顺序、优先级和协作者" .-> CHAIN
    CHAIN -. "Processor 按需调用" .-> COLLAB["StandardExpressionParser / Evaluator<br/>MessageResolver / LinkBuilder / Fragment"]
```

运行步骤如下：

1. `TemplateEngine` 首次调用时安装默认 `StringTemplateResolver`，构造并冻结
   `EngineConfiguration`；
2. `DialectSetConfiguration` 按 TemplateMode 和 Processor 类型聚合方言能力；
3. `TemplateManager` 先检查模板缓存，再按 Resolver order 解析模板；
4. `TemplateResolution` 决定资源、模式、缓存有效性和解耦逻辑；
5. 六种 Parser 把资源转换成模板事件，并由 `ModelBuilderTemplateHandler` 构成不可变
   `TemplateModel`；缓存有效性只决定是否保存该模型，不改变后续 Handler 语义；
6. Handler 链依次执行 PreProcessor、`ProcessorTemplateHandler`、PostProcessor 和输出；
7. Standard Dialect 注册标准 `th:*` Processor，表达式 Parser 负责外层语法、预处理和缓存；
8. 完整渲染写入 `TemplateWriter`，节流渲染复用同一模型和 Processor 链，由 FlowController
   控制推进。

### 3.4 配置聚合是核心控制平面

CodeGraph 给出的初始化调用链是：

```text
TemplateEngine::initialize
  → EngineConfiguration::new
    → DialectSetConfiguration::build
      → TemplateManager::new
```

`DialectSetConfiguration` 不是普通配置 DTO。它在 Engine 对外可用前完成以下一次性工作：

1. 按模板模式和 Processor 类型分类、包装并排序全部方言 Processor；
2. 聚合 PreProcessor、PostProcessor、ExecutionAttribute 和 ExpressionObjectFactory；
3. 构造 `ElementDefinitions` 与 `AttributeDefinitions`，并回注需要定义感知能力的对象；
4. 固定 Standard Dialect 是否存在及其实际前缀；
5. 把冲突执行属性、非法 Processor 集合和 Handler 构造能力等问题转化为初始化失败。

这形成了清晰的状态边界：

| 平面 | 创建时机 | 可变性 | 失败影响 |
|:---|:---|:---|:---|
| 配置控制平面 | Engine 首次初始化 | 初始化后冻结 | 阻止 Engine 进入可服务状态 |
| 解析与模型平面 | 模板首次解析或缓存未命中 | `TemplateModel` 不可变，可缓存重放 | 当前模板解析失败 |
| 请求执行平面 | 每次渲染 | Context 与 Handler 游标请求级隔离 | 当前渲染失败 |
| 交付平面 | 输出阶段 | Full 固定；Stream 按 Frame 推进 | 同步错误或流内晚期错误 |

因此，第三方 Dialect 的扩展点位于配置控制平面和请求执行平面的交界处，而不是宿主
Web 适配层。框架适配器不能自行聚合 Processor、改变 precedence 或创建第二套
ExpressionObjectFactory。

### 3.5 忠实迁移与 Rust 原生替代

| 保留的可观察语义 | Rust 实现策略 |
|:---|:---|
| Resolver 顺序、缓存键、TemplateMode | 稳定排序、类型化 key、Rust enum |
| 不可变、可缓存、可重放的 TemplateModel | `Arc<TemplateModel>` 和事件 trait |
| Dialect 与 Processor precedence | trait SPI、`ProcessorSet` 和能力方法 |
| Context 层级、Selection、Locale、变量不存在与 null | trait capability、`TemplateValue`、`Locale` |
| Java UTF-16 字符、哈希、索引边界 | `Utf16String` 保存 UTF-16 code unit |
| 完整与节流双输出 | `TemplateWriter`、`IThrottledTemplateProcessor`、中立 HTTP Body |
| OGNL 常用只读语义 | 内建 AST 和安全求值器，不开放任意反射/Class/静态调用 |
| Java SoftReference 自动回收 | 显式缓存策略；不伪造 GC 行为 |

Spring、Servlet、Reactive Streams 和 SpEL 类型不进入核心；其宿主能力由
`thymeleaf::web` 与独立 `thymeleaf-{framework}` crate 承接。

## 4. 当前总体架构

### 4.1 核心组件与依赖方向

```mermaid
flowchart TB
    APP["调用方<br/>TemplateSpec + IContext"] --> API

    subgraph CRATE["thymeleaf：单一、完整、框架中立的核心 crate"]
        subgraph API["公共入口"]
            API_ENGINE["TemplateEngine / ITemplateEngine"]
            API_SPEC["TemplateSpec"]
            API_CONTEXT["Context / IContext"]
            API_SPEC --> API_ENGINE
            API_CONTEXT --> API_ENGINE
        end

        subgraph CONTROL["应用级、初始化后冻结的控制平面"]
            CONFIG["EngineConfiguration"]
            DIALECT_SET["DialectSetConfiguration<br/>Processor 分类/排序 · Pre/Post<br/>ExecutionAttribute · ExpressionObjectFactory"]
            SPI["Dialect · Processor · TemplateResolver<br/>MessageResolver · LinkBuilder · CacheManager"]
            SPI --> DIALECT_SET --> CONFIG
        end

        subgraph INGEST["解析与模型平面"]
            MANAGER["TemplateManager"]
            CACHE_HIT{"Template Cache 命中？"}
            RESOLVER["有序 TemplateResolver 链"]
            RESOURCE["TemplateResolution + TemplateResource<br/>TemplateMode + Validity"]
            PARSER["HTML / XML / TEXT / JS / CSS / RAW Parser"]
            BUILDER["ModelBuilderTemplateHandler"]
            MODEL["不可变、可重放 TemplateModel<br/>IEngineTemplateEvent 序列"]
            TEMPLATE_CACHE["Template Cache<br/>Arc&lt;TemplateModel&gt;"]

            MANAGER --> CACHE_HIT
            CACHE_HIT -->|是| MODEL
            CACHE_HIT -->|否| RESOLVER --> RESOURCE --> PARSER --> BUILDER --> MODEL
            MODEL -. "仅 Validity 允许时保存" .-> TEMPLATE_CACHE
            TEMPLATE_CACHE -. "后续读取" .-> CACHE_HIT
        end

        subgraph EXECUTION["请求级事件执行平面"]
            CONTEXT_FACTORY["IEngineContextFactory<br/>StandardEngineContextFactory"]
            CONTEXT_MANAGER["EngineContextManager<br/>根层创建 · 嵌套复用 · 层级恢复"]
            ENGINE_CONTEXT["EngineContext<br/>变量、Locale、Selection、Web Capability"]
            CHAIN["PreProcessor → ProcessorTemplateHandler → PostProcessor"]
            SERVICES["Expression · Message · Link · Fragment<br/>TemplateValue + 安全求值"]
            EVENTS["处理后的模板事件"]

            CONTEXT_FACTORY --> CONTEXT_MANAGER --> ENGINE_CONTEXT
            ENGINE_CONTEXT --> CHAIN
            MODEL --> CHAIN --> EVENTS
            CHAIN -. "按 Processor 语义调用" .-> SERVICES
        end

        subgraph DELIVERY["交付平面"]
            FULL["process / process_to_writer<br/>OutputTemplateHandler → TemplateWriter"]
            THROTTLED["process_throttled<br/>ThrottledTemplateProcessor + FlowController"]
            EVENTS --> FULL
            EVENTS --> THROTTLED
            FULL --> FULL_RESULT["Utf16String / Writer"]
            THROTTLED --> THROTTLED_RESULT["IThrottledTemplateProcessor"]
        end

        API_ENGINE --> MANAGER
        API_ENGINE --> CONTEXT_MANAGER
        CONFIG -. "固定顺序、优先级与策略" .-> MANAGER
        CONFIG -. "提供冻结后的 Context Factory" .-> CONTEXT_FACTORY
        CONFIG -. "提供 Processor 与协作者" .-> CHAIN
    end

    FULL_RESULT --> RESULT["非 Web 完整输出"]
    THROTTLED_RESULT --> RESULT

    FRAMEWORK["Axum / Actix Web / Rocket 等原生类型"] -. "不得进入核心签名" .-> CRATE
    VERNAL["Vernal 原生类型"] -. "不得进入核心签名" .-> CRATE
```

这不是“Parser 直接生成 HTML”的流水线，而是配置平面驱动事件平面的执行引擎：

1. `TemplateEngine` 首次执行时冻结 Resolver、Dialect、Processor、Cache、Message 和
   Link 等有序协作者；
2. `TemplateManager` 决定缓存命中、资源解析和模板模式；当前 Rust 实现在缓存未命中时
   始终通过 `ModelBuilderTemplateHandler` 物化 `TemplateModel`，`Validity` 只决定
   是否把模型保存进缓存；
3. `EngineContext` 和 Handler Chain 属于一次渲染，`ProcessorTemplateHandler` 消费
   统一事件协议，结构处理结果继续作为事件流动；
4. 完整与节流入口复用相同 Resolver、Parser、Model、Processor 和表达式语义，只在
   输出驱动与背压边界分叉；
5. `thymeleaf::web::ThymeleafRenderer` 位于核心执行管线之外，只通过
   `ITemplateEngine` 调用 `process` 或 `process_throttled`，再把结果封装为中立 HTTP
   输出；框架专属类型不能反向进入 Engine、Parser、Expression 或 Processor SPI。

根模板与嵌套模板的 Context 生命周期也只有一条路径：根层由
`StandardEngineContextFactory` 检查 `IWebContext` capability，创建普通
`EngineContext` 或保持同一 exchange 身份的 `WebEngineContext`；嵌套层由
`EngineContextManager` 复用现有对象，先提升 level，再压入新的 `TemplateData`，
退出时按层级恢复。`WebContext` 与 `WebExpressionContext` 因而共享同一工厂和处理链，
Web capability 不会把核心分叉为另一套 Engine。

关键对象的生命周期和扩展边界：

| 对象/平面 | 生命周期 | 状态所有权 | 允许扩展 | 禁止进入 |
|---|---|---|---|---|
| `TemplateEngine` / `EngineConfiguration` | 应用级 | 冻结后的 Resolver、Dialect、Processor、Cache 配置 | 注册中立 SPI | Request/Response、框架容器类型 |
| `TemplateManager` / Parser | 随 Engine | 模板解析器与缓存访问 | `ITemplateResolver`、`ITemplateResource` | Controller、Middleware |
| `TemplateModel` | 可跨请求缓存 | 不可变事件序列与 `TemplateData` | Processor 对事件的请求级解释 | 宿主 Session、连接状态 |
| `EngineContext` / Handler Chain | 请求级 | 变量、Locale、Selection、执行游标 | Expression、Message、Link、Fragment 协作者 | 跨请求共享可变状态 |
| `ThymeleafRenderer` / `RenderedTemplate` | 响应级 | Charset、Header、Full/Stream Body | 标准 `http` / `http-body` 消费方 | Axum、Actix Web、Rocket 等原生类型 |

错误边界同样属于核心架构合同：

| 失败位置 | 当前传播方式 | Header 是否已固定 | 适配器责任 |
|:---|:---|:---:|:---|
| Engine 初始化、Dialect 聚合 | `RenderError` 同步返回 | 否 | 映射宿主错误页或 5xx |
| Resolver、Parser、Full Processor | `process` 错误同步返回 | 否 | 不得重试执行模板，除非应用显式定义策略 |
| Stream 创建前的元数据/线程创建 | `render_stream` 同步返回错误 | 否 | 映射宿主错误 |
| Stream 工作线程中的解析/处理 | Body 产生 `Err(RenderError)` 项并终止 | 是 | 无损转发并记录，不能再替换状态/Header |
| 客户端断连、Body 丢弃 | Receiver 关闭，工作线程停止后续发送 | 是 | 观察断连并释放宿主请求资源 |

这张表限定了适配器能做什么：它可以映射错误类型和连接生命周期，但不能吞掉流内错误、
重新执行 Processor 链或在 Header 发出后伪造第二个响应。

核心一次完整渲染的实际调用关系如下：

```mermaid
sequenceDiagram
    participant App as 调用方
    participant Engine as TemplateEngine
    participant Manager as TemplateManager
    participant Cache as Template Cache
    participant Resolver as TemplateResolver
    participant Parser as Mode Parser
    participant Model as TemplateModel
    participant Chain as Processor Handler Chain
    participant Writer as TemplateWriter

    App->>Engine: process(TemplateSpec, IContext)
    Engine->>Manager: parse_and_process(...)
    Manager->>Cache: get(TemplateCacheKey)
    alt 缓存命中
        Cache-->>Manager: Arc<TemplateModel>
    else 缓存未命中
        Manager->>Resolver: resolve_template(...)
        Resolver-->>Manager: TemplateResolution
        Manager->>Parser: parse_standalone(..., ModelBuilder)
        Parser-->>Manager: TemplateModel
        opt Resolution Validity 可缓存
            Manager->>Cache: put(Arc<TemplateModel>)
        end
    end
    Manager->>Chain: replay model events
    Chain->>Writer: processed output events
    Writer-->>Engine: UTF-16 output / I/O result
    Engine-->>App: Utf16String or Result
```

### 4.2 中立 Web 与宿主适配架构

```mermaid
flowchart LR
    subgraph HOST["宿主框架"]
        HOST_INPUT["Request / Session / Application"]
        HOST_OUTPUT["Response / Responder / Reply / Service"]
    end

    subgraph INTEGRATION["整合 crate：只做协议适配"]
        INPUT_ADAPTER["入站 Capability Wrapper"]
        OUTPUT_ADAPTER["Status / Header / Body 转换"]
    end

    subgraph NEUTRAL["thymeleaf：中立 Web 与渲染合同"]
        INPUT_PORTS["IWebApplication · IWebExchange<br/>IWebRequest · IWebSession"]
        CONTEXT["WebContext / IContext<br/>可选 Web Capability"]
        PLAIN["普通 IContext<br/>非 Web 渲染"]
        REQUEST["TemplateSpec + Arc&lt;dyn IContext&gt;"]
        RENDERER["ThymeleafRenderer"]
        CORE["同一 ITemplateEngine<br/>Resolver / Parser / Model / Processor"]
        FULL_ENGINE["process → Utf16String"]
        STREAM_ENGINE["process_throttled → IThrottledTemplateProcessor"]
        FULL["Charset 编码 + Content-Length<br/>RenderedTemplateBody::Full(Bytes)"]
        STREAM["工作线程驱动 Processor<br/>容量 1 Frame 通道形成背压"]
        DATA["DataDrivenTemplateIterator + Signal<br/>可选数据驱动输入"]
        RESULT["RenderedTemplate<br/>StatusCode + HeaderMap + RenderedTemplateBody"]

        INPUT_PORTS --> CONTEXT --> REQUEST
        PLAIN --> REQUEST
        REQUEST --> RENDERER
        RENDERER --> CORE
        CORE --> FULL_ENGINE --> FULL --> RESULT
        CORE --> STREAM_ENGINE --> STREAM --> RESULT
        DATA -. "喂入并唤醒同一节流路径" .-> STREAM_ENGINE
    end

    HOST_INPUT --> INPUT_ADAPTER --> INPUT_PORTS
    RESULT --> OUTPUT_ADAPTER --> HOST_OUTPUT

    DIRECT["thymeleaf-{framework}<br/>独立集成"] -. "实现 Wrapper/转换；依赖 thymeleaf" .-> INTEGRATION
    VERNAL["thymeleaf-vernal<br/>可选、平级集成"] -. "实现 Vernal Bridge；依赖 thymeleaf" .-> INTEGRATION
```

Web 中立边界是双向端口，而不是单纯统一 Response：

- **入站端口**：`IWebApplication`、`IWebExchange`、`IWebRequest`、`IWebSession`
  只表达模板引擎可观察的 Web 能力；它们支持 Context 变量、URL 构建、Session/
  Application 属性和 Web TemplateResource，但不泄漏 Axum、Actix Web、Rocket 等类型；
- **执行端口**：`ThymeleafRenderer` 接收 `TemplateSpec` 与普通或 Web `IContext`，
  再调用同一个 `ITemplateEngine`；因此 Web 渲染与非 Web 渲染共享 Resolver、Parser、
  Model 和 Processor Pipeline；
- **出站端口**：`RenderedTemplate` 统一状态、Header 和 Body，宿主适配器只完成原生
  Response/Responder/Reply/Service 转换。

因此，整合层只有两个合法触点：在请求进入时实现中立 Web Capability，在渲染完成后
消费中立 `RenderedTemplate`。控制平面、解析与模型平面、Processor 执行平面都属于
`thymeleaf` 的语义内核，不是适配器扩展面。这个“双端口、单内核”约束同时适用于
`thymeleaf-{framework}` 和 `thymeleaf-vernal`。

这里必须区分三个层次：`ITemplateEngine` 产生 Java 语义完整输出或节流处理器；
`ThymeleafRenderer` 负责字符集、HTTP 元数据、工作线程和 Frame 通道；整合 crate
最后才把中立结果转换为宿主类型。任何适配器都不能跳过 `ThymeleafRenderer` 后自行
定义另一套 Charset、背压或流内错误语义。

CodeGraph 当前只在 `thymeleaf-hyper` 中发现生产级 `HostWebRequest`、
`HostWebExchange` 和 `HostWebSession` 入站实现；13 个框架 crate 均已消费
`RenderedTemplate` 出站合同。所以上图是已经固定的统一端口架构，但不能据此声称所有
框架的入站 Wrapper 已完成。其余宿主必须在真实 HTTP 验收时逐一补齐或明确复用
Hyper/http 层桥接。

当前态、目标态和验收缺口必须分开：

| 能力 | 当前状态 | 目标状态 | 验收证据 |
|---|---|---|---|
| 核心渲染语义 | 已实现，行为验证持续推进 | 与固定 Java 上游保持显式语义处置 | Golden、JUnit SOURCE_PARITY、`.thtest` |
| 中立出站合同 | `RenderedTemplate`、Full/Stream Body 已实现 | 所有宿主无损消费同一状态、Header 和帧语义 | 核心 Web 合同测试 + 适配器合同测试 |
| 中立入站合同 | Capability trait 已实现；Hyper Host Wrapper 已实现 | 每个宿主实现原生 Request/Session/Application 映射，或明确复用共享桥 | 真实请求、Session、URL、Web Resource 测试 |
| 独立框架适配 | 13 个整合 crate 已进入 Workspace | 每个适配器独立发布、独立配置、独立端到端验收 | Full/Stream/Error/Cancellation/Trailer |
| Vernal 集成 | `thymeleaf-vernal` 协议桥已实现 | 与各 `vernal-{framework}` 组合时保持同一渲染语义 | Vernal 宿主端到端合同 |
| 流式执行 | 容量 1 通道提供下游背压；当前每个流启动工作线程 | 明确线程预算、取消传播和受控执行器策略 | 负载、断连、慢消费者、资源上限测试 |

三种部署模式是平级能力，而不是逐层叠加关系：

| 模式 | 编译期依赖路径 | Context/能力来源 | 最终输出 |
|:---|:---|:---|:---|
| 非 Web 渲染 | 业务库/任务 → `thymeleaf` | 普通 `IContext` | `Utf16String`、`TemplateWriter` 或节流处理器 |
| 独立 Web 集成 | 应用 → `thymeleaf-{framework}` → `thymeleaf` | 框架适配器包装 Request/Session/Application | 框架原生 Response/Body |
| Vernal Web 集成 | Vernal 应用 → `thymeleaf-vernal` → `thymeleaf` | Vernal Bridge 提供同一组中立 Capability | Vernal View/HTTP Body |

因此，使用 `thymeleaf-axum` 不需要 Vernal；使用 `thymeleaf-vernal` 也不要求应用直接
依赖 `thymeleaf-axum`。两条 Web 路径在 `thymeleaf` 的中立端口汇合，而不是相互包装。

完整与流式 Web 调用的真实时序如下：

```mermaid
sequenceDiagram
    participant Host as 宿主 Handler
    participant Adapter as thymeleaf-{framework} / thymeleaf-vernal
    participant Renderer as ThymeleafRenderer
    participant Engine as ITemplateEngine
    participant Worker as Render Worker
    participant Processor as IThrottledTemplateProcessor
    participant Body as RenderedTemplateBody

    Host->>Adapter: 原生 Request + View Model
    Adapter->>Renderer: TemplateSpec + IContext
    alt Full
        Renderer->>Engine: process(...)
        Engine-->>Renderer: Utf16String
        Renderer->>Renderer: Charset 编码 + Content-Length
        Renderer-->>Adapter: RenderedTemplate::Full(Bytes)
    else Stream / Data Stream
        Renderer->>Engine: get_configuration()
        Renderer->>Worker: spawn，移交 TemplateSpec + Arc<IContext>
        Renderer-->>Adapter: RenderedTemplate::Stream
        Worker->>Engine: process_throttled(...)
        Engine-->>Worker: IThrottledTemplateProcessor
        loop 直到 is_finished 或取消/错误
            Worker->>Processor: process_output_stream(chunk_size)
            Processor-->>Body: Frame<Bytes> / RenderError
        end
    end
    Adapter-->>Host: 原生 Response / Responder / Reply / Service
```

流式模式在返回 `RenderedTemplate::Stream` 之前只同步完成配置与响应元数据检查；
Resolver、Parser 和 Processor 的后续错误可能发生在工作线程中，此时只能作为 Body
错误结束流，不能再修改已经提交的状态码和 Header。

出站中立边界的具体职责是：

- `ThymeleafRenderer` 负责调用核心 Engine、字符集编码、Content-Type、完整/节流选择；
- `RenderedTemplate` 只保存标准 `StatusCode`、`HeaderMap` 和中立 Body；
- `RenderedTemplateBody::Full` 提供确定长度的 `Bytes`；
- `RenderedTemplateBody::Stream` 提供 `http-body` 数据帧和下游背压；
- `render_data_stream` 只是以 `DataDrivenTemplateIterator` 驱动相同节流管线，不创建
  第二套模板语义；
- 整合 crate 只转换 Status、Header、Body、Responder、Reply 或 Service 类型；
- `thymeleaf-vernal` 与其他整合 crate 平级，不能成为独立适配器的传递依赖。

责任矩阵：

| 能力 | `thymeleaf` 核心/中立 Web 合同 | `thymeleaf-{framework}` | `thymeleaf-vernal` |
|---|---|---|---|
| 模板解析与处理语义 | 唯一实现 | 禁止复制 | 禁止复制 |
| Request/Session/Application | 定义最小 Capability trait | 包装宿主原生对象 | 包装/桥接 Vernal 对象 |
| Link、Message、Fragment、Expression | 唯一实现和 SPI | 只提供 Context 能力 | 可注册 Vernal 侧实现 |
| Charset、Content-Type、Full/Stream | 统一生成 | 转换为宿主 Body | 转换为 Vernal HTTP Body |
| 请求作用域与断连 | 暴露渲染生命周期和错误 | 观察宿主生命周期 | 连接 Vernal 生命周期 |
| 业务 Handler/Controller | 不负责 | 框架应用负责 | Vernal 应用负责 |

依赖方向可以压缩为：

```mermaid
flowchart LR
    HOST["框架原生应用"] --> DIRECT["thymeleaf-{framework}<br/>Topcoat · Actix Web · Axum · Gotham · Hyper · Ntex · Poem<br/>Rocket · Salvo · Tide · Warp · Tower · Tonic"]
    VHOST["vernal-{framework} 应用"] --> VERNAL["thymeleaf-vernal"]
    DIRECT --> CORE["thymeleaf<br/>核心 + 中立 Web 合同"]
    VERNAL --> CORE

    CORE -. "不得依赖" .-> DIRECT
    CORE -. "不得依赖" .-> VERNAL
    DIRECT -. "不依赖" .-> VERNAL
    VERNAL -. "不依赖" .-> DIRECT
```

上图只表达 Cargo/API 依赖方向；运行时数据从核心产生 `RenderedTemplate` 后再流向
适配器。把“运行时输出流向”和“编译期依赖方向”分开，是避免错误理解中立性的关键。

### 4.3 可执行的中立性约束

“框架中立”必须由依赖、API 和测试共同保证，而不是文档口号：

1. `thymeleaf` 的正常依赖图不得包含任何目标 Web 框架或 Vernal crate；
2. `thymeleaf` 公共签名不得出现框架原生 Request、Response、Body、Extractor 或
   Middleware 类型；
3. 每个整合 crate 只能依赖 `thymeleaf` 的公共 API，不得访问内部 Parser、Processor
   实现或复制 Standard Dialect；
4. 同一模板、Context 和渲染选项经不同适配器得到的正文、Content-Type 和错误语义必须
   一致；
5. Full、Stream、Error、Cancellation、Trailer 和客户端断连必须在每个真实宿主中验收；
6. `thymeleaf-vernal` 与所有 `thymeleaf-{framework}` 平级，任何一方不得成为另一方的
   必选或传递依赖；
7. Tonic 只消费动态 String/Bytes、Gateway 或服务内容，不把普通 HTML Body 伪装成
   gRPC 协议响应。

可以用下面的判定表审查任何新适配器：

| 检查项 | 通过条件 | 失败示例 |
|:---|:---|:---|
| 核心依赖 | `thymeleaf` 的 normal dependency 不出现宿主或 Vernal | 核心直接依赖 `axum` |
| 公共类型 | Engine/Parser/Processor API 只出现核心或标准中立类型 | `TemplateEngine::render` 返回 `actix_web::HttpResponse` |
| 模板语义 | Resolver、Parser、Expression、Processor 只在核心实现 | 适配器自行解析 `th:*` |
| Full/Stream | 两者均消费 `RenderedTemplate`，不另建 Body 协议 | Warp 与 Axum 使用不同 Chunk/Error 规则 |
| 错误与取消 | 流内错误无损传递，断连停止消费，不自动重跑模板 | Header 发出后改写为第二个 500 Response |
| Vernal 关系 | 直接适配与 Vernal 适配平级、互不传递依赖 | `thymeleaf-axum` 强制依赖 `thymeleaf-vernal` |

### 4.4 Crate 模型

引擎只发布一个核心 crate。Engine、Context、TemplateModel、事件、错误、Processor/Dialect SPI、各模板 Parser、表达式系统、Standard Dialect、中立 Web 输出和核心测试设施均属于 `thymeleaf` 的内部模块，不再拆分成独立 crate。

| 类型 | Crate | 职责 |
|---|---|---|
| 核心 | `thymeleaf` | 完整模板引擎、稳定公共 API 与中立渲染协议 |
| 整合 | `thymeleaf-topcoat` | Topcoat 独立适配 |
| 整合 | `thymeleaf-actix-web` | Actix Web 独立适配 |
| 整合 | `thymeleaf-axum` | Axum 独立适配 |
| 整合 | `thymeleaf-gotham` | Gotham 独立适配 |
| 整合 | `thymeleaf-hyper` | Hyper 独立适配 |
| 整合 | `thymeleaf-ntex` | Ntex 独立适配 |
| 整合 | `thymeleaf-poem` | Poem 独立适配 |
| 整合 | `thymeleaf-rocket` | Rocket 独立适配 |
| 整合 | `thymeleaf-salvo` | Salvo 独立适配 |
| 整合 | `thymeleaf-tide` | Tide 独立适配 |
| 整合 | `thymeleaf-warp` | Warp 独立适配 |
| 整合 | `thymeleaf-tower` | Tower Service/Layer 独立适配 |
| 整合 | `thymeleaf-tonic` | Tonic 动态内容生成和服务集成 |
| 整合 | `thymeleaf-vernal` | 可选 Vernal bridge、starter、自动配置和 ViewResolver 注册 |

除 `thymeleaf` 外，发布的 crate 都是整合层。框架整合是正式支持面，而不是 Vernal 的附属实现；它们应保持为薄层并独立发布、独立 feature gate、独立测试，不得复制模板解析、表达式求值、Processor 或渲染逻辑。`thymeleaf-vernal` 与上述框架整合并列存在，负责 Vernal 场景的自动装配。

## 5. 核心领域模型

### 5.1 TemplateEngine

`TemplateEngine` 是线程安全、首次处理时完成初始化并冻结配置的引擎入口。当前公开核心
合同由 `ITemplateEngine` 固定：

```rust
process(&TemplateSpec, &dyn IContext) -> TemplateEngineResult<Utf16String>
process_to_writer(&TemplateSpec, &dyn IContext, Box<dyn TemplateWriter>) -> TemplateEngineResult<()>
process_throttled(&TemplateSpec, &dyn IContext)
    -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>>
```

HTTP 结果不是 `TemplateEngine` 的返回类型。`thymeleaf::web::ThymeleafRenderer` 在
该合同之上提供 `render_full`、`render_stream` 和 `render_data_stream`，从而让核心
执行语义与 Web 交付策略保持分离。

当前职责包括：

- 首次调用时构造并冻结 `EngineConfiguration`；
- 聚合 TemplateResolver、Dialect、Processor、MessageResolver、LinkBuilder 与 Cache；
- 通过 `TemplateManager` 解析、缓存并重放 `TemplateModel`；
- 为每次渲染创建请求级 EngineContext 与 Handler Chain；
- 提供完整 Writer 输出和 Java 语义节流处理器。

生产化仍需补齐的横切能力包括 tracing/metrics 预算、流式执行器资源上限以及面向发布的
缓存与诊断策略；这些目标不能写成当前已完成的 API。

Parser 和渲染 Processor 本质上主要是 CPU 工作，不应为了“全异步”而让每一个模板事件都经过 `async fn`。

推荐：

- TemplateResolver 可提供同步和异步实现；
- 模板解析和普通表达式求值保持同步；
- 输出层将渲染事件转换为受背压控制的异步 Stream；
- 真正需要异步访问的数据应在 Controller/Service 中准备；
- 后续再引入显式 `AsyncValue` 或 data-driver，而不是让所有 Processor 异步化。

### 5.2 TemplateModel

推荐使用不可变模型：

```text
Arc<TemplateModel>
```

内部可以采用：

- 紧凑事件数组；
- Arena 索引；
- Interned element/attribute names；
- SourceSpan；
- 子模型范围；
- 预解析表达式引用；
- Fragment 索引。

不建议默认使用可变 DOM。主要原因：

- 每个请求复制 DOM 成本高；
- Processor 的执行顺序难以保持；
- 流式输出能力下降；
- 缓存模型的共享粒度差；
- 原始 HTML 格式容易被 DOM Serializer 规范化。

### 5.3 TemplateEvent

建议支持的事件包括：

```text
TemplateStart
TemplateEnd
DocumentType
XmlDeclaration
OpenElement
CloseElement
StandaloneElement
Attribute
Text
Comment
CData
ProcessingInstruction
Raw
```

每个事件应尽量保留：

- SourceSpan；
- 原始文本片段；
- 规范化名称；
- TemplateMode；
- 当前模板标识；
- 父元素或模型范围；
- 预解析的 Processor 匹配信息。

### 5.4 Processor 与 Dialect

Processor 推荐分为：

- Element Tag Processor；
- Element Model Processor；
- Attribute Processor；
- Text Processor；
- Comment Processor；
- Template Boundary Processor；
- PreProcessor；
- PostProcessor。

每个 Processor 应有确定性排序键：

```text
dialect_precedence
processor_precedence
registration_order
```

最后一个字段用于保证同 precedence 下结果稳定，避免 HashMap 遍历顺序影响模板输出。

Dialect 应负责提供：

- Processor 集合；
- Expression Object；
- Execution Attribute；
- PreProcessor；
- PostProcessor；
- Prefix；
- Dialect precedence。

## 6. 表达式系统设计

### 6.1 外层语法必须由 Thymeleaf-Rust 管理

Thymeleaf 表达式包含不同语义：

| 表达式 | 含义 | 推荐实现 |
|---|---|---|
| `${...}` | Variable Expression | 委托核心 `NativeVariableExpressionEvaluator` |
| `*{...}` | Selection Variable Expression | 设置 Selection Root 后委托同一原生 evaluator |
| `#{...}` | Message Expression | 委托 MessageResolver |
| `@{...}` | Link URL Expression | 委托 LinkBuilder |
| `~{...}` | Fragment Expression | 委托 FragmentResolver |
| `__${...}__` | Preprocessing | 首期谨慎支持或延后 |

`thymeleaf::expression` 内部模块必须提供框架无关的 Parser、AST、值模型和 evaluator。
`thymeleaf-vernal` 只负责把 Vernal Web 请求/响应和上下文接到该合同，不能用
`vernal-expression`（SpEL 语义）替换 OGNL evaluator，也不能让核心反向依赖 Vernal。
Rhai、CEL、evalexpr、JSONPath/JMESPath 都不是 Java OGNL 语法兼容实现，不能作为
默认求值器偷换语言语义。

crates.io 上的 `ognlib` 也只是名称相似的练习项目，不是 Object-Graph Navigation
Language 实现，不能作为迁移依赖。Rhai 适合另行提供完整嵌入式脚本，CEL 适合安全
规则和 Guardrails，evalexpr 适合简单算术/布尔表达式，JSONPath/JMESPath 适合查询
`serde_json::Value`；这些都是应用可选的其他语言，不进入 `thymeleaf` 的 OGNL
兼容合同。

不能直接将整个属性内容交给某一个通用表达式引擎，因为 `#{...}` 在 Thymeleaf 中是消息表达式，而通用模板表达式 Parser 可能将其解释为另一种嵌入表达式。

### 6.2 Rust 值模型

Rust 没有 Java 式反射，因此需要显式值模型。

建议提供：

- `TemplateValue`；
- `ExpressionValue` 适配；
- `PropertyAccessor`；
- `IndexAccessor`；
- `MethodResolver`；
- `FunctionRegistry`；
- `BeanResolver`；
- `TypeConverter`；
- `#[derive(TemplateModel)]` 或等价宏；
- `serde::Serialize` 到模板值的受控转换。

推荐支持的基础值：

```text
Null
Bool
Integer
Float
Decimal
String
Bytes
List
Map
DateTime
Object
Function
SafeHtml
```

### 6.3 OGNL 兼容边界

Rust 生态目前没有成熟、主流且兼容 Java OGNL 语法的实现，因此核心采用：

```text
OGNL source
  → Native Parser / AST
  → TemplateValue
  → property / index / method capability
  → ACL
  → evaluator
```

默认兼容面面向 Thymeleaf 模板的只读求值，覆盖属性和索引访问、方法调用、集合
projection/selection、条件/逻辑/算术/包含/位运算、局部表达式变量、静态成员和构造器
语法。Java 反射无法在 Rust 中隐式复制，宿主对象通过 `TemplateObject` 暴露属性与
方法；静态成员、构造器和宿主类型关系通过 `OgnlRuntime` 显式注册，并继续执行
Thymeleaf 的类型、成员和受限执行上下文 ACL。

`serde_json::Value` 可以作为一种输入适配，但不能成为唯一内部值模型，否则会丢失
Java UTF-16 字符串、数字包装类型、对象身份以及“变量不存在”和“值为 null”的区别。
默认运行时不开放任意对象写入、任意类型反射或脚本执行；这些能力既不是安全的模板
渲染默认值，也不能通过换用另一门脚本语言来冒充 OGNL 兼容。

上游 `.thtest` 的 `%CONTEXT` 是测试夹具，不是模板语法。Rust runner 先通过生产
`VariableExpression`/OGNL 求值器顺序建立 `Context`，再处理未经包装的原模板；不能
把整段 Context 塞进一个虚构的 `th:with`，否则会错误改变 TEXT 模式节点、双花括号
集合字面量、源码行列和赋值作用域。Java 测试专用 Bean、lazy variable、静态工厂与
构造器通过窄化 `TemplateObject`/`OgnlRuntime` 夹具注册，不得扩大默认反射权限。

固定上游 `3.1.5.RELEASE` 语料的统一验证结果为：Parsing 69 / 69、Plain 200 / 200、
Standard Engine Context 996 / 996、专用 Dialect 61 / 61。前三者中 Parsing 被 Plain
包含，Context 与 Plain 不重叠；专用 Dialect 批次包含 13 个 `elementstack` 和 48 个
`inlining/nostandard` 用例，也不与前两批重叠，因此 `verified` 统一范围当前结算
1,257 / 1,257 个不同 `.thtest`。专用批次按 Java 测试的真实方言组合执行，并验证了
Dialect/Processor 优先级、属性/文本/模型处理、片段插入、元素栈以及没有
Standard Dialect 时不执行标准内联。

全部 500 个 `%EXCEPTION` 用例随后作为独立语义域统一执行，严格校验 Java 异常类、
消息和 cause 链；结果为 500 / 500。该批次覆盖 Reader 与嵌套模板异常、模板名和行列、
内联模式、命名模板模式、Dialect 优先级、lazy variable、危险链接协议以及 OGNL
类型化异常和属性 ACL。它与 `verified` 不重叠，因此 `validated` 并集为
1,757 / 1,757。后续语义域继续覆盖 directive、multi-input、Link、内联交互、
Conversion、Aggregation、Markup、Context、Precedence、Web exchange 以及
remove/replace/surround Processor。最终 2,608 个可执行资源全部处置：2,595 个不同
用例通过 Rust 行为验证，12 个上游已禁用 `execinfo` 资源和 1 个任意 Java 反射链
具名处置，0 未解释，语义功能覆盖率为 100%。CI 原样全 workspace 源码覆盖率为
region 59.61%、function 55.82%、line 61.19%，仅作为后续补测诊断指标，不作为
语义迁移和发布的硬门槛。

上游 `instancestaticrestrictions29.thtest` 中
`''.getClass().getClass().getName()` 依赖 Java `Class` 反射链。该语义被登记为默认
安全配置下的有意差异：核心不会为复刻这一用例向模板暴露任意运行时类型对象。宿主若
确有受控类型查询需求，应通过 `TemplateObject`/`OgnlRuntime` 显式注册窄能力，而不是
启用通用反射。

### 6.4 表达式 Guardrails

默认安全策略应包括：

- 禁止任意类型定位；
- 禁止任意构造器调用；
- 禁止进程、文件和网络访问；
- 禁止反射式私有字段访问；
- 默认禁用表达式赋值；
- 只允许显式注册的方法、函数和 Bean；
- 限制表达式 AST 深度；
- 限制集合投影和递归计算量；
- 限制模板递归深度；
- 限制单次渲染输出大小；
- 对求值错误输出模板名和 SourceSpan，但不泄露敏感值。

## 7. 可选 Vernal 集成建议

Thymeleaf-Rust 不依赖 Vernal。根据 `Spring-组件替换约定.md`，Vernal Web 层采用全栈应用线与 API 后端线并存的双轨架构，因此可以通过 `thymeleaf-vernal` 将中立渲染引擎接入两条轨道。

### 7.1 Vernal 场景建议接入的组件

| Vernal 组件 | Thymeleaf-Rust 用途 | 结论 |
|---|---|---|
| `vernal-web` | Vernal `ViewEngine`、`ViewResolver`、`ModelAndView` | Vernal bridge 接入 |
| `vernal-webmvc` | Controller 返回模板名和 Model | Vernal MVC 场景 |
| `vernal-webflux` | 流式 HTML、背压、取消传播 | Vernal 响应式场景 |
| `vernal-http` | 将中立 `RenderedTemplate` 转成 Vernal Response/Body | Vernal bridge 接入 |
| `vernal-tower` | Service/Layer 及请求作用域生命周期 | Vernal Tower 场景 |
| `vernal-expression` | 可选的 `${}`、`*{}` 求值实现 | 通过 ExpressionEvaluator 适配 |
| `vernal-cache` | 可选的模板和表达式缓存实现 | 通过 Cache trait 适配 |
| Moka | 默认或 Vernal 场景的本地缓存实现 | 可选实现，不进入公共合同 |
| `vernal-context` | Request Scope、Locale、请求变量 | Vernal Context 桥接 |
| `vernal-beans` | BeanResolver 和组件注入 | Vernal IoC 桥接 |
| `serde`/`serde_json` | Model 转换、JS 内联序列化 | 中立基础依赖，可由双方复用 |
| `tracing` | 解析和渲染链路追踪 | 中立观测协议，可由双方复用 |
| `metrics`/`vernal-actuator` | 缓存和渲染指标 | Vernal 生产集成 |
| `mime` | Content-Type | 中立 Web 层依赖 |
| `validator` | 表单绑定与错误展示 | 可选 Vernal 表单集成 |

### 7.2 Vernal 双层集成模型

同一个 Thymeleaf-Rust Engine 在 Vernal 中有两层使用方式：

1. `thymeleaf-vernal` 提供统一的 `ThymeleafViewEngine`、配置、Bean、Context、Expression、Cache 和 HTTP bridge；
2. 每个 `vernal-{framework}` 场景将 `ThymeleafViewEngine` 的结果转换成对应框架响应。

```mermaid
flowchart LR
    CTRL["Controller / Handler"] --> VIEW["thymeleaf-vernal<br/>ThymeleafViewEngine"]
    VIEW --> ENGINE["thymeleaf<br/>中立 TemplateEngine"]
    ENGINE --> RESULT["RenderedTemplate"]
    RESULT --> HTTP["vernal-http::HttpBody"]

    HTTP --> TOP["vernal-topcoat"]
    HTTP --> ACTIX["vernal-actix-web"]
    HTTP --> AXUM["vernal-axum"]
    HTTP --> GOTHAM["vernal-gotham"]
    HTTP --> HYPER["vernal-hyper"]
    HTTP --> NTEX["vernal-ntex"]
    HTTP --> POEM["vernal-poem"]
    HTTP --> ROCKET["vernal-rocket"]
    HTTP --> SALVO["vernal-salvo"]
    HTTP --> TIDE["vernal-tide"]
    HTTP --> WARP["vernal-warp"]
    HTTP --> TOWER["vernal-tower"]
    HTTP --> TONIC["vernal-tonic"]
```

### 7.3 当前 Vernal 的真实缺口

在当前源码中，需要优先处理以下问题：

1. `Spring-组件替换约定.md` 规划了 `View`、`ViewResolver` 和 `WebHandler`，但 `vernal-web` 当前尚未提供完整实现；
2. `vernal-webmvc` 和 `vernal-webflux` 仍处于待建状态；
3. `vernal-web::DataBuffer` 内部使用 `Vec<u8>`，部分操作产生复制，不适合作为模板流式输出热路径；
4. 文档中的 Moka 示例使用异步 API，但 Workspace 当前启用的是 Moka `sync` feature；
5. 多个框架适配器已经处理 Request Scope 和 Body 生命周期，但尚未形成统一 View 渲染协议；
6. Topcoat 尚未出现在 Vernal Workspace 依赖中，文档版本与公开版本需要在实施前重新校准。

这些缺口只影响 Vernal bridge，不得阻塞或污染 Thymeleaf-Rust 的独立框架适配。

### 7.4 Moka 缓存建议

模板 AST、表达式 AST 和 Fragment 索引是同步 CPU 对象，推荐使用 Moka sync cache：

```text
TemplateModelCache
ExpressionCache
FragmentSelectorCache
```

只有以下场景需要 Moka future cache：

- 远程模板加载；
- 对象存储模板；
- 配置中心模板；
- 需要异步 single-flight 的模板刷新。

文档和实际 Cargo feature 应统一，避免 API 示例与实现不一致。

## 8. View 抽象建议

### 8.1 ModelAndView

建议在 `vernal-web` 中新增框架无关模型：

```text
ModelAndView
├── view_name
├── model
├── status
├── headers
├── locale
└── render_hints
```

`render_hints` 可包括：

- 完整渲染或流式渲染；
- TemplateMode；
- Content-Type；
- 输出字符集；
- 是否允许缓存；
- 最大输出大小；
- Fragment selector；
- SSE/data-driven 提示。

### 8.2 ViewEngine

概念接口：

```rust
pub trait ViewEngine: Send + Sync {
    fn name(&self) -> &str;

    fn supports(&self, view_name: &str, media_type: Option<&mime::Mime>) -> bool;

    fn render(
        &self,
        request: ViewRenderRequest,
    ) -> ViewRenderFuture;
}
```

由于稳定 Rust 中 trait async 的对象安全和装箱策略需要统一，正式接口可选择：

- `async-trait`；
- 返回 `Pin<Box<dyn Future<...>>>`；
- 静态泛型接口与 dyn-compatible 接口分层。

### 8.3 RenderedView

建议统一为：

```text
RenderedView
├── Full(Bytes)
└── Stream(Stream<Frame<Bytes>>)
```

并携带：

- StatusCode；
- HeaderMap；
- Content-Type；
- 可选 Content-Length；
- 可选 CancellationToken；
- 渲染诊断信息。

`RenderedView` 可以被独立框架适配器直接转换，也可以由 `thymeleaf-vernal` 转换为 `vernal_http::HttpResponse`。

## 9. HTML Parser 技术选型

### 9.1 解析器要求

HTML Parser 需要同时满足：

- 容忍不规范 HTML；
- 支持自然模板属性；
- 尽量保留原始空白、注释、属性顺序和文本；
- 提供 SourceSpan；
- 支持 Fragment selector；
- 支持 Element/Attribute/Text 事件；
- 支持子模型捕获和重放；
- 支持大模板；
- 不依赖浏览器 DOM；
- 满足 `#![forbid(unsafe_code)]` 项目约束。

### 9.2 候选方案

#### html5ever

优势：

- 提供 HTML5 Tokenizer；
- 提供 Tree Builder；
- 支持 fragment parsing；
- 生态成熟。

困难：

- 完整 DOM/Tree Builder 可能规范化不规范 HTML；
- Serializer 可能改变原始属性和空白表现；
- 自然模板希望尽可能保留设计稿原貌；
- 需要自定义 TreeSink 或事件模型。

#### lol_html

优势：

- 低延迟流式 HTML 重写；
- 内置 CSS Selector；
- 提供 SourceLocation；
- 对大文档和低内存场景友好。

困难：

- 目标是流式重写器，不是模板 AST；
- `th:each`、递归 Fragment 和模型替换需要缓存子树；
- 某些不规范 HTML 场景无法在纯流式模式下回溯；
- Processor precedence 和跨模板 Fragment 不属于它的职责。

#### quick-xml

适合：

- XML TemplateMode；
- XHTML 严格模式；
- 事件式 XML 读取和输出。

不适合直接作为 HTML 模式解析器，因为 HTML 不是严格 XML。

### 9.3 推荐方案

首期建议：

1. 定义 Thymeleaf-Rust 自己的 Parser 和 TemplateEvent 接口；
2. HTML 模式评估 `html5ever` tokenizer 或 `lol_html` tokenizer/rewriter能力；
3. 输出统一转换为自有不可变事件模型；
4. 不在公共 API 中泄漏第三方 Parser 类型；
5. XML 模式使用 `quick-xml`；
6. TEXT/JS/CSS/RAW 模式实现独立轻量 Parser。

Parser 技术选型应通过兼容性原型决定，而不是只比较 benchmark。

原型至少覆盖：

- 非法嵌套 HTML；
- 可选闭合标签；
- `<script>`/`<style>` 内容；
- HTML entity；
- Unicode；
- `th:each` 子树捕获；
- Fragment selector；
- `th:replace`；
- Comment 和 Doctype；
- 原始空白和属性顺序；
- SourceSpan 错误定位。

## 10. Processor 和结构修改难点

简单 Processor 如 `th:text` 并不困难，真正复杂的是结构级 Processor。

### 10.1 结构操作

需要支持：

- 删除元素但保留子内容；
- 删除整个元素和子树；
- 替换当前元素；
- 在元素内部插入模型；
- 在元素之前或之后插入模型；
- 重复当前子模型；
- 设置局部变量；
- 改变 selection target；
- 改变 inliner；
- 修改当前元素属性；
- 重新处理插入的模型；
- 控制 Processor 是否继续执行。

### 10.2 th:each

`th:each` 必须：

1. 捕获当前 ElementModel；
2. 求值集合或迭代器；
3. 为每次迭代建立局部 Context level；
4. 注入迭代变量和状态变量；
5. 重放子模型；
6. 在每次迭代后恢复 Context；
7. 支持嵌套循环；
8. 对大集合提供流式输出；
9. 对无限流或异步数据源提供显式 data-driver，而不是隐式阻塞。

### 10.3 Fragment

Fragment 系统至少需要：

- `template :: fragment`；
- 当前模板 Fragment；
- Fragment 参数；
- 命名和位置参数；
- `th:insert`；
- `th:replace`；
- Fragment selector；
- 空 Fragment；
- 无操作 Fragment；
- 模板栈；
- 循环引用检测；
- 最大递归深度；
- Fragment 模型缓存。

建议 Fragment 持有：

```text
Arc<TemplateModel>
ParameterBindings
SyntheticParameterFlag
SourceTemplate
Selector
```

## 11. 输出转义与安全

### 11.1 输出上下文

至少需要区分：

- HTML Text；
- HTML Attribute；
- XML Text；
- XML Attribute；
- URL；
- JavaScript String；
- JavaScript Inline Object；
- CSS String；
- CSS URL；
- RAW；
- SafeHtml。

不能用一个通用 `escape_html()` 处理所有场景。

### 11.2 th:text 与 th:utext

- `th:text`：默认按当前 TemplateMode 和输出位置转义；
- `th:utext`：明确输出不转义内容；
- `SafeHtml`：只能由显式受信任 API 创建；
- 普通 String 不得自动升级为 SafeHtml；
- 日志和错误中不得输出完整敏感 Model 值。

### 11.3 JavaScript 内联

JS 内联应使用 `serde_json` 生成合法 JSON/JavaScript 数据，不能通过字符串拼接实现。

还需要处理：

- `</script>` 提前结束；
- U+2028/U+2029；
- HTML-sensitive 字符；
- JSON null、数字和对象；
- Safe JavaScript 与普通 String 的类型区分。

## 12. 流式渲染与背压

### 12.1 两种渲染模式

#### Full Render

适用：

- 普通页面；
- 小型 Fragment；
- 需要 Content-Length；
- 渲染失败后必须完整返回错误页。

流程：

```mermaid
sequenceDiagram
    participant H as Handler
    participant R as ThymeleafRenderer
    participant E as TemplateEngine
    participant M as TemplateManager
    participant P as Processor Pipeline

    H->>R: render_full(TemplateSpec, Context)
    R->>E: process()
    E->>M: parse_and_process()
    M->>P: parse/replay TemplateModel
    P-->>E: Utf16String
    E-->>R: UTF-16 output
    R->>R: encode charset + Content-Length
    R-->>H: RenderedTemplate::Full(Bytes)
```

#### Stream Render

适用：

- 大页面；
- 大集合；
- 数据驱动渲染；
- 需要降低首字节时间；
- SSE 或分块输出。

流程：

```mermaid
sequenceDiagram
    participant H as Framework Handler
    participant R as ThymeleafRenderer
    participant W as Render Worker
    participant P as ThrottledTemplateProcessor
    participant C as Capacity-1 Channel
    participant B as Framework Body

    H->>R: render_stream(TemplateSpec, Arc<Context>)
    R->>W: spawn thymeleaf-render
    R-->>H: RenderedTemplate::Stream
    W->>P: process_output_stream(chunk_size)
    P->>C: Frame<Bytes>
    Note over C: 容量为 1；下游变慢时发送端阻塞
    B->>C: poll_next()
    C-->>B: Frame<Bytes>
    W->>P: 重复推进，直到 is_finished
    W->>C: close_channel()
    C-->>B: EOF
```

当前实现用独立 OS 线程隔离请求级非并发 Processor，并以容量为 1 的 channel 形成真实
背压。该模型语义清晰，但高并发下的线程数量、栈内存、调度成本和取消延迟必须通过负载
测试量化；必要时应引入可配置执行器或受控线程池，而不能无依据改为无界任务创建。

### 12.2 流式错误策略

HTTP Header 一旦发送，后续渲染错误不能再切换为标准 500 页面。

需要明确：

- Header 发送前的错误：返回完整 Problem Details/错误页；
- Header 发送后的错误：终止 Body，并记录 tracing error；
- 开发模式可向 HTML 注入安全注释；
- 生产模式不得把表达式、路径或 Model 值发送给客户端；
- 记录已输出字节数、模板名、SourceSpan；
- 客户端断开时停止后续发送；长时间运行的单个 Processor 是否能及时取消需要专门测试。

## 13. 缓存设计

### 13.1 TemplateModel Cache

缓存键建议包括：

```text
owner_template
template_name
template_selectors
template_mode
resolver_attributes
locale_if_required
resolver_version
engine_configuration_version
```

缓存值建议为：

```text
Arc<TemplateModel>
```

### 13.2 Expression Cache

缓存键必须区分：

- 外层表达式类型；
- 原始表达式文本；
- Parser configuration；
- TemplateMode；
- 安全策略版本。

缓存值为不可变 Expression AST。

### 13.3 Fragment Selector Cache

对于常用 layout 和 Fragment，可缓存：

```text
(template_model_id, selector) -> ModelRange
```

### 13.4 开发热更新

开发模式可以使用：

- 文件 mtime；
- 文件大小；
- 内容哈希；
- 文件监听；
- 手动清理 API。

生产模式建议：

- 默认不轮询文件；
- 有界容量；
- 可配置 TTL/TTI；
- 暴露缓存命中和逐出指标；
- 支持 Actuator 清理指定模板缓存。

## 14. Web 框架适配评估

### 14.1 双通道支持要求

每个宿主框架都必须允许两种接入方式：

1. 不使用 Vernal：直接依赖 `thymeleaf-{framework}`；
2. 使用 Vernal：通过 `thymeleaf-vernal` 与对应 `vernal-{framework}` 组合。

| 宿主框架 | 独立集成 | Vernal 场景 | 主要集成形态 |
|---|---|---|---|
| Topcoat | `thymeleaf-topcoat` | `thymeleaf-vernal` + `vernal-topcoat` | ViewEngine、Page、Fragment、动态 RawHtml |
| Actix Web | `thymeleaf-actix-web` | `thymeleaf-vernal` + `vernal-actix-web` | Responder、MessageBody、Stream |
| Axum | `thymeleaf-axum` | `thymeleaf-vernal` + `vernal-axum` | IntoResponse、Body、Extension |
| Gotham | `thymeleaf-gotham` | `thymeleaf-vernal` + `vernal-gotham` | Handler、Response、Middleware |
| Hyper | `thymeleaf-hyper` | `thymeleaf-vernal` + `vernal-hyper` | `Response<Body>`、Service |
| Ntex | `thymeleaf-ntex` | `thymeleaf-vernal` + `vernal-ntex` | Responder、Service、Body |
| Poem | `thymeleaf-poem` | `thymeleaf-vernal` + `vernal-poem` | IntoResponse、Endpoint、Stream |
| Rocket | `thymeleaf-rocket` | `thymeleaf-vernal` + `vernal-rocket` | Responder、ByteStream、Fairing |
| Salvo | `thymeleaf-salvo` | `thymeleaf-vernal` + `vernal-salvo` | Writer、Handler、ResBody |
| Tide | `thymeleaf-tide` | `thymeleaf-vernal` + `vernal-tide` | Endpoint、Response、Body |
| Warp | `thymeleaf-warp` | `thymeleaf-vernal` + `vernal-warp` | Reply、Filter、Rejection |
| Tower | `thymeleaf-tower` | `thymeleaf-vernal` + `vernal-tower` | Service、Layer、Response Body |
| Tonic | `thymeleaf-tonic` | `thymeleaf-vernal` + `vernal-tonic` | 动态 String/Bytes、状态详情、Gateway/Service 集成 |

发布阶段可以按优先级逐步完成，但架构和公共 API 不得把任何一个框架定义成二等集成，也不得要求独立用户引入 Vernal。

当前 CodeGraph 结果表明，`RenderedTemplate` 有 27 个适配调用者，覆盖
Actix Web、Axum、Gotham、Ntex、Poem、Rocket、Salvo、Tide、Warp、Hyper、Tower、
Tonic、Topcoat 和 Vernal 等边界。适配器已经是薄层：主要执行状态码、Header 和 Body
转换；模板处理逻辑仍由核心 `ThymeleafRenderer` 完成。

当前共享中立 Web 测试覆盖：

- 完整 Body、Content-Type 和 Content-Length；
- 分块输出和下游逐帧消费；
- 节流处理器完成信号；
- 非法 Charset 错误。

尚未闭合的是每个宿主框架自身的真实 HTTP Full、Stream、Error、Cancellation 和
Trailer 测试。仅编译通过不能等同于宿主运行时兼容。

### 14.2 Axum

推荐提供：

```text
ThymeleafView -> IntoResponse
Result<ThymeleafView, ThymeleafError> -> IntoResponse
```

完整和流式 Body 都可以通过 `axum::body::Body::new()` 包装标准 `http_body::Body`。

Axum 是首批适配的最佳选择之一。

### 14.3 Actix Web

推荐提供：

```text
ThymeleafView -> Responder
TemplateStream -> MessageBody
```

也可以将 Stream 转换为 Actix streaming response。

需要重点测试：

- 非 `Send` Body 边界；
- ServiceResponse 生命周期；
- 请求作用域保持到 Body 结束；
- 客户端取消；
- 错误类型转换。

### 14.4 Poem 与 Salvo

两者都适合提供薄适配器：

- 将 `RenderedView::Full` 转换为 Bytes Body；
- 将 `RenderedView::Stream` 转换为框架 Stream Body；
- 复制 Status、Header 和 Content-Type；
- 保持 CancellationToken。

### 14.5 Rocket

Rocket 原始 Stream 不能直接作为 Handler 返回值，需要：

- 自定义 `Responder`；
- 或使用 ByteStream/ReaderStream；
- 处理 Rocket Response 的生命周期参数；
- 将模板错误映射为 Rocket Status/Catcher。

### 14.6 Warp

Warp 的主要适配难点不是模板渲染，而是：

- Filter 组合类型；
- Reply 抽象；
- Body 类型封装；
- Rejection 错误映射。

应提供一个简单的 `ThymeleafReply`，避免业务侧感知底层 Body 转换。

### 14.7 Hyper 与 Tower

Hyper 和 Tower 更适合作为底层协议和 Service 抽象，而不是面向业务的 MVC 开发体验。

它们对 Thymeleaf-Rust 的价值很高：

- Hyper 提供标准 HTTP Server 和 Body；
- Tower 提供 Service/Layer；
- Axum 和多个框架可以共享 Tower 适配；
- 独立适配器可以直接消费 `RenderedTemplateBody`；
- Vernal 场景则由 `vernal-tower` 处理请求作用域与 Body 生命周期，并使用 `vernal_http::HttpBody`。

### 14.8 Topcoat

Topcoat 自身已经提供：

- `view!` 编译期宏；
- 类型检查的 Rust 模板；
- async component；
- 服务端渲染；
- signal；
- shard；
- 客户端响应式表达式。

Thymeleaf-Rust 的定位与它并不相同：

| Topcoat View | Thymeleaf-Rust |
|---|---|
| Rust 源码中的 `view!` 宏 | 外部 HTML 模板 |
| 编译期类型检查 | 运行时解析和表达式求值 |
| 原生组件和 shard | Fragment 和 Dialect |
| 支持客户端响应式转换 | 主要是服务端 SSR |
| Rust 开发者友好 | Java/Spring/设计师迁移友好 |

独立 Topcoat 应用可以注册 Thymeleaf-Rust View/Page adapter；Vernal Topcoat 应用则通过 `thymeleaf-vernal` 注册不同 `ViewEngine`：

```text
TopcoatViewEngine
ThymeleafViewEngine
```

根据 View 名、文件扩展名或 Content-Type 选择。

Thymeleaf 模板不能自动获得 Topcoat 的客户端响应式能力。若需要二者组合，应使用：

- Thymeleaf 渲染完整 MPA 页面；
- Thymeleaf 页面嵌入 Topcoat shard 占位；
- Topcoat 页面将 Thymeleaf 渲染结果作为受控 RawHtml Fragment；
- 不允许未经安全标记的模板输出直接进入 RawHtml。

Topcoat 仍处于 early-stage。无论独立集成还是 Vernal 集成，都必须由适配器隔离其 API，不应让 Topcoat 类型泄漏到 Thymeleaf-Rust 核心。

### 14.9 Tonic

Tonic 是正式支持的独立集成目标，但集成语义是“动态内容渲染”，不能只用传统 MVC HTML Response 衡量。

`thymeleaf-tonic` 应提供：

- gRPC 方法返回 HTML String/Bytes 字段；
- 邮件模板、消息模板或报告模板内部使用 Thymeleaf-Rust；
- 将模板结果写入 Protobuf `string`/`bytes` 字段；
- 为 gRPC 状态详情、调试页面或管理协议生成动态内容；
- 在 Tonic Service/Interceptor 中注入共享 TemplateEngine；
- 为 gRPC Gateway/BFF 提供模板渲染服务；
- Tonic 与 Hyper/Axum 在同一进程或端口运行，HTML 请求由 HTTP Router 处理；
- gRPC Gateway 将数据交给 Web 层后渲染；
- 后台管理服务内部生成静态 HTML。

`vernal-tonic` 场景通过 `thymeleaf-vernal` 获取同一个 `ThymeleafViewEngine`、Context、Expression、Cache 和观测能力。Tonic adapter 不应把普通 HTML 冒充为 gRPC 协议响应，而应明确地把渲染结果映射到消息字段、Gateway HTTP Response 或内部业务输出。

## 15. 主要技术困难

### 15.1 HTML 解析保真与结构语义冲突

模板引擎既希望保留原始 HTML，又需要理解不规范 HTML 的真实结构。

纯 Token 模型难以处理浏览器式树纠正，完整 DOM 又可能改变原始输出。需要在以下目标之间平衡：

- 自然模板保真；
- HTML5 容错；
- SourceSpan；
- Processor 结构修改；
- Fragment selector；
- 流式输出；
- 内存占用。

这是整个项目最难、最应该先做原型验证的部分。

### 15.2 Java 动态对象到 Rust 强类型模型的迁移

Rust 无法无成本复制：

- 任意 getter；
- 任意方法调用；
- Class/静态方法访问；
- 任意 Bean；
- 动态 Proxy；
- Java Collection 统一接口。

需要明确 Rust 风格的值、Accessor 和 derive 宏，否则模板使用体验会明显弱于 Java Thymeleaf。

### 15.3 Processor precedence 和重处理

结构被 Processor 修改后：

- 后续 Processor 是否处理新结构；
- 原元素 Processor 是否继续；
- 插入 Fragment 是否重新执行；
- 局部 Context 如何恢复；
- Processor 如何跳过自身避免无限递归；
- Dialect 间顺序如何保持稳定。

这些行为必须在 Engine Contract 中明确，并通过兼容性测试固定。

### 15.4 Fragment、Layout 和递归

Fragment 是模板引擎从“变量替换”走向“页面组件系统”的关键，也容易引入：

- 循环依赖；
- 深度递归；
- 参数遮蔽；
- Context 泄漏；
- selector 性能问题；
- 跨模板缓存失效；
- Fragment 输出模式冲突。

### 15.5 流式渲染错误

输出一旦发送就无法回滚到完整错误页。必须定义：

- 何时提交 Header；
- 首个 Chunk 前缓冲多少；
- 中途错误如何终止；
- 是否允许模板自行声明容错区域；
- 客户端断开时如何释放 Context；
- Drop 是否足以完成清理；
- tracing span 何时结束。

### 15.6 多框架生命周期差异

不同框架在以下方面存在差异：

- Body 类型；
- Send/Sync 要求；
- Response 生命周期；
- 错误/拒绝模型；
- Request Scope 存活时间；
- Cancellation；
- Trailer 支持；
- 流式 Response API；
- Header 默认行为。

统一 `http::Response<HttpBody>` 可以大幅降低差异，但不能完全消除专属适配代码。

当前优先风险不是适配器代码量，而是宿主生命周期验证：`RenderedTemplate` 的共享合同
已有测试，但各框架的 Body poll、请求作用域、客户端断开和错误转换仍需真实服务器测试。

### 15.7 流式执行器与资源预算

当前 `render_stream` 和 `render_data_stream` 为每个响应创建一个 OS 线程，并使用容量
为 1 的 channel 建立背压。这能避免请求级 Processor 跨线程并发，但必须量化：

- 高并发下线程数和栈内存；
- 慢客户端造成的发送阻塞时间；
- 客户端取消后工作线程退出延迟；
- 单个慢 Processor 无法及时抢占的问题；
- 数据驱动渲染等待信号时的调度开销。

若压测证明该模型超出预算，应提供可配置执行器或有界渲染线程池，同时保留同一
Processor 不并发执行的语义。

### 15.8 安全和 XSS

模板引擎会直接生成浏览器可执行内容，因此安全要求高于普通字符串模板：

- 转义上下文错误会导致 XSS；
- URL 表达式可能产生开放重定向；
- Bean 调用可能越权；
- Fragment 路径可能目录穿越；
- `th:utext` 和 RawHtml 可能绕过转义；
- 表达式求值可能造成 DoS；
- 热更新目录可能被非授权写入。

### 15.9 兼容性测试规模

需要长期维护 Java Thymeleaf 与 Rust 引擎的差异矩阵。

兼容性不是简单的“模板能运行”，而包括：

- Processor 执行顺序；
- 空值和布尔转换；
- 数字和日期格式；
- HTML entity；
- 属性序列化；
- Fragment 参数；
- Locale 和 Message；
- URL 编码；
- 异常位置；
- 缓存失效；
- 流式 Chunk 边界。

## 16. 主要优势

### 16.1 跨框架统一 SSR

一套模板可以在多个 Rust Web 框架中运行，业务模板不再绑定 Axum、Actix Web 或 Rocket。

### 16.2 Spring/Thymeleaf 迁移路径

对于已有 Spring MVC/Thymeleaf 团队：

- 模板语法熟悉；
- 页面设计方式熟悉；
- Controller + Model + View 模式熟悉；
- Fragment/Layout 思维熟悉；
- 表达式和 Message 使用方式接近。

这能显著降低 Java 到 Rust 的迁移成本。

### 16.3 自然模板

包含 `th:*` 属性的 HTML 可以尽量保持为浏览器和设计工具可打开的普通页面。

这比完全写在 Rust 宏中的模板更适合：

- 前端设计人员；
- UI 外包协作；
- 运营页面；
- 邮件模板；
- 主题系统；
- 运行时可更新页面。

### 16.4 Rust 运行时优势

- 无 JVM；
- 启动快；
- 内存更容易控制；
- 不依赖 GC；
- 线程安全约束更明确；
- `Send + Sync` 边界可验证；
- 错误模型可使用 `thiserror`；
- 可以满足 `#![forbid(unsafe_code)]`；
- 与 Tokio、Bytes、http-body 原生集成。

### 16.5 流式和低复制输出

通过 `Bytes`、`Frame<Bytes>` 和标准 `http_body::Body` 可以实现：

- 避免构造超大 String；
- 降低首字节时间；
- 支持背压；
- 支持取消；
- 保留 Trailer；
- 让 Hyper/Axum/Tower 直接消费；
- 让其他框架只做薄转换。

### 16.6 可选 Vernal 生态复用

在使用 Vernal 的部署场景中，`thymeleaf-vernal` 可以选择性桥接和复用：

- SpEL 风格表达式；
- Bean Container；
- Request Scope；
- Cache；
- HTTP Body；
- Tower Service；
- Web 适配器；
- tracing；
- metrics；
- Actuator；
- Validator。

这可以减少 Vernal 应用的重复建设，但不会改变 Thymeleaf-Rust 的中立性；不使用 Vernal 的应用仍可以只依赖 Core、中立 Web 合同和目标框架适配器。

### 16.7 Dialect 生态

未来可以建立 Rust 方言：

- `vernal-security-dialect`；
- `vernal-form-dialect`；
- `vernal-layout-dialect`；
- `vernal-i18n-dialect`；
- `vernal-htmx-dialect`；
- `vernal-topcoat-dialect`；
- `vernal-assets-dialect`；
- `vernal-csrf-dialect`。

## 17. 可观测性与生产要求

### 17.1 tracing

建议 span：

```text
thymeleaf.render
thymeleaf.resolve
thymeleaf.parse
thymeleaf.cache.lookup
thymeleaf.expression.parse
thymeleaf.expression.evaluate
thymeleaf.fragment.resolve
thymeleaf.processor.execute
thymeleaf.output.stream
```

关键属性：

- template name；
- template mode；
- fragment selector；
- cache hit/miss；
- processor count；
- render mode；
- output bytes；
- chunk count；
- locale；
- framework adapter；
- error kind。

禁止记录：

- 完整 Model；
- 密码、Token、Cookie；
- 未脱敏表达式结果；
- 用户输入的完整敏感值。

### 17.2 metrics

建议指标：

```text
thymeleaf_render_duration_seconds
thymeleaf_render_total
thymeleaf_render_errors_total
thymeleaf_parse_duration_seconds
thymeleaf_template_cache_hits_total
thymeleaf_template_cache_misses_total
thymeleaf_expression_cache_hits_total
thymeleaf_fragment_depth
thymeleaf_output_bytes
thymeleaf_stream_chunks
thymeleaf_stream_cancellations_total
```

### 17.3 Actuator

建议端点：

```text
/actuator/thymeleaf
/actuator/thymeleaf/caches
/actuator/thymeleaf/templates
```

写操作必须受权限控制：

- 清理全部模板缓存；
- 清理指定模板；
- 重新加载 Resolver；
- 查看配置摘要；
- 不返回模板源码和敏感 Context。

## 18. 测试和评估体系

完整迁移采用独立、可审计的治理基线：

- [迁移路线图](migration/迁移路线图.md)：定义上游基线、完成口径、阶段依赖和发布门禁；
- [对象级对照表](migration/对象级对照表.md)：逐项登记 Java 主对象、内部对象及 Rust 落点；
- [方法级对照表](migration/方法级对照表.md)：登记 Java 方法、构造器、重载、参数与 Rust API；
- [语义迁移对照表](migration/语义迁移对照表.md)：固定处理链、模板模式、表达式、Processor、Web 与错误语义；
- [对象名称一致性检查](migration/对象名称一致性检查.md)：约束类型名、文件名、目录布局和允许的 Rust 化例外。

### 18.1 测试金字塔

```mermaid
flowchart TB
    E2E["跨框架 E2E<br/>全部独立适配器 + 全部 vernal-* 组合"]
    COMPAT["Java Thymeleaf 对照测试<br/>Golden Output"]
    INTEGRATION["Engine 集成测试<br/>Resolver + Parser + Processor + Cache"]
    UNIT["单元测试<br/>Parser / Expression / Escaping / Context"]
    PROPERTY["Property/Fuzz<br/>非法 HTML、递归、Unicode、安全"]

    E2E --> COMPAT
    COMPAT --> INTEGRATION
    INTEGRATION --> UNIT
    UNIT --> PROPERTY
```

### 18.2 Java 对照运行器

建议维护独立测试工具：

```text
template
context.json
configuration.json
expected.html
expected-error.json
```

同一测试分别由：

- Thymeleaf Java 3.1；
- Thymeleaf-Rust；

执行，然后比较：

- 输出；
- 错误类型；
- 错误位置；
- Fragment 结果；
- 转义结果；
- 缓存行为。

对于非字节级兼容场景，需要使用规范化比较器，并明确为什么允许差异。

### 18.3 Fuzz 与安全测试

重点目标：

- HTML Tokenizer；
- 表达式 Parser；
- Fragment Parser；
- URL Builder；
- JS/CSS Inline；
- SourceSpan；
- 深度递归；
- 超大属性；
- 非法 Unicode；
- HTML entity；
- Processor 重处理；
- 模板路径规范化。

## 19. 历史分阶段实施路线

本节保留立项时的阶段边界，用于解释当前对象和 crate 为什么这样组织，不再代表实时
待办。核心语义迁移及适配器生产代码已经越过这些阶段；当前优先级以第 24 节为准。

### 19.1 Phase 0：架构合同与技术原型

目标：

- 固定核心 trait；
- 固定 Core 不依赖 Web 框架和 Vernal 的中立性合同；
- 验证 HTML Parser；
- 验证不可变 TemplateModel；
- 验证中立 `RenderedTemplateBody` 流式输出；
- 验证适配器可以在不修改 Core 的情况下转换 Full/Stream 结果。

交付物：

- `TemplateResolver`；
- `TemplateParser`；
- `TemplateModel`；
- `TemplateEvent`；
- `Processor`；
- `ExpressionEvaluator`；
- `TemplateCache`；
- `ViewEngine`；
- `RenderedView`；
- `RenderedTemplateBody`；
- HTML Parser 对比报告；
- Axum Hello Template；
- Hyper/Tower Body 原型；
- 兼容性测试骨架。

退出条件：

- 可以解析和原样输出普通 HTML；
- SourceSpan 正确；
- TemplateModel 可缓存和并发重放；
- Axum 和 Hyper 能独立返回 Full 和 Stream Body；
- 客户端取消能终止流。

### 19.2 Phase 1：可用 MVP

支持：

- HTML TemplateMode；
- FileSystemResolver；
- EmbeddedResolver；
- `${}`；
- `th:text`；
- `th:utext`；
- `th:if`；
- `th:unless`；
- `th:each`；
- `th:with`；
- `th:attr`；
- 常用固定属性 Processor；
- `th:classappend`；
- `th:styleappend`；
- `th:insert`；
- `th:replace`；
- 命名 Fragment；
- HTML 转义；
- Template/Expression Cache；
- 完整 Bytes 响应；
- Axum、Actix Web、Hyper、Tower 和 Topcoat 独立原型适配；
- `thymeleaf-vernal` bridge 原型，但不成为 MVP Engine 的依赖。

### 19.3 Phase 2：独立框架适配套件

支持：

- `thymeleaf-topcoat`；
- `thymeleaf-actix-web`；
- `thymeleaf-axum`；
- `thymeleaf-gotham`；
- `thymeleaf-hyper`；
- `thymeleaf-ntex`；
- `thymeleaf-poem`；
- `thymeleaf-rocket`；
- `thymeleaf-salvo`；
- `thymeleaf-tide`；
- `thymeleaf-warp`；
- `thymeleaf-tower`；
- `thymeleaf-tonic`；
- 每个适配器的 Full、Stream、取消、错误映射和 E2E 测试；
- 各适配器独立 feature、依赖和版本管理。

### 19.4 Phase 3：Vernal Web 正式集成

支持：

- `thymeleaf-vernal`；
- `vernal-web::ViewEngine`；
- `ViewResolver`；
- `ModelAndView`；
- `vernal-webmvc`；
- `vernal-webflux`；
- `vernal-http::HttpBody` 流式渲染；
- 对接所有已存在的 `vernal-{framework}` 适配器；
- Vernal Bean、Context、Expression、Cache、Tracing 和 Metrics 桥接；
- 确保 Vernal bridge 不改变独立适配器公共合同；
- CancellationToken；
- MessageResolver；
- LinkBuilder；
- Selection Expression；
- Locale；
- 热更新；
- tracing；
- metrics。

### 19.5 Phase 4：方言与高级模式

支持：

- Dialect SPI；
- PreProcessor/PostProcessor；
- XML；
- TEXT；
- JAVASCRIPT；
- CSS；
- RAW；
- Decoupled Logic；
- data-driven rendering；
- 表单绑定；
- Validator；
- Security Dialect；
- Layout Dialect。

### 19.6 Phase 5：兼容性与生产化

目标：

- Thymeleaf 3.1 核心功能兼容矩阵；
- 性能基准；
- 故障注入；
- 模板资源限流；
- 缓存运维；
- Actuator；
- 安全审计；
- fuzz；
- 多框架 E2E；
- 版本迁移策略；
- 稳定 Dialect API。

## 20. 历史 MVP 范围控制

本节记录最初的范围约束。当前实现范围与证据状态以第 2、14、18 和 24 节为准。

### 20.1 首期应实现

- HTML；
- 常用变量、条件、循环和属性；
- Fragment；
- 自动转义；
- 文件和嵌入模板；
- 缓存；
- 完整渲染；
- 基本流式渲染；
- 至少两个独立 Web 框架接入；
- 可选 Vernal bridge 原型；
- Golden Test。

### 20.2 首期不应实现

- 全部 Spring Security Dialect；
- 全部 Thymeleaf Extras；
- Servlet/WebFlux API 一比一复制；
- 任意 Java 方法语义；
- 完整 OGNL 兼容；
- 客户端响应式框架；
- 远程分布式模板缓存；
- 所有 Web 框架适配器同时达到 GA，但它们仍属于正式规划的独立支持面；
- XML/JS/CSS 模式全部一次完成；
- 对外承诺完全兼容。

## 21. 性能设计原则

建议遵循：

- TemplateModel 使用 `Arc` 共享；
- 模板名、属性名和 Processor 匹配信息尽量 intern；
- Expression 在模板解析阶段预解析；
- 缓存命中后不重新扫描全部 `th:*` 属性；
- Full Render 使用 `BytesMut`；
- Stream Render 使用有界 Chunk；
- 避免每个事件进行 async 调度；
- 避免 `Vec<u8> -> String -> Vec<u8>` 往返转换；
- 避免在热路径使用 `dyn Any` 多次 downcast；
- 对 Processor 列表预排序；
- 对 Fragment selector 建立索引；
- 限制单次循环和递归深度；
- benchmark 同时测量吞吐、P50/P95/P99、分配和首字节时间。

建议基准场景：

1. 静态 HTML；
2. 100 个表达式；
3. 1000 项 `th:each`；
4. 嵌套 Fragment/Layout；
5. 缓存冷启动；
6. 缓存热命中；
7. Full Render；
8. Stream Render；
9. 多线程并发；
10. 客户端中途取消。

## 22. 许可证和命名

Thymeleaf 源码采用 Apache License 2.0。

如果复用或翻译其源码、测试或实现细节，需要：

- 保留 Apache-2.0 License；
- 保留适用的版权声明；
- 分发时包含 NOTICE；
- 标注修改；
- 不暗示项目由 Thymeleaf 官方维护；
- 谨慎使用 Thymeleaf 名称和 Logo；
- 在 README 中明确“兼容/受启发”而非“官方 Rust 版本”。

推荐描述：

> Thymeleaf-Rust is an independent Rust implementation inspired by and aiming for compatibility with Thymeleaf template semantics. It is not an official Thymeleaf project.

正式发布前仍应进行独立的许可证和商标审查。

## 23. 架构决策建议

### ADR-001：内核不依赖具体 Web 框架

状态：已接受并实现。

决策：

- Core 不依赖 Axum、Actix Web、Topcoat、Vernal 或其他宿主框架；
- Web 层依赖 `http`、`bytes`、`http-body`；
- 每个宿主框架提供独立、可选、薄适配器；
- Vernal 集成放入独立的 `thymeleaf-vernal`；
- 独立适配和 Vernal 适配是并列通道，不是上下级关系。

### ADR-002：采用不可变、可重放的 TemplateModel

状态：已接受并实现。

决策：

- Parser 输出不可变模型；
- 缓存使用 `Arc<TemplateModel>`；
- 每次渲染只创建 Context 和执行游标；
- Processor 通过结构指令改变输出，不直接共享可变 DOM。

### ADR-003：Thymeleaf 外层表达式与 SpEL 求值分离

状态：已接受；核心安全求值器已实现，Vernal 可插拔验证待加强。

决策：

- `${}`、`*{}`、`#{}`、`@{}`、`~{}` 由 Thymeleaf-Rust Parser 识别；
- `${}`、`*{}` 的内部 AST 委托中立 `ExpressionEvaluator`；
- `vernal-expression` 只是 `thymeleaf-vernal` 可注册的一个实现；
- Message、URL、Fragment 使用专属 Resolver；
- 避免 `#{}` 语义冲突。

### ADR-004：统一输出为中立 RenderedTemplate 和标准 HTTP Body

状态：已接受并实现。

决策：

- Full Render 输出 `Bytes`；
- Stream Render 输出 `Stream<Frame<Bytes>>`；
- `thymeleaf::web` 模块提供中立 `RenderedTemplateBody`；
- 独立适配器将其转换为各框架 Response；
- `thymeleaf-vernal` 将其转换为 `vernal_http::HttpBody`；
- Core 不暴露任何框架或 Vernal 类型。

### ADR-005：Topcoat 与 Thymeleaf-Rust 作为并列 ViewEngine

状态：已接受；独立适配器已实现，双 ViewEngine 真实宿主验证待加强。

决策：

- 不让 Thymeleaf-Rust 核心依赖 Topcoat；
- 不尝试把运行时模板自动编译成 Topcoat reactive expression；
- 允许独立 Topcoat 应用直接集成 Thymeleaf-Rust；
- 允许同一 Vernal Topcoat 应用同时注册两个 ViewEngine；
- 通过文件扩展名、View 前缀或显式配置选择。

### ADR-006：Tonic 是正式的动态内容集成目标

状态：已接受；独立适配器已实现，Gateway/Service 端到端测试待加强。

决策：

- 提供独立的 `thymeleaf-tonic`；
- 提供 `thymeleaf-vernal` + `vernal-tonic` 组合；
- 支持将渲染结果映射到 Protobuf String/Bytes、Gateway Response 和内部动态内容；
- 支持在 Tonic Service/Interceptor 中注入共享 TemplateEngine；
- 不把普通 HTML Body 冒充为 gRPC 协议响应。

### ADR-007：仓库名称与发布名称分离

状态：已接受并落实到 Cargo Workspace。

决策：

- 项目和 Git 仓库名称固定为 `thymeleaf-rust`；
- crates.io 主 crate 固定为 `thymeleaf`；
- 对外 Rust API 根路径固定为 `thymeleaf::...`；
- 框架整合 crate 统一命名为 `thymeleaf-{framework}`；
- 禁止创建或发布 `thymeleaf-rust-*` crate；
- 禁止使用 `thymeleaf_rust` 作为 Rust 根模块；
- `thymeleaf-vernal` 表示 Thymeleaf 面向 Vernal 的可选整合 crate，命名遵循与 `thymeleaf-spring` 相同的“核心在前、宿主在后”约定。

## 24. 最终结论与下一阶段

核心架构、语义清单和中立适配边界已经落位。下一阶段不再回到逐对象实现，而是沿以下
生产化主线推进：

```mermaid
flowchart LR
    A["已闭合<br/>核心对象、方法与模板语义"] --> B["适配器真实 HTTP 验收"]
    B --> C["流式线程、背压与取消压测"]
    C --> D["第三方 Dialect / Processor 兼容套件"]
    D --> E["固定能力矩阵与安全边界"]
    E --> F["crates.io 发布审计"]
    F --> G["版本化兼容与持续回归"]
```

最重要的工程判断是：

1. 复用 Thymeleaf 的事件模型、Processor Pipeline、Fragment、Dialect 和缓存思想；
2. 不复制 Java 继承、反射和 Spring 类型；
3. 使用 Rust 原生的不可变模型、显式值访问、`Arc`、`Bytes` 和 `http_body::Body`；
4. Core 不依赖任何 Web 框架，也不依赖 Vernal；
5. 为 Topcoat、Actix Web、Axum、Gotham、Hyper、Ntex、Poem、Rocket、Salvo、Tide、Warp、Tower、Tonic 提供独立适配；
6. 通过 `thymeleaf-vernal` 将同一 Engine 接入所有对应 `vernal-*` 场景；
7. 将 Topcoat 视为可以独立组合、也可以在 Vernal 中并列注册的编译期响应式 ViewEngine；
8. 将 Tonic 视为正式的动态 String/Bytes、Gateway 和服务内容渲染目标；
9. 用 Java/Rust Golden、SOURCE_PARITY 和共享 `.thtest` 持续驱动兼容性；
10. 不用源码覆盖率或对象文件存在替代行为证据；
11. 发布前优先解决各适配器端到端验证和每流一个 OS 线程的资源预算。

建议项目命名：

- 引擎项目：`thymeleaf-rust`
- 对外主 crate：`thymeleaf`
- 独立适配器：`thymeleaf-{framework}`
- Vernal 集成：`thymeleaf-vernal`
- Vernal 宿主组合：`thymeleaf-vernal` + `vernal-{framework}`
- 对外定位：独立、中立、面向 Thymeleaf 核心模板语义兼容的 Rust 动态内容渲染引擎
