# DTD 验证集成实施计划（oxixml-dtd）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 thymeleaf-rust 的 XML 模式下引入可选 DTD 验证能力（oxixml-dtd 0.1.2 push 接口），内嵌 W3C XHTML DTD 文件集，通过 MemoryResolver 实现零网络 DTD 解析，默认 Disabled 保持上游行为兼容。

**Architecture:** 新增 `thymeleaf/src/dtd/` 模块（embedded_dtd.rs + validator.rs + entity_budget.rs），在 `parse_xml` 中以 `Option<Validator>` 驱动 push 验证；HTML 模式不引入任何新代码路径；feature gate `dtd-validation` 控制二进制体积。

**Tech Stack:**
- 已有：quick_xml 0.41（XML 流式解析）、StandardTranslationDocTypeProcessor（DTD 翻译）、doc_type_translation_golden_diff（golden 测试）
- 新增依赖：oxixml-dtd = "=0.1.2"（钉死版本）
- 内嵌资产：W3C XHTML 1.0 Strict/Transitional/Frameset + XHTML 1.1 完整 DTD 文件集

**Related Design Doc:** `docs/superpowers/specs/2026-09-03-dtd-validation-design.md`

---

## 全局约定

- **feature gate**：所有 DTD 验证代码在 `#[cfg(feature = "dtd-validation")]` 下；默认 features 不含此 gate——零体积/编译时间影响
- **DTD 文件管理**：`dtd-files/xhtml1/` 和 `dtd-files/xhtml11/` 目录存放原始 DTD 文本；`build.rs` 或 `include_str!` 编译时嵌入
- **版本钉死**：`oxixml-dtd = "=0.1.2"`（避免 patch 版本 API 漂移）
- **提交约定**：conventional commits；每 Task 一 commit；每 Stage 结束跑 `cargo check -p thymeleaf --all-targets` + `cargo clippy -p thymeleaf --all-targets -- -D warnings` + `cargo test -p thymeleaf`
- **并行性**：Stage 1/2 相互独立可并行；Stage 3 依赖 1/2 完成；Stage 4 依赖全部

---

## 实施阶段总览

| Stage | 目标 | 预期 Task 数 |
|-------|------|-------------|
| 0 | 依赖引入 + DTD 文件获取与校验 | 3 |
| 1 | dtd 模块实现（embedded_dtd + validator + entity_budget） | 3 |
| 2 | parse_xml 集成 + 配置接口 | 3 |
| 3 | 测试（单元 + 集成 + golden） | 3 |
| 4 | 收尾（feature gate + CHANGELOG + 文档） | 2 |

---

## Stage 0 — 依赖引入 + DTD 文件获取与校验

### Task 0.1：Cargo.toml 添加 oxixml-dtd 依赖

**Files:**
- Modify: `thymeleaf/Cargo.toml`（`[dependencies]` 新增 oxixml-dtd；`[features]` 新增 `dtd-validation`）

**要点**：
- `oxixml-dtd = { version = "=0.1.2", optional = true }`
- `[features] dtd-validation = ["oxixml-dtd"]`
- 运行 `cargo check -p thymeleaf` 确认无冲突

**验收**：`cargo check -p thymeleaf` 绿；`cargo check -p thymeleaf --features dtd-validation` 绿。

### Task 0.2：获取 W3C XHTML DTD 文件集

**Files:**
- Create: `dtd-files/xhtml1/xhtml1-strict.dtd`
- Create: `dtd-files/xhtml1/xhtml1-strict-model-1.mod`
- Create: `dtd-files/xhtml1/xhtml1-framework-1.mod`
- Create: `dtd-files/xhtml1/xhtml1-lat1.ent`
- Create: `dtd-files/xhtml1/xhtml1-special.ent`
- Create: `dtd-files/xhtml1/xhtml1-symbol.ent`
- Create: `dtd-files/xhtml1/xhtml1-transitional.dtd`（+ 对应 .mod/.ent）
- Create: `dtd-files/xhtml1/xhtml1-frameset.dtd`（+ 对应 .mod/.ent）
- Create: `dtd-files/xhtml11/xhtml11.dtd`（+ 对应 .mod/.ent）
- Create: `dtd-files/README.md`（来源、版本、SHA-256 校验清单）

**要点**：
- 从 W3C 官方 `http://www.w3.org/TR/xhtml1/DTD/` 和 `http://www.w3.org/TR/xhtml11/DTD/` 获取
- 记录每个文件的 SHA-256 哈希（用于后续完整性校验）
- 每个 DTD 的 `SYSTEM` 引用路径必须与 MemoryResolver 注册的 key 完全一致

**验收**：所有 DTD 文件存在；`dtd-files/README.md` 包含来源 URL + SHA-256 清单。

### Task 0.3：DTD 文件完整性校验测试

**Files:**
- Create: `thymeleaf/tests/dtd_file_integrity.rs`（`#[cfg(feature = "dtd-validation")]`）

