# DTD 验证集成设计（oxixml-dtd）

- **日期**：2026-09-03
- **状态**：待实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2`）
- **外部依赖**：oxixml-dtd 0.1.2（crates.io，`https://github.com/cool-japan/oxixml`）
- **关联**：`processor/standard_translation_doc_type_processor.rs`（DTD 翻译）、`markup/abstract_markup_template_parser.rs`（parse_xml/parse_html）、`doc_type_translation_golden_diff.rs`（现有 golden 测试）

## 1. 目标与范围

在 thymeleaf-rust 的 **XML 模式**下引入 DTD 验证能力，使模板解析器在解析元素/属性事件的同时检查文档是否符合其声明的 DTD。这是对 Java 上游（attoparser `validateProlog=false`、`NOT_VALIDATED`）的**超越性增强**——Java 侧不做任何 DTD 验证，Rust 侧将提供可选的严格验证。

**核心判断**：

| 维度 | 现状（thymeleaf-rust） | 目标 |
|------|----------------------|------|
| HTML 模式 DTD | `StandardTranslationDocTypeProcessor` 翻译 Thymeleaf 专有 → W3C 标准 | 不变——HTML 模式不做 DTD 验证（WHATWG 容错语义） |
| XML 模式 DTD | quick_xml 流式解析，不验证 DTD | 新增：oxixml-dtd push 验证器在 parse_xml 中同步驱动 |
| DTD 来源 | 无——DOCTYPE 仅作事件传递 | 内嵌 W3C XHTML DTD 文件（MemoryResolver），零网络访问 |
| 实体展开 | quick_xml 原生处理 | oxixml-dtd `expand_general_entity` 带 expansion bomb 防护 |
| 上游对齐 | 1:1 镜像（不做验证） | **超越上游**——可选严格验证，不破坏现有行为 |

**非目标**：

- 不在 HTML 模式做 DTD 验证（WHATWG 规范要求容错，HTML5 已废弃 DTD 概念）
- 不做网络 DTD 加载（MemoryResolver 白名单 + DenyExternalEntities 默认拒绝）
- 不修改 `StandardTranslationDocTypeProcessor` 的翻译逻辑（翻译与验证是两个独立关注点）
- 不在首次交付中支持用户自定义 DTD（仅内嵌 W3C XHTML 系列）

## 2. 现有架构分析

### 2.1 DTD 翻译路径（HTML 模式，PRECEDENCE 1000）

```
模板输入 → parse_html (html5gum) → Token::Doctype → emit_doctype
  → TemplateHandlerAdapterMarkupHandler::doc_type
    → ProcessorTemplateHandler::handle_doc_type
      → StandardTranslationDocTypeProcessor::process
        → translate_system_id: 16 个 Thymeleaf 专有 SystemID → 4 组 W3C DOCTYPE
        → structure_handler.set_doc_type (替换 public_id + system_id)
```

关键代码：`standard_translation_doc_type_processor.rs` L108-158

- 仅触发条件：`type == "SYSTEM"` 且 `system_id` 以 `http://www.thymeleaf.org/dtd/` 开头
- 翻译映射：`xhtml1-{strict|transitional|frameset}-thymeleaf-{1..4}.dtd` → 对应 W3C DTD
- `xhtml11-thymeleaf-{1..4}.dtd` → `xhtml11.dtd`
- 仅注册于 HTML 模式（`TemplateMode::HTML`），XML 模式原样传递

### 2.2 XML 解析路径（当前无验证）

```
模板输入 → parse_xml (quick_xml::Reader) → Event::DocType → emit_doctype
  → TemplateHandlerAdapterMarkupHandler::doc_type
    → ProcessorTemplateHandler::handle_doc_type
      → [现有 DocType Processor 链]
```

关键代码：`abstract_markup_template_parser.rs` L605-779

