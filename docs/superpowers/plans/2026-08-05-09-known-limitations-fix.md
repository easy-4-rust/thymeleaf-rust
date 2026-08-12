# 已知限制修正计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 3 项已知限制（literal substitution 无限递归、markup_selector 零前进、then_some 下溢），建立进度不变量审查规范。

**Architecture:** 三个同源 bug 收敛到同一条移植债——把 Java 的迭代算法搬成 Rust 的递归 + RAII + 手写推进循环，没有为"病态输入下不前进"的失败模式做结构性兜底。修复同时建立编码规范防止同类问题复发。

**Tech Stack:** proptest（fuzz 恢复）、cargo-fuzz（decoupled 路径覆盖）

**Related Design Doc:** `docs/superpowers/specs/2026-08-09-raii-guard-invariants.md`

---

## 全局约定

- **修复优先**：生产稳定性优先于完整规格流程
- **结构防御**：修复后必须建立编译期/审查期规则防止同类问题
- **fuzz 恢复**：修复后恢复 proptest 和 cargo-fuzz 覆盖

---

## 实施阶段总览

| Stage | 目标 | 日期 |
|-------|------|------|
| A | literal substitution 无限递归修复 | 2026-08-04 |
| B | markup_selector 零前进修复 | 2026-08-09 |
| C | then_some 下溢修复 | 2026-08-09 |
| D | 进度不变量审查规范 | 2026-08-09 |
| E | Fuzz 覆盖恢复 | 2026-08-05~09 |

---

## Stage A — literal substitution 无限递归修复

### Task A.1：LiteralSubstitutionUtil 修复

- [x] **Step 1:** 步骤 2 递归加 `substituted != selector` 进度守卫（相等时落到主状态机）
- [x] **Step 2:** 递归入口加深度上限 16
- [x] **Step 3:** Java 3.1.5 实测 ground truth：`th:text="${${||}}"` 在模板解析期抛 `TemplateInputException`
- [x] **Step 4:** Rust 现在同样快速返回 Err（parity 锁定）

### Task A.2：表达式解析入口守卫

- [x] **Step 1:** ExpressionParsingUtil/native OGNL 解析入口长度上限 4096 UTF-16 units
- [x] **Step 2:** 递归深度上限 256
- [x] **Step 3:** parse_internal 模板字节上限 64MB

---

## Stage B — markup_selector 零前进修复

### Task B.1：parse_attributes 零前进修复

- [x] **Step 1:** 空名时跳过该字符保证前进（`<L/ꟓ>`、`<L=x>` 类输入）
- [x] **Step 2:** 属性合法性仍由 adapter 侧校验
- [x] **Step 3:** 修复前：无限 `Vec::push` 内存膨胀（14GB）+ 100% CPU 挂起

### Task B.2：decoupled_template_logic_builder 同源修复

- [x] **Step 1:** 属性扫描零前进同源 bug——markup_selector 修复漏网的双胞胎

### Task B.3：tokenizer 输入/进度守卫

- [x] **Step 1:** parse_html token 进度守卫（span.end 连续不前进即中止）

---

## Stage C — then_some 下溢修复

### Task C.1：深度守卫惰性构造

- [x] **Step 1:** `bool::then_some(带 Drop 的值)` 改为 `then(|| Guard)` 惰性构造
- [x] **Step 2:** 修复前：entered=false 时 Guard 被构造+丢弃，但计数没自增 → drop 时计数下溢 panic

---

## Stage D — 进度不变量审查规范

### Task D.1：编码规范建立

**Files:**
- Create: `docs/coding/raii-guard-and-progress-invariants.md`

- [x] **Step 1:** 规则 1：带 Drop 的守卫必须惰性构造
- [x] **Step 2:** 规则 2：手写推进循环必须保证每轮 position 必增
- [x] **Step 3:** 规则 3：递归下降必须有深度上限 + 进度不变量
- [x] **Step 4:** 历史 bug 案例索引（4 个 bug）
- [x] **Step 5:** PR 审查清单

---

## Stage E — Fuzz 覆盖恢复

### Task E.1：Proptest 恢复

- [x] **Step 1:** render smoke proptest 恢复（DiscardingWriter + shrink 钳制 + proptest timeout 60s 兜底）
- [x] **Step 2:** html parser fuzz 恢复

### Task E.2：Decoupled 路径 fuzz 覆盖

- [x] **Step 1:** decoupled 路径 fuzz 覆盖
- [x] **Step 2:** HTML 解析器 span 切片加 char-boundary 钳制（修复多字节 Unicode panic）

### Task E.3：CI public-api 硬门禁

- [x] **Step 1:** 固定 nightly-2026-07-28
- [x] **Step 2:** public-api 步骤移除 continue-on-error 改为硬门禁
- [x] **Step 3:** 修正 nightly toolchain 放置位置——避免默认 toolchain 切换影响 fmt