**要点**：
- 对每个内嵌 DTD 文件计算 SHA-256 并与 `dtd-files/README.md` 中的预期值比对
- 确保 `MemoryResolver` 能成功解析主 DTD 对 .mod/.ent 的 `SYSTEM` 引用
- 测试 `DtdParser::new().with_resolver(Box::new(resolver)).parse_internal_subset(main_dtd)` 不报错

**验收**：`cargo test -p thymeleaf --features dtd-validation --test dtd_file_integrity` 绿。

---

## Stage 1 — dtd 模块实现

### Task 1.1：embedded_dtd.rs（MemoryResolver 构建）

**Files:**
- Create: `thymeleaf/src/dtd/mod.rs`
- Create: `thymeleaf/src/dtd/embedded_dtd.rs`

**要点**：
- `pub fn build_xhtml_resolver() -> MemoryResolver` — 构建并返回包含所有 XHTML DTD 的 MemoryResolver
- 每个 `resolver.insert(system_id, include_str!(...))` 调用的 system_id 必须与 W3C DTD 中的 `SYSTEM` 引用完全匹配
- 注册顺序：先 .ent（实体文件），再 .mod（模块文件），最后 .dtd（主文件）
- `#[cfg(feature = "dtd-validation")]` gate 包裹整个模块

**验收**：`cargo check -p thymeleaf --features dtd-validation` 绿；函数签名稳定。

### Task 1.2：entity_budget.rs（展开预算管理）

**Files:**
- Create: `thymeleaf/src/dtd/entity_budget.rs`

**要点**：
- `pub fn default_expansion_limits() -> ExpansionLimits` — 返回保守的默认预算
- `pub fn default_budget() -> Budget` — 从默认 limits 创建运行时 budget
- 参数选择（基于设计 spec 6.2 节）：`max_entity_depth=10`、`max_entity_expansions=1000`、`max_expanded_bytes=1MB`
- 公开允许用户自定义 limits 的接口（预留，首次不实现）

**验收**：`cargo check -p thymeleaf --features dtd-validation` 绿。

### Task 1.3：validator.rs（DTD 验证器封装）

**Files:**
- Create: `thymeleaf/src/dtd/validator.rs`

**要点**：
- `pub struct DtdValidator` — 封装 `oxixml_dtd::Validator` + `Budget`
- `pub fn new(system_id: &str) -> Option<Self>` — 匹配已知 system_id，构建 Dtd + Validator；未知 system_id 返回 None
- `pub fn start_element(&mut self, name: &str, attrs: &[(&str, &str)])` — 委托 `Validator::start_element`
- `pub fn characters(&mut self, text: &str)` — 委托 `Validator::characters`
- `pub fn end_element(&mut self, name: &str)` — 委托 `Validator::end_element`
- `pub fn finish(self) -> Vec<ValidityError>` — 委托 `Validator::finish`
- `system_id` 匹配表：`http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd` 等 4 组
- quick_xml 属性格式转换：`Vec<Attribute>` → `Vec<(&str, &str)>`（name, value 对）

**验收**：`cargo check -p thymeleaf --features dtd-validation` 绿；`DtdValidator::new` 对已知 system_id 返回 `Some`，对未知返回 `None`。

---

## Stage 2 — parse_xml 集成 + 配置接口

### Task 2.1：ValidationPolicy 配置枚举

**Files:**
- Modify: `thymeleaf/src/engine_configuration.rs`（新增字段 + getter）
- Modify: `thymeleaf/src/i_engine_configuration.rs`（新增 trait 方法）

**要点**：
- `pub enum ValidationPolicy { Disabled, Warn, Strict }`（在 `thymeleaf/src/dtd/validator.rs` 中定义，re-export）
- `IEngineConfiguration` 新增 `fn get_dtd_validation_policy(&self) -> ValidationPolicy`（默认返回 `Disabled`）
- `EngineConfiguration` 新增 `dtd_validation_policy` 字段 + builder 方法
- 仅在 `#[cfg(feature = "dtd-validation")]` 下编译——无 feature 时 trait 方法提供默认实现返回 `Disabled`

**验收**：`cargo check -p thymeleaf` 绿（无 feature）；`cargo check -p thymeleaf --features dtd-validation` 绿。

### Task 2.2：parse_xml 集成 DTD 验证器

**Files:**
- Modify: `thymeleaf/src/markup/abstract_markup_template_parser.rs`（`parse_xml` 函数）

**要点**：
- 在 `parse_xml` 函数签名中新增 `validation_policy: ValidationPolicy` 参数（或从 configuration 获取）
- 在 DocType 事件处理分支中：提取 system_id，尝试构建 `DtdValidator`
- 在 Start/Empty 事件分支中：调用 `validator.start_element(name, attrs)`
- 在 Text 事件分支中：调用 `validator.characters(text)`
- 在 End 事件分支中：调用 `validator.end_element(name)`
- 在 EOF 之前：调用 `validator.finish()`，根据 policy 处理错误
- `Disabled` 策略：validator 始终为 None，所有验证调用被跳过（零开销）
- `Warn` 策略：验证失败 → `tracing::warn!` 日志，继续解析
- `Strict` 策略：验证失败 → 返回 `TemplateParserError`
- 属性格式转换：quick_xml 的 `attributes()` → `Vec<(&str, &str)>`（处理 QName 前缀、值解码）

