# thymeleaf-rust

Thymeleaf 3.1.5.RELEASE（Java）的 Rust 移植，工作区采用多 crate 布局（参考 freemarker-rust）：

| 目录 | 角色 | 发布 |
|---|---|---|
| [`thymeleaf/`](./thymeleaf/) | 主 crate：框架无关的 Thymeleaf 兼容模板引擎（README 见 `thymeleaf/README.md`） | ✅ |
| [`thymeleaf-examples/`](./thymeleaf-examples/) | Java `examples/core`（GTVG）示例移植 | ❌ |
| [`thymeleaf-test/`](./thymeleaf-test/) | Java `tests` + `lib/testing` 的差分验收：2608 例语料 + 对象级 parity + source-test 门禁 | ❌ |
| [`integrations/`](./integrations/) | 14 个 web 框架适配 crate（axum/actix/hyper/rocket/…） | ✅ |
| [`docs/`](./docs/) | 迁移文档（可行性设计、对象级对照表、迁移测试对照表等） | — |
| [`scripts/`](./scripts/) | 黄金文件再生成脚本与台账生成脚本 | — |
| [`xtask/`](./xtask/) | `migration-check` 门禁工具（`cargo xtask migration-check`） | — |

## 常用命令

```sh
cargo build --workspace            # 构建全部
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask migration-check --upstream <Java 上游 checkout> --baseline 10f9dd2eb8cbd98515ce14b149d115e0287d0add
THYMELEAF_UPSTREAM=<Java 上游 checkout> THYMELEAF_SCOPE=semantic_all \
  cargo test -p thymeleaf-test --test thtest_upstream_plain_batch   # 2608 例语料
cargo llvm-cov --workspace --all-features --summary-only            # 覆盖率
```

详细说明见 [`thymeleaf/README.md`](./thymeleaf/README.md) 与 [`docs/`](./docs/)。