- `quick_xml::Reader` 流式解析，`check_end_names=true`、`allow_unmatched_ends=false`
- DocType 事件被识别（L757-765）并通过 adapter 传递给 handler 链
- **不验证** DTD 内容模型、属性默认值、ID/IDREF 唯一性

### 2.3 HTML 解析路径（不做验证的理由）

```
模板输入 → parse_html (html5gum::Tokenizer) → Token::Doctype → emit_doctype
  → [同上 handler 链]
```

关键代码：`abstract_markup_template_parser.rs` L311-569

- html5gum 是 WHATWG HTML5 tokenizer——容错设计，不依赖 DTD
- HTML5 规范已废弃 DTD 概念（`<!DOCTYPE html>` 是模式切换信号，非验证声明）
- Thymeleaf HTML 模板的 DTD 仅用于翻译（StandardTranslationDocTypeProcessor），不用于验证

## 3. oxixml-dtd 集成架构

### 3.1 组件关系

```
thymeleaf crate
├── dtd/                          # 新模块
│   ├── mod.rs                    # 公开入口
│   ├── embedded_dtd.rs           # 内嵌 DTD 文件清单 + MemoryResolver 构建
│   ├── validator.rs              # DTD 验证器封装（oxixml-dtd Validator 的薄包装）
│   └── entity_budget.rs          # 实体展开预算管理
├── markup/
│   └── abstract_markup_template_parser.rs  # parse_xml 中集成验证器
└── Cargo.toml                    # 新增 oxixml-dtd 依赖
```

### 3.2 内嵌 DTD 文件清单

从 W3C 标准获取的完整 XHTML DTD 文件集（需内嵌二进制）：

**XHTML 1.0 Strict**（4 个主文件 + 模块文件）：
- `xhtml1-strict.dtd` — 主 DTD
- `xhtml1-strict-model-1.mod` — 内容模型模块
- `xhtml1-strict-legacy.mod` — 遗留实体模块（若存在）
- `xhtml1-framework-1.mod` — 框架模块
- `xhtml1-lat1.ent` / `xhtml1-special.ent` / `xhtml1-symbol.ent` — 字符实体文件

**XHTML 1.0 Transitional**（同结构）：
- `xhtml1-transitional.dtd` + 对应 .mod / .ent 文件

**XHTML 1.0 Frameset**（同结构）：
- `xhtml1-frameset.dtd` + 对应 .mod / .ent 文件

**XHTML 1.1**（同结构）：
- `xhtml11.dtd` + 对应 .mod / .ent 文件

**内嵌方式**：使用 `include_bytes!` 或 `include_str!` 在编译时将 DTD 文本嵌入二进制，通过 `MemoryResolver::insert(system_id, content)` 注册。

### 3.3 MemoryResolver 组织

```rust
// dtd/embedded_dtd.rs
use oxixml_dtd::resolver::MemoryResolver;

pub fn build_xhtml_resolver() -> MemoryResolver {
    let mut resolver = MemoryResolver::new();
    // XHTML 1.0 Strict
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd",
        include_str!("../../dtd-files/xhtml1/xhtml1-strict.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict-model-1.mod",
        include_str!("../../dtd-files/xhtml1/xhtml1-strict-model-1.mod"),
    );
    // ... 其余 .mod / .ent 文件
    // XHTML 1.0 Transitional / Frameset / XHTML 1.1 同理
    resolver
}
```

**安全属性**：
- `MemoryResolver` 仅返回编译时嵌入的白名单内容——零文件系统访问、零网络访问
- `DenyExternalEntities` 是 oxixml-dtd 默认 resolver——任何未在 MemoryResolver 注册的实体引用均被拒绝
- 两者组合形成**白名单安全模型**：已知 DTD 内嵌 + 未知 DTD 拒绝

### 3.4 验证触发时机（XML 模式）

集成点在 `parse_xml` 函数（`abstract_markup_template_parser.rs` L605-779）：

