# 版本治理与发布门禁

- **日期**：2026-07-28
- **作者**：thymeleaf-rust 团队
- **状态**：已实施
- **上游基线**：Thymeleaf 3.1.5.RELEASE（commit `10f9dd2eb8cbd98515ce14b149d115e0287d0add`)
- **相关计划**：`docs/superpowers/plans/2026-08-03-governance-hardening.md`

---

# 版本治理与发布门禁

> 本文件定义 thymeleaf-rust 从 `0.1.0-alpha` 到稳定版的版本晋级规则、发布门禁与
> 语义化版本承诺。核心 crate `thymeleaf` 与 15 个整合 crate 共享 `[workspace.package]`
> 版本号（`Cargo.toml`），保持同批发布。

## 1. 当前版本与目标

- 当前：`0.1.0-alpha.1`（2026-08-02 起，阶段 0-1 治理完成后的基线）
- 阶段目标：维持 alpha 系列（`0.1.0-alpha.x`）直至治理、鲁棒性与发布流程全部就绪，
  再按 §3 晋级 `0.1.0`（首个稳定版），此后按 SemVer 承诺演化。

## 2. SemVer 承诺（1.0 前）

alpha 阶段不承诺稳定 API；但以下红线始终有效：

- **破坏性变更必须记录在 CHANGELOG.md 的对应 Unreleased 条目**；
- **`thymeleaf` 核心 crate 的公开 API 面**（由 `cargo public-api` 基线锁定）不得在
  无评审的情况下扩大/收缩；
- 迁移语义（Java Thymeleaf 3.1.5 基线 `10f9dd2eb8cbd98515ce14b149d115e0287d0add`）
  不因版本迭代而改变：2608 语料差分、SOURCE_PARITY 875、acceptance 2686 为不可回归门禁。

## 3. 晋级规则

### 3.1 alpha → beta（`0.1.0-alpha.x` → `0.1.0-beta.x`）

满足全部条件后发起：

1. 通用迁移布局审计 `strict_blockers=0` 且 `warnings=0`（`--fail-on-warning` 通过）；
2. `migration-check` PASS（491 对象、0 missing）；
3. 严格 Clippy、fmt、workspace 全测试、2608 语料、acceptance 2686 全绿；
4. 发布演练 `cargo package --verify` 覆盖全部可发布 crate；
5. `cargo-deny`（依赖许可/来源/重复）与 `cargo-audit`（漏洞）无未豁免告警；
6. fuzz（proptest 常驻 + cargo-fuzz 目标）与基准（criterion）基线落档；
7. 安全模型文档（docs/release/security.md）评审通过。

### 3.2 beta → 1.0（`0.1.0-beta.x` → `1.0.0`）

在 beta 条件基础上追加：

1. API 冻结：`cargo public-api` 基线 + `cargo semver-checks` 全绿；
2. 破坏性变更清单确认：1.0 发布时清理所有 alpha/beta 期间的临时 API；
3. 外部集成依赖发布完成（vernal/sa-token 等阻断点解除）；
4. 双人评审记录（对象级对照表、layout_approvals.json 批准清单）。

### 3.3 晋级动作

- 更新 `Cargo.toml` 的 `[workspace.package] version`
- CHANGELOG.md 移动 Unreleased → 版本条目
- git tag：`v<version>`（annotated），GitHub Release 自动生成
- 重新生成 coverage/审计/语料台账快照并落档

## 4. 版本号纪律

- 版本号单一事实来源：`Cargo.toml` `[workspace.package].version`（15 个 support
  crate 已全部 `*.workspace = true` 继承，禁止手写版本号）；
- `topcoat` 的 `rust-version` 已对齐 1.88（workspace 继承）；若未来某依赖强制更高
  MSRV，须在 `docs/release/versioning.md` 登记批准例外并单独标注该 crate 的 MSRV；
- 根 `Cargo.lock` 不跟踪（库 crate 发布惯例），CI 使用 `--locked` 时以
  `xtask/Cargo.lock` 为准。

## 5. 评审与责任

- 本文件变更需随版本晋级提交一并评审；
- 每个晋级条件对应一个可执行门禁（见 §3 引用），不允许"口头确认"。
