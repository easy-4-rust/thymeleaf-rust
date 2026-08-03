<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**A framework-neutral Thymeleaf-compatible dynamic template engine for Rust**

[![Build](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/easy-4-rust/thymeleaf-rust/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange)](#3-rust-baseline)
[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

[English](./README.md) · [简体中文](./README.zh-CN.md)

[Overview](#1-overview) · [Maturity](#2-maturity) · [Workspace](#4-workspace-layout) ·
[Quick Start](#6-quick-start) · [Quality](#13-quality-gates) · [Roadmap](#16-roadmap)

</div>

---

> **Version**: `0.1.0-alpha.1` · **MSRV**: Rust 1.95 · **Edition**: 2024 · **Resolver**: 3
>
> **Upstream baseline**: Thymeleaf `3.1.5.RELEASE` @ `10f9dd2e`

## 1. Overview

thymeleaf-rust is a behavioral port of Java Thymeleaf 3.1.5 to Rust. It parses and
renders HTML/XML/TEXT templates with expression evaluation, dialect processors,
caching, and a framework-neutral web contract. The project does not depend on the
JVM at runtime — all Java semantics are re-implemented in pure Rust.

### 1.1 Status Evidence

| Claim | Value | Evidence |
|:---|:---|:---|
| Semantic parity corpus | 2,608 `.thtest` cases, 2,595 behaviorally identical | CI `thtest_upstream_plain_batch` |
| Object-level coverage | 491 main objects, 4,291 methods, 0 missing | `cargo xtask migration-check` |
| Source-parity ledger | 413 core test entries (Spring excluded by policy) | `source_parity_inventory.json` |
| Acceptance gate | 2,686 assets SHA-256 verified | `thymeleaf-test/tests/acceptance.rs` |
| Tests | 295 (lib) + 964 (integration) + 45 (adapters) | `cargo test --workspace` |
| CI platforms | ubuntu-latest, macos-latest | GitHub Actions matrix |
| unsafe | `forbid` across all crates | `[lints.rust] unsafe_code = "forbid"` |

### 1.2 Non-Goals

- No JVM/bytecode interop at runtime (Java is only the behavior oracle).
- No Spring/JSP/Servlet runtime integration (Java-only modules excluded by policy).
- No 1:1 internal implementation copy — Java idioms are mapped to Rust ownership/trait/error patterns.

## 2. Maturity

### 2.1 Feature Matrix

| Feature | Status | Crate | Limitation |
|:---|:---:|:---|:---|
| HTML/XML/TEXT template parsing | ✅ Stable | `thymeleaf` | html5gum tokenizer has pathological-input memory risk |
| Expression evaluation (OGNL subset) | ✅ Stable | `thymeleaf` | No JVM reflection; ACL-gated static method whitelist |
| Standard dialect processors | ✅ Stable | `thymeleaf` | 2,608 corpus cases verified |
| Template cache & resolvers | ✅ Stable | `thymeleaf` | String/File/Class/URL/Multi/ByteArray loaders |
| Auto-escaping & output formats | ✅ Stable | `thymeleaf` | HTML/XML/JavaScript/CSS/JSON/RTF/PlainText |
| Decoupled template logic | ✅ Stable | `thymeleaf` | `.th.xml` sidecar |
| Framework-neutral web contract | ✅ Stable | `thymeleaf` | IWebExchange / IWebRequest / IWebSession |
| Framework adapters | 🧪 Preview | `thymeleaf-support/*` | 13 publishable + 2 non-published (tide, vernal) |
| sa-token security dialect | 🧪 Preview | `thymeleaf-sa-token` | 12 contract tests |
| Fuzz (property tests) | 🚧 Partial | `thymeleaf-test` | XML/TEXT parser proptest; HTML/render excluded (see Known Limitations) |

### 2.2 Upstream Compatibility

| Dimension | Scope | Method |
|:---|:---|:---|
| Behavioral | 2,595 / 2,608 executable cases match Java byte-for-byte | Corpus differential |
| Policy differences | 13 cases (12 `execinfo` upstream-disabled + 1 arbitrary reflection chain) | Named disposition |
| Source parity | 413 core Java test classes tracked (Spring excluded) | `source_parity_inventory.json` |
| Object parity | 491 / 491 main objects, 4,291 / 4,291 methods | `migration-check` |

## 3. Rust Baseline

| Item | Value |
|:---|:---|
| MSRV | 1.95 |
| Edition | 2024 |
| Resolver | 3 |
| Clippy | `-D warnings` |
| rustfmt | stable |
| unsafe | `forbid` (all crates) |
| missing_docs | `deny` (`thymeleaf` crate) |

## 4. Workspace Layout

```text
[Downstream crate]
        │ cargo add thymeleaf / thymeleaf-<framework>
        ▼
┌──────────────────────────────────────────────────────────┐
│ thymeleaf-rust Workspace                                 │
│                                                          │
│ thymeleaf               Core engine, public API, web     │
│ thymeleaf-test          Java parity corpus, golden tests │
│ thymeleaf-examples      GTVG sample port                 │
│ thymeleaf-support/*     15 framework adapters            │
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

| Crate | Publish | Role |
|:---|:---:|:---|
| `thymeleaf` | ✅ | Core engine |
| `thymeleaf-actix-web` | ✅ | Actix-web adapter |
| `thymeleaf-axum` | ✅ | Axum adapter |
| `thymeleaf-gotham` | ✅ | Gotham adapter |
| `thymeleaf-hyper` | ✅ | Hyper adapter |
| `thymeleaf-ntex` | ✅ | Ntex adapter |
| `thymeleaf-poem` | ✅ | Poem adapter |
| `thymeleaf-rocket` | ✅ | Rocket adapter |
| `thymeleaf-sa-token` | ✅ | Sa-Token security dialect |
| `thymeleaf-salvo` | ✅ | Salvo adapter |
| `thymeleaf-tonic` | ✅ | Tonic adapter |
| `thymeleaf-topcoat` | ✅ | Topcoat adapter |
| `thymeleaf-tower` | ✅ | Tower adapter |
| `thymeleaf-warp` | ✅ | Warp adapter |
| `thymeleaf-tide` | ❌ | Tide adapter (unmaintained upstream) |
| `thymeleaf-vernal` | ❌ | Vernal adapter (git deps, pending crates.io) |
| `thymeleaf-test` | ❌ | Test harness (internal) |
| `thymeleaf-examples` | ❌ | Examples (internal) |

## 5. Security Model

Expression evaluation defaults to a read-only safe subset:

- **`restrict_external_access = true`** by default — `new`, `param`, `@Type@` syntax blocked.
- **Arbitrary classes and reflection blocked** — 10 blocked package prefixes (`java.`/`javax.`/`jakarta.`/`jdk.`/…), 53 allowed classes (wrappers, collections, time, math).
- **Restricted static method whitelist** — `Math.abs/sqrt/…`, `Integer.parseInt`, `LocalDateTime.of`, `String.format` on 9 classes; all others rejected by `ThymeleafACLClassResolver`.
- **`unsafe_code = "forbid"`** across all crates — zero unsafe in workspace source.
- Hosts can further restrict via `OgnlRuntime` (opt-in).

## 6. Quick Start

### From Git (not yet on crates.io)

```toml
[dependencies]
thymeleaf = { git = "https://github.com/easy-4-rust/thymeleaf-rust.git" }
```

### Minimal Example

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

### CI Pipeline (24 steps)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask migration-check --upstream <upstream> --baseline 10f9dd2e...
python3 scripts/audit_migration_layout.py --fail-on-warning   # layout audit 0/0/0
cargo deny check                                               # licenses/bans/sources/advisories
cargo audit                                                    # vulnerabilities
cargo llvm-cov --workspace --all-features --summary-only
THYMELEAF_SCOPE=semantic_all cargo test -p thymeleaf-test --test thtest_upstream_plain_batch  # 2,608 corpus
```

### Test Types

| Type | Count | Purpose |
|:---|:---:|:---|
| Unit (lib) | 295 | Core logic |
| Integration (parity) | 964 | Java 1:1 differential |
| Adapter contracts | 45 | Framework integration smoke |
| Corpus | 2,608 | Upstream behavioral parity |
| Acceptance | 2,686 assets | SHA-256 byte-identical |
| Fuzz (proptest) | 2 active | XML/TEXT parser robustness |

## 9. Known Limitations

- **html5gum tokenizer**: Pathological Unicode input (isolated surrogates, special sequences) can cause internal memory inflation. HTML parser fuzz excluded; robustness covered by 2,608 corpus.
- **Render smoke proptest**: Random expression injection can cause `process_template` timeout (>60s). Excluded; covered by corpus + workspace tests.
- **API baseline CI**: `cargo public-api` requires nightly; CI uses stable → `continue-on-error` (alpha stage).

## 10. Roadmap

| Phase | Status | Items |
|:---|:---:|:---|
| Semantic alignment | ✅ Done | 2,608 corpus, 491 objects, 4,291 methods |
| Governance audit | ✅ Done | strict blockers 0, warnings 0, CI enforced |
| Fuzz OOM fix | ✅ Done | DiscardingWriter + shrink clamp + serial |
| Release ecosystem | 🚧 In progress | `cargo package --verify`, docs.rs, adapter contracts |
| Version 0.1.0 | 🗓️ Planned | API freeze, CHANGELOG, tag |
| Benchmark suite | 🗓️ Planned | Criterion render/parse/expression |

## 11. Contributing

Run the basic gates before submitting:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

New public APIs must include docs, tests, and SemVer/MSRV impact notes.

## 12. License

[MIT](./LICENSE)

This project ports behavior from [Thymeleaf](https://www.thymeleaf.org/) (Apache 2.0).
Upstream license, source commit, and modification scope are documented in `docs/`.

---

<div align="center">

[Back to top](#readme-top) · [Actions](https://github.com/easy-4-rust/thymeleaf-rust/actions) · [Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
