# Thymeleaf-Rust 迁移技术要求

> **用途**：规定从 Thymeleaf Java 到 `thymeleaf` Rust 核心 crate 的实现、测试、
> 证据和验收边界。本文件是迁移变更的强制门禁，不是架构愿景。
>
> **固定上游**：Thymeleaf `3.1.5.RELEASE`，
> `10f9dd2eb8cbd98515ce14b149d115e0287d0add`
>
> **最后更新**：2026-07-30

## 1. 目标与非目标

迁移目标是功能语义完全迁移：相同输入、状态和调用顺序应产生相同输出、事件、
副作用与错误类别。不能用“Rust 更惯用”为理由删除 Java 可观察行为。

`thymeleaf-rust` 是中立项目，发布 crate 名为 `thymeleaf`。核心不得依赖某个 Web
框架或 Vernal；`thymeleaf-axum`、`thymeleaf-actix-web`、`thymeleaf-vernal` 等
整合 crate 只负责宿主请求/响应和生命周期适配。

下列结果不能单独证明迁移完成：

- Rust 文件或同名类型已经存在；
- 可以编译；
- 有较高或 100% Rust 覆盖率；
- 用 Tera、Askama 等替代实现得到相似页面；
- 只通过手工挑选的 happy path。

## 2. 权威基线与证据等级

每次验证必须记录上游 commit，不允许对浮动分支做兼容性声明。证据优先级为：

| 等级 | 证据 | 可支持的结论 |
|:---:|:---|:---|
| V0 | 文件/类型存在 | 只能说明形态存在 |
| V1 | `cargo check`、布局审计 | 可编译、无显式 STUB |
| V2 | Rust 单元测试 | Rust 内部合同成立 |
| V3 | 固定 Java Golden 与 Rust 逐记录差分 | 已覆盖输入上的跨语言行为一致 |
| V4 | 上游 JUnit/参数化 case 逐项处置 | 上游测试语义没有静默遗漏 |
| V5 | `.thtest` 逐项迁移 | 模板级兼容证据 |
| V6 | 调用链、动态分派与副作用审计 | 对象在真实链路中的语义成立 |
| V7 | 全量门禁、性能/并发/模糊测试 | 发布候选级证据 |

当前两个仓库均不存在 `.codegraph/`。设计阶段的 CodeGraph 统计只能作为历史证据；
本轮使用固定源码、调用方检查和机器清单，不能声称已获得动态分派图证据。索引恢复后，
必须补审 trait/接口分派、反射/SPI 和回调路径。

## 3. 对象、文件与名称

1. 目录、文件、方法和参数使用 `snake_case`，类型使用 `PascalCase`。
2. 一个 Java 主对象对应一个包含真实逻辑的 `.rs` 文件。
3. Java 内部类、内部 enum 和紧耦合 Builder 可与主对象同文件。
4. Java 子包按约定映射到最后一级 Rust 目录。
5. `mod.rs` 和 `lib.rs` 只做模块声明、文档和重导出。
6. 禁止以 `compat.rs`、大而全的 `lib.rs` 或重导出空文件伪造对象覆盖。
7. 生产代码禁止 wildcard import。
8. 不允许 `todo!()`、`unimplemented!()`、空函数体或无依据的默认返回值。
9. Java 主对象、方法、重载和内部对象必须能从对象级、方法级清单追踪到 Rust 落点。

## 4. 注释与 API

- 每个迁移对象必须有中文 `///` 文档并标注 Java 全限定名。
- 每个 `pub`/`pub(crate)` 方法必须说明参数、返回值、错误和 Java 来源。
- Java Javadoc 的校验顺序、状态、副作用和线程安全语义必须翻译保留。
- 私有复杂算法要用中文行内注释说明不明显的不变量，不能逐行复述代码。
- Java overload 可以映射为不同 Rust 方法或参数对象，但每个入口必须在方法表单独登记。

## 5. 语义映射

### 5.1 值与文本

- Java `char`、`char[]`、`String.length`、substring offset 和哈希均以 UTF-16 code
  unit 为准；不能直接替换为 Rust UTF-8 字节或 Unicode scalar 数量。
- `null`、空值、未指定和缺省值必须分别建模，通常使用 `Option<T>`。
- Java 集合的顺序、身份、可变性、防御性复制和重复元素规则必须显式保留。
- Java `int`/`long` 的溢出位置使用 wrapping 运算，不得由 debug overflow panic 改写语义。

### 5.2 错误

- checked exception 映射为 `Result<T, E>`；原始错误类别、消息、cause、行列和发生顺序必须保留。
- JVM RuntimeException 可用类型化 error/panic 边界表达，但不能合并不同 Java 类别。
- Java Error 对应的未知 Rust panic 必须在 finally/清理语义完成后继续传播。
- 对每个错误入口同时验证：错误类型、消息、cause、校验顺序、已发生副作用和清理。

### 5.3 并发与资源

- `synchronized`/并发 Map 按共享可变性选择 `Mutex`、`RwLock`、`DashMap` 或原子类型。
- 不能因 Rust 锁中毒而增加 Java 不存在的永久失败；若选择恢复，必须有测试。
- Reader/Writer、缓存、缓冲池和网络资源要覆盖成功、部分读取、零读取、失败和关闭。
- 异步只在原语义或宿主整合需要时引入，不能把同步核心强制绑定到 Tokio。

## 6. 测试三台账

生产语义迁移完成后，必须一次性生成并处置三类测试台账，格式见
[迁移测试对照表](迁移测试对照表.md)：

