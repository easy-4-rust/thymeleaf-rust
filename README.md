<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**A framework-neutral, Thymeleaf-inspired dynamic content rendering engine for Rust.**

[![Project status: migration in progress](https://img.shields.io/badge/status-migration%20in%20progress-blue)](#project-status)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

[English](README.md) | [简体中文](README.zh-CN.md)

[Overview](#overview) · [Architecture](#architecture) · [Crate model](#crate-model) ·
[Integrations](#integration-model) · [Roadmap](#roadmap) · [Contributing](#contributing)

</div>

---

> **Project status: migration in progress**
>
> The `thymeleaf` Cargo workspace and the first behavior-verified foundation slice now exist. The rendering engine, parsers, processors, integrations, stable public API, and crates.io release are still incomplete; no whole-engine compatibility claim is made.

## Overview

`thymeleaf-rust` is the project and repository name for a planned Rust implementation of a dynamic content rendering engine inspired by [Thymeleaf](https://www.thymeleaf.org/) template semantics.

The future public core crate will be published as **`thymeleaf`**. Framework integration crates use the `thymeleaf-{framework}` pattern; no published crate or Rust module will use a `thymeleaf-rust-*` prefix.

The project is intended to support two equal integration paths:

1. direct, independent integration with Rust web frameworks;
2. optional integration as the dynamic rendering engine for Vernal and its web adapters.

The engine core will remain independent of Topcoat, Actix Web, Axum, Gotham, Hyper, Ntex, Poem, Rocket, Salvo, Tide, Warp, Tower, Tonic, and Vernal.

This is an independent project. It is not an official Thymeleaf project.

## Goals and boundaries

### Goals

- Provide natural HTML templates with Thymeleaf-style processing semantics.
- Support variables, selection expressions, messages, links, fragments, processors, and dialects.
- Build immutable, cacheable, replayable template models.
- Support complete and backpressure-aware streaming rendering.
- Expose a neutral `RenderedTemplate`/HTTP body contract.
- Offer independently versioned adapters for supported Rust web frameworks.
- Offer an optional `thymeleaf-vernal` bridge without making Vernal a core dependency.
- Develop upstream compatibility through traceable parity and golden tests.

### Non-goals

- Reproduce Java inheritance, reflection, Servlet, or Spring types in the core.
- Claim complete Thymeleaf compatibility before a versioned compatibility matrix exists.
- Make a framework-specific response type part of the engine API.
- Turn runtime templates automatically into Topcoat reactive components.
- Treat gRPC payloads as ordinary browser HTML responses.
- Present planned crates, APIs, tests, or benchmarks as implemented.

## Architecture

```text
Templates + model + locale + render options
                    │
                    ▼
┌──────────────────────────────────────────────────────────────┐
│                  thymeleaf neutral engine                    │
│ resolver → parser → template model → processors → renderer   │
└──────────────────────────────┬───────────────────────────────┘
                               │
                    RenderedTemplate
                    Full(Bytes) / Stream(Frame<Bytes>)
                               │
              ┌────────────────┴────────────────┐
              ▼                                 ▼
  Independent framework adapters       Optional thymeleaf-vernal
  thymeleaf-{framework}                 bridge
              │                                 │
              ▼                                 ▼
  Native framework response             vernal-{framework}
```

Core dependency rule:

```text
thymeleaf crate ← neutral contracts ← integration crates

Never:
thymeleaf crate → framework integration
thymeleaf crate → Vernal
```

See the detailed [feasibility and architecture proposal](docs/Thymeleaf-Rust-可行性与架构设计.md).

## Naming and publication contract

| Layer | Name | Status |
|:---|:---|:---:|
| Project and Git repository | `thymeleaf-rust` | Confirmed |
| Future crates.io core | `thymeleaf` | Workspace created; not published |
| Public Rust path | `thymeleaf::...` | Foundation API implemented |
| Integration crates | `thymeleaf-{framework}` | Planned |
| Optional Vernal integration | `thymeleaf-vernal` | Planned |

Names such as `thymeleaf-rust-core`, `thymeleaf-rust-axum`, and the Rust root module `thymeleaf_rust` are explicitly excluded.

## Crate model

The engine is planned as one cohesive core crate. Parser modes, expression handling, the standard dialect, neutral web output, and test support are internal modules or test infrastructure of `thymeleaf`; they are not separate published crates.

| Planned core crate | Responsibility |
|:---|:---|
| `thymeleaf` | Engine, context, template model, parser modes, expression evaluation, standard dialect and `th:*` processors, neutral rendered output, stable public API, and core test infrastructure |

Everything else is an integration crate: `thymeleaf-{framework}` adapts the neutral `thymeleaf` output to one host framework, while `thymeleaf-vernal` provides the optional Vernal integration. Its name follows the same core-first convention as `thymeleaf-spring`. Integration crates must remain thin and must not duplicate parsing or rendering logic.

## Integration model

Every target framework is planned to support both direct use and optional Vernal composition.

| Host | Independent adapter | Vernal composition | Intended output |
|:---|:---|:---|:---|
| Topcoat | `thymeleaf-topcoat` | `thymeleaf-vernal` + `vernal-topcoat` | View, page, fragment, controlled raw HTML |
| Actix Web | `thymeleaf-actix-web` | `thymeleaf-vernal` + `vernal-actix-web` | Responder, message body, stream |
| Axum | `thymeleaf-axum` | `thymeleaf-vernal` + `vernal-axum` | IntoResponse and body |
| Gotham | `thymeleaf-gotham` | `thymeleaf-vernal` + `vernal-gotham` | Handler and response |
| Hyper | `thymeleaf-hyper` | `thymeleaf-vernal` + `vernal-hyper` | Standard HTTP response/body |
| Ntex | `thymeleaf-ntex` | `thymeleaf-vernal` + `vernal-ntex` | Responder, service, body |
| Poem | `thymeleaf-poem` | `thymeleaf-vernal` + `vernal-poem` | IntoResponse, endpoint, stream |
| Rocket | `thymeleaf-rocket` | `thymeleaf-vernal` + `vernal-rocket` | Responder and byte stream |
| Salvo | `thymeleaf-salvo` | `thymeleaf-vernal` + `vernal-salvo` | Handler and response body |
| Tide | `thymeleaf-tide` | `thymeleaf-vernal` + `vernal-tide` | Endpoint and response |
| Warp | `thymeleaf-warp` | `thymeleaf-vernal` + `vernal-warp` | Reply and rejection mapping |
| Tower | `thymeleaf-tower` | `thymeleaf-vernal` + `vernal-tower` | Service, layer, response body |
| Tonic | `thymeleaf-tonic` | `thymeleaf-vernal` + `vernal-tonic` | Dynamic string/bytes, gateway, service content |

Release order may be phased, but the architecture must not require independent users to depend on Vernal.

## Project status

| Deliverable | Status | Evidence |
|:---|:---:|:---|
| Feasibility and architecture proposal | Available | [`docs/Thymeleaf-Rust-可行性与架构设计.md`](docs/Thymeleaf-Rust-可行性与架构设计.md) |
| Naming and neutrality decisions | Documented | Architecture proposal ADRs |
| Cargo workspace | Available | [`Cargo.toml`](Cargo.toml) |
| Public Rust API | Verified slices | Foundation/configuration APIs, cache families, `StandardCache`, `TemplateResolution`, and the template-resource SPI/string/file resources; URL and JVM soft-reference runtime edges remain pending |
| Framework adapters | Planned | No adapter manifests or code |
| Upstream compatibility matrix | In progress | 491 objects, 4,291 methods, and 6,936 parameters inventoried |
| Migration governance | Automated | `cargo xtask migration-check` validates baseline, manifest, layout, documentation, and red lines |
| Tests and CI | Slice gates passing | 91 unit tests, ten Java/Rust Golden tests with 756 records, 100% line/function/region coverage |
| crates.io package | Not published | `thymeleaf` remains a planned publication name |

## Documentation quick start

The rendering engine is not executable yet. To check the implemented S1/S2/S5 slices:

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

Then read:

- [Feasibility and architecture proposal](docs/Thymeleaf-Rust-可行性与架构设计.md)
- [Migration roadmap](docs/migration/迁移路线图.md)
- [Object-level mapping](docs/migration/对象级对照表.md)
- [Method-level mapping](docs/migration/方法级对照表.md)
- [Semantic migration matrix](docs/migration/语义迁移对照表.md)
- [Object naming consistency check](docs/migration/对象名称一致性检查.md)
- [Chinese README](README.zh-CN.md)

## Compatibility direction

The initial design targets the core semantics of the Thymeleaf 3.1 line. Exact upstream versions, supported processors, expression behavior, error parity, exclusions, and compatibility percentages will be published only after they are pinned and verified.

Evidence now includes a fixed Java API inventory and a Foundation Golden differential test. Remaining planned evidence includes:

- public API and processor inventories;
- Java/Rust golden-output comparisons;
- fragment, escaping, URL, locale, and error parity tests;
- differential tests for malformed and boundary inputs;
- an explicit difference register with migration guidance.

The upstream Thymeleaf project is licensed under Apache License 2.0. Any source, tests, or fixtures adapted from upstream must preserve the applicable copyright, license, NOTICE, attribution, and modification requirements.

## Roadmap

| Phase | Planned deliverable | Exit condition |
|:---|:---|:---|
| Phase 0 | Neutral contracts and parser/body prototypes | Model replay and full/stream output validated |
| Phase 1 | HTML and standard-dialect MVP | One real template path works end to end |
| Phase 2 | Independent framework adapters | Each adapter has full/stream/error/cancellation tests |
| Phase 3 | `thymeleaf-vernal` bridge | All relevant `vernal-*` hosts consume the same engine |
| Phase 4 | Additional modes and dialect SPI | Versioned capability matrix is published |
| Phase 5 | Compatibility and release readiness | Packaging, documentation, security, and parity gates pass |

Dates are intentionally omitted until implementation capacity and dependency choices are confirmed.

## Design sketch: future API

The following is a non-runnable design sketch. Names and signatures may change.

```rust
use thymeleaf::{Context, TemplateEngine};

fn render(engine: &TemplateEngine) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = Context::new();
    context.set("name", "Rust");
    Ok(engine.render("home", &context)?)
}
```

Do not use `cargo add thymeleaf` until this README explicitly marks the crate as published.

## Contributing

The project currently welcomes design review in these areas:

- HTML parser fidelity and source spans;
- immutable event/model representation;
- expression safety and Rust value access;
- processor and dialect contracts;
- full and streaming output semantics;
- framework adapter boundaries;
- upstream compatibility and test methodology.

When implementation begins, changes will be expected to include documentation, tests, compatibility impact, and clear dependency-direction checks.

## Security

There is no executable parser or rendering runtime in the repository yet. Before those layers are accepted, the project will define limits for input size, recursion, expression evaluation, template resolution, output size, cancellation, and unescaped content.

Do not publish suspected vulnerabilities or sensitive proof-of-concept data in a public issue. A private reporting channel will be finalized before executable releases.

## License

This repository is licensed under the [MIT License](LICENSE).

Upstream-derived material remains subject to its original license and attribution requirements.

---

<div align="center">

[Back to top](#readme-top) · [Architecture](docs/Thymeleaf-Rust-可行性与架构设计.md) ·
[Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
