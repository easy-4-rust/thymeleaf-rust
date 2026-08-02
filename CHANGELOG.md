# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与
[语义化版本](https://semver.org/lang/zh-CN/)（晋级规则见
[docs/release/versioning.md](docs/release/versioning.md)）。

## [Unreleased]

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
- **发布配套文档**：`docs/release/versioning.md`（alpha→beta→1.0 晋级规则）、
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
