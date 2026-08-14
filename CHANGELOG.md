# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与
[语义化版本](https://semver.org/lang/zh-CN/)（晋级规则见
[docs/superpowers/specs/2026-07-28-versioning-governance.md](docs/superpowers/specs/2026-07-28-versioning-governance.md)）。

## [Unreleased]

### Added

- **Web 适配器 P0 做厚**（superpowers plan `2026-08-15-web-adapter-p0`）：
  - `thymeleaf-axum` / `thymeleaf-actix-web` / `thymeleaf-topcoat` 补齐
    `HostWeb` 四件套（`IWebRequest` ~24 方法 + `IWebExchange`/`IWebSession`/
    `IWebApplication`），契约测试与 `thymeleaf-hyper` 标杆逐断言对齐
    （axum 13 / actix 13 / topcoat 12 测试全绿，含端到端渲染）。
  - 三件新增 `ThymeleafView::render_async`：axum/topcoat 走 tokio
    `spawn_blocking`，actix 走原生 `web::block`；同步入口并存。
  - topcoat API 勘察落档（`2026-08-15-topcoat-api-notes.md`）：topcoat 0.5.0
    router 层为 http 1.x 类型族 re-export，与 axum 路线同构；响应式 view
    宏与 Thymeleaf SSR 互补。
  - vernal-webmvc 对接缺口清单（`2026-08-15-webmvc-view-integration-notes.md`）：
    ViewResolver trait 缺失等 5 项桥接，作为下一计划输入。

### Changed

- `thymeleaf-gotham` / `thymeleaf-tide` / `thymeleaf-warp` 降级**实验性**
  （上游停滞/被取代；测试移入 `full` feature，CI `--all-features` 仍看护）。
- Vernal 约定文档新增 4.7 视图层选型节（thymeleaf-rust 替代
  ThymeleafViewResolver，状态 `[待验证]` → 本轮 P0 完成后 `[已验证]`）。



### Changed

- **命名 Rust 化（语义锁定，行为零变更）**：全量移除 `Java*` 前缀类型与
  `java_*` 边界方法，改用 Rust 领域命名——`JavaString`→`Utf16String`、
  `JavaLocale`→`Locale`、`JavaWriter`→`TemplateWriter`、`JavaTemporal`→
  `TemporalValue`、`JavaNumber`→`NumberValue`、`JavaList`→`ListValue`、
  `JavaDate`→`DateValue`、`JavaEvaluation*`→`Evaluation*` 等 40+ 类型；
  `ClassNotFoundException`→`ClassNotFoundError`、`NoSuchMethodException`→
  `NoSuchMethodError`、`OgnlException`→`OgnlError`；
  `TemplateObject` 边界方法 `java_class_name`→`class_name`、
  `java_invoke_method`→`invoke_method` 等 8 项。
  模板语言可见的 Java 对象契约（UTF-16 语义、toString 格式、corpus 断言、
  `%EXCEPTION` 字符串键）全部保持。另清理 60+ 个内部 `java_*` 辅助函数
  （`java_trim`→`trim`、`java_message`→`message`、`java_hash_code`→`hash_code`、
  `CharSequenceValue` 的 `java_length`/`java_char_at`/`java_to_string`→
  `length`/`char_at`/`to_utf16_string` 等）。API baseline 已用 cargo-public-api 再生成。
- **安全模型修正（预先存在的 CI 失败）**：`ExpressionUtils::is_type_forbidden`
  改为默认拒绝（仅白名单 + `java.time.*`/`org.thymeleaf.*` 受信前缀放行；
  无包名裸类名仍由解析器报 `ClassNotFound`，保持 corpus 契约）；
  `is_member_forbidden(None, ...)` 无目标上下文默认拒绝危险成员；
  GTVG 示例测试固定进程默认 Locale，消除 CI 平台 LANG 差异。

### Added

- **治理门禁**：通用迁移布局审计 vendor 为 `scripts/audit_migration_layout.py`
  （复制自 rust-java-migration 技能），新增批准清单机制
  （`docs/migration/layout_approvals.json`，含 reason/owner/exit_criteria 契约）
  与 no-op 来源证据识别；CI 新增通用审计 job 与 `migration-check` 双轨并存，
  `--require-source-comments --fail-on-warning` 全量启用。
- **来源注释生成器**：`scripts/generate_java_source_comments.py`，从对象级对照表
  与 Java API 清单推导"对应 Java"注释（可验证方法 → `Class#method()`；
  接口/继承方法 → `Java 接口/超类方法`；Rust 侧辅助 → 显式标注）。
- **独立对象拆分**：`JavaString`/`JavaStringResult` 从 `util/logging_utils.rs`
  拆分为 `util/java_string.rs`（重导出保持外部零 churn）。
- **发布配套文档**：`docs/superpowers/specs/2026-07-28-versioning-governance.md`（alpha→beta→1.0 晋级规则）、
  `docs/release/` 目录建立。

### Fixed

- **仓库键冲突竞态**：`ElementNames`/`AttributeNames` 全局仓库在并行访问下，
  `forHTMLName("xmlns:th")` 与 `forHTMLNameWithPrefix("xmlns","th")` 共享键
  `xmlns:th` 产生顺序依赖（Java 固有的首注册者胜语义被 Rust 并行测试暴露）。
  修复为 keep-first（返回既有绑定，对应 Java 读路径 short-circuit），并令
  HTML withPrefix(xmlns) 断言序无关；parity 测试以 `serial_test` 镜像 JUnit
  单线程模型。

### Governance

- `multiple_public_objects` 97 → 0、`missing_object_file` 17 → 0、
  `stub_logic` 22 → 0、`missing_java_source_comment` 1280 → 0
  （通用审计 `errors=0 warnings=0 strict_blockers=0`）。
- 15 个 `thymeleaf-support/*` crate 的 version/edition/rust-version/license
  改为 `[workspace.package]` 继承；`topcoat` rust-version 对齐 1.88。

### 测试（Java 1:1 差分补移植）

- `WebEngineContextTest` 14/14、`EngineContextTest` 10/10、
  `AttributeNamesTest` 6/6、`ElementNamesTest` 6/6、
  `LazyContextVariableTest` 10/10 全部方法 1:1 锁定（含 setVariable null 语义、
  表示串活 exchange 感知、别名 assertSame 系列）。

### Known Limitations（已修复）

- **TemplateEngine.render smoke 超时（真实 bug，已修复）**：根因有两个
  Rust 侧真实缺陷——
  1. `LiteralSubstitutionUtil` 对 `${${||}}` 以完全相同输入无限递归
     （Java 上游是单遍迭代状态机、零递归）。修复：步骤 2 递归加
     `substituted != selector` 进度守卫（相等时落到主状态机），递归入口
     加深度上限 16。Java 3.1.5 实测 ground truth：`th:text="${${||}}"` 在
     模板解析期抛 `TemplateInputException`（嵌套 `${||}` OGNL 语法错误），
     Rust 现在同样快速返回 Err（parity 锁定）。
  2. `markup_selector::parse_attributes` 对 `<L/ꟓ>`、`<L=x>` 类输入
     （自闭合斜杠/`=` 落在属性名位置）空名 push 后永不前进——无限
     `Vec::push` 内存膨胀（14GB） + 100% CPU 挂起。修复：空名时跳过该
     字符保证前进，属性合法性仍由 adapter 侧校验。
  附加结构防御：`ExpressionParsingUtil`/native OGNL 解析入口长度上限
  4096 UTF-16 units + 递归深度上限 256；`parse_internal` 模板字节上限
  64MB；`parse_html` token 进度守卫（span.end 连续不前进即中止）。
  render smoke proptest 与 html parser fuzz 均已恢复（DiscardingWriter +
  shrink 钳制 + proptest timeout 60s 兜底）。
- **html5gum tokenizer**：历史 SIGKILL 根因是输出侧无界缓冲（CapturedWriter，
  已由 DiscardingWriter 消除）；tokenizer 0.8.4 内部缓冲 O(n) 有界，隔离
  驱动验证 `<L/ꟓ>` 等输入 token 流正常。HTML parser fuzz 已恢复。
- **API baseline CI**：`cargo public-api` 需要 nightly toolchain——CI 新增
  固定 `nightly-2026-07-28`（与 `docs/release/api-baseline.txt` 生成版本
  一致，本地已验证 diff 完全匹配），public-api 步骤移除 `continue-on-error`
  改为硬门禁；alpha 阶段 API 漂移必须显式更新 baseline。
