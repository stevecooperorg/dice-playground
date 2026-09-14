# Source map for the extracted documents

**Preservation record — approved on 2026-09-14.** [Product requirements](../requirements/README.md) · [Technical design](README.md)

## Scope and method

All **13 files** formerly in `_bmad-output/` were read, including both hidden `.decision-log.md` files. The project owner approved the extracted documents and removal of the original output on 2026-09-14. The baseline for supplemental implementation inspection was `main` at `2d00070`.

Historical paths below are provenance, not live links or dependencies. The original files remain available in Git history at commit `2d00070`; use `git show 2d00070:<historical-path>` to inspect one after cleanup. The maintained design no longer needs the old output directory or an installed workflow system.

The extraction uses these rules:

1. Preserve explicit product decisions, including their rationale and non-goals.
2. Let specific companion designs refine earlier broad plans where that refinement is clear.
3. Do not treat research alternatives, assumptions, or old “today” descriptions as settled requirements.
4. Keep genuine conflicts in the [review notes](../requirements/review-notes.md), rather than choosing a new design during extraction.
5. Distinguish desired behaviour from source code observed today and from historical claims that something was tested.

The first brief was inferred from the repository, not based on a formal PRD, user interview, market sizing, or established business metrics. Its decision log then records owner decisions about the report direction, single file, whole-document Run, and literate tutorials. The current request’s focus on casual TTRPG designers is an explicit audience refinement.

## Original-output inventory

### S01 — Product brief

`_bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/brief.md`

Preserved in [product purpose](../requirements/product.md), [experience](../requirements/experience.md), [architecture](architecture.md), and [learning](learning-and-llm.md): exact-odds purpose, audience, report-oriented direction, one-shot execution, single-source lessons, browser/CLI/static parity, images/sharing ambitions, scope, and non-goals. Unmeasured success criteria and business/market assumptions remain labelled as such in [review notes](../requirements/review-notes.md).

### S02 — Product addendum

`_bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/addendum.md`

Preserved in [experience](../requirements/experience.md), [documents](literate-documents.md), [architecture](architecture.md), and [learning](learning-and-llm.md): decided document/execution model, shared markdown pipeline, static docs, current-UI versus intended-UX distinction, mechanic coverage, and unresolved workspace, images, export, and positioning questions. Historical “still to build” descriptions are not treated as current status.

### S03 — Product decision log

`_bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/.decision-log.md`

Preserved in the product’s [guiding choices](../requirements/product.md) and this provenance record: the brief’s inferred origin; the shift from an IDE endpoint to readable reports; then explicit single-file, whole-document, and executable-tutorial decisions. The initial uncertainty about execution and encoding is superseded by those recorded decisions. Workflow completion markers are not application requirements.

### S04 — Architecture decisions

`_bmad-output/planning-artifacts/architecture.md`

Preserved in [architecture](architecture.md), [documents](literate-documents.md), and [reports](reports-and-visualisation.md): D1–D12, layer boundaries, native/WASM/static consumers, stack and library choices, orchestration data, source maps, sanitization, size limits, output binding, chart separation, tests, and implementation dependency order. Proposed API names are distinguished from current names. The detailed coverage table below maps all twelve decisions.

### S05 — Migration plan

`_bmad-output/planning-artifacts/literate-dice-migration-plan-2026-06-12.md`

