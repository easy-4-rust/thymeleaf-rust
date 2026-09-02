# 性能基准（criterion）

> 本文件记录 thymeleaf-rust 渲染引擎的性能基线与 drift 记录。
> 基建：`thymeleaf/benches/render_baseline.rs`（criterion，harness=false）。
> 运行：`cargo bench -p thymeleaf --bench render_baseline`（单基准过滤：
> `-- full_document` 位置参数）。
> 测量纪律：基线数据须在**无并发负载**时采集（llvm-cov/测试并行跑会引入
> ±20% 噪声——2026-08-16 首采教训）。

## 1. 基线（2026-09-03 首采）

环境：Apple M 系列（darwin arm64）/ stable rustc / release bench profile /
引擎与 Context 复用（缓存命中稳态）。

| 基准 | 中位耗时 | 吞吐 | 说明 |
|------|---------:|-----:|------|
| `render_simple_variable/single_interpolation` | **10.18 µs** | 2.44 MiB/s | 单变量插值（解析缓存命中） |
| `render_each_100/list_iteration` | **145.02 µs** | 397 KiB/s | th:each 100 行迭代 |
| `render_full_document/mixed_processors` | **225.54 µs** | 1.556 MiB/s | 混合文档（if/each/text/utext/attr + 表达式链） |

## 2. Drift 记录

| 日期 | 基准 | 变化 | 提交/说明 |
|------|------|------|----------|
| 2026-09-03 | 全部 | 首采基线 | — |

## 3. 已知热点（待优化验证，未动代码）

- `Utf16String` 转换密度：native 表达式求值器单文件 44 处 `from_*` 构造；
- `Arc<TemplateValue>` 克隆：值传递链（上下文 → 表达式 → 输出）；
- 上述为图谱 + 代码证据的假设，优化须先以 flamegraph 验证，且不得改变输出字节
  （行为零变更为最高约束，见版本治理）。

## 4. 变更记录

| 日期 | 变更 |
|------|------|
| 2026-09-03 | 初版——criterion 三基准首采（S11「性能」组件基建落地） |
