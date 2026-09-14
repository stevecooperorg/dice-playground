# Application architecture

**High-level design and implementation constraints — approved.** [Design index](README.md)

## One engine, three consumers

The architecture keeps the meaning of a `.dice` document in a shared Rust engine. Three consumers use that engine:

1. the browser playground, compiled to WebAssembly;
2. the native `dice` command-line tool; and
3. the static documentation build, which invokes the command-line renderer.

The browser does not send a document to an application server for evaluation. Static publication happens at build time, not through a server-side rendering service. CDN or Cloudflare Workers hosting delivers the generated site and WASM assets.

This choice supports a forkable, statically deployable open-source application and avoids implementing one language for the website and another for automation.

## Boundaries matter more than the screen layout

| Area | Responsibility | Not its responsibility |
|---|---|---|
| `src/engine/` | Distributions, pool rules, the Starlark guest, document parsing, evaluation, diagnostic mapping, report formatting, and sanitization. | Leptos components, browser DOM manipulation, or UI state. |
| `src/ui/` | Editing, workspace interactions, Run wiring, diagnostics, tabs, browser state, and chart mounting. | Reimplementing probability rules or parsing a separate version of the literate language. |
| `src/bin/dice.rs` | Native commands that expose engine behaviour. | A competing evaluator or report formatter. |
| Documentation build | Select sources, invoke rendering, add navigation and shared assets, and assemble the static site. | Maintain copied lesson calculations or depend on a browser to render lesson prose. |

The direction of dependency is **UI → engine**, not engine → UI. In particular, the engine must not depend on Leptos, `web-sys`, or UI-only types. Browser-only chart lifecycle work stays outside it.

The recorded architecture continues the existing single Rust crate rather than introducing a new starter application or a service split. It identifies Rust 2021, Starlark 0.14, Leptos 0.8 CSR, Trunk, and Wrangler as the chosen stack. These are recorded implementation choices, not additional user-facing features.

## The document pipeline

```text
.dice source
    │
    ├── detect legacy script ────────────────┐
    │                                       │
    └── parse literate document              │
            └── tangle code + source map    │
                         │                  │
                         └──────┬───────────┘
                                ▼
                      expand dice/range notation
                                ▼
                       evaluate one Starlark module
                                ▼
                         structured output entries
                                │
                ┌───────────────┼──────────────────┐
                ▼               ▼                  ▼
           plain text      JSON / chart data    weave / format HTML
                                                   ▼
                                                sanitize
                                                   ▼
                                     browser fragment or static page
```

The document layer sits **around** the existing probability evaluator. Markdown prose is not added to the Starlark grammar. [Literate documents](literate-documents.md) describes detection, tangling, scope, and error locations; [the probability engine](probability-engine.md) describes evaluation.

## The engine-facing API

There is no HTTP API requirement. The browser and native tools make in-process calls.

The intent is one document-aware check/evaluation path that selects legacy or literate mode internally. The architecture proposed names such as `eval_document` and `EvalDocumentResponse`; the inspected implementation instead extends `eval_program` and `EvalProgramResponse`. The shared behaviour matters more than freezing a proposal’s name.

The successful response needs to preserve these distinct pieces:

| Information | Why keep it separate? |
|---|---|
| Structured `outputs` | Numerical truth for JSON, charts, and alternative renderers. |
| Plain `text` | CLI parity and copyable output, including for literate files. |
| Full `report_html` | Prose and fence-bound results for a literate document. |
| Outputs-only `outputs_html` | Formatted results for a legacy script with no prose. |
| Return value, where exposed | Existing script behaviour must not disappear during the report migration. |

The output-panel design adds `outputs_html` as an optional/defaulted serialized field, so older responses without it remain understandable. Literate responses need not duplicate the same output sections in both HTML fields. An explicit legacy/literate mode field is optional; it was not a requirement for telemetry collection.

Checks and failed evaluations also need useful diagnostics. Source locations should refer to the author’s document even if parsing or execution occurred on tangled and desugared code. A shared API does not justify exposing hidden intermediate line numbers.

## Markdown and sanitization are engine responsibilities

The chosen markdown renderer is **`pulldown-cmark` 0.13**, with default features disabled and the `html` feature enabled. The intended subset is CommonMark plus explicitly enabled GFM tables. This preserves the small, shared Rust path demonstrated by the original WASM spike.

The primitive renders headings and links, returns empty or whitespace-only HTML for empty input, and treats fenced code as displayed code when used on its own. Executing selected fences is the separate literate layer’s job.

The reasons for this choice are portability, a small API surface, and avoiding a second renderer with different behaviour in JavaScript or Pandoc. `comrak` was deferred because its larger extension set was not needed. A custom parser was reserved as a possible response to measured bundle pressure, not a preferred implementation. SIMD should not be enabled for WASM without testing it.

