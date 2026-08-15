# vernal-webmvc ViewResolver 集成实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 补齐 `2026-08-15-webmvc-view-integration-notes.md` 列出的 5 项桥接缺口（ViewResolver trait、Model 桥、Locale 协商、缓存配置、前缀后缀解析），打通 thymeleaf-rust → vernal Web 栈的 Spring MVC 视图解析路径。

**Architecture:** View/ViewResolver/Model 契约定义在 **vernal-web**（已存在、thymeleaf-vernal 已 git 依赖——不新建 vernal-webmvc crate，它是 Topcoat 轨道的未来物）；thymeleaf-vernal 实现 `ThymeleafViewResolver`（Spring `org.thymeleaf.spring6.view.ThymeleafViewResolver` 语义子集：视图名 + prefix/suffix → 模板、Model → WebContext 变量、Locale 注入、cacheable 开关）。

**Tech Stack:** vernal-web（trait 新增）/ vernal-http（HttpResponse 已通）/ thymeleaf（prefix/suffix API 已有：`set_prefix`/`set_suffix`；cache：`set_cache_manager` + `clear_template_cache`；Locale 已有）

**Related Design Doc:** `2026-08-15-webmvc-view-integration-notes.md`（缺口来源）、`2026-08-15-web-adapter-p0-design.md`（前置 P0）

---

## 全局约定

- **跨仓工作流**：先 vernal-framework（trait + push 拿新 rev）→ 再 thymeleaf-rust（更新 rev + 实现）。DNS workaround：`git -c http.curloptResolve="github.com:443:140.82.112.3" push`。
- **Spring 语义锚点**：`ViewResolver#resolveViewName(name, locale)`、`View#render(model, request, response)`、`org.springframework.ui.Model`（attribute map）。
- **thymeleaf-vernal 现有锚点**：`VernalWebExchange::new(request_context, snapshot)`、`ThymeleafView::into_http_response()`。
- 每 Task 一 commit；vernal 仓库 commit 前跑其 workspace check。

---

## Stage 0 — 计划落档

- [x] spec + plan 写入 `docs/superpowers/`

## Stage 1 — vernal-web 视图契约（vernal-framework 仓库）

### Task 1.1：view.rs（View / ViewResolver trait + Model）

**Files:**
- Create: `crates/vernal-web/src/view.rs`（`View` trait：`render(&self, model: &Model, locale, exchange) -> Result<HttpResponse, ...>`；`ViewResolver` trait：`resolve_view_name(&self, name, locale) -> Option<View>`；`Model`：`add_attribute/get_attribute/contains_attribute/as_map`，内部 `IndexMap<String, TemplateValue?>`——不，vernal-web 不依赖 thymeleaf，Model 值类型用 `Arc<dyn Any + Send + Sync>` 或 serde Value？**决策**：`Model` 值为 `vernal_core` 通用值（若有）或 `Arc<dyn Any + Send + Sync>`；thymeleaf 侧负责 Any→TemplateValue 转换）
- Modify: `crates/vernal-web/src/lib.rs`（mod + pub use）

### Task 1.2：单测 + commit + push（拿新 rev）

- Model 属性顺序/身份语义测试；trait 编译验证
- commit + push origin main

## Stage 2 — ThymeleafViewResolver（thymeleaf-rust 仓库）

### Task 2.1：更新 vernal rev + view_resolver.rs

**Files:**
- Modify: `thymeleaf-support/thymeleaf-vernal/Cargo.toml`（rev → Stage 1 push 后的 SHA）
- Create: `thymeleaf-support/thymeleaf-vernal/src/view_resolver.rs`

**要点（5 项缺口逐一落）**：
1. `ThymeleafViewResolver`：`new(template_engine, template_resolver)` + `set_prefix("classpath:/templates/")`/`set_suffix(".html")`（Spring 默认值）→ 视图名 `home` 解析为 `templates/home.html`（resource 路径归一化）
2. `resolve_view_name(name, locale)` → `ThymeleafView`（渲染推迟到 render）
3. `render(model, locale, exchange)`：Model 的 Any 值 → `TemplateValue`（string/number/bool 直转，其余 `to_string_lossy`）→ exchange attributes / WebContext 变量
4. Locale 注入 `WebContext`（thymeleaf `Context::set_locale` 若有，否则 exchange attribute `thymeleaf.locale`——以核心 API 为准）
5. `set_cacheable(bool)`：false 时每次 `clear_template_cache_for`（或 NullCacheManager 路线，以核心 API 实况定）

### Task 2.2：集成测试

- 端到端：`ThymeleafViewResolver` + prefix/suffix + Model{name} + locale → 渲染 `<p th:text="${name}">` → HttpResponse 断言 body/status
- cacheable=false 的缓存绕过断言（改模板文件后重渲染取新值——用 StringTemplateResolver 语义等价验证）

### Task 2.3：fmt/clippy/test + commit

## Stage 3 — 收尾

### Task 3.1：文档回填 + push + CI

- vernal 4.7 节：ViewResolver 桥完成记录
- thymeleaf-rust CHANGELOG [Unreleased] + push dev → CI 绿 → main

## 完成定义（DoD）

- [x] vernal-web 暴露 View/ViewResolver/Model 契约（Spring 语义）
- [x] thymeleaf-vernal 提供 ThymeleafViewResolver（5 项缺口全落）
- [x] 端到端集成测试绿（视图名→模板→变量→Locale→响应）
- [x] 双仓 CI 绿（vernal dev push；thymeleaf-rust CI 见下）