Preserved in [architecture’s phased approach](architecture.md#why-the-migration-was-phased) and [learning](learning-and-llm.md): compatibility first, engine before UI/content, pilot before bulk conversion, temporary dual builds, CLI/static parity, complete-source playground handoff, corpus evaluation, removal of duplicate lesson sources/Pandoc paths, LSP/LLM hardening, and bundle measurement. Roles, sprint labels, and workflow invocations are excluded. The old split-UI plan is superseded by S10.

### S06 — Technical research on prose encoding

`_bmad-output/planning-artifacts/research/technical-prose-encoding-in-dice-files-research-2026-06-12.md`

Preserved in [documents](literate-documents.md), [architecture](architecture.md), and [review notes](../requirements/review-notes.md): why markdown-first plus legacy fallback fits the product; why tangling precedes Starlark; source-map needs; library trade-offs; and why Noweb, directives, prose helper calls, and JSON notebooks were not the chosen path. Research suggestions for immediate placeholders, alternate fence labels, and a Pandoc-flavoured surface are superseded or narrowed by the later format. External comparisons are rationale, not product promises.

### S07 — Literate capability specification

`_bmad-output/specs/spec-literate-dice/SPEC.md`

Preserved in [documents](literate-documents.md) and the [experience review scenarios](../requirements/experience.md#review-scenarios): CAP-1 single executable document; CAP-2 one module/whole-file execution; CAP-3 sanitized woven reports; CAP-4 legacy compatibility; CAP-5 original-source diagnostics. Constraints, non-goals, the pilot success signal, and the initial sanitizer/chart-binding uncertainties are retained or refined by later companions.

### S08 — Literate decision log

`_bmad-output/specs/spec-literate-dice/.decision-log.md`

Preserved in [mode detection](literate-documents.md#file-identity-and-mode-detection) and [fence rules](literate-documents.md#which-code-blocks-execute): the explicit decision that a bare fence defaults to dice, equivalence with the lowercase tag, and the potential legacy-detection risk. Its earlier preservation/self-validation verdict is historical process evidence, not proof that this extraction or the implementation is complete.

### S09 — Literate format v1

`_bmad-output/specs/spec-literate-dice/literate-dice-format.md`

Preserved in [documents](literate-documents.md): UTF-8, mode detection, backtick fences, case sensitivity, non-executable/inline code, front-matter exclusion, tangle bodies/metadata/maps, shared scope, output ordering and duplicate names, weave fragments, markdown subset, sanitization, 256/64 KiB limits, consumer matrix, examples, and versioning. Newline, whitespace, and binding ambiguities are explicit in [review notes](../requirements/review-notes.md#document-rules). Later table/panel/chart companions are explained in the reports document rather than left as missing links.

### S10 — Unified Output panel design

`_bmad-output/specs/spec-literate-dice/output-panel-html-tab.md`

Preserved in [experience](../requirements/experience.md) and [reports](reports-and-visualisation.md): one panel, tab names/order, full versus outputs-only HTML, text/JSON preservation, successful-Run defaults/reset, empty/error states, security boundary, accessibility, shared styling, and optional/defaulted response field. Old UI section-removal tasks are expressed as intended behaviour, not a new implementation checklist. Unclear legacy default and empty-graph policy remain open.

### S11 — GFM table output design

`_bmad-output/specs/spec-literate-dice/tabular-output-gfm.md`

Preserved in [report tables](reports-and-visualisation.md#real-tables-in-html-plain-text-in-terminals): semantic HTML tables through a shared GFM builder, unchanged structured data and compression, all output kinds, captions/mean, teaching versus selected-format columns, sample-space headers, escaping, readable CSS, plain-text compatibility, and tests. Markdown CLI mode, pivots, and optional row caps remain deferred. An “implemented” heading does not settle incomplete column-option details.

### S12 — Inline charts design

`_bmad-output/specs/spec-literate-dice/graphs-in-report-html.md`

Preserved in [inline charts](reports-and-visualisation.md#inline-charts-chart-first-table-second): chart above table per output; eligible kinds; 32-row table threshold; full rather than compressed PMFs; stable name/kind attributes; parallel structured data rather than inline JSON; sanitizer allowances; UI mounting and cleanup; legacy parity; empty static placeholders; retained graph gallery; test expectations; and deferred structured-segment/static-chart alternatives. Status remains a recorded design, not an inferred blanket completion claim.

### S13 — WASM markdown spike

`_bmad-output/implementation-artifacts/spec-wasm-markdown-html-spike.md`

Preserved in [architecture](architecture.md#markdown-and-sanitization-are-engine-responsibilities): `pulldown-cmark` 0.13, minimal features, engine-only API, native and WASM compilation intent, rejected/deferred renderer alternatives, ordinary heading/link/empty/fenced-code behaviour, and avoiding untested SIMD expansion. The spike records successful tests and a WASM check, but explicitly lacks the final release-size measurement. Its temporary “no full tangle/UI work in this spike” scope is not a permanent product non-goal.

## Architecture-decision coverage

| Original decision | Preserved meaning | Destination |
|---|---|---|
| D1 | Markdown prose with bare or `dice` executable fences. | [Document format](literate-documents.md) |
| D2 | Parse → tangle → desugar → evaluate → weave. | [Architecture pipeline](architecture.md#the-document-pipeline) |
| D3 | Executable-fence detection; case-sensitive exact tag. | [Mode detection](literate-documents.md#file-identity-and-mode-detection) |
| D4 | Shared `pulldown-cmark` renderer with minimal features. | [Markdown implementation](architecture.md#markdown-and-sanitization-are-engine-responsibilities) |
| D5 | Engine-side HTML sanitization. | [Security boundary](architecture.md#markdown-and-sanitization-are-engine-responsibilities) |
| D6 | Preserve legacy full-file evaluation. | [Compatibility](literate-documents.md#compatibility-and-limits) |
| D7 | One document-aware orchestration API; proposal names need not freeze implementation. | [Engine API](architecture.md#the-engine-facing-api) |
| D8 | Initial outputs below their contributing fences. | [Output placement](literate-documents.md#output-placement-and-identity) |
| D9 | Prose placeholders deferred, not v1. | [Deferred placeholders](literate-documents.md#deferred-prose-placeholders) |
| D10 | CLI HTML page shell and shared stylesheet. | [CLI path](architecture.md#command-line-and-editor-paths) |
| D11 | 256 KiB literate source; retain existing guards; optional tangled limit unsettled. | [Limits](architecture.md#size-responsiveness-and-resource-limits) |
| D12 | Engine chart placeholders/data, UI-owned drawing. Later companion chooses parallel data rather than inline JSON. | [Inline charts](reports-and-visualisation.md#inline-charts-chart-first-table-second) |

## Supplemental repository evidence

These sources fill gaps in the old output, especially the mathematical core. They are not silently reclassified as old product decisions.

| ID | Evidence | Use in the extraction |
|---|---|---|
| **R01** | `src/engine/die_roll.rs`, `dice_pool.rs`, `enumerate.rs`, `core.rs`, `ordinal.rs`, `int_band.rs`, `face_spec.rs`, `poly_explode.rs`, and their unit tests. | PMFs, floating-point representation, independence, convolution, transformations, conditioning, pool reduction, ordered labels, finite explosions, cost/limits, and mathematical test identities. |
| **R02** | `src/engine/starlark_guest/eval.rs`, value bindings, `output_format.rs`, `src/engine/sugar.rs`, and `range_sugar.rs`. | Script semantics, distribution reuse, concrete helpers and defaults, context-sensitive notation, inclusive ranges, callbacks, outputs, and reconstructed fraction formatting. |
| **R03** | `llms.txt`, `docs/README.md`, `docs/references/api-conventions.md`, `docs/references/stdlib.md`, `docs/tutorial/*.dice`, and `docs/cookbook/*.dice`. | Teaching concepts, concrete corpus coverage, authoring workflow, external LLM use, and documentation drift observations. |
| **R04** | `src/engine/playground.rs`, `literate/`, `markdown_html.rs`, `html_sanitize.rs`, `output_html.rs`, `output_chart.rs`, `markdown_page.rs`; `src/ui/app.rs`, `eval_client.rs`, `output_panel.rs`, `report_html_host.rs`, `output_graph.rs`, `static_site.rs`, `playground_handoff.rs`, and `storage.rs`; `src/engine/lsp.rs`. | Current orchestration, heuristic fence binding, sanitized reports, chart architecture, tab data, source handoff gaps, and incomplete literate LSP path. |
| **R05** | `bin/build-tutorial-site.sh`, `Makefile`, `src/bin/dice.rs`, `src/engine/starlark_guest/docs.rs`, and integration tests under `tests/`. | Static build, generated-reference process, corpus evaluation, render/check commands, test scope, and distinction between native smoke tests and browser execution. |
| **R06** | `README.md`, `docs/AGENT.md`, `docs/wasm-bundle-size.md`, deployment guides, `Cargo.toml`, `LICENSE`, and `.agents/skills/dice-playground-standards/`. | Existing stack, static deploy, MIT licence, modularity, explanatory documentation, quality expectations, and release measurement. |

The additional mathematical formulas and small worked results are explanations derived from these operations. They are not evidence that a symbolic engine, automatic approximation error reporting, or every suggested test already exists.

## Material deliberately not promoted into requirements

- Workflow front matter, completed-step markers, skill commands, agent personas, handoff boilerplate, and sprint ownership.
- Temporary spike boundaries that would contradict the subsequent literate implementation if treated as permanent.
- Old “today” descriptions of Pandoc, split lesson files, or detached panels where later decisions or current code show progression.
- Unvalidated business metrics, market differentiation, growth plans, monetisation, and PDF-sharing assumptions.
- Research alternatives and future extensions that the final format narrowed or deferred.
- Historical self-validation or test-pass labels as evidence of current correctness.

The initial repository inventory also found `_bmad/` configuration/runtime material, BMad skills, and GitHub agent wrappers; no separate `_bmad_method` directory was found. These are tooling, not application requirements. Installation and skill removals occurred separately from this extraction. The approved cleanup removes the old output, not the remaining GitHub agent wrappers or the independent project standards.

The former code reference to an old output file in `src/engine/literate/mod.rs` now points to the maintained document design. See the [approval and cleanup record](../requirements/review-notes.md#approval-and-cleanup).
