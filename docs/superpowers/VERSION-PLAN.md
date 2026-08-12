# 版本规划

> 本文件定义 thymeleaf-rust 的版本快照、路线图与晋级门禁映射。
> 详细版本治理规则见 `specs/2026-07-28-versioning-governance.md`。

## 1. 当前版本快照

| 指标 | 值 |
|------|-----|
| 当前版本 | 0.1.0-alpha.1 |
| 基线日期 | 2026-08-02 |
| 上游基线 | Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`） |
| 源码规模 | 569 个 .rs 文件，123,121 行（`thymeleaf/src/`） |
| 测试规模 | 208 个 .rs 文件，61,784 行（`thymeleaf-test/tests/`） |
| Corpus | 2,665 个 .thtest（2,608 可执行，2,595 运行，13 项排除） |
| 对象级 parity | 109 对象级 parity 测试 |
| Git commits | 222（2026-07-28 ~ 2026-08-09） |

## 2. 迁移阶段路线图

| 阶段 | 主对象数 | 交付主题 | 状态 |
|:---:|---:|:---|:---:|
| S0 | 0 | 基线、对象表、语义表、名称门禁 | ✅ 完成 |
| S1 | 20 | crate 骨架、根 API、错误、模板模式 | ✅ 完成 |
| S2 | 30 | Resource、Resolver、Cache | ✅ 完成（29/30 BEHAVIOR_VERIFIED） |
| S3 | 128 | Context、Engine、Event、Model、Handler | ✅ 完成 |
| S4 | 41 | 六种模板模式 Parser 与解耦逻辑 | ✅ 完成（5/41 BEHAVIOR_VERIFIED） |
| S5 | 48 | Processor/Dialect/Pre/Post/Inline SPI | ✅ 完成（32/48 BEHAVIOR_VERIFIED） |
| S6 | 64 | 核心表达式对象、Message、Link、Util | ✅ 完成（17/64 BEHAVIOR_VERIFIED） |
| S7 | 88 | Standard Expression、Inline、Serializer | ✅ 完成 |
| S8 | 56 | Standard `th:*` Processor | ✅ 完成 |
| S9 | 16 | 中立 Web 合同和 Servlet 语义等价迁移 | ✅ 完成 |
| S10 | — | `thymeleaf-{framework}`、`thymeleaf-vernal` | ✅ 完成 |
| S11 | — | 全量 Parity、安全、性能和发布 | 🔄 进行中 |

## 3. 后迁移里程碑

| 里程碑 | 目标 | 前置条件 |
|--------|------|---------|
| alpha.2 | 命名 Rust 化收尾 + 已知限制修正 | S11 部分完成 |
| beta.0 | 治理门禁全绿 + API 冻结准备 | alpha 系列稳定 |
| beta.1 | 外部集成验证 + 双人评审 | beta.0 通过 |
| 1.0.0 | API 冻结 + SemVer 承诺 | beta 系列稳定 |

## 4. 晋级门禁映射

### 4.1 alpha → beta 门禁

| 门禁 | 当前状态 | 备注 |
|------|---------|------|
| 通用迁移布局审计 `strict_blockers=0` | ✅ 通过 | `scripts/audit_migration_layout.py` |
| `migration-check` PASS | ✅ 通过 | 491 对象，0 missing |
| 严格 Clippy + fmt | ✅ 通过 | CI 硬门禁 |
| workspace 全测试 | ✅ 通过 | 2,595/2,595 .thtest |
| `cargo-deny` + `cargo-audit` | ✅ 通过 | CI 硬门禁 |
| fuzz 基线 | ✅ 落档 | proptest + cargo-fuzz |
| 安全文档 | ✅ 评审通过 | docs/release/security.md |
| `cargo package --verify` | ⏳ 待执行 | 发布演练 |

### 4.2 beta → 1.0 门禁

| 门禁 | 当前状态 | 备注 |
|------|---------|------|
| API 冻结 | ⏳ 待执行 | `cargo public-api` 基线 |
| `cargo semver-checks` | ⏳ 待执行 | 破坏性变更检查 |
| 外部集成依赖发布 | ⏳ 待执行 | vernal/sa-token 等 |
| 双人评审记录 | ⏳ 待执行 | 对象级对照表 + layout_approvals.json |

## 5. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-07-28 | 初始版本规划——基于迁移路线图和 CHANGELOG |
