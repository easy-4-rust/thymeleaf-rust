<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**框架无关的 Thymeleaf 兼容动态模板引擎（Rust 实现）**

[![Build](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange)](#3-rust-基线)
[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

[English](./README.md) · [简体中文](./README.zh-CN.md)

[概述](#1-概述) · [成熟度](#2-成熟度) · [工作区](#4-工作区布局) ·
[快速开始](#6-快速开始) · [质量门禁](#8-质量门禁) · [路线图](#10-路线图)

</div>

---

> **版本**：`0.2.0` · **MSRV**：Rust 1.95 · **Edition**：2024 · **Resolver**：3
>
> [crates.io](https://crates.io/crates/thymeleaf/0.2.0) · [docs.rs](https://docs.rs/thymeleaf/0.2.0/thymeleaf/) · [GitHub Release](https://github.com/easy-4-rust/thymeleaf-rust/releases/tag/v0.2.0)
>
> **上游基线**：Thymeleaf `3.1.5.RELEASE` @ `10f9dd2e`

## 1. 概述

thymeleaf-rust 是 Java Thymeleaf 3.1.5 的 Rust 行为级移植。它解析和渲染
HTML/XML/TEXT 模板，支持表达式求值、方言处理器、模板缓存和框架无关的中立 Web 合同。
项目运行时不依赖 JVM——已支持的模板语义由 Rust 实现。

### 1.1 状态证据

| 声明 | 值 | 证据 |
|:---|:---|:---|
| 语义对齐语料 | 2,608 个 `.thtest` 用例，2,595 个通过差分测试 | [语料测试](./thymeleaf-test/tests/thtest_upstream_plain_batch.rs) |
| 对象级覆盖 | 491 主对象、4,291 方法、0 缺失 | `cargo xtask migration-check` |
| 源码 parity 台账 | 413 个核心 Java 测试方法、1,154 个运行用例 | [源码对照台账](./docs/migration/baseline/source_parity_inventory.json) |
| 验收门禁 | 按 SHA-256 校验登记的测试资产 | [资产验收测试](./thymeleaf-test/tests/acceptance.rs) |
| CI 平台 | ubuntu-latest, macos-latest | GitHub Actions 矩阵 |
| unsafe | 核心与适配器 crate 使用 `forbid` | `[lints.rust] unsafe_code = "forbid"` |

### 1.2 非目标

- 运行时不做 JVM/字节码互操作（Java 仅作行为 oracle）。
- 不做 Spring/JSP/Servlet 运行时集成（Java 专用模块按策略排除）。
- 不做内部实现逐行复制——Java 习惯用法映射到 Rust 所有权/trait/错误模式。

## 2. 成熟度

### 2.1 功能矩阵

| 功能 | 状态 | Crate | 限制 |
|:---|:---:|:---|:---|
| HTML/XML/TEXT 模板解析 | ✅ 稳定 | `thymeleaf` | 各模式的解析与渲染测试覆盖不同 |
| 表达式求值（OGNL 子集） | ✅ 稳定 | `thymeleaf` | 无 JVM 反射；ACL 门控的静态方法白名单 |
| Standard 方言处理器 | ✅ 稳定 | `thymeleaf` | 2,595 个用例通过语义差分 |
| 模板缓存与解析器 | ✅ 稳定 | `thymeleaf` | String/File/Class/URL/Multi/ByteArray 加载器 |
| 自动转义与输出格式 | ✅ 稳定 | `thymeleaf` | HTML/XML/JavaScript/CSS/JSON/RTF/PlainText |
| 解耦模板逻辑 | ✅ 稳定 | `thymeleaf` | `.th.xml` 边车 |
| 可选 DTD 验证（XML 模式） | 🧪 预览 | `thymeleaf`（`dtd-validation` feature） | 内嵌 W3C XHTML DTD；`Disabled`/`Warn`/`Strict` 三策略，默认 `Disabled`；含内部子集的 DOCTYPE 跳过验证 |
| 框架无关中立 Web 合同 | ✅ 稳定 | `thymeleaf` | IWebExchange / IWebRequest / IWebSession |
| 框架适配器 | 🧪 预览 | `thymeleaf-support/*` | Axum、Actix Web、Topcoat 提供 `render_async` |
| Tide 适配器（仅源码） | 🧪 实验 | `thymeleaf-tide` | Tide / async-std 上游已停止维护，排除默认 Workspace、CI 与安全依赖图；不发布 |
| Vernal 视图层 | 🧪 预览 | `thymeleaf-vernal` | `ThymeleafView` 转为 Vernal 响应，`ThymeleafViewResolver` 提供视图解析 |
| Sa-Token 安全方言 | 🧪 预览 | `thymeleaf-sa-token` | 提供基于授权状态的模板处理器 |
| Fuzz（属性测试） | 🚧 部分 | `thymeleaf-test` | XML 和 TEXT 解析器属性测试 |

### 2.2 上游兼容

| 维度 | 范围 | 方法 |
|:---|:---|:---|
| 行为 | 2,608 个可执行用例中有 2,595 个通过差分测试 | 语料差分 |
| 策略差异 | 13 例（12 个 `execinfo` 上游禁用 + 1 个任意反射链） | 具名处置 |
| 源码 parity | 413 个核心 Java 测试方法（Spring 排除） | [源码对照台账](./docs/migration/baseline/source_parity_inventory.json) |
| 对象 parity | 491 / 491 主对象、4,291 / 4,291 方法 | `migration-check` |

## 3. Rust 基线

| 项目 | 值 |
|:---|:---|
| MSRV | 核心 1.95，Topcoat 适配器 1.99 |
| Edition | 2024 |
| Resolver | 3 |
| Clippy | `-D warnings` |
| rustfmt | stable |
| unsafe | `forbid`（核心及适配器） |
| missing_docs | `deny`（`thymeleaf` crate） |

## 4. 工作区布局

```text
[下游 crate]
        │ cargo add thymeleaf
        ▼
┌──────────────────────────────────────────────────────────┐
│ thymeleaf-rust 工作区                                    │
│                                                          │
│ thymeleaf               核心引擎、公开 API、Web 合同     │
│ thymeleaf-test          Java parity 语料、golden 测试    │
│ thymeleaf-examples      GTVG 示例移植                    │
│ thymeleaf-support/*     14 个 Workspace 适配器           │
│   ├── thymeleaf-actix-web   thymeleaf-axum               │
│   ├── thymeleaf-hyper       thymeleaf-rocket              │
│   ├── thymeleaf-sa-token    thymeleaf-salvo  ...          │
├──────────────────────────────────────────────────────────┤
│ xtask                   migration-check 门禁工具         │
│ scripts/                golden 再生成、审计脚本           │
│ docs/                   迁移文档、发布策略                │
└──────────────────────────────────────────────────────────┘
```

### Crate 映射

| Crate | v0.2.0 crates.io 状态 | 职责 |
|:---|:---:|:---|
| `thymeleaf` | [crates.io](https://crates.io/crates/thymeleaf/0.2.0) | 核心引擎 |
| `thymeleaf-actix-web` | 源码 | Actix-web 适配器 |
| `thymeleaf-axum` | 源码 | Axum 适配器 |
| `thymeleaf-gotham` | 源码 | Gotham 适配器 |
| `thymeleaf-hyper` | 源码 | Hyper 适配器 |
| `thymeleaf-ntex` | 源码 | Ntex 适配器 |
| `thymeleaf-poem` | 源码 | Poem 适配器 |
| `thymeleaf-rocket` | 源码 | Rocket 适配器 |
| `thymeleaf-sa-token` | 源码 | Sa-Token 安全方言 |
| `thymeleaf-salvo` | 源码 | Salvo 适配器 |
| `thymeleaf-tonic` | 源码 | Tonic 适配器 |
| `thymeleaf-topcoat` | 源码 | Topcoat 适配器 |
| `thymeleaf-tower` | 源码 | Tower 适配器 |
| `thymeleaf-warp` | 源码 | Warp 适配器 |
| `thymeleaf-tide` | 未发布 | Tide 适配器（不属于 Workspace，上游未维护） |
| `thymeleaf-vernal` | 未发布 | Vernal 适配器（使用 Git 依赖） |
| `thymeleaf-test` | 未发布 | 测试基建（内部） |
| `thymeleaf-examples` | 未发布 | 示例（内部） |

## 5. 安全模型

受限表达式上下文可阻止外部访问。静态类和方法访问遵循 `ThymeleafACLClassResolver` 与 `OgnlRuntime` 执行的策略。应用应使用可信模板渲染数据。

[表达式上下文策略](./thymeleaf/src/expression/standard_expression_execution_context.rs)

## 6. 快速开始

### 从 crates.io 安装（推荐）

将依赖加入 `Cargo.toml`，把示例保存为 `src/main.rs`，运行 `cargo run`。

```toml
[dependencies]
thymeleaf = "0.2.0"
```

0.2.0 保留模板表达式对象 `#dates` 和 `#temporals`。直接构建日期或时间值的 Rust 调用方使用 Jiff 类型。详见[时间 API 迁移说明](./docs/superpowers/specs/2026-11-18-chrono-migration.md)。

### 从 Git 使用（开发快照）

```toml
[dependencies]
thymeleaf = { git = "https://github.com/easy-4-rust/thymeleaf-rust.git" }
```

### 最小示例

```rust
use std::sync::Arc;
use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{TemplateEngine, TemplateMode};

fn main() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);

    let engine = TemplateEngine::new();
    engine.set_template_resolver(Arc::new(resolver)).expect("configure resolver");

    let context = Context::new();
    context.set_variable(
        Some(Utf16String::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str("Rust")))),
    );

    let output = engine
        .process_template(r#"<p th:text="${name}">fallback</p>"#, &context)
        .expect("render template");
    println!("{}", output.to_string_lossy());
}
```

预期输出 `<p>Rust</p>`.

XML 模式使用 DTD 校验时，启用 `dtd-validation` Cargo feature，并在渲染前调用 `TemplateEngine::set_dtd_validation_policy`，策略选用 `thymeleaf::dtd::ValidationPolicy::Warn` 或 `Strict`。

## 7. Java → Rust 语义映射

| Java | Rust | 原因 |
|:---|:---|:---|
| Checked exception | `Result<T, E>` + `thiserror` 枚举 | 显式错误传播 |
| `null` | `Option<T>` | 空值可见 |
| `synchronized` / `ConcurrentHashMap` | `Arc<RwLock<_>>` | 所有权并发 |
| 反射 / `Class.forName` | `ThymeleafACLClassResolver` + `OgnlRuntime` trait | 无动态类加载；ACL 门控 |
| 内部类 | 同文件类型族（审计批准） | Rust 模块惯例 |
| `ExecutorService` | `futures` + 同步核心 | 核心同步；异步在适配层 |

## 8. 质量门禁

在仓库根目录运行以下命令。工具链须满足整个工作区要求，包括 Topcoat 适配器要求的 Rust 1.99。

```bash
cargo fmt --all --check
python3 scripts/generate_test_asset_manifest.py --output thymeleaf-test/source-test-parity.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

将固定版本的 Java 上游源码检出到 `upstream-thymeleaf` 后，可单独运行语料差分。

```bash
THYMELEAF_UPSTREAM="$PWD/upstream-thymeleaf" THYMELEAF_SCOPE=semantic_all \
  cargo test -p thymeleaf-test --test thtest_upstream_plain_batch
```

[CI 工作流](./.github/workflows/ci.yml)记录了 Java 基线检出、回归及迁移审计步骤。另见[覆盖率](./.github/workflows/coverage.yml)、[安全审计](./.github/workflows/security.yml)和[发布预检](./.github/workflows/release-preflight.yml)。

## 9. 已知限制

- **模糊测试范围**。工作区的[属性测试](./thymeleaf-test/tests/robustness_fuzz_smoke.rs)覆盖 XML 和 TEXT 解析。定时[模糊测试工作流](./.github/workflows/fuzz.yml)覆盖 HTML 渲染和表达式求值。
- **API 基线 CI**：`cargo public-api` 基线为 CI 硬门禁（固定 nightly-2026-07-28），任何公开 API 漂移即失败。

## 10. 路线图

| 阶段 | 状态 | 项 |
|:---|:---:|:---|
| 语义对齐 | ✅ 完成 | 2,608 语料、491 对象、4,291 方法 |
| 治理审计 | ✅ 完成 | strict blockers 0、warnings 0、CI 强制 |
| Fuzz OOM 修复 | ✅ 完成 | DiscardingWriter + shrink 钳制 + serial |
| 核心 v0.2.0 | ✅ 已发布 | [crates.io](https://crates.io/crates/thymeleaf/0.2.0) · [docs.rs](https://docs.rs/thymeleaf/0.2.0/thymeleaf/) · [GitHub Release](https://github.com/easy-4-rust/thymeleaf-rust/releases/tag/v0.2.0) |
| 渲染基准 | ✅ 已运行 | [Criterion 渲染回归](./.github/workflows/bench-regression.yml) |
| 适配器验收 | 🚧 进行中 | 扩展宿主层流式输出、错误与取消测试 |

## 11. 贡献

提交前运行[质量检查](#8-质量门禁)。

新增公开 API 必须包含文档、测试和 SemVer/MSRV 影响说明。

## 12. 许可证

[MIT](./LICENSE)

本项目移植自 [Thymeleaf](https://www.thymeleaf.org/)（Apache 2.0）的行为语义。
上游许可证、来源提交和修改范围记录在 `docs/` 中。

---

<div align="center">

[返回顶部](#readme-top) · [Actions](https://github.com/easy-4-rust/thymeleaf-rust/actions) · [Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