```
parse_xml 开始
  ↓
[新增] 从 DocType 事件提取 system_id
  ↓
[新增] 匹配 system_id → 构建 Dtd（MemoryResolver 解析 .mod/.ent 引用）
  ↓
[新增] 创建 Validator::new(&dtd, ValidationOptions::default())
  ↓
quick_xml 事件循环（现有）
  ├── Event::Start(tag) | Event::Empty(tag)
  │     → [新增] validator.start_element(name, attrs) — 检查内容模型 + 属性默认值
  │     → [现有] adapter.element_start_with_injected(...)
  ├── Event::Text(_)
  │     → [新增] validator.characters(text) — 检查 #PCDATA 位置
  │     → [现有] adapter.text(...)
  ├── Event::End(_)
  │     → [新增] validator.end_element() — 检查闭合合法性
  │     → [现有] adapter.element_end(...)
  └── Event::DocType(_)
        → [现有] emit_doctype (传递给 handler 链)
        → [新增] 从 system_id 构建 Dtd + Validator
  ↓
parse_xml 结束
  ↓
[新增] validator.finish() → 收集 ValidityError 列表
  ↓
[决策] 验证失败处理策略（见 3.5）
```

**关键设计决策**：

1. **DocType 事件必须先于元素事件**（XML 规范要求）——在 DocType 事件时构建 Dtd，在后续元素事件时驱动 Validator
2. **无 DocType 时不做验证**——`validator` 为 `Option<Validator>`，`None` 时跳过所有验证调用
3. **验证与解析同步**——不预先扫描 DocType（避免两遍解析），在事件流中实时构建

### 3.5 验证失败处理策略

```
ValidationPolicy enum {
    Strict,    // 验证失败 → 返回 TemplateParserError（中断解析）
    Warn,      // 验证失败 → tracing::warn! 日志，继续解析
    Disabled,  // 不做验证（默认，保持上游行为兼容）
}
```

**默认行为**：`Disabled`——不破坏现有用户。用户通过 `TemplateEngine` 配置显式启用。

**Strict 模式下的错误格式**：
```
DTD validation error at line {line}, col {col}: {error_kind}
  in template: {template_name}
  element: {element_name}
  constraint: {constraint_description}
```

### 3.6 实体展开与安全防护

```rust
// dtd/entity_budget.rs
use oxixml_dtd::limits::{Budget, ExpansionLimits};

/// 默认实体展开预算（防止 expansion bomb）。
pub fn default_expansion_limits() -> ExpansionLimits {
    ExpansionLimits {
        max_entity_depth: 10,
        max_entity_expansions: 1000,
        max_expanded_bytes: 1024 * 1024, // 1MB
    }
}
```

- `Dtd::expand_general_entity(name, &mut budget)` 在展开时消耗预算
- 预算耗尽 → 返回 `ExpansionLimit` 错误（不 panic、不无限展开）
- 与 `DenyExternalEntities` 组合：外部实体被拒绝 + 内部实体有预算上限

## 4. HTML/XML 模式差异处理

| 维度 | HTML 模式 | XML 模式 |
|------|-----------|----------|
| DTD 验证 | **不做**——WHATWG 容错语义 | **做**——oxixml-dtd push 验证 |
| DTD 翻译 | `StandardTranslationDocTypeProcessor`（已有） | 不翻译——XML 模式原样传递 DOCTYPE |
| 实体展开 | html5gum 原生处理 | oxixml-dtd `expand_general_entity`（带预算） |
| 配置控制 | 无——HTML 模式始终不验证 | `ValidationPolicy` 三态（Strict/Warn/Disabled） |
| 默认行为 | 不变 | `Disabled`（保持上游兼容） |

**设计理由**：

1. HTML5 规范已废弃 DTD 概念——`<!DOCTYPE html>` 仅是模式切换信号，不引用可验证的 DTD
2. html5gum tokenizer 不产生 DTD 验证所需的结构化事件（无 content-model 检查）
3. Thymeleaf 专有 DTD（`http://www.thymeleaf.org/dtd/...`）是翻译目标，不是验证目标——翻译后变为 W3C 标准 DTD，但 HTML 模式下仍不验证

