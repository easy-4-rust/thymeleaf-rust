<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**A framework-neutral Thymeleaf-compatible dynamic template engine for Rust**

[![Build](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange)](#3-rust-baseline)
[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

[English](./README.md) · [简体中文](./README.zh-CN.md)

[Overview](#1-overview) · [Maturity](#2-maturity) · [Workspace](#4-workspace-layout) ·
[Quick Start](#6-quick-start) · [Quality](#8-quality-gates) · [Roadmap](#10-roadmap)

</div>

---

> **Version**: `0.2.0` · **MSRV**: Rust 1.95 · **Edition**: 2024 · **Resolver**: 3
>
> [crates.io](https://crates.io/crates/thymeleaf/0.2.0) · [docs.rs](https://docs.rs/thymeleaf/0.2.0/thymeleaf/) · [GitHub Release](https://github.com/easy-4-rust/thymeleaf-rust/releases/tag/v0.2.0)
>
> **Upstream baseline**: Thymeleaf `3.1.5.RELEASE` @ `10f9dd2e`

## 1. Overview

thymeleaf-rust is a behavioral port of Java Thymeleaf 3.1.5 to Rust. It parses and
renders HTML/XML/TEXT templates with expression evaluation, dialect processors,
caching, and a framework-neutral web contract. The project does not depend on the
JVM at runtime — the supported template semantics run in Rust.

### 1.1 Status Evidence

| Claim | Value | Evidence |
|:---|:---|:---|
| Semantic parity corpus | 2,608 `.thtest` cases, 2,595 verified by the differential test | [Corpus test](./thymeleaf-test/tests/thtest_upstream_plain_batch.rs) |
| Object-level coverage | 491 main objects, 4,291 methods, 0 missing | `cargo xtask migration-check` |
| Source-parity ledger | 413 core Java test methods and 1,154 runtime cases | [Source parity inventory](./docs/migration/baseline/source_parity_inventory.json) |
| Acceptance gate | Registered test assets checked against SHA-256 | [Acceptance test](./thymeleaf-test/tests/acceptance.rs) |
| CI platforms | ubuntu-latest, macos-latest | GitHub Actions matrix |
| unsafe | `forbid` in the core and adapter crates | `[lints.rust] unsafe_code = "forbid"` |

### 1.2 Non-Goals

- No JVM/bytecode interop at runtime (Java is only the behavior oracle).
- No Spring/JSP/Servlet runtime integration (Java-only modules excluded by policy).
- No 1:1 internal implementation copy — Java idioms are mapped to Rust ownership/trait/error patterns.

## 2. Maturity

### 2.1 Feature Matrix

| Feature | Status | Crate | Limitation |
|:---|:---:|:---|:---|
| HTML/XML/TEXT template parsing | ✅ Stable | `thymeleaf` | Parser and rendering coverage varies by mode |
| Expression evaluation (OGNL subset) | ✅ Stable | `thymeleaf` | No JVM reflection; ACL-gated static method whitelist |
| Standard dialect processors | ✅ Stable | `thymeleaf` | 2,595 cases passed semantic comparison |
| Template cache & resolvers | ✅ Stable | `thymeleaf` | String/File/Class/URL/Multi/ByteArray loaders |
| Auto-escaping & output formats | ✅ Stable | `thymeleaf` | HTML/XML/JavaScript/CSS/JSON/RTF/PlainText |
| Decoupled template logic | ✅ Stable | `thymeleaf` | `.th.xml` sidecar |
| Optional DTD validation (XML mode) | 🧪 Preview | `thymeleaf` (`dtd-validation` feature) | Embedded W3C XHTML DTDs; `Disabled`/`Warn`/`Strict` policy, default `Disabled`; internal-subset DOCTYPEs skip validation |
| Framework-neutral web contract | ✅ Stable | `thymeleaf` | IWebExchange / IWebRequest / IWebSession |
| Framework adapters (P0) | ✅ Stable | `thymeleaf-axum`/`thymeleaf-actix-web`/`thymeleaf-topcoat` | Full IWebExchange suite + contract tests aligned with hyper benchmark + `render_async` |
| Framework adapters (thin) | 🧪 Preview | hyper/tower/ntex/poem/rocket/salvo/tonic | Response-level adapters; IWebExchange via hyper benchmark |
| Framework adapters (experimental) | 🧪 Experimental | gotham/warp | Upstream stalled/replaced; best-effort, tests behind `full` feature |
| Tide adapter (source-only) | 🧪 Experimental | `thymeleaf-tide` | Excluded from workspace CI/security graph: Tide and async-std are discontinued; `publish = false`; retained for reference only |
| sa-token security dialect | 🧪 Preview | `thymeleaf-sa-token` | Authorization-aware template processors |
| Vernal view layer | 🧪 Preview | `thymeleaf-vernal` | `ThymeleafView` renders to a Vernal response. `ThymeleafViewResolver` resolves views. |
| Fuzz (property tests) | 🚧 Partial | `thymeleaf-test` | XML and TEXT parser property tests |

### 2.2 Upstream Compatibility

| Dimension | Scope | Method |
|:---|:---|:---|
| Behavioral | 2,595 of 2,608 executable cases pass the differential test | Corpus differential |
| Policy differences | 13 cases (12 `execinfo` upstream-disabled + 1 arbitrary reflection chain) | Named disposition |
| Source parity | 413 core Java test methods (Spring excluded) | [Source parity inventory](./docs/migration/baseline/source_parity_inventory.json) |
| Object parity | 491 / 491 main objects, 4,291 / 4,291 methods | `migration-check` |

## 3. Rust Baseline

| Item | Value |
|:---|:---|
| MSRV | Core 1.95, Topcoat adapter 1.99 |
| Edition | 2024 |
| Resolver | 3 |
| Clippy | `-D warnings` |
| rustfmt | stable |
| unsafe | `forbid` (core and adapters) |
| missing_docs | `deny` (`thymeleaf` crate) |

## 4. Workspace Layout

```text
[Downstream crate]
        │ cargo add thymeleaf
        ▼
┌──────────────────────────────────────────────────────────┐
│ thymeleaf-rust Workspace                                 │
│                                                          │
│ thymeleaf               Core engine, public API, web     │
│ thymeleaf-test          Java parity corpus, golden tests │
│ thymeleaf-examples      GTVG sample port                 │
│ thymeleaf-support/*     14 workspace adapters            │
│   ├── thymeleaf-actix-web   thymeleaf-axum               │
│   ├── thymeleaf-hyper       thymeleaf-rocket              │
│   ├── thymeleaf-sa-token    thymeleaf-salvo  ...          │
├──────────────────────────────────────────────────────────┤
│ xtask                   migration-check tool             │
│ scripts/                golden regeneration, audit        │
│ docs/                   migration docs, release policy    │
└──────────────────────────────────────────────────────────┘
```

### Crate Map

| Crate | crates.io in v0.2.0 | Role |
|:---|:---:|:---|
| `thymeleaf` | [crates.io](https://crates.io/crates/thymeleaf/0.2.0) | Core engine |
| `thymeleaf-actix-web` | Source | Actix-web adapter |
| `thymeleaf-axum` | Source | Axum adapter |
| `thymeleaf-gotham` | Source | Gotham adapter |
| `thymeleaf-hyper` | Source | Hyper adapter |
| `thymeleaf-ntex` | Source | Ntex adapter |
| `thymeleaf-poem` | Source | Poem adapter |
| `thymeleaf-rocket` | Source | Rocket adapter |
| `thymeleaf-sa-token` | Source | Sa-Token security dialect |
| `thymeleaf-salvo` | Source | Salvo adapter |
| `thymeleaf-tonic` | Source | Tonic adapter |
| `thymeleaf-topcoat` | Source | Topcoat adapter |
| `thymeleaf-tower` | Source | Tower adapter |
| `thymeleaf-warp` | Source | Warp adapter |
| `thymeleaf-tide` | Unpublished | Tide adapter (excluded from workspace; unmaintained upstream) |
| `thymeleaf-vernal` | Unpublished | Vernal adapter (Git dependencies) |
| `thymeleaf-test` | Unpublished | Test harness (internal) |
| `thymeleaf-examples` | Unpublished | Examples (internal) |

## 5. Security Model

Restricted expression contexts can block external access. Static class and method access follows the policies enforced by `ThymeleafACLClassResolver` and `OgnlRuntime`. Use trusted templates when rendering application data.

[Expression context policies](./thymeleaf/src/expression/standard_expression_execution_context.rs)

## 6. Quick Start

### From crates.io (recommended)

Add the dependency to `Cargo.toml`. Put the example in `src/main.rs` and run `cargo run`.

```toml
[dependencies]
thymeleaf = "0.2.0"
```

0.2.0 keeps the template expression objects `#dates` and `#temporals`. Rust callers that construct date or time values use Jiff types. See the [time API migration guide](./docs/superpowers/specs/2026-11-18-chrono-migration.md).

### From Git (development snapshots)

```toml
[dependencies]
thymeleaf = { git = "https://github.com/easy-4-rust/thymeleaf-rust.git" }
```

### Minimal Example

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

Expected output `<p>Rust</p>`.

Enable the `dtd-validation` Cargo feature for XML DTD validation. Before rendering, call `TemplateEngine::set_dtd_validation_policy` with `thymeleaf::dtd::ValidationPolicy::Warn` or `Strict`.

## 7. Java → Rust Semantic Mapping

| Java | Rust | Reason |
|:---|:---|:---|
| Checked exceptions | `Result<T, E>` + `thiserror` enums | Explicit error propagation |
| `null` | `Option<T>` | Null safety |
| `synchronized` / `ConcurrentHashMap` | `Arc<RwLock<_>>` / `DashMap`-style | Ownership-based concurrency |
| Reflection / `Class.forName` | `ThymeleafACLClassResolver` + `OgnlRuntime` trait | No dynamic class loading; ACL-gated |
| Inner classes | Same-file types (audit-approved type families) | Rust module conventions |
| `ExecutorService` | `futures` + synchronous core | Core is sync; async at adapter layer |

## 8. Quality Gates

Run from the repository root using a Rust toolchain that satisfies the complete workspace, including the Topcoat adapter's Rust 1.99 requirement.

```bash
cargo fmt --all --check
python3 scripts/generate_test_asset_manifest.py --output thymeleaf-test/source-test-parity.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Check out the pinned Java baseline at `upstream-thymeleaf` to reproduce the semantic corpus.

```bash
THYMELEAF_UPSTREAM="$PWD/upstream-thymeleaf" THYMELEAF_SCOPE=semantic_all \
  cargo test -p thymeleaf-test --test thtest_upstream_plain_batch
```

The [CI workflow](./.github/workflows/ci.yml) records the upstream checkout, Java regression and migration audit commands. See also [coverage](./.github/workflows/coverage.yml), [security](./.github/workflows/security.yml) and [release preflight](./.github/workflows/release-preflight.yml).

## 9. Known Limitations

- **Fuzz coverage**. The workspace [property tests](./thymeleaf-test/tests/robustness_fuzz_smoke.rs) exercise XML and TEXT parsing. The scheduled [fuzz workflow](./.github/workflows/fuzz.yml) exercises HTML rendering and expression evaluation.
- **API baseline CI**: `cargo public-api` baseline is a hard CI gate on pinned nightly (`nightly-2026-07-28`); any public-API drift fails CI.

## 10. Roadmap

| Phase | Status | Items |
|:---|:---:|:---|
| Semantic alignment | ✅ Done | 2,608 corpus, 491 objects, 4,291 methods |
| Governance audit | ✅ Done | strict blockers 0, warnings 0, CI enforced |
| Fuzz OOM fix | ✅ Done | DiscardingWriter + shrink clamp + serial |
| Core v0.2.0 | ✅ Published | [crates.io](https://crates.io/crates/thymeleaf/0.2.0) · [docs.rs](https://docs.rs/thymeleaf/0.2.0/thymeleaf/) · [GitHub Release](https://github.com/easy-4-rust/thymeleaf-rust/releases/tag/v0.2.0) |
| Render benchmark | ✅ Running | [Criterion regression](./.github/workflows/bench-regression.yml) |
| Adapter acceptance | 🚧 In progress | Expand host-level streaming, error and cancellation tests |

## 11. Contributing

Run the [quality checks](#8-quality-gates) before submitting.

New public APIs must include docs, tests, and SemVer/MSRV impact notes.

## 12. License

[MIT](./LICENSE)

This project ports behavior from [Thymeleaf](https://www.thymeleaf.org/) (Apache 2.0).
Upstream license, source commit, and modification scope are documented in `docs/`.

---

<div align="center">

[Back to top](#readme-top) · [Actions](https://github.com/easy-4-rust/thymeleaf-rust/actions) · [Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
