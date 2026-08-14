# Web 适配器 P0 做厚实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 thymeleaf-support 的 axum/actix-web/topcoat 三个适配器从薄层（44-106 行）做厚到 `thymeleaf-hyper` 标杆深度（IWebExchange 四件套 + 契约测试），gotham/tide/warp 降级实验性，并在 Vernal 约定文档立项视图层选型。

**Architecture:** 每个 P0 适配器补齐 `host_web_request/exchange/session/application` 四件套（impl 核心 crate 的 `IWebRequest` 等 trait，~24 必需方法），契约测试与 hyper 标杆逐断言对齐；`ThymeleafView` 增加 `render_async`（spawn_blocking 包同步渲染）。

**Tech Stack:**
- 已有：thymeleaf 核心（同步渲染、IWeb* trait、2609 corpus）、thymeleaf-hyper 标杆（710 行四件套）、adapter_contract.rs 测试模式、tokio
- 新增依赖：axum 0.8（`form_urlencoded` query 解析）、actix-web 4.11（test 工具）、topcoat（版本 Stage 3 勘察后钉死）

**Related Design Doc:** `docs/superpowers/specs/2026-08-15-web-adapter-p0-design.md`

---

## 全局约定

- **四件套模板**：结构对照 `thymeleaf-support/thymeleaf-hyper/src/`（host_web_request.rs 263 行 / host_web_exchange.rs 172 行 / host_web_session.rs 119 行 / host_web_application.rs 126 行）
- **契约测试口径**：新适配器 `tests/adapter_contract.rs` 的断言集与 hyper 的契约测试**逐条对齐**；不允许各测各的
- **trait 锚点**：`thymeleaf/src/web/i_web_request.rs`（~24 必需方法）、`i_web_exchange.rs`、`i_web_session.rs`、`i_web_application.rs`
- **Session 策略**：axum/actix 均不做框架级 session 依赖，`IWebSession` 请求级占位（对齐 hyper 的 `existing()/new()` 语义），README 标注"会话存储由宿主桥接"
- **提交约定**：conventional commits；每 Task 一 commit；每 Stage 结束跑 `cargo check -p <crate> --all-targets` + `cargo clippy -p <crate> --all-targets -- -D warnings` + `cargo test -p <crate>`（本地受限验证，最终以 CI 为准）
- **并行性**：Stage 1/2/3 相互独立可并行；Stage 4 依赖 1-3 全部完成

---

## 实施阶段总览

| Stage | 目标 | 预期 Task 数 |
|-------|------|-------------|
| 0 | Vernal 约定文档立项（视图层选型落档） | 1 |
| 1 | thymeleaf-axum 做厚（四件套 + 契约测试 + render_async） | 4 |
| 2 | thymeleaf-actix-web 做厚（同上） | 4 |
| 3 | thymeleaf-topcoat 做厚（API 勘察 + 同上） | 5 |
| 4 | 降级 gotham/tide/warp 为实验性（CI 收窄） | 2 |
| 5 | 收尾（CHANGELOG + README 矩阵表 + 全量验证） | 2 |

---

## Stage 0 — Vernal 约定文档立项

### Task 0.1：Spring-组件替换约定.md 补"视图层"选型节

**Files:**
- Modify: `/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/docs/Spring-组件替换约定.md`（第四节后新增小节，并同步第十一节总览表 + web-integration-manifest.toml 状态）

**内容**：
- 新增表格：`ThymeleafViewResolver`/`View` → `thymeleaf-vernal`（`ThymeleafView`）+ `thymeleaf-{axum|actix-web|topcoat}` 适配器；状态 `[待验证]`（P0 完成后升 `[已验证]`）
- 注明与 tera 分工：tera 管通用文本模板（context-support 邮件），thymeleaf 管 HTML 服务端渲染（webmvc 视图解析）
- 引用本 spec 作为设计依据

