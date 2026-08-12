# S11 统一验证计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 统一执行 Java/Rust 差分、`.thtest`、SOURCE_PARITY 和 Golden 验证，按证据批量晋级对象状态。

**Architecture:** S1-S10 生产语义全部闭合后，一次性导入 Java 测试资产、批量运行差分与 `.thtest`，再依据证据更新对象和语义状态。采用"三台账"模型：SOURCE_PARITY（上游已有测试）、RUST_OBLIGATION（语言映射新增合同）、VALUE_ADD（高风险补测）。

**Tech Stack:** Java Oracle（固定 JDK 21、en_US）、Golden harness、`.thtest` 批量运行、`cargo-llvm-cov`（覆盖率）

**Related Design Doc:** `docs/superpowers/specs/2026-07-28-migration-test-mapping.md`

---

## 全局约定

- **验证顺序**：Java Golden → Rust 单元测试 → `.thtest` → 覆盖率 → 模糊测试
- **状态晋级**：NOT_STARTED → IMPLEMENTED_UNVERIFIED → SOURCE_PARITY 已处置 → Golden/合同差分通过 → 全量门禁通过 → BEHAVIOR_VERIFIED
- **禁止伪完成**：文件存在、编译通过、tasks 勾选不等于完成

---

## 实施阶段总览

| Stage | 目标 | 日期 |
|-------|------|------|
| A | Java Oracle Golden 生成 | 2026-08-01 |
| B | Rust 差分消费 | 2026-08-01 |
| C | .thtest 批量运行 | 2026-08-02 |
| D | SOURCE_PARITY 闭合 | 2026-08-02 |
| E | 对象状态批量晋级 | 2026-08-02 |

---

## Stage A — Java Oracle Golden 生成

### Task A.1：固定 Java Oracle 生成器

**Files:**
- Create: `thymeleaf-test/` 中的 Java Oracle 生成器

- [x] **Step 1:** 固定 JDK 21、`en_US` 环境
- [x] **Step 2:** 生成 49 组 Golden 记录（覆盖 S1 阶段全部对象）
- [x] **Step 3:** 生成 3,756 条 Java 记录

---

## Stage B — Rust 差分消费

### Task B.1：Rust Golden 消费器

**Files:**
- Create: `thymeleaf-test/tests/` 中的 parity 测试文件

- [x] **Step 1:** 逐记录比较 Java 输出与 Rust 输出
- [x] **Step 2:** 覆盖重载、空值、边界、错误和副作用
- [x] **Step 3:** 248 个 Rust 单元测试通过

---

## Stage C — .thtest 批量运行

### Task C.1：上游 .thtest 导入与运行

**Files:**
- Create: `thymeleaf/tests/fixtures/` 中的 .thtest 文件

- [x] **Step 1:** 导入 2,609 个 .thtest 用例
- [x] **Step 2:** 2,608 可执行，2,595 运行通过
- [x] **Step 3:** 13 项 code-level 排除（POLICY_DIFFERENCE）

---

## Stage D — SOURCE_PARITY 闭合

### Task D.1：静态清单生成

**Files:**
- Create: `docs/migration/baseline/migration_test_static_inventory.json`
- Create: `docs/migration/baseline/source_parity_inventory.json`

- [x] **Step 1:** 静态审计识别 875 个 Java 测试方法/注解
- [x] **Step 2:** 展开为五模块 2,156 个运行时 case
- [x] **Step 3:** 875/875 源码入口、2,156/2,156 运行时 case 均已登记，MISSING=0

---

## Stage E — 对象状态批量晋级

### Task E.1：按证据批量晋级

- [x] **Step 1:** 202 个主对象达到 BEHAVIOR_VERIFIED
- [x] **Step 2:** 277 个主对象为 IMPLEMENTED_UNVERIFIED
- [x] **Step 3:** 12 个 Servlet 运行时对象为 JAVA_ONLY_EXEMPT
- [x] **Step 4:** 同步更新对象级对照表、方法级对照表、语义迁移对照表
