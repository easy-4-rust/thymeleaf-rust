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

> **版本**：`0.1.0-alpha.1` · **MSRV**：Rust 1.95 · **Edition**：2024 · **Resolver**：3
>
> **上游基线**：Thymeleaf `3.1.5.RELEASE` @ `10f9dd2e`

## 1. 概述

thymeleaf-rust 是 Java Thymeleaf 3.1.5 的 Rust 行为级移植。它解析和渲染
HTML/XML/TEXT 模板，支持表达式求值、方言处理器、模板缓存和框架无关的中立 Web 合同。
项目运行时不依赖 JVM——所有 Java 语义均以纯 Rust 重新实现。

### 1.1 状态证据

| 声明 | 值 | 证据 |
|:---|:---|:---|
| 语义对齐语料 | 2,608 个 `.thtest` 用例，2,595 行为一致 | CI `thtest_upstream_plain_batch` |
| 对象级覆盖 | 491 主对象、4,291 方法、0 缺失 | `cargo xtask migration-check` |
| 源码 parity 台账 | 413 个核心测试条目（Spring 按策略排除） | `source_parity_inventory.json` |
| 验收门禁 | 2,686 资产 SHA-256 字节校验 | `thymeleaf-test/tests/acceptance.rs` |
| 测试 | 295（库）+ 964（集成）+ 45（适配器） | `cargo test --workspace` |
| CI 平台 | ubuntu-latest, macos-latest | GitHub Actions 矩阵 |
| unsafe | 全 crate `forbid` | `[lints.rust] unsafe_code = "forbid"` |

### 1.2 非目标

- 运行时不做 JVM/字节码互操作（Java 仅作行为 oracle）。
- 不做 Spring/JSP/Servlet 运行时集成（Java 专用模块按策略排除）。
- 不做内部实现逐行复制——Java 习惯用法映射到 Rust 所有权/trait/错误模式。

## 2. 成熟度

### 2.1 功能矩阵

| 功能 | 状态 | Crate | 限制 |
|:---|:---:|:---|:---|
| HTML/XML/TEXT 模板解析 | ✅ 稳定 | `thymeleaf` | html5gum tokenizer 对病态 Unicode 有内存膨胀风险 |
| 表达式求值（OGNL 子集） | ✅ 稳定 | `thymeleaf` | 无 JVM 反射；ACL 门控的静态方法白名单 |
| Standard 方言处理器 | ✅ 稳定 | `thymeleaf` | 2,608 语料验证 |
| 模板缓存与解析器 | ✅ 稳定 | `thymeleaf` | String/File/Class/URL/Multi/ByteArray 加载器 |
| 自动转义与输出格式 | ✅ 稳定 | `thymeleaf` | HTML/XML/JavaScript/CSS/JSON/RTF/PlainText |
| 解耦模板逻辑 | ✅ 稳定 | `thymeleaf` | `.th.xml` 边车 |
| 框架无关中立 Web 合同 | ✅ 稳定 | `thymeleaf` | IWebExchange / IWebRequest / IWebSession |
| 框架适配器 | 🧪 预览 | `thymeleaf-support/*` | 13 个可发布 + 2 个不发布（tide, vernal） |
| sa-token 安全方言 | 🧪 预览 | `thymeleaf-sa-token` | 12 个合同测试 |
| Fuzz（属性测试） | 🚧 部分 | `thymeleaf-test` | XML/TEXT parser proptest；HTML/render 排除（见已知限制） |

### 2.2 上游兼容

| 维度 | 范围 | 方法 |
|:---|:---|:---|
| 行为 | 2,595 / 2,608 可执行用例与 Java 逐字节一致 | 语料差分 |
| 策略差异 | 13 例（12 个 `execinfo` 上游禁用 + 1 个任意反射链） | 具名处置 |
| 源码 parity | 413 个核心 Java 测试类（Spring 排除） | `source_parity_inventory.json` |
| 对象 parity | 491 / 491 主对象、4,291 / 4,291 方法 | `migration-check` |

## 3. Rust 基线

| 项目 | 值 |
|:---|:---|
| MSRV | 1.95 |
| Edition | 2024 |
| Resolver | 3 |
| Clippy | `-D warnings` |
| rustfmt | stable |
| unsafe | `forbid`（全 crate） |
| missing_docs | `deny`（`thymeleaf` crate） |

## 4. 工作区布局