**验收**：文档 diff 只增不删；第十一节总览表新增行；manifest.toml 对应条目状态更新。

---

## Stage 1 — thymeleaf-axum 做厚

### Task 1.1：host_web_request.rs（IWebRequest 实现）

**Files:**
- Create: `thymeleaf-support/thymeleaf-axum/src/host_web_request.rs`
- Modify: `thymeleaf-support/thymeleaf-axum/src/lib.rs`（mod + pub use）
- Modify: `thymeleaf-support/thymeleaf-axum/Cargo.toml`（加 `form_urlencoded`；确认 tokio dev-dep）

**要点**：
- `from_request(request: &axum::extract::Request, application_path: &str)` 构造（对照 hyper 的 `from_request`）
- query 解析：`Uri::query()` + `form_urlencoded::parse`（多值聚合为 `Vec`，对齐 Java `getParameterMap`）
- cookie 解析：`Cookie` header 手动解析（`;` 分割 + `name=value`），多 cookie 聚合
- ~24 个 trait 方法逐一实现；`get_request_path`/`get_request_url`/`get_header_value` 等默认实现直接继承

### Task 1.2：host_web_exchange/session/application.rs（其余三件套）

**Files:**
- Create: `thymeleaf-support/thymeleaf-axum/src/host_web_exchange.rs`
- Create: `thymeleaf-support/thymeleaf-axum/src/host_web_session.rs`
- Create: `thymeleaf-support/thymeleaf-axum/src/host_web_application.rs`
- Modify: `thymeleaf-support/thymeleaf-axum/src/lib.rs`

**要点**：exchange 持 request/session/可选 locale，`set_content_type`/`set_character_encoding` 可写回（对照 hyper 172 行）；session 请求级占位（`existing()`/`new()`）；application 资源根解析（对照 hyper 126 行）。

### Task 1.3：契约测试对齐 hyper 标杆

**Files:**
- Modify: `thymeleaf-support/thymeleaf-axum/tests/adapter_contract.rs`（已有骨架，扩展为全量契约）

**要点**：
- 断言集逐条对齐 hyper 契约测试：同组请求构造（method/scheme/server/path/query/header/parameter/cookie）→ 同组 `IWebExchange` 语义断言
- 端到端：`tower::ServiceExt::oneshot` 起路由，`ThymeleafView` 响应渲染一个含 `${...}` 的模板，断言 body 与 content-type
- 差异点（如 axum 无内建 session）在测试注释中显式标注

### Task 1.4：render_async（spawn_blocking）

**Files:**
- Modify: `thymeleaf-support/thymeleaf-axum/src/thymeleaf_view.rs`
- Test: 同文件 `#[cfg(test)]`（tokio::test）

**要点**：`render_async(engine: Arc<TemplateEngine>, ...)` = `spawn_blocking(move || engine.process_template(...))`；同步入口保留并存；doc comment 写明"每次渲染付线程切换成本，换取渲染核心零改动"。

---

## Stage 2 — thymeleaf-actix-web 做厚

### Task 2.1：host_web_request.rs

**Files:**
- Create: `thymeleaf-support/thymeleaf-actix-web/src/host_web_request.rs`
- Modify: `thymeleaf-support/thymeleaf-actix-web/src/lib.rs`、`Cargo.toml`

**要点**：`actix_web::HttpRequest` 自带 `query_string()`/`cookies()`/`headers()`/`peer_addr()`/`connection_info()`——映射最直接；query 多值解析同样用 `form_urlencoded`；cookie 用 `request.cookies()`。

### Task 2.2：host_web_exchange/session/application.rs

同 Task 1.2 结构（对照 hyper）。

### Task 2.3：契约测试对齐 hyper 标杆

**Files:**
- Create/Modify: `thymeleaf-support/thymeleaf-actix-web/tests/adapter_contract.rs`

**要点**：`actix_web::test` 构造请求；断言集同 Task 1.3；端到端走 `ThymeleafBody`（已有 MessageBody 适配）+ `Responder`。

