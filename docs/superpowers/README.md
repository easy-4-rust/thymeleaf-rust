# thymeleaf-rust Superpowers 规格驱动开发体系

> 本目录遵循 [obra/superpowers](https://github.com/obra/superpowers) 执行方法层约定，
> 为 thymeleaf-rust 项目提供 plans（实施计划）与 specs（设计规格）的结构化管理。

## 目录结构

```
docs/superpowers/
├── README.md              # 本文件——约定与索引
├── VERSION-PLAN.md        # 版本规划（快照 + 路线图 + 晋级门禁映射）
├── AUDIT-SUMMARY.md       # 历史计划合规审计总结
├── baseline/              # CI 门禁依赖的 JSON 台账（从 docs/migration/baseline/ 迁移）
│   ├── java_api_inventory.json
│   ├── migration_test_static_inventory.json
│   ├── source_parity_inventory.json
│   └── thtest_inventory.json
├── plans/                 # 实施计划
│   └── YYYY-MM-DD-<kebab-name>.md
└── specs/                 # 设计规格
    └── YYYY-MM-DD-<kebab-name>-design.md
```

## 项目概况

| 指标 | 值 |
|------|-----|
| 项目 | thymeleaf-rust，Thymeleaf 3.1.5.RELEASE 的 Rust 语义兼容移植 |
| 上游 Java | Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`） |
| 当前版本 | 0.1.0-alpha.1（[Unreleased]） |
| 源码 | `thymeleaf/src/`：569 个 .rs 文件，123,121 行 |
| 测试 | `thymeleaf-test/tests/`：208 个 .rs 文件，61,784 行 |
| Corpus | 2,665 个 .thtest（2,608 可执行，2,595 运行，13 项排除） |
| 对象级 parity | 109 对象级 parity 测试 |
| Git 历史 | 222 commits（2026-07-28 ~ 2026-08-09） |
| 命名 Rust 化 | 8 批完成（JavaString→Utf16String 等，语义锁定行为零变更） |
| 安全模型 | ACL deny-by-default（Thymeleaf 自有） |

## Plans 约定

**命名规则**：`YYYY-MM-DD-<kebab-name>.md`

**日期**：使用真实 git 提交日期或版本发布日期，不编造。

**格式**（参照 liteflow / freemarker-rust 约定）：

```markdown
# <计划标题>

> **For agentic workers:** REQUIRED SUB-SKILL: ...

**Goal:** 一句话目标
**Architecture:** 架构概要
**Tech Stack:** 技术栈
**Related Design Doc:** `docs/superpowers/specs/...`

---

## 全局约定

---

## 实施阶段总览

| Stage | 目标 | 预期 Task 数 |
|-------|------|-------------|
| 1     | ...  | N           |

## Stage N — <阶段标题>

### Task N.M：<任务标题>

**Files:**
- Create: ...
- Modify: ...
- Test: ...

- [ ] **Step 1: ...**
- [ ] **Step 2: ...**
```

**Task 状态标记**（审计时使用）：
- `- [x]` 已完成
- `- [ ]` 未完成
- `- [~]` 部分完成（附说明）

## Specs 约定

**命名规则**：`YYYY-MM-DD-<kebab-name>-design.md`

**定位**：specs 是**完整迁移**，包含设计细节和元数据。旧 `docs/` 下的文档内容
已完整搬迁到 `specs/` 目录。

**格式**：

```markdown
# <规格标题>

- **日期**：YYYY-MM-DD
- **作者**：thymeleaf-rust 团队
- **状态**：已实施 | 实施中 | 待实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit 10f9dd2eb8cbd98515ce14b149d115e0287d0add）

## 1. 目标与范围

## 2. 设计来源

| 文档 | 路径 | 核心内容 |
|------|------|----------|
| ...  | ...  | ...      |

## 3. 关键设计决策

## 4. 验收标准
```

## 迁移文档索引

| Superpowers 产物 | 迁移来源 | 性质 |
|------------------|---------|------|
| `specs/2026-07-28-architecture-design.md` | `docs/Thymeleaf-Rust-可行性与架构设计.md` | 核心架构 spec |
| `specs/2026-07-28-migration-technical-requirements.md` | `docs/migration/Thymeleaf-Rust-迁移技术要求.md` | 迁移技术 spec |
| `specs/2026-07-28-semantic-migration-mapping.md` | `docs/migration/语义迁移对照表.md` | 参考 spec |
| `specs/2026-07-28-object-level-mapping.md` | `docs/migration/对象级对照表.md` | 参考 spec |
| `specs/2026-07-28-method-level-mapping.md` | `docs/migration/方法级对照表.md` | 参考 spec |
| `specs/2026-07-28-naming-consistency-check.md` | `docs/migration/对象名称一致性检查.md` | 参考 spec |
| `specs/2026-07-28-migration-test-mapping.md` | `docs/migration/迁移测试对照表.md` | 参考 spec |
| `specs/2026-07-28-java-test-class-mapping.md` | `docs/migration/Java测试类对照.md` | 参考 spec |
| `specs/2026-07-28-java-test-method-mapping.md` | `docs/migration/Java测试方法对照.md` | 参考 spec |
| `specs/2026-08-09-raii-guard-invariants.md` | `docs/coding/raii-guard-and-progress-invariants.md` | 编码 spec |

## 历史计划索引

| 计划文件 | 对应阶段 | 日期 | 核心交付 |
|---------|---------|------|---------|
| `2026-07-28-s0-s10-batch-migration.md` | S0-S10 批量迁移 | 2026-07-28~31 | 全量生产语义域批量实施 |
| `2026-08-02-s11-parity-verification.md` | S11 统一验证 | 2026-08-01~02 | Java/Rust 差分 + .thtest + SOURCE_PARITY |
| `2026-08-03-governance-hardening.md` | 治理收口 | 2026-08-03 | CI 门禁 + parity ledger + Maven scope + fuzz |
| `2026-08-04-naming-rustification.md` | 命名 Rust 化 | 2026-08-04 | 8 批改名 + 安全模型修正 + release prep |
| `2026-08-05-09-known-limitations-fix.md` | 已知限制修正 | 2026-08-05~09 | 3 项 bug 修复 + 进度不变量审查规范 |

## CI 门禁依赖的台账文件

以下文件不迁移，保留在原位或移到 `baseline/`：

| 文件 | 原路径 | 用途 |
|------|--------|------|
| `java_api_inventory.json` | `docs/migration/baseline/` | Java API 清单 |
| `migration_test_static_inventory.json` | `docs/migration/baseline/` | 迁移测试静态清单 |
| `source_parity_inventory.json` | `docs/migration/baseline/` | SOURCE_PARITY 台账 |
| `thtest_inventory.json` | `docs/migration/baseline/` | .thtest 清单 |
| `layout_approvals.json` | `docs/migration/` | CI 门禁批准清单 |
| `api-baseline.txt` | `docs/release/` | cargo-public-api 基线 |

## 变更记录

| 日期 | 变更 |
|------|------|
| 2026-07-28 | 初始迁移——从 docs/ 迁移全部现有文档到 superpowers 体系 |
