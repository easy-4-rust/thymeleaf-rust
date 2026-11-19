<a id="readme-top"></a>

<div align="center">

# thymeleaf-rust

**A framework-neutral Rust template engine migrating Thymeleaf core semantics.**

[![crates.io](https://img.shields.io/crates/v/thymeleaf)](https://crates.io/crates/thymeleaf/0.2.0)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/LICENSE)

[English](https://github.com/easy-4-rust/thymeleaf-rust/blob/main/thymeleaf/README.md) | [简体中文](https://github.com/easy-4-rust/thymeleaf-rust/blob/main/thymeleaf/README.zh-CN.md)

[Overview](#overview) · [Architecture](#architecture) · [Crate model](#crate-model) ·
[Integrations](#integration-model) · [Remaining roadmap](#remaining-roadmap) ·
[Contributing](#contributing)

</div>

---

> **Thymeleaf core v0.2.0** · Rust 1.95 · Rust edition 2024
>
> [crates.io](https://crates.io/crates/thymeleaf/0.2.0) · [API documentation](https://docs.rs/thymeleaf/0.2.0/thymeleaf/) · [GitHub Release](https://github.com/easy-4-rust/thymeleaf-rust/releases/tag/v0.2.0)
>
> The upstream differential suite covers 2,608 executable cases. It verifies 2,595 and records 13 explicit dispositions.

## Overview

`thymeleaf-rust` is the project and repository name for a Rust dynamic content rendering
engine that migrates [Thymeleaf](https://www.thymeleaf.org/) 3.1.5.RELEASE core template
semantics at object, method, and behavioral levels.

The public core crate name is **`thymeleaf`**, available on crates.io. Framework
integration crates use the `thymeleaf-{framework}` pattern; no published crate or
Rust module uses a `thymeleaf-rust-*` prefix.

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
template + model + locale
          │
          ▼
┌────────────────────────────── thymeleaf ──────────────────────────────┐
│ CONTROL      EngineConfiguration · DialectSetConfiguration · cache   │
│     │                                                                │
│     ▼                                                                │
│ MODEL        resolver → resource → parser → immutable TemplateModel  │
│     │                                                                │
│     ▼                                                                │
│ EXECUTION    pre → ProcessorTemplateHandler → post → output events   │
│     │                                                                │
│     ▼                                                                │
│ DELIVERY     process → String/Writer                                 │
│              process_throttled → IThrottledTemplateProcessor         │
│                                                                      │
│ Shared semantics: expression · message · link · fragment · dialect   │
│ Neutral Web: ThymeleafRenderer calls the delivery APIs               │
│              → RenderedTemplate(status, headers, Full/Stream body)    │
└──────────────────────────────────┬───────────────────────────────────┘
                                   │ stable neutral contracts
                  ┌────────────────┴─────────────────┐
                  ▼                                  ▼
        thymeleaf-{framework}                thymeleaf-vernal
        direct host adapter                  optional peer bridge
                  │                                  │
                  ▼                                  ▼
        native framework response             vernal-{framework}
```

The arrows above describe runtime data flow. Cargo dependencies point in the opposite
direction: applications and integration crates depend on `thymeleaf`; the core never
depends on a host framework or Vernal.

```text
host application → thymeleaf-{framework} → thymeleaf
vernal application → thymeleaf-vernal → thymeleaf

Never:
thymeleaf crate → framework integration
thymeleaf crate → Vernal
```

### Core rendering call chain

The current implementation, verified with CodeGraph, retains Thymeleaf's event-driven
topology instead of delegating templates to Tera, Askama, or Handlebars:

```mermaid
flowchart TB
    INPUT["TemplateSpec + IContext"] --> ENGINE["TemplateEngine<br/>initialize once, then freeze configuration"]

    subgraph CONTROL["1 · application-scoped control plane"]
        DIALECTS["DialectSetConfiguration<br/>aggregate/order processors · pre/post-processors<br/>execution attributes · expression-object factories"]
        COLLABORATORS["resolvers · cache manager<br/>message resolvers · link builders"]
        CONFIG["EngineConfiguration<br/>immutable runtime snapshot"]
        DIALECTS --> CONFIG
        COLLABORATORS --> CONFIG
    end

    subgraph MODEL_PLANE["2 · resolution, parsing, and model plane"]
        MANAGER["TemplateManager"]
        CACHE{"template cache hit?"}
        RESOLVER["ordered TemplateResolver chain"]
        RESOURCE["TemplateResolution + TemplateResource<br/>TemplateMode + Validity"]
        PARSER["HTML / XML / TEXT / JS / CSS / RAW parser"]
        BUILDER["ModelBuilderTemplateHandler"]
        MODEL["immutable, replayable TemplateModel"]
        STORE["Arc&lt;TemplateModel&gt; cache entry"]
        MANAGER --> CACHE
        CACHE -->|yes| MODEL
        CACHE -->|no| RESOLVER --> RESOURCE --> PARSER --> BUILDER --> MODEL
        MODEL -. "store only if Validity allows" .-> STORE
        STORE -. "later lookup" .-> CACHE
    end

    subgraph EXECUTION["3 · request-scoped event execution plane"]
        CONTEXT_MANAGER["EngineContextManager<br/>root creation · nested reuse · level restoration"]
        CONTEXT_FACTORY["IEngineContextFactory<br/>StandardEngineContextFactory"]
        CONTEXT["EngineContext<br/>variables · locale · selection · optional Web capability"]
        HANDLERS["PreProcessor → ProcessorTemplateHandler → PostProcessor"]
        SERVICES["Expression · Message · Link · Fragment"]
        EVENTS["processed output events"]
        CONTEXT_FACTORY --> CONTEXT_MANAGER --> CONTEXT
        CONTEXT --> HANDLERS
        MODEL --> HANDLERS --> EVENTS
        HANDLERS -. "processor-selected" .-> SERVICES
    end

    subgraph DELIVERY["4 · delivery plane"]
        FULL["process / process_to_writer<br/>OutputTemplateHandler → TemplateWriter"]
        THROTTLED["process_throttled<br/>ThrottledTemplateProcessor + FlowController"]
        EVENTS --> FULL
        EVENTS --> THROTTLED
    end

    ENGINE --> MANAGER
    ENGINE --> CONTEXT_MANAGER
    CONFIG -. "frozen ordering and policies" .-> MANAGER
    CONFIG -. "supplies the context factory" .-> CONTEXT_FACTORY
    CONFIG -. "processor/runtime services" .-> HANDLERS
    FULL --> FULL_RESULT["Utf16String / Writer"]
    THROTTLED --> STREAM_RESULT["IThrottledTemplateProcessor<br/>caller-driven backpressure"]
```

The current Rust implementation materializes a `TemplateModel` on every cache miss.
Cache validity controls whether that model is retained, not whether parsing bypasses
the model. Full and throttled rendering share resolution, parsing, processor,
expression, and output-event semantics; only the output driver and backpressure
boundary differ.

At the root template boundary, `StandardEngineContextFactory` selects `EngineContext`
or `WebEngineContext` from the source context's `IWebContext` capability. Nested
templates reuse the same engine context: `EngineContextManager` raises its level,
pushes `TemplateData`, and restores the previous level on disposal. Ordinary and Web
rendering therefore diverge only by context capability, not by creating separate
template engines.

Four constrained planes make up the core architecture. The configuration plane
aggregates dialect capabilities and freezes them during first initialization. The
parsing/model plane turns resources into replayable events. The request execution plane
interprets those events through processors. Only the delivery plane selects full,
throttled, or HTTP output. A Web integration has exactly two host-facing ports:
an inbound capability wrapper that supplies request/session/application observations,
and an outbound conversion from `RenderedTemplate` to the native response/body. It must
not manipulate configuration, parsers, template models, processor chains, or expression
semantics.

### Neutral web layer and adapters

```mermaid
flowchart LR
    subgraph HOST["Host boundary"]
        REQUEST["native request / session / application"]
        RESPONSE["native response / responder / reply / service"]
    end

    subgraph ADAPTER["Integration boundary"]
        IN["request capability wrapper"]
        OUT["response/body conversion"]
    end

    subgraph THYMELEAF["thymeleaf — framework-neutral"]
        PORTS["IWebApplication · IWebExchange<br/>IWebRequest · IWebSession"]
        CONTEXT["WebContext / IContext<br/>optional Web capability"]
        PLAIN["plain IContext<br/>non-Web rendering"]
        CALL["TemplateSpec + IContext"]
        RENDERER["ThymeleafRenderer"]
        ENGINE["ITemplateEngine<br/>same resolver/parser/model/processor semantics"]
        FULL_ENGINE["process → Utf16String"]
        STREAM_ENGINE["process_throttled → throttled processor"]
        FULL["charset encode + Content-Length<br/>RenderedTemplateBody::Full(Bytes)"]
        STREAM["render worker + capacity-one Frame channel<br/>RenderedTemplateBody::Stream"]
        DATA["optional DataDrivenTemplateIterator<br/>signals the same throttled path"]
        RESULT["RenderedTemplate<br/>StatusCode + HeaderMap + RenderedTemplateBody"]
        PORTS --> CONTEXT --> CALL
        PLAIN --> CALL
        CALL --> RENDERER
        RENDERER --> ENGINE
        ENGINE --> FULL_ENGINE --> FULL --> RESULT
        ENGINE --> STREAM_ENGINE --> STREAM --> RESULT
        DATA -. "feeds" .-> STREAM_ENGINE
    end

    REQUEST --> IN --> PORTS
    RESULT --> OUT --> RESPONSE

    DIRECT["thymeleaf-{framework}<br/>direct adapter"] -. "implements IN/OUT; depends on" .-> ADAPTER
    VERNAL["thymeleaf-vernal<br/>optional peer adapter"] -. "implements IN/OUT; depends on" .-> ADAPTER
```

Neutrality is bidirectional. On input, adapters expose only the web capabilities needed
by contexts, link builders, and web template resources. On output, they convert only
status, headers, and body types. They must not reimplement resolvers, parsers,
expressions, processors, charset handling, or streaming control. Direct adapters and
`thymeleaf-vernal` are peer consumers of the same contracts; neither path depends on
the other. See the detailed
[feasibility and architecture design](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-architecture-design.md) for
call-chain evidence and risks.

The neutral inbound traits are implemented today by the Hyper host bridge. The other
framework crates currently focus on outbound `RenderedTemplate` conversion; equivalent
native request/session/application wrappers remain part of their acceptance work.

| Boundary | Owned by `thymeleaf` | Owned by an integration crate |
|:---|:---|:---|
| Request side | Neutral web capability traits and template context semantics | Native request/session/application wrappers |
| Rendering | Resolver, parser, model, processors, expressions, links, messages, fragments, cache | No rendering semantics |
| Response side | `RenderedTemplate`, metadata, charset encoding, full/stream body | Native Response/Responder/Reply/Service conversion |
| Lifecycle | Throttled progress, frame errors, data-driven signaling | Request scope, disconnect observation, host error mapping |

Failures follow the same boundary:

| Failure stage | Observable behavior | Owner |
|:---|:---|:---|
| Engine initialization / dialect aggregation | Fails synchronously before rendering; no response body exists | `thymeleaf` |
| Resolver / parser / processor in Full mode | `render_full` returns synchronously with an error; no response body exists | `thymeleaf` |
| Resolver / parser / processor in Stream mode | Terminates `RenderedTemplateBody::Stream` with an error item; already-sent headers cannot be replaced | Defined by `thymeleaf`, forwarded losslessly by the adapter |
| Client disconnect / dropped body | Consumption stops and the sender closes; the host observes connection lifecycle | Integration crate |
| Native response conversion | Maps to the host error type without re-running the template | Integration crate |

Neutrality therefore covers success, initialization failure, rendering failure, and
late streaming failure—not only a shared success response.

The same core supports three deployment modes without changing template semantics:

| Mode | Dependency path | Web capabilities | Output |
|:---|:---|:---|:---|
| Non-Web rendering | application → `thymeleaf` | none; plain `IContext` | `Utf16String` or `TemplateWriter` |
| Direct Web integration | application → `thymeleaf-{framework}` → `thymeleaf` | adapter wraps native request/session/application objects | native framework response |
| Vernal integration | Vernal application → `thymeleaf-vernal` → `thymeleaf` | Vernal bridge supplies the same neutral capabilities | Vernal HTTP/view result |

Direct adapters and `thymeleaf-vernal` are peers. Neither is the canonical route through
which the other must pass.

```mermaid
flowchart TB
    CORE["thymeleaf<br/>one semantic core + neutral Web contracts"]
    DIRECT["thymeleaf-{framework}<br/>Topcoat · Actix Web · Axum · Gotham · Hyper · Ntex · Poem<br/>Rocket · Salvo · Tide · Warp · Tower · Tonic"]
    VERNAL["thymeleaf-vernal<br/>optional Vernal bridge"]
    DIRECT_APP["native framework application"]
    VERNAL_APP["vernal-{framework} application"]

    DIRECT_APP --> DIRECT --> CORE
    VERNAL_APP --> VERNAL --> CORE
    DIRECT -. "does not depend on" .-> VERNAL
    VERNAL -. "does not depend on" .-> DIRECT
    CORE -. "must not depend back on adapters" .-> DIRECT
    CORE -. "must not depend back on Vernal" .-> VERNAL
```

Solid arrows show Cargo/API dependencies. Runtime rendered data moves in the opposite
direction: `thymeleaf` produces `RenderedTemplate`, and the selected adapter converts
it into the host response.

## Naming and publication contract

| Layer | Name | Status |
|:---|:---|:---:|
| Project and Git repository | `thymeleaf-rust` | Confirmed |
| crates.io core | `thymeleaf` | Published; version on crates.io |
| Public Rust path | `thymeleaf::...` | Foundation API implemented |
| Integration crates | `thymeleaf-{framework}` | 13 adapter crates implemented; basic response contracts pass; end-to-end verification pending |
| Optional Vernal integration | `thymeleaf-vernal` | HTTP protocol bridge implemented; status/header/data/trailer contract passes |

Names such as `thymeleaf-rust-core`, `thymeleaf-rust-axum`, and the Rust root module `thymeleaf_rust` are explicitly excluded.

## Crate model

The engine is one cohesive core crate. Parser modes, expression handling, the standard dialect, neutral web output, and test support are internal modules or test infrastructure of `thymeleaf`; they are not separate published crates.

| Core crate | Responsibility |
|:---|:---|
| `thymeleaf` | Engine, context, template model, parser modes, expression evaluation, standard dialect and `th:*` processors, neutral rendered output, stable public API, and core test infrastructure |

Everything else is an integration crate: `thymeleaf-{framework}` adapts the neutral `thymeleaf` output to one host framework, while `thymeleaf-vernal` provides the optional Vernal integration. Its name follows the same core-first convention as `thymeleaf-spring`. Integration crates must remain thin and must not duplicate parsing or rendering logic.

## Integration model

The architecture defines both direct use and optional Vernal composition for every
target framework. Direct adapters are implemented today; each `vernal-{framework}`
combination remains subject to the corresponding Vernal host adapter.

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
| Tide (source only) | `thymeleaf-tide` | `thymeleaf-vernal` + `vernal-tide` | Endpoint and response |
| Warp | `thymeleaf-warp` | `thymeleaf-vernal` + `vernal-warp` | Reply and rejection mapping |
| Tower | `thymeleaf-tower` | `thymeleaf-vernal` + `vernal-tower` | Service, layer, response body |
| Tonic | `thymeleaf-tonic` | `thymeleaf-vernal` + `vernal-tonic` | Dynamic string/bytes, gateway, service content |

Release order may be phased, but the architecture must not require independent users to depend on Vernal.

## Project status

| Deliverable | Status | Evidence |
|:---|:---:|:---|
| Feasibility and architecture baseline | Available; CodeGraph reviewed | [`docs/superpowers/specs/2026-07-28-architecture-design.md`](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-architecture-design.md) |
| Naming and neutrality decisions | Documented | Architecture proposal ADRs |
| Cargo workspace | Available | [`Cargo.toml`](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/Cargo.toml) |
| Rust core API | Semantic inventory closed | Engine, configuration, resolvers, six parsers, event model, context, caches, standard dialect, processor SPI, safe expression subset, and neutral web output have real implementations |
| Framework adapters | Available; host tests need expansion | Workspace and source-only adapters are listed in the [repository README](https://github.com/easy-4-rust/thymeleaf-rust#4-workspace-layout) |
| Upstream compatibility matrix | Structural inventory and registered parity corpus verified | All 491 main objects and 4,291 methods have dispositions; the repository's migration ledger records 479 verified objects and 12 Java-only host equivalents; 2,595 of 2,608 semantic cases match with 13 approved differences |
| Migration governance | Automated | `cargo xtask migration-check` validates baseline, manifest, layout, documentation, and red lines |
| Tests and CI | Core Java baseline and Rust differential tests | [CI workflow](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/.github/workflows/ci.yml) |
| crates.io package | Published | [crates.io/crates/thymeleaf](https://crates.io/crates/thymeleaf) |

## Documentation quick start

Add the published core crate with `cargo add thymeleaf@0.2.0`. The repository README provides a [runnable example](https://github.com/easy-4-rust/thymeleaf-rust#6-quick-start) and [workspace validation commands](https://github.com/easy-4-rust/thymeleaf-rust#8-quality-gates).

Then read:

- [Feasibility and architecture baseline](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-architecture-design.md)
- [Migration roadmap](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-migration-roadmap.md)
- [Object-level mapping](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-object-level-mapping.md)
- [Method-level mapping](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-method-level-mapping.md)
- [Semantic migration matrix](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-semantic-migration-mapping.md)
- [Object naming consistency check](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-naming-consistency-check.md)
- [Migration technical requirements](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-migration-technical-requirements.md)
- [Migration test ledger](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-migration-test-mapping.md)
- [Chinese README](https://github.com/easy-4-rust/thymeleaf-rust/blob/main/thymeleaf/README.zh-CN.md)

## Compatibility direction

Compatibility is pinned to Thymeleaf 3.1.5.RELEASE commit
`10f9dd2eb8cbd98515ce14b149d115e0287d0add`. Evidence includes object and method
inventories, a [source parity ledger](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/migration/baseline/source_parity_inventory.json) covering 413 core Java test methods and 1,154 runtime cases, and the Rust differential test covering 2,608 upstream `.thtest` cases.

Remaining evidence work focuses on raising conservative object-level maturity labels,
expanding adapter full/stream/error/cancellation tests, and maintaining the explicit
difference register.

The upstream Thymeleaf project is licensed under Apache License 2.0. Any source, tests, or fixtures adapted from upstream must preserve the applicable copyright, license, NOTICE, attribution, and modification requirements.

## Remaining roadmap

| Phase | Planned deliverable | Exit condition |
|:---|:---|:---|
| P0 | Framework-adapter acceptance | Every adapter has real HTTP full/stream/error/cancellation tests |
| P0 | Streaming execution model | Load-test one-thread-per-response behavior; introduce a configurable executor or bounded pool if required |
| P1 | Third-party dialect compatibility suite | Custom processors, pre/post-processors, and expression-object factories pass plugin contracts |
| P1 | Published capability matrix | Document the safe OGNL subset, host policy differences, MSRV, and supported targets |
| P2 | crates.io updates | Package verification, documentation, security review, and parity gates pass for every version |

Release history is tracked in the repository CHANGELOG and crates.io. Adapter and
streaming-load acceptance remains a separate, ongoing verification effort.

## Usage status

The core API is published as [`thymeleaf`](https://crates.io/crates/thymeleaf).
Use `cargo add thymeleaf@0.2.0` for a registry dependency or Git checkout when reproducing
migration and Java parity tests; consult crates.io for the current published version.

## Contributing

The project currently welcomes implementation and verification work in these areas:

- HTML parser fidelity and source spans;
- immutable event/model representation;
- expression safety and Rust value access;
- processor and dialect contracts;
- full and streaming output semantics;
- framework adapter boundaries;
- upstream compatibility and test methodology.

Changes are expected to include documentation, tests, compatibility impact, and clear
dependency-direction checks.

## Security

Restricted expression contexts can block external access. Static class and method access follows the policies enforced by `ThymeleafACLClassResolver` and `OgnlRuntime`. Render application data with trusted templates.

Do not publish suspected vulnerabilities or sensitive proof-of-concept data in a public issue. A private reporting channel will be finalized before executable releases.

## License

This repository is licensed under the [MIT License](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/LICENSE).

Upstream-derived material remains subject to its original license and attribution requirements.

---

<div align="center">

[Back to top](#readme-top) · [Architecture](https://github.com/easy-4-rust/thymeleaf-rust/blob/v0.2.0/docs/superpowers/specs/2026-07-28-architecture-design.md) ·
[Issues](https://github.com/easy-4-rust/thymeleaf-rust/issues)

</div>