```text
[下游 crate]
        │ cargo add thymeleaf / thymeleaf-<framework>
        ▼
┌──────────────────────────────────────────────────────────┐
│ thymeleaf-rust 工作区                                    │
│                                                          │
│ thymeleaf               核心引擎、公开 API、Web 合同     │
│ thymeleaf-test          Java parity 语料、golden 测试    │
│ thymeleaf-examples      GTVG 示例移植                    │
│ thymeleaf-support/*     15 个框架适配器                  │
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

| Crate | 发布 | 职责 |
|:---|:---:|:---|
| `thymeleaf` | ✅ | 核心引擎 |
| `thymeleaf-actix-web` | ✅ | Actix-web 适配器 |
| `thymeleaf-axum` | ✅ | Axum 适配器 |
| `thymeleaf-gotham` | ✅ | Gotham 适配器 |
| `thymeleaf-hyper` | ✅ | Hyper 适配器 |
| `thymeleaf-ntex` | ✅ | Ntex 适配器 |
| `thymeleaf-poem` | ✅ | Poem 适配器 |
| `thymeleaf-rocket` | ✅ | Rocket 适配器 |
| `thymeleaf-sa-token` | ✅ | Sa-Token 安全方言 |
| `thymeleaf-salvo` | ✅ | Salvo 适配器 |
| `thymeleaf-tonic` | ✅ | Tonic 适配器 |
| `thymeleaf-topcoat` | ✅ | Topcoat 适配器 |
| `thymeleaf-tower` | ✅ | Tower 适配器 |
| `thymeleaf-warp` | ✅ | Warp 适配器 |
| `thymeleaf-tide` | ❌ | Tide 适配器（上游未维护） |
| `thymeleaf-vernal` | ❌ | Vernal 适配器（git 依赖，待 crates.io） |
| `thymeleaf-test` | ❌ | 测试基建（内部） |
| `thymeleaf-examples` | ❌ | 示例（内部） |

## 5. 安全模型

表达式求值默认使用只读安全子集：

- **`restrict_external_access = true`** 默认开启——`new`、`param`、`@Type@` 语法被封禁。
- **任意 Class 和反射被阻断**——10 个封禁包前缀（`java.`/`javax.`/`jakarta.`/`jdk.`/…），53 个允许类（包装类、集合、时间、数学）。
- **受限静态方法白名单**——`Math.abs/sqrt/…`、`Integer.parseInt`、`LocalDateTime.of`、`String.format` 共 9 个类；其余全部被 `ThymeleafACLClassResolver` 拒绝。
- **`unsafe_code = "forbid"`** 覆盖全 crate——工作区源码零 unsafe。
- 宿主可通过 `OgnlRuntime` 进一步收紧（opt-in）。

## 6. 快速开始

### 从 Git 使用（尚未发布到 crates.io）

```toml
[dependencies]
thymeleaf = { git = "https://github.com/easy-4-rust/thymeleaf-rust.git" }
```

### 最小示例

```rust
use thymeleaf::{TemplateEngine, TemplateMode};
use thymeleaf::context::Context;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::ITemplateResolver;
use std::sync::Arc;

fn main() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");

    let ctx = Context::new();
    let output = engine.process_template("<p th:text=\"${msg}\">fallback</p>", &ctx)
        .expect("render");
    println!("{}", output.to_string_lossy());
}
```

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

### CI 流水线（24 步）

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask migration-check --upstream <upstream> --baseline 10f9dd2e...
python3 scripts/audit_migration_layout.py --fail-on-warning   # 布局审计 0/0/0
cargo deny check                                               # 许可/禁止/来源/公告
cargo audit                                                    # 漏洞扫描
cargo llvm-cov --workspace --all-features --summary-only
THYMELEAF_SCOPE=semantic_all cargo test -p thymeleaf-test --test thtest_upstream_plain_batch  # 2,608 语料
```

### 测试类型

| 类型 | 数量 | 目的 |
|:---|:---:|:---|
| 单元（库） | 295 | 核心逻辑 |
| 集成（parity） | 964 | Java 1:1 差分 |
| 适配器合同 | 45 | 框架集成冒烟 |
| 语料 | 2,608 | 上游行为 parity |
| 验收 | 2,686 资产 | SHA-256 字节一致 |
| Fuzz（proptest） | 2 活跃 | XML/TEXT parser 鲁棒性 |

## 9. 已知限制

- **html5gum tokenizer**：病态 Unicode 输入（孤立代理对、特殊序列）可能导致内部内存膨胀。HTML parser fuzz 排除；鲁棒性由 2,608 语料覆盖。
- **渲染 smoke proptest**：随机表达式注入可能导致 `process_template` 超时（>60s）。排除；由语料 + workspace 测试覆盖。
- **API 基线 CI**：`cargo public-api` 需要 nightly；CI 用 stable → `continue-on-error`（alpha 阶段）。

## 10. 路线图

| 阶段 | 状态 | 项 |
|:---|:---:|:---|
| 语义对齐 | ✅ 完成 | 2,608 语料、491 对象、4,291 方法 |
| 治理审计 | ✅ 完成 | strict blockers 0、warnings 0、CI 强制 |
| Fuzz OOM 修复 | ✅ 完成 | DiscardingWriter + shrink 钳制 + serial |
| 发布生态 | 🚧 进行中 | `cargo package --verify`、docs.rs、适配器合同 |
| 版本 0.1.0 | 🗓️ 计划 | API 冻结、CHANGELOG、tag |
| 基准套件 | 🗓️ 计划 | Criterion 渲染/解析/表达式 |

## 11. 贡献

提交前运行基础门禁：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

新增公开 API 必须包含文档、测试和 SemVer/MSRV 影响说明。

## 12. 许可证

[MIT](./LICENSE)

本项目移植自 [Thymeleaf](https://www.thymeleaf.org/)（Apache 2.0）的行为语义。
上游许可证、来源提交和修改范围记录在 `docs/` 中。

---

<div align="center">

[返回顶部](#readme-top) · [Actions](https://github.com/easy-4-rust/thymeleaf-rust/actions) · [Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