| 台账 | 含义 | 必填追踪 |
|:---|:---|:---|
| `SOURCE_PARITY` | 上游已有 JUnit、参数化行、动态测试或 `.thtest` | Java 测试/case → Rust 测试或明确处置 |
| `RUST_OBLIGATION` | 由语言/运行时映射新增的必要合同 | 映射规则 → Rust 测试 |
| `VALUE_ADD` | 超出上游但能杀死高风险错误的测试 | 风险/变异 → Rust 测试 |

处置状态只能是：

- `MAPPED`：一对一迁移；
- `SPLIT`：一个源测试拆为多个 Rust 测试；
- `MERGED`：多个源 case 由同一参数化/Golden 测试覆盖；
- `NOT_APPLICABLE`：必须有可复核的技术理由；
- `MISSING`：尚未迁移，不能算完成。

静态审计当前识别 875 个 Java 测试方法/注解和 147 个需要展开复核的参数化/动态
候选；2,609 个 `.thtest` 单独计数。完整原始枚举见
[`baseline/migration_test_static_inventory.json`](baseline/migration_test_static_inventory.json)。
静态扫描只是发现工具，不等于 case 已逐项处置。

### 6.1 批量迁移与统一验证顺序

迁移执行顺序是强制的：

1. 冻结 Java 对象、方法、参数、内部对象、动态入口和测试资产清单；
2. 按 Engine、Parser、Expression、Processor、Web/Host 等语义域批量完成全部生产
   逻辑；
3. 用静态审计确认每个 Java 方法具有显式方法、动态调用、Rust 惯用等价、
   trait/闭包/流程合并或私有合并落点；
4. 完成全部 `thymeleaf-{framework}` 和 `thymeleaf-vernal` 生产适配；
5. 最后统一执行 Java Golden、Rust 测试、2,609 个 `.thtest`、覆盖率、模糊测试、
   性能和发布门禁；
6. 根据统一证据批量回填对象、方法和语义状态。

禁止把“迁移一个对象 → 写一组测试 → 更新一次状态”作为主执行循环。生产迁移期间
可以运行 `cargo fmt`、`cargo check`、清单审计和 STUB/命名红线扫描，以防止批量
改动失去基本可编译性；这些快速反馈不构成行为验证。

## 7. Golden 与测试价值

Golden harness 必须：

1. 编译固定上游源码或固定测试模块；
2. 输出稳定、可读、可逐记录比较的结果；
3. 记录异常类、消息、cause、位置、状态和必要的身份/顺序；
4. 覆盖重载、空值、边界、错误和副作用；
5. 提供可重复生成脚本；
6. 将 fixture 提交到仓库，CI 不依赖本地未固定环境；
7. 在文档登记组数、记录数和生成命令。

每个新增测试应能说明它会杀死什么错误实现。只执行构造器、只断言 `is_ok()`、
只做快照但不校验关键字段、或重复已有 happy path 的测试不计作充分证据。

## 8. 覆盖率与门禁

覆盖率是 Rust 实现内部“是否执行”的证据，不是 Java 语义一致性的替代品。当前项目
要求全工作区行、函数和 region 均为 100%，且不允许用忽略目标文件掩盖缺口：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features \
  --fail-under-lines 100 \
  --fail-under-functions 100 \
  --fail-under-regions 100 \
  --summary-only
```

迁移工具还必须通过：

```bash
cargo fmt --manifest-path xtask/Cargo.toml --all --check
cargo clippy --manifest-path xtask/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path xtask/Cargo.toml
cargo xtask migration-check \
  --upstream /absolute/path/to/thymeleaf \
  --baseline 10f9dd2eb8cbd98515ce14b149d115e0287d0add \
  --json target/migration-check.json
```

## 9. 状态更新协议

对象只能按下列顺序晋级；S1–S10 批量生产迁移期间冻结现有验证状态，S11 统一验证
后再按证据批量晋级：

```mermaid
flowchart LR
    A["NOT_STARTED"] --> B["真实实现"]
    B --> C["IMPLEMENTED_UNVERIFIED"]
    C --> D["SOURCE_PARITY 已处置"]
    D --> E["Golden / 合同差分通过"]
    E --> F["全量门禁通过"]
    F --> G["BEHAVIOR_VERIFIED"]
```

统一验证批次结案时必须同步：

1. 对象级对照表；
2. 方法级对照表；
3. 语义迁移对照表；
4. 对象名称一致性检查；
5. 迁移测试对照表；
6. 迁移路线图和 README 状态数字。

任何一个数字必须能从机器清单、测试输出或固定 fixture 重算。发现文档互相冲突时，
先降级不可靠声明，再以机器证据更新，不能选择对项目更有利的数字。

## 10. 完成定义

单对象 `BEHAVIOR_VERIFIED` 至少要求：

- 对象与全部内部对象均有真实实现；
- 全部 Java 方法、构造器、重载和可观察私有算法已登记；
- 上游相关测试/case 已在 `SOURCE_PARITY` 处置；
- Rust 运行时映射风险已在 `RUST_OBLIGATION` 覆盖；
- Java Golden 或可证明等价的差分证据通过；
- 全工作区编译、lint、测试、迁移检查和 100% 覆盖率通过；
- 文档状态与机器报告一致。

项目完成还要求 491 个主对象、69 个内部类型和 2,609 个 `.thtest` 全部完成处置。
局部切片通过不得写成“Thymeleaf 已完全兼容”。
