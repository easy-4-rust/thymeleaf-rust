# Web 适配器 P0 做厚计划 — thymeleaf-support 核心三件套

- **日期**：2026-08-15
- **状态**：待实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2`）
- **决策依据**：`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/docs/Spring-组件替换约定.md` v1.5（4.2/4.3 节适配器矩阵 + 第十节未决项）

## 1. 目标与范围

把 thymeleaf-rust 从"15 个薄适配器骨架"推进到"**三个生产级深适配器 + 十个薄层保持 + 三个降级实验性**"的稳定形态，并在 Vernal 约定文档正式立项视图层选型。

**核心判断**（基于 2026-08-15 勘察）：

| 现状 | 数据 |
|---|---|
| 适配器总数 | 15 crate（13 web 框架 + vernal + sa-token） |
| 深度标杆 | `thymeleaf-hyper` 710 行（IWebExchange 四件套完整） |
| P0 三件薄层现状 | axum 103 行 / actix 106 行 / topcoat 44 行（**均缺 IWebExchange 四件套**） |
| 渲染入口 | 全同步（`process_template` 等无 async fn）→ spawn_blocking 策略 |
| 契约测试基础 | poem/vernal/tide/gotham/tower/topcoat/warp 已有 `adapter_contract.rs` 模式 |
| CI | `cargo test --workspace --all-features` 已覆盖 `thymeleaf-support/*` |

**非目标**：

- 不做渲染循环 async 化（远期，基于 throttled writer 单点改造）
- 不给 gotham/tide/warp 增加任何新功能（仅降级标记 + 现有测试保持绿）
- 不实现 axum/actix 的完整 Session 存储（只做 `IWebSession` 语义的最小正确实现）

## 2. 四件套契约（P0 适配器的"做厚"定义）

对齐 `thymeleaf-hyper` 标杆结构，每个 P0 适配器补齐：

```
thymeleaf-{axum|actix-web|topcoat}/src/
├── lib.rs                    # 重导出
├── thymeleaf_view.rs         # 已有：渲染结果 → 框架 Response
├── host_web_request.rs       # 新增：impl IWebRequest（~24 个必需方法）
├── host_web_exchange.rs      # 新增：impl IWebExchange（request/session/locale/content-type）
├── host_web_session.rs       # 新增：impl IWebSession（最小语义）
└── host_web_application.rs   # 新增：impl IWebApplication（资源根/上下文路径）
```

**IWebRequest 必需方法清单**（以 `thymeleaf/src/web/i_web_request.rs` trait 定义为准，~24 个）：

- 请求线：`get_method` / `get_scheme` / `get_server_name` / `get_server_port` / `is_secure`
- 路径：`get_application_path` / `get_path_within_application` / `get_query_string`
- Header ×7：`contains_header` / `get_header_count` / `get_all_header_names` / `get_header_map` / `get_header_values`（+ 默认实现的 `get_header_value`）
- Parameter ×7：`contains_parameter` / `get_parameter_count` / `get_all_parameter_names` / `get_parameter_map` / `get_parameter_values`（query 解析 + form 支持）
- Cookie ×5：`contains_cookie` / `get_cookie_count` / `get_all_cookie_names` / `get_cookie_values`
- Locale 与其余默认实现方法按 trait 锚定

**验收口径**：新适配器的 `adapter_contract.rs` 必须与 `thymeleaf-hyper` 的契约测试**逐断言对齐**（同一组请求构造 → 同一组 IWebExchange 语义断言），不允许"各测各的"。

## 3. 框架映射要点（每适配器的语义难点）

### axum 0.8
- Request = `axum::extract::Request`（`http::Request<Body>`）；query 需自行解析 `Uri`（`form_urlencoded`）；cookie 自行解析 `Cookie` header
- 无内建 session → `IWebSession` 用请求级占位实现（`existing()`/`new()` 语义对齐 hyper 标杆），文档标注"会话存储由宿主经 tower_sessions 自行桥接"
- 测试用 `axum::body::Body` + `tower::ServiceExt::oneshot`

### actix-web 4.11
- `actix_web::HttpRequest` 自带 header/cookie/peer/scheme/qs（`query_string()`/`cookies()`），映射最直接
- `ThymeleafBody` 已有（MessageBody 适配），补 `Responder` 全语义
- session 不依赖 actix-session（保持零额外依赖），同 axum 占位策略
- 测试用 `actix_web::test`

### topcoat（轨道一主线）
- topcoat 2026-07-22 发布，API 需以当时 crates.io 文档为准——**Stage 3 第一个 Task 即 API 勘察落档**
- 目标形态对齐 vernal-webmvc 视图解析：`ThymeleafView` 直接成为 vernal-webmvc 的 View 实现后端

## 4. 异步策略（两步走的第一步）

**本计划只做第一步**：每个 P0 适配器的 `ThymeleafView` 增加 `render_async` 入口：

```rust
/// spawn_blocking 包同步渲染（每次渲染付一次线程切换成本，
/// 换取渲染核心零改动、正确性零风险）。
pub async fn render_async(engine: Arc<TemplateEngine>, ...) -> Result<RenderedTemplate, ...> {
    tokio::task::spawn_blocking(move || engine.process_template(...)).await...
}
```

流式/SSE 的第二阶段（基于 `throttled_template_writer`）留待远期计划，不在本计划范围。

## 5. 降级决策

| crate | 动作 | 理由 |
|---|---|---|
| thymeleaf-gotham | README + Cargo.toml 标 `experimental`；移出 CI 必测（feature 隔离） | 上游 2022 后停滞 |
| thymeleaf-tide | 同上 | 0.17-beta，维护停滞 |
| thymeleaf-warp | 同上 | 被 axum 取代中（vernal 优先级 4 → 降级为可用但非重点） |

与 vernal `web-integration-manifest.toml` 的优先级排序一致性：axum(1)/actix(2) 做厚正好对应其 priority 1/2。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| topcoat API 漂移（新框架迭代快） | Stage 3 首个 Task 做版本钉死 + API 勘察落档后再写码 |
| axum query/form 解析语义与 Java `request.getParameterMap` 差异 | 契约测试锚定 hyper 标杆行为；差异点文档化 |
| 13 适配器全量 CI 拖慢 | 降级三件走 feature 隔离；workspace 测试保持默认绿 |
| spawn_blocking 在低并发场景反而更慢 | `render_async` 与同步入口并存，由宿主选择 |