The markdown primitive is not itself the security boundary. **User-authored HTML and output labels are untrusted.** Report HTML must be sanitized in the engine before the browser receives it or the CLI writes it into a report. The recorded sanitizer choice is `ammonia` or an equivalent strict allowlist; the current dependency is `ammonia`.

The policy removes scripts, event handlers, and disallowed URLs. Output labels must be escaped as text, and attribute values must be escaped before constructing chart placeholders. Necessary structural, table, and chart attributes can be allowed deliberately. A report view should insert only engine-produced, sanitized HTML, not arbitrary editor contents. An iframe sandbox was not required by the output-panel design.

The complete raw-HTML and external-resource policy remains a [review topic](../requirements/review-notes.md#safety-and-operational-targets). Ordinary markdown image syntax does not imply an image-upload service.

## Size, responsiveness, and resource limits

The recorded limits distinguish document size from computational complexity:

| Limit or goal | Meaning |
|---|---|
| Literate source: **256 KiB** | Measured as UTF-8 bytes; prose needs more room than a short script. |
| Legacy source: **64 KiB** | Preserve the existing `MAX_SOURCE_BYTES` limit unless deliberately revised. |
| Output count | Retain the existing `MAX_OUTPUT_COUNT` guard; it is **500** in the inspected implementation. |
| Joint enumeration | Protect the engine from the combinatorial cost of large pools; see the mathematical notes. |
| Ordinary Run responsiveness | Millisecond-scale evaluation is an aspiration in the brief, not a quantified service-level target. |
| WASM download size | Measure release builds as literate rendering and sanitization are added. No accepted hard budget was supplied. |

A separate, stricter limit on tangled code was suggested but not fixed. It should not be invented during extraction. Likewise, “one Run is usually quick” does not eliminate the need for resource guards on hostile or simply oversized documents.

Bundle work should use release measurements, not the much larger debug build. [WASM bundle notes](../wasm-bundle-size.md) explain the measurement process. The old spike proved compilation; it did not measure the complete feature’s final download size.

## Command-line and editor paths

- **`dice eval`** accepts legacy or literate source, preserves text/JSON behaviour, and evaluates the whole document.
- **`dice render path.dice -o out.html`** evaluates and weaves a literate source into a full HTML page. The recorded design wraps the report fragment in a page shell linked to shared `tutorial.css` styling.
- **`dice lsp`** is the native stdio language-server path. The intended literate integration checks tangled code and maps diagnostics back; highlighting should distinguish prose and executable fences.
- Documentation and other existing CLI commands remain supporting tools, not casualties of the migration.

Literate-aware LSP behaviour is a requirement of the hardening phase, not a claim about the current LSP implementation. Full export details beyond HTML, and source/preview editor arrangements, remain open.

## Quality expectations

The retained engineering intent is modular, understandable, well-tested Rust. The project’s contribution standards supply the detailed rules: no unsafe production code, no production `unwrap()`/`expect()`, `anyhow` errors with context, and documented public APIs with runnable examples. Small focused modules are preferred to a monolithic parser or UI component.

Important validation follows the boundaries:

| Concern | Evidence to seek |
|---|---|
| Mathematics | Small enumerable examples, normalization and probability identities, failure paths, and approximation bounds. |
| Documents | Legacy regression fixtures, multiple fences sharing scope, exact source mapping, malformed fences, size limits. |
| Reports | Correct output placement, real HTML tables, safe label escaping, sanitizer tests, stable structured outputs. |
| Browser | Tab availability/defaults, chart cleanup across Runs, keyboard access, and WASM compilation. |
| Documentation | Every lesson and recipe evaluates unchanged; representative renders and static build succeed. |

The established commands are `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -Dwarnings`, and `make check` for the combined native gate. `make check-wasm` checks `wasm32-unknown-unknown --no-default-features`, excluding native CLI/LSP features. `make release-static` verifies the assembled static site and supports bundle measurement. Document approval does not certify a release build or deployment, and a successful compile alone is not evidence of complete browser behaviour.

## Why the migration was phased

The original dependency order remains useful design rationale, even though parts have since been implemented:

1. Prove the small markdown-to-HTML primitive on native and WASM.
2. Add parsing, tangling, source maps, legacy compatibility, and tests.
3. Add sanitized weaving and a CLI HTML renderer.
4. Add the report-oriented playground interface.
5. Publish one end-to-end pilot lesson before bulk conversion.
6. Convert the rest of the tutorial and cookbook, then remove duplicate lesson sources and Pandoc paths for that corpus.
7. Finish LSP integration, authoring guidance, and performance/bundle hardening.

The pilot protects against converting all content before the format, styling, diagnostics, and publication path agree. Temporary dual builds were migration mechanics, not a requirement to keep two lesson formats forever. The later unified Output panel supersedes the earlier plan for separate literate and legacy result layouts.

---

**Basis:** [source map](source-map.md), S01–S05, S07, S09–S13; implementation names and current constants from `playground.rs`, `Cargo.toml`, `Makefile`, and the project standards.