## 5. 与现有 StandardTranslationDocTypeProcessor 的关系

```
HTML 模式:
  模板 → parse_html → DocType 事件
    → StandardTranslationDocTypeProcessor (翻译 Thymeleaf → W3C)
    → [不做 DTD 验证]

XML 模式:
  模板 → parse_xml → DocType 事件
    → [不做翻译——XML 模式无 StandardTranslationDocTypeProcessor]
    → [新增] DTD 验证器（从 system_id 构建 Dtd，验证后续元素）
```

**两个关注点完全分离**：
- `StandardTranslationDocTypeProcessor`：负责 **翻译**（Thymeleaf 专有 SystemID → W3C 标准 DOCTYPE）
- DTD 验证器：负责 **验证**（检查文档是否符合 DTD 声明的内容模型）

**不修改** `StandardTranslationDocTypeProcessor`——它的 PRECEDENCE 1000、HTML-only 限制、翻译逻辑均保持不变。

## 6. 安全模型

```
┌─────────────────────────────────────────────────┐
│                  安全边界                         │
│                                                  │
│  ┌──────────────┐    ┌──────────────────────┐   │
│  │ MemoryResolver│    │ DenyExternalEntities │   │
│  │ (白名单)      │    │ (默认拒绝)            │   │
│  │              │    │                      │   │
│  │ xhtml1-*.dtd │    │ 任何未注册的实体      │   │
│  │ xhtml11.dtd  │    │ → ResolverError      │   │
│  └──────────────┘    └──────────────────────┘   │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ ExpansionLimits / Budget                  │   │
│  │ - max_entity_depth: 10                    │   │
│  │ - max_entity_expansions: 1000             │   │
│  │ - max_expanded_bytes: 1MB                 │   │
│  └──────────────────────────────────────────┘   │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ ValidationPolicy                          │   │
│  │ - Disabled (默认): 不验证，零开销          │   │
│  │ - Warn: 验证但不中断                       │   │
│  │ - Strict: 验证失败中断解析                  │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

**三层防护**：
1. **白名单**：MemoryResolver 只返回编译时嵌入的 DTD——无法注入恶意 DTD
2. **默认拒绝**：DenyExternalEntities 阻止任何外部实体解析——零网络/文件访问
3. **预算限制**：ExpansionLimits 防止 entity expansion bomb——即使内部实体也有上限

## 7. 性能影响评估

| 场景 | 影响 |
|------|------|
| HTML 模式 | **零影响**——不引入任何新代码路径 |
| XML 模式 + Disabled（默认） | **近零影响**——仅多一次 `Option::is_none()` 检查 |
| XML 模式 + Warn/Strict | **可控开销**——Dtd 解析一次（DocType 事件时），Validator push 调用与元素事件 1:1 对应 |
| 内嵌 DTD 二进制大小 | 约 200-400KB（XHTML 1.0 全系列 DTD + 模块 + 实体文件），可通过 feature gate 控制 |

## 8. 风险

| 风险 | 缓解 |
|------|------|
| oxixml-dtd 0.1.x API 演进 | 钉死版本 `=0.1.2`；API 变更时在独立 PR 中升级 |
| 内嵌 DTD 文件与 W3C 标准不一致 | 内嵌前逐文件校验 SHA-256；测试中比对已知 DTD 内容 |
| XML 模式下 quick_xml 与 oxixml-dtd 事件模型不匹配 | quick_xml `Event::Start` 的属性格式需转换为 `&[(&str, &str)]`；封装层处理 |
| DTD 验证失败的用户体验 | 默认 Disabled；Strict 模式错误消息包含行/列/约束描述 |
| 二进制体积增长 | feature gate（`dtd-validation`）允许不编译内嵌 DTD |

## 9. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-09-03 | 初版——oxixml-dtd 集成架构、DTD 打包方案、HTML/XML 模式差异、安全模型 |
