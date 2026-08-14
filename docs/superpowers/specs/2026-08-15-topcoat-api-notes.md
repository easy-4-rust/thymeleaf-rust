# topcoat 0.5.0 API 勘察笔记（Task 3.1 前置落档）

- **日期**：2026-08-15
- **状态**：已确认（依据本地 cargo 缓存 topcoat-0.5.0 / topcoat-core-0.5.0 / topcoat-router-0.5.0 源码）
- **结论**：topcoat 是 **Leptos 风格响应式全栈框架**（`Cx` 上下文 + view 宏 + signal），非传统 handler(Request)->Response MVC；但其 router 层的请求/响应类型是 **http 1.x 类型族的直接 re-export**，Thymeleaf 适配可行且与 axum 路线同构。

## 1. 版本与类型钉死

| 项 | 值 | 来源 |
|---|---|---|
| topcoat 版本 | `=0.5.0`（Cargo.toml 已钉） | thymeleaf-topcoat/Cargo.toml |
| `topcoat::router::Request<T>` | `pub type Request<T = Body> = http::Request<T>`（**http 1.x**） | topcoat-router/src/request.rs:11 |
| `topcoat::router::Response<T>` | `pub type Response<T = Body> = http::Response<T>` | topcoat-router/src/response.rs:15 |
| `HeaderMap` | http 1.x（与 hyper/axum 同族） | topcoat-router re-export |

## 2. 响应侧（已有，无需重做）

`IntoResponse for ThymeleafView`（现有 `thymeleaf_view.rs` 44 行）：trait 方法签名为 `fn into_response(self, cx: &Cx) -> Result<Response>`——**带 Cx 参数**是 topcoat 特色；`RenderedTemplate::into_parts()` 组装保留状态/Header/流式 Body。

## 3. 请求侧（本计划补齐）

- `HostWebRequest::from_request(&http::Request<B>, application_path)` 可**逐行复刻 axum 版**（同为 http 1.x 类型族：`uri().authority()`/`headers()`/`form_urlencoded` query 解析/Cookie 头直解）。
- `IntoResponse` 依赖的 `Cx` 不参与请求快照构造（快照在 handler 入口从 `Request` 提取，与 Cx 无耦合）。

## 4. 与 vernal-webmvc 对接的定位（Task 3.5 输入）

topcoat 的响应式 view（`topcoat_view` 宏）与 Thymeleaf 服务端模板是**两种视图模型**：
- 响应式组件（view 宏）= 客户端岛屿/islands 场景
- Thymeleaf SSR = 服务端整页渲染场景（对应 vernal-webmvc 视图解析空缺）

二者在 topcoat 生态**互补而非竞争**：route handler 返回 `ThymeleafView`（`IntoResponse`）即整页 SSR；返回响应式 view 即 islands。对接缺口清单见 `2026-08-15-webmvc-view-integration-notes.md`。

## 5. render_async 路线

topcoat 是 Tokio-first（serve/tower 生态），用 `tokio::task::spawn_blocking` 包同步渲染（同 axum 路线）。
