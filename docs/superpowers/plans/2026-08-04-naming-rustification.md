# 命名 Rust 化计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 全量移除 `Java*` 前缀类型与 `java_*` 边界方法，改用 Rust 领域命名，语义锁定行为零变更。

**Architecture:** 8 批改名按依赖顺序执行，每批改名后同步更新 layout_approvals.json、source_parity_inventory、Golden 测试名和文档引用。安全模型修正同步进行（ACL deny-by-default）。

**Tech Stack:** `cargo-public-api`（API 基线再生成）、`scripts/audit_migration_layout.py`（布局审计）

**Related Design Doc:** `docs/superpowers/specs/2026-07-28-naming-consistency-check.md`

---

## 全局约定

- **语义锁定**：改名不改变任何可观察行为
- **批次顺序**：按依赖关系严格排序，避免中间态编译失败
- **同步更新**：每批改名后同步更新 approvals、inventory、Golden 测试名

---

## 实施阶段总览

| Stage | 目标 | 日期 |
|-------|------|------|
| 批1 | 内部型 Rust 化改名 | 2026-08-04 |
| 批2 | JavaString→Utf16String 全量改名 | 2026-08-04 |
| 批3+4 | Locale/TemplateWriter/TemporalValue | 2026-08-04 |
| 批5+6 | 异常三件套 + util Java* 类型 | 2026-08-04 |
| 批7 | TemplateObject trait 边界方法 | 2026-08-04 |
| 批8 | 内部 java_* 辅助函数全量 Rust 化 | 2026-08-04 |
| 安全 | 安全模型修正 | 2026-08-04 |
| 发布 | Release prep | 2026-08-04 |

---

## Stage 批1 — 内部型 Rust 化改名

### Task 1.1：内部类型改名

- [x] **Step 1:** `IteratorValue`/`MapEntryValue`/`StreamValue`/`CharsetDecoder`/`string_case_utils`/`double_string`/`ClassObjectValue` 改名
- [x] **Step 2:** 语义锁定，行为零变更

---

## Stage 批2 — JavaString→Utf16String 全量改名

### Task 2.1：Utf16String 改名

- [x] **Step 1:** 类型/模块/方法全量改名
- [x] **Step 2:** layout_approvals.json 路径同步——java_string.rs→utf16_string.rs
- [x] **Step 3:** multiple_public_objects 批准随批2改名更新

---

## Stage 批3+4 — Locale/TemplateWriter/TemporalValue

### Task 3.1：Locale/TemplateWriter/TemporalValue 改名

- [x] **Step 1:** `JavaLocale`→`Locale`、`JavaWriter`→`TemplateWriter`、`JavaTemporal`→`TemporalValue`
- [x] **Step 2:** layout_approvals.json 同步批3-6 改名——temporal_value.rs 路径 + 33 处旧类型名

---

## Stage 批5+6 — 异常三件套 + util Java* 类型

### Task 5.1：异常与工具类型改名

- [x] **Step 1:** `ClassNotFoundException`→`ClassNotFoundError`、`NoSuchMethodException`→`NoSuchMethodError`、`OgnlException`→`OgnlError`
- [x] **Step 2:** `JavaNumber`→`NumberValue`、`JavaList`→`ListValue`、`JavaDate`→`DateValue`、`JavaEvaluation*`→`Evaluation*`

---

## Stage 批7 — TemplateObject trait 边界方法

### Task 7.1：边界方法去 java_ 前缀

- [x] **Step 1:** `java_class_name`→`class_name`、`java_invoke_method`→`invoke_method` 等 8 项
- [x] **Step 2:** 语义锁定，行为零变更

---

## Stage 批8 — 内部 java_* 辅助函数全量 Rust 化

### Task 8.1：辅助函数改名

- [x] **Step 1:** 清理 60+ 个内部 `java_*` 辅助函数
- [x] **Step 2:** `java_trim`→`trim`、`java_message`→`message`、`java_hash_code`→`hash_code`
- [x] **Step 3:** `CharSequenceValue` 的 `java_length`/`java_char_at`/`java_to_string`→`length`/`char_at`/`to_utf16_string`
- [x] **Step 4:** 台账证据标记同步批8 改名——golden 测试名更新（脚本 + baseline inventory）
- [x] **Step 5:** 五份迁移对照表同步批1-8 改名——Rust 类型/文件路径引用更新

---

## Stage 安全 — 安全模型修正

### Task S.1：ACL deny-by-default

- [x] **Step 1:** 修复预先存在的 CI 测试失败——ACL 默认拒绝 + gtvg Locale 固定
- [x] **Step 2:** util_family 类型门禁断言同步 Rust 安全模型——es/de/com.whatever 默认拒绝
- [x] **Step 3:** is_type_forbidden 裸类名放行——恢复 corpus instancestaticrestrictions 23/26 的 ClassNotFoundException 契约

### Task S.2：安全模型文档

- [x] **Step 1:** ExpressionUtils::is_type_forbidden 改为默认拒绝（仅白名单 + java.time.*/org.thymeleaf.* 受信前缀放行）
- [x] **Step 2:** is_member_forbidden(None, ...) 无目标上下文默认拒绝危险成员
- [x] **Step 3:** GTVG 示例测试固定进程默认 Locale

---

## Stage 发布 — Release Prep

### Task R.1：发布准备

- [x] **Step 1:** 补全 19 个 Java 镜像 stub + 资源文件——保证 1:1 存在性
- [x] **Step 2:** 重写根 README 中英双语（full-stack-doc Rust 模板 + 上游兼容剖面）
- [x] **Step 3:** 第一批——README 安全模型修正 + docs.rs + 多 OS CI + 安全测试套件 + 已知限制
- [x] **Step 4:** 第二批 2.1——vernal publish=false（git 依赖阻断 crates.io）
- [x] **Step 5:** 收尾同步——README/架构设计/api-baseline 再生成 + CHANGELOG 命名 Rust 化条目
- [x] **Step 6:** cargo-deny 容器 action 限定 Linux——macOS runner 不支持容器步骤
