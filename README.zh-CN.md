<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**面向 Rust 的框架中立、受 Thymeleaf 启发的动态内容渲染引擎。**

[![项目状态：迁移实施中](https://img.shields.io/badge/status-migration%20in%20progress-blue)](#项目状态)
[![许可证：MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

[English](README.md) | [简体中文](README.zh-CN.md)

[项目概览](#项目概览) · [架构](#架构) · [Crate 模型](#crate-模型) ·
[集成模型](#集成模型) · [路线图](#路线图) · [参与贡献](#参与贡献)

</div>

---

> **项目状态：迁移实施中**
>
> `thymeleaf` Cargo Workspace 和首个行为验证 Foundation 切片已经存在。渲染引擎、Parser、Processor、整合层、稳定公共 API 与 crates.io 发布仍未完成；当前不声明整体兼容。

## 项目概览

`thymeleaf-rust` 是项目和 Git 仓库名称，目标是使用 Rust 实现一个受 [Thymeleaf](https://www.thymeleaf.org/) 模板语义启发的动态内容渲染引擎。

未来对外发布的核心 crate 名称是 **`thymeleaf`**。框架整合 crate 使用 `thymeleaf-{framework}` 模式；不会发布或创建任何 `thymeleaf-rust-*` crate 或 Rust 模块。

项目计划同时支持两条地位相同的集成路径：

1. 与 Rust Web 框架直接、独立集成；
2. 作为 Vernal 及其 Web 适配器的可选动态内容渲染引擎。

引擎 Core 将保持对 Topcoat、Actix Web、Axum、Gotham、Hyper、Ntex、Poem、Rocket、Salvo、Tide、Warp、Tower、Tonic 和 Vernal 的完全中立。

本项目是独立项目，不是 Thymeleaf 官方项目。

## 目标与边界

### 项目目标

- 提供具有 Thymeleaf 风格处理语义的自然 HTML 模板。
- 支持变量、选择表达式、消息、链接、Fragment、Processor 和 Dialect。
- 构建不可变、可缓存、可重放的模板模型。
- 支持完整渲染和具有背压能力的流式渲染。
- 暴露中立的 `RenderedTemplate`/HTTP Body 合同。
- 为目标 Rust Web 框架提供独立版本的适配器。
- 提供可选的 `thymeleaf-vernal` bridge，同时避免 Vernal 成为 Core 依赖。
- 通过可追溯的 Parity Test 和 Golden Test 推进上游兼容。

### 非目标

- 在 Core 中复制 Java 继承、反射、Servlet 或 Spring 类型。
- 在发布版本化兼容矩阵前宣称完全兼容 Thymeleaf。
- 让某个框架的 Response 类型进入引擎公共 API。
- 自动把运行时模板转换成 Topcoat 响应式组件。
- 把 gRPC Payload 当作普通浏览器 HTML Response。
- 把规划中的 crate、API、测试或性能结果描述为已经实现。

## 架构

```text
模板 + Model + Locale + 渲染选项
                    │
                    ▼
┌──────────────────────────────────────────────────────────────┐
│                   thymeleaf 中立引擎                         │
│ Resolver → Parser → TemplateModel → Processor → Renderer    │
└──────────────────────────────┬───────────────────────────────┘
                               │
                    RenderedTemplate
                    Full(Bytes) / Stream(Frame<Bytes>)
                               │
              ┌────────────────┴────────────────┐
              ▼                                 ▼
       独立框架适配器                   可选 thymeleaf-vernal
       thymeleaf-{framework}            bridge
              │                                 │
              ▼                                 ▼
       框架原生 Response                 vernal-{framework}
```

Core 依赖规则：

```text
thymeleaf crate ← 中立合同 ← 整合 crate

禁止：
thymeleaf crate → 框架整合
thymeleaf crate → Vernal
```

详细设计见[可行性与架构提案](docs/Thymeleaf-Rust-可行性与架构设计.md)。

## 命名与发布合同

| 层级 | 名称 | 状态 |
|:---|:---|:---:|
| 项目与 Git 仓库 | `thymeleaf-rust` | 已确认 |
| 未来 crates.io 核心 | `thymeleaf` | Workspace 已创建，尚未发布 |
| Rust 公共路径 | `thymeleaf::...` | Foundation API 已实现 |
| 整合 crate | `thymeleaf-{framework}` | 规划中 |
| 可选 Vernal 整合 | `thymeleaf-vernal` | 规划中 |

明确排除 `thymeleaf-rust-core`、`thymeleaf-rust-axum` 和 Rust 根模块 `thymeleaf_rust` 等名称。

## Crate 模型

引擎规划为一个内聚的核心 crate。Parser 模式、表达式处理、Standard Dialect、中立 Web 输出和测试支持均作为 `thymeleaf` 的内部模块或测试基础设施存在，不拆分为独立发布的 crate。

| 规划核心 crate | 职责 |
|:---|:---|
| `thymeleaf` | Engine、Context、TemplateModel、各 Parser 模式、表达式求值、Standard Dialect 与 `th:*` Processor、中立渲染输出、稳定公共 API 和核心测试基础设施 |

其余 crate 均为整合 crate：`thymeleaf-{framework}` 将 `thymeleaf` 的中立输出适配到单个宿主框架，`thymeleaf-vernal` 提供可选 Vernal 整合，其名称与 `thymeleaf-spring` 一样遵循“核心在前”的命名约定。整合 crate 必须保持薄层，禁止复制解析或渲染逻辑。

## 集成模型

每个目标框架均规划支持独立使用和可选 Vernal 组合两种方式。

| 宿主 | 独立适配器 | Vernal 组合 | 预期输出 |
|:---|:---|:---|:---|
| Topcoat | `thymeleaf-topcoat` | `thymeleaf-vernal` + `vernal-topcoat` | View、Page、Fragment、受控 RawHtml |
| Actix Web | `thymeleaf-actix-web` | `thymeleaf-vernal` + `vernal-actix-web` | Responder、MessageBody、Stream |
| Axum | `thymeleaf-axum` | `thymeleaf-vernal` + `vernal-axum` | IntoResponse 与 Body |
| Gotham | `thymeleaf-gotham` | `thymeleaf-vernal` + `vernal-gotham` | Handler 与 Response |
| Hyper | `thymeleaf-hyper` | `thymeleaf-vernal` + `vernal-hyper` | 标准 HTTP Response/Body |
| Ntex | `thymeleaf-ntex` | `thymeleaf-vernal` + `vernal-ntex` | Responder、Service、Body |
| Poem | `thymeleaf-poem` | `thymeleaf-vernal` + `vernal-poem` | IntoResponse、Endpoint、Stream |
| Rocket | `thymeleaf-rocket` | `thymeleaf-vernal` + `vernal-rocket` | Responder 与 ByteStream |
| Salvo | `thymeleaf-salvo` | `thymeleaf-vernal` + `vernal-salvo` | Handler 与 Response Body |
| Tide | `thymeleaf-tide` | `thymeleaf-vernal` + `vernal-tide` | Endpoint 与 Response |
| Warp | `thymeleaf-warp` | `thymeleaf-vernal` + `vernal-warp` | Reply 与 Rejection 映射 |
| Tower | `thymeleaf-tower` | `thymeleaf-vernal` + `vernal-tower` | Service、Layer、Response Body |
| Tonic | `thymeleaf-tonic` | `thymeleaf-vernal` + `vernal-tonic` | 动态 String/Bytes、Gateway、服务内容 |

发布顺序可以分阶段推进，但架构不得要求独立用户依赖 Vernal。

## 项目状态

| 交付物 | 状态 | 证据 |
|:---|:---:|:---|
| 可行性与架构提案 | 已有 | [`docs/Thymeleaf-Rust-可行性与架构设计.md`](docs/Thymeleaf-Rust-可行性与架构设计.md) |
| 命名与中立性决策 | 已记录 | 架构提案 ADR |
| Cargo Workspace | 已有 | [`Cargo.toml`](Cargo.toml) |
| Rust 公共 API | 已验证切片 | Foundation/配置 API、缓存对象族、`StandardCache`、`TemplateResolution`、模板资源 SPI/字符串资源/文件资源、标准表达式字面量/执行上下文/转换服务/NO-OP/Token 字符语义、`EvaluationUtils`/`Bools`、聚合/数组/List/Set/Map/Object facade、模式、版本、日志与内容类型工具；URL 与 JVM 软引用运行时边界仍待补齐 |
| 框架适配器 | 规划中 | 不存在适配器 Manifest 或代码 |
| 上游兼容矩阵 | 实施中 | 已登记 491 个对象、4,291 个方法和 6,936 个参数 |
| 迁移治理 | 已自动化 | `cargo xtask migration-check` 校验基线、清单、布局、来源注释和红线 |
| 测试与 CI | 切片门禁通过 | 183 个单元测试、26 个共 2,496 条记录的 Java/Rust Golden 测试、行/函数/区域覆盖率均为 100% |
| crates.io 包 | 未发布 | `thymeleaf` 仍是规划发布名 |

## 文档快速开始

渲染引擎尚不可执行。验证当前已实现的 S1/S2/S5 垂直切片：

```bash
git clone --branch dev https://github.com/easy-4-rust/thymeleaf-rust.git
cd thymeleaf-rust
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features \
  --fail-under-lines 100 \
  --fail-under-functions 100 \
  --fail-under-regions 100 \
  --summary-only
```

然后阅读：

- [可行性与架构提案](docs/Thymeleaf-Rust-可行性与架构设计.md)
- [迁移路线图](docs/migration/迁移路线图.md)
- [对象级对照表](docs/migration/对象级对照表.md)
- [方法级对照表](docs/migration/方法级对照表.md)
- [语义迁移对照表](docs/migration/语义迁移对照表.md)
- [对象名称一致性检查](docs/migration/对象名称一致性检查.md)
- [English README](README.md)

## 兼容方向

初始设计以 Thymeleaf 3.1 系列的核心语义为目标。只有在固定并验证具体上游版本、Processor、表达式行为、错误语义、排除项和统计口径后，才会发布兼容性结论和百分比。

当前已经具备固定 Java API 清单和 Foundation Golden 差分测试。后续规划证据包括：

- 公共 API 与 Processor 清单；
- Java/Rust Golden Output 对比；
- Fragment、转义、URL、Locale 与错误 Parity Test；
- 面向非法和边界输入的 Differential Test；
- 包含迁移建议的显式差异登记。

Thymeleaf 上游采用 Apache License 2.0。任何从上游调整而来的源码、测试或 Fixture 都必须保留适用的版权、许可证、NOTICE、署名和修改说明。

## 路线图

| 阶段 | 规划交付物 | 退出条件 |
|:---|:---|:---|
| Phase 0 | 中立合同与 Parser/Body 原型 | Model 重放以及 Full/Stream 输出得到验证 |
| Phase 1 | HTML 与 Standard Dialect MVP | 一个真实模板路径端到端运行 |
| Phase 2 | 独立框架适配器 | 每个适配器具备 Full/Stream/Error/Cancellation 测试 |
| Phase 3 | `thymeleaf-vernal` bridge | 所有相关 `vernal-*` 宿主消费同一个 Engine |
| Phase 4 | 更多模式与 Dialect SPI | 发布版本化能力矩阵 |
| Phase 5 | 兼容与发布准备 | Package、文档、安全和 Parity 门禁通过 |

在实施容量和依赖选型确认前，路线图不提供时间承诺。

## 设计草图：未来 API

以下代码是当前不可运行的设计草图，名称和签名可能变化。

```rust
use thymeleaf::{Context, TemplateEngine};

fn render(engine: &TemplateEngine) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = Context::new();
    context.set("name", "Rust");
    Ok(engine.render("home", &context)?)
}
```

在 README 明确标记 crate 已发布之前，请勿执行 `cargo add thymeleaf`。

## 参与贡献

项目当前欢迎以下方面的设计评审：

- HTML Parser 保真与 SourceSpan；
- 不可变 Event/Model 表示；
- 表达式安全与 Rust 值访问；
- Processor 与 Dialect 合同；
- Full/Stream 输出语义；
- 框架适配边界；
- 上游兼容与测试方法。

进入实现阶段后，变更应包含文档、测试、兼容影响和清晰的依赖方向检查。

## 安全

当前仓库还没有可执行 Parser 或渲染 Runtime。在接受这些层之前，项目将明确输入大小、递归、表达式求值、模板解析、输出大小、取消和未转义内容等限制。

请勿在公开 Issue 中披露疑似漏洞或敏感验证数据。项目会在发布可执行版本前确定私密报告渠道。

## 许可证

本仓库采用 [MIT License](LICENSE)。

上游衍生材料仍受其原许可证和署名要求约束。

---

<div align="center">

[返回顶部](#readme-top) · [架构文档](docs/Thymeleaf-Rust-可行性与架构设计.md) ·
[Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
