# 安全模型（Security Model）

> 本文档描述 thymeleaf-rust 的安全边界、策略与鲁棒性基线。
> 全部事实来自代码实测（取证日期 2026-08-16；核对命令随文附注）。
> 本文件是 alpha→beta 晋级门禁之一（specs/2026-07-28-versioning-governance.md §条件 8）。

## 1. 威胁模型与信任边界

thymeleaf-rust 是**宿主内嵌模板引擎库**，信任边界划分为三层：

| 层 | 信任级别 | 说明 |
|----|---------|------|
| 宿主程序（引擎调用方） | 完全信任 | 控制 Configuration/Dialect/Resolver 注册 |
| 模板作者 | 半信任 | 模板文本中的 `th:*` 属性与内联表达式受表达式求值限制约束（§3） |
| 模板数据（context 变量） | 不信任 | 仅数据，不携带可执行语义；引擎不对其做代码求值 |

Java 上游的 Spring 集成语义（spring5/6、springsecurity、webflux）**不在迁移范围**
（对象级对照表范围声明）；12 个 Servlet 运行时对象采用宿主等价映射
（`JAVA_ONLY_EXEMPT`，layout_approvals 与对象账本登记），Servlet/HTTP 边界由宿主
框架负责。

## 2. unsafe 政策

**核心 crate 零 unsafe**（实测）：

```bash
grep -rn "unsafe " thymeleaf/src --include="*.rs" | wc -l   # → 0
```

`thymeleaf/src`、`thymeleaf-support`、`thymeleaf-test/src` 无任何 `unsafe` 块。
未来引入 unsafe 须在本文件登记 SAFETY 论证并经评审。

## 3. 表达式求值限制（受限子集模型）

标准表达式的静态类型引用与方法调用走三层限制
（`expression/native_variable_expression_evaluator.rs`）：

1. **禁止类型 ACL**（先于一切类型解析）：
   `ThymeleafACLClassResolver::class_for_name` →
   `ExpressionUtils::is_type_forbidden(type_name)` 命中即报
   *"Access is forbidden for type '...' in this expression context."*
   ——1:1 移植 Java `OGNLVariableExpressionEvaluator.ThymeleafACLClassResolver`；
2. **静态方法白名单**：`invoke_static_method` 仅放行登记过的受限函数集
   （`java.lang.Math` abs/ceil/floor/sqrt/cbrt/三角/log/pow/min/max/round、
   `java.lang.Integer` parseInt/valueOf 等）；未登记组合不可达；
3. **宿主运行时优先**：若宿主注册了 OGNL 兼容运行时
   （`current_ognl_runtime()`），表达式求值交由宿主——安全边界随宿主策略。

动态方法调用（`invoke_dynamic_method`）仅作用于**上下文中的既有对象**，
不存在按类名加载任意类型再实例化的路径（`ThymeleafDefaultClassResolver`
不做 `java.lang.` 隐式补全，空/短类型名直接拒绝）。

## 4. 依赖治理（CI 硬门禁）

- **cargo-deny**：licenses 显式 allowlist（`deny.toml [licenses]`）；
  advisories 检查启用，例外逐条登记并注明理由（如 sa-token 外部链的
  notice/unmaintained 豁免，deny.toml:80 注释）；
- **cargo-audit**：RUSTSEC 公告门禁；
- **来源约束**：crates.io 之外的新依赖源须增补 sources 白名单并评审
  （2026-08-16 曾为 aspect-rs git 源做过一次增补，commit d59d2fa）。

## 5. 鲁棒性基线（fuzz）

`thymeleaf-test/tests/robustness_fuzz_smoke.rs`（proptest 实现，245 行）：

- 默认 64 用例；本地加深：`PROPTEST_CASES=10000 cargo test -p thymeleaf-test
  --test robustness_fuzz_smoke`；
- shrink 钳制：`max_shrink_iters: 256` + `max_shrink_time: 10s`；
- 单 case 超时 60s（防卡死用例）；
- 覆盖面：模板解析/渲染的恶意输入 smoke（畸形标签、深嵌套、超长 token）。

## 6. 公开 API 面门禁

`docs/release/api-baseline.txt`（cargo-public-api 快照）+ CI diff 门禁：
任何未登记的公开 API 变更（增/删/改签名）使构建失败。API 冻结状态与
晋级关系见 `../superpowers/specs/2026-07-28-versioning-governance.md`。

## 7. 已知边界（非漏洞，如实登记）

- 12 个 Servlet 运行时对象为宿主等价映射（`JAVA_ONLY_EXEMPT`）——不在 Rust
  侧提供 Servlet 容器语义；
- 表达式白名单与 Java OGNL 环境的函数集并非逐一枚举等价：宿主接入 OGNL
  运行时后，白名单语义由宿主运行时决定（行为差异由宿主负责）；
- corpus 排除项 13 例（POLICY_DIFFERENCE）为策略性差异，登记于迁移路线图。

## 8. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-08-16 | 初版——基于代码实测（unsafe=0 / ACL+白名单 / deny 策略 / fuzz 基线 / package 演练通过）；修正 VERSION-PLAN 门禁虚标 |