### Task 2.4：render_async

同 Task 1.4（actix 侧 `ThymeleafView`）。

---

## Stage 3 — thymeleaf-topcoat 做厚（轨道一主线）

### Task 3.1：topcoat API 勘察落档（写码前置）

**Files:**
- Create: `docs/superpowers/specs/2026-08-15-topcoat-api-notes.md`

**要点**：钉死 topcoat 版本（Cargo.toml `=x.y.z`）；落档其 Request/Response/中间件模型、与 tower 的关系、view/responder 扩展点；若其模型与 tower 高度同构，评估四件套直接复用 thymeleaf-tower 的实现（减少一份语义苦工）。

### Task 3.2：host_web_request.rs

依 3.1 勘察结论实施（若复用 tower 路线，则本 Task 改为"从 thymeleaf-tower 提取/引用 + topcoat 特化"）。

### Task 3.3：host_web_exchange/session/application.rs

同构。

### Task 3.4：契约测试对齐 hyper 标杆

同 Task 1.3 口径。

### Task 3.5：vernal-webmvc 视图解析对接验证（设计验证，非实施）

**Files:**
- Create: `docs/superpowers/specs/2026-08-15-webmvc-view-integration-notes.md`

**要点**：验证 `ThymeleafView` 能否直接作为 vernal-webmvc 的 View 后端；列出对接缺口清单（如需要 vernal-web trait 桥），作为下一个计划的输入。**不在本计划内实现对接**。

---

## Stage 4 — 降级 gotham/tide/warp

### Task 4.1：实验性标记

**Files:**
- Modify: `thymeleaf-support/thymeleaf-gotham/Cargo.toml` + `README` 头注释
- Modify: `thymeleaf-support/thymeleaf-tide/Cargo.toml` + `README` 头注释
- Modify: `thymeleaf-support/thymeleaf-warp/Cargo.toml` + `README` 头注释

**要点**：description 加 `"(experimental)"`；README 头部注明上游维护状态（gotham 2022 停滞 / tide 0.17-beta / warp 被 axum 取代）+ "尽力维护，不推荐新项目"。

### Task 4.2：CI 必测矩阵收窄

**Files:**
- Modify: `thymeleaf-support/thymeleaf-gotham/Cargo.toml`、`thymeleaf-tide/Cargo.toml`、`thymeleaf-warp/Cargo.toml`（测试移到 feature 后面，如 `[features] full = []` + `[[test]] required-features = ["full"]`）
- Modify: `.github/workflows/ci.yml`（如需：默认 workspace 测试不含三件的 full feature；可选加一个 allow-failure 的实验性 job）

**验收**：`cargo test --workspace` 默认绿且不跑三件实验性测试；三件现有代码保持编译（`cargo check` 仍覆盖）。

---

## Stage 5 — 收尾

### Task 5.1：CHANGELOG + README 矩阵

**Files:**
- Modify: `CHANGELOG.md`（[Unreleased] 新条目：适配器分级 + render_async + 降级决策）
- Modify: `thymeleaf/README.md` + `README.zh-CN.md`（适配器支持矩阵表：P0 三件"生产级"、七件"稳定薄层"、三件"实验性"）

### Task 5.2：全量验证

- 本地：`cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace --all-features`
- 提交推送 dev；CI 全绿（ubuntu + macos）后 fast-forward 合并 main
- 回填 Task 0.1 的约定文档状态：`[待验证]` → `[已验证]`

---

## 完成定义（DoD）

- [ ] axum/actix/topcoat 四件套齐备，契约测试与 hyper 逐断言对齐
- [ ] 三件 `render_async` 可用（spawn_blocking）
- [ ] gotham/tide/warp 标实验性且移出必测矩阵
- [ ] Vernal 约定文档视图层立项 + 状态回填
- [ ] CI 双平台绿，main 合并完成
