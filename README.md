<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**A framework-neutral, Thymeleaf-inspired dynamic content rendering engine for Rust.**

[![Project status: design stage](https://img.shields.io/badge/status-design%20stage-blue)](#project-status)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

[English](README.md) | [简体中文](README.zh-CN.md)

[Overview](#overview) · [Architecture](#architecture) · [Planned crates](#planned-crates) ·
[Integrations](#integration-model) · [Roadmap](#roadmap) · [Contributing](#contributing)

</div>

---

> **Project status: design stage**
>
> This repository currently contains architecture and implementation planning only. It does not yet provide a Cargo workspace, an installable crate, stable public APIs, runnable examples, CI results, or compatibility claims.

## Overview

`thymeleaf-rust` is the project and repository name for a planned Rust implementation of a dynamic content rendering engine inspired by [Thymeleaf](https://www.thymeleaf.org/) template semantics.

The future public facade crate will be published as **`thymeleaf`**. Planned subcrates and adapters use the `thymeleaf-*` prefix; no published crate or Rust module will use a `thymeleaf-rust-*` prefix.

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
- Offer an optional `vernal-thymeleaf` bridge without making Vernal a core dependency.
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
  Independent framework adapters       Optional vernal-thymeleaf
  thymeleaf-{framework}                 bridge
              │                                 │
              ▼                                 ▼
  Native framework response             vernal-{framework}
```

Core dependency rule:

```text
thymeleaf core ← neutral contracts ← adapters

Never:
thymeleaf core → framework adapter
thymeleaf core → Vernal
```

See the detailed [feasibility and architecture proposal](docs/Thymeleaf-Rust-可行性与架构设计.md).

## Naming and publication contract

| Layer | Name | Status |
|:---|:---|:---:|
| Project and Git repository | `thymeleaf-rust` | Confirmed |
| Future crates.io facade | `thymeleaf` | Planned |
| Public Rust path | `thymeleaf::...` | Planned |
| Subcrates | `thymeleaf-*` | Planned |
| Independent framework adapters | `thymeleaf-{framework}` | Planned |
| Optional Vernal bridge | `vernal-thymeleaf` | Planned |

Names such as `thymeleaf-rust-core`, `thymeleaf-rust-axum`, and the Rust root module `thymeleaf_rust` are explicitly excluded.

## Planned crates

All entries in this table are plans; no Cargo manifests or published artifacts exist yet.

| Planned crate | Responsibility |
|:---|:---|
| `thymeleaf` | Stable facade and public re-exports |
| `thymeleaf-core` | Engine, context, model, events, errors, processor and dialect contracts |
| `thymeleaf-parser` | Parser abstractions and shared parsing facilities |
| `thymeleaf-parser-html` | HTML template mode |
| `thymeleaf-parser-xml` | XML template mode |
| `thymeleaf-parser-text` | Text, JavaScript, CSS, and raw modes |
| `thymeleaf-expression` | Thymeleaf expression grammar and neutral evaluator contract |
| `thymeleaf-standard` | Standard dialect and `th:*` processors |
| `thymeleaf-web` | Neutral view model, rendered output, HTTP headers, and MIME handling |
| `thymeleaf-testkit` | Parity, golden, integration, and adapter test support |

## Integration model

Every target framework is planned to support both direct use and optional Vernal composition.

| Host | Independent adapter | Vernal composition | Intended output |
|:---|:---|:---|:---|
| Topcoat | `thymeleaf-topcoat` | `vernal-thymeleaf` + `vernal-topcoat` | View, page, fragment, controlled raw HTML |
| Actix Web | `thymeleaf-actix-web` | `vernal-thymeleaf` + `vernal-actix-web` | Responder, message body, stream |
| Axum | `thymeleaf-axum` | `vernal-thymeleaf` + `vernal-axum` | IntoResponse and body |
| Gotham | `thymeleaf-gotham` | `vernal-thymeleaf` + `vernal-gotham` | Handler and response |
| Hyper | `thymeleaf-hyper` | `vernal-thymeleaf` + `vernal-hyper` | Standard HTTP response/body |
| Ntex | `thymeleaf-ntex` | `vernal-thymeleaf` + `vernal-ntex` | Responder, service, body |
| Poem | `thymeleaf-poem` | `vernal-thymeleaf` + `vernal-poem` | IntoResponse, endpoint, stream |
| Rocket | `thymeleaf-rocket` | `vernal-thymeleaf` + `vernal-rocket` | Responder and byte stream |
| Salvo | `thymeleaf-salvo` | `vernal-thymeleaf` + `vernal-salvo` | Handler and response body |
| Tide | `thymeleaf-tide` | `vernal-thymeleaf` + `vernal-tide` | Endpoint and response |
| Warp | `thymeleaf-warp` | `vernal-thymeleaf` + `vernal-warp` | Reply and rejection mapping |
| Tower | `thymeleaf-tower` | `vernal-thymeleaf` + `vernal-tower` | Service, layer, response body |
| Tonic | `thymeleaf-tonic` | `vernal-thymeleaf` + `vernal-tonic` | Dynamic string/bytes, gateway, service content |

Release order may be phased, but the architecture must not require independent users to depend on Vernal.

## Project status

| Deliverable | Status | Evidence |
|:---|:---:|:---|
| Feasibility and architecture proposal | Available | [`docs/Thymeleaf-Rust-可行性与架构设计.md`](docs/Thymeleaf-Rust-可行性与架构设计.md) |
| Naming and neutrality decisions | Documented | Architecture proposal ADRs |
| Cargo workspace | Not created | No `Cargo.toml` |
| Public Rust API | Design only | No source implementation |
| Framework adapters | Planned | No adapter manifests or code |
| Upstream compatibility matrix | Planned | No parity harness |
| Tests and CI | Not created | No test or workflow claims |
| crates.io package | Not published | `thymeleaf` remains a planned publication name |

## Documentation quick start

There is no executable quick start yet. To review the current design:

```bash
git clone --branch dev https://github.com/easy-4-rust/thymeleaf-rust.git
cd thymeleaf-rust
```

Then read:

- [Feasibility and architecture proposal](docs/Thymeleaf-Rust-可行性与架构设计.md)
- [Chinese README](README.zh-CN.md)

## Compatibility direction

The initial design targets the core semantics of the Thymeleaf 3.1 line. Exact upstream versions, supported processors, expression behavior, error parity, exclusions, and compatibility percentages will be published only after they are pinned and verified.

Planned evidence includes:

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
| Phase 3 | `vernal-thymeleaf` bridge | All relevant `vernal-*` hosts consume the same engine |
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

There is no executable parser or runtime in the repository yet. Before implementation is accepted, the project plans to define limits for input size, recursion, expression evaluation, template resolution, output size, cancellation, and unescaped content.

Do not publish suspected vulnerabilities or sensitive proof-of-concept data in a public issue. A private reporting channel will be finalized before executable releases.

## License

This repository is licensed under the [MIT License](LICENSE).

Upstream-derived material remains subject to its original license and attribution requirements.

---

<div align="center">

[Back to top](#readme-top) · [Architecture](docs/Thymeleaf-Rust-可行性与架构设计.md) ·
[Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