**验收**：`cargo check -p thymeleaf --features dtd-validation` 绿；现有 XML 模式测试不回归。

### Task 2.3：parse_internal 传递 validation_policy

**Files:**
- Modify: `thymeleaf/src/markup/abstract_markup_template_parser.rs`（`parse_internal` 方法）

**要点**：
- 从 `configuration.get_dtd_validation_policy()` 获取策略
- 传递给 `parse_xml` 函数
- HTML 模式路径（`parse_html`）不受影响——不传递、不使用验证器

**验收**：`cargo check -p thymeleaf --features dtd-validation` 绿。

---

## Stage 3 — 测试

### Task 3.1：DTD 验证器单元测试

**Files:**
- Create: `thymeleaf/src/dtd/validator.rs`（`#[cfg(test)] mod tests`）

**要点**：
- 测试 `DtdValidator::new` 对 4 组已知 system_id 返回 `Some`
- 测试 `DtdValidator::new` 对未知 system_id 返回 `None`
- 测试简单有效文档（`<html><body><p>text</p></body></html>`）验证通过（`finish()` 返回空）
- 测试无效文档（违反内容模型）验证失败
- 测试属性默认值填充（DTD 定义的默认属性值在 `start_element` 返回中可见）

**验收**：`cargo test -p thymeleaf --features dtd-validation -- dtd::validator` 绿。

### Task 3.2：XML 模式集成测试

**Files:**
- Create: `thymeleaf-test/tests/dtd_validation_integration.rs`（`#[cfg(feature = "dtd-validation")]`）

**要点**：
- 测试 XHTML 1.0 Strict 模板（含正确 DOCTYPE）在 Strict 策略下验证通过
- 测试违反内容模型的模板（如 `<html><p></html>` 缺少 `<body>`）在 Strict 策略下验证失败
- 测试 Warn 策略下验证失败不中断解析
- 测试 Disabled 策略下不做验证（零开销确认）
- 测试无 DocType 的 XML 模板不做验证
- 测试 Thymeleaf 专有 DTD（XML 模式下不翻译，原样传递）不触发验证（system_id 不匹配 W3C）

**验收**：`cargo test -p thymeleaf-test --features dtd-validation --test dtd_validation_integration` 绿。

### Task 3.3：现有测试不回归

**Files:**
- 无新文件——运行现有测试套件

**要点**：
- `cargo test -p thymeleaf --all-features` 全绿（含 doc_type_translation_golden_diff）
- `cargo test -p thymeleaf-test --all-features` 全绿
- `cargo clippy -p thymeleaf --all-targets --all-features -- -D warnings` 无新警告

**验收**：全量测试通过；clippy 零警告。

---

## Stage 4 — 收尾

### Task 4.1：feature gate 文档 + Cargo.toml 清理

**Files:**
- Modify: `thymeleaf/Cargo.toml`（`[package.metadata.docs.rs]` 确认 `all-features = true`）
- Modify: `thymeleaf/README.md`（新增 "DTD Validation" 小节说明 feature gate）

**要点**：
- README 说明：`cargo add thymeleaf --features dtd-validation` 启用；默认不启用
- 说明安全模型：MemoryResolver 白名单 + DenyExternalEntities + ExpansionLimits
- 说明与上游差异：Java 不做 DTD 验证，Rust 可选启用

**验收**：文档 diff 只增不删。

### Task 4.2：CHANGELOG + 全量验证

**Files:**
- Modify: `CHANGELOG.md`（`[Unreleased]` 新条目）

**要点**：
- CHANGELOG 条目：`feat: optional DTD validation in XML mode via oxixml-dtd (feature gate: dtd-validation)`
- 本地全量验证：`cargo fmt --all --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo test --workspace --all-features`
- CI 全绿后合并

**验收**：CHANGELOG 更新；全量验证通过。

---

## 完成定义（DoD）

- [ ] oxixml-dtd = "=0.1.2" 引入，feature gate `dtd-validation` 可用
- [ ] W3C XHTML DTD 文件集内嵌（4 组主 DTD + .mod/.ent），SHA-256 校验
- [ ] MemoryResolver 白名单 + DenyExternalEntities 默认拒绝 + ExpansionLimits 三重安全
- [ ] parse_xml 中 DTD 验证器 push 接口集成（start_element/characters/end_element/finish）
- [ ] ValidationPolicy 三态（Disabled/Warn/Strict）可通过 EngineConfiguration 配置
- [ ] HTML 模式零影响——不引入任何新代码路径
- [ ] 默认 Disabled——不破坏现有用户行为
- [ ] 单元测试 + 集成测试 + 现有测试不回归
- [ ] CI 双平台绿，main 合并完成
