# Review notes and open decisions

**Approved review record — 2026-09-14.** [Product reading guide](README.md) · [Technical design](../design/README.md)

The project owner approved all 11 extracted documents and authorised removal of the old BMad output. The open questions below are intentionally retained as open. Approval accepts the documentation, not a claim that every feature is implemented or a decision to implement every future idea.

## What is already clear in the sources

The repeated, explicit decisions are:

- one `.dice` document contains explanation and executable calculations;
- Run evaluates the entire document once, not individual cells;
- the report is the intended primary reading experience;
- existing plain-Starlark scripts remain compatible;
- the same engine serves browser, CLI, and static publication;
- tutorials and cookbook recipes are runnable literate source, not paired prose/script copies;
- the engine owns document processing and sanitization; the UI owns presentation and browser charts;
- the probability approach is deterministic distribution calculation, not production Monte Carlo; and
- static, open-source deployment remains the form factor, without a required application server or account service.

The current request emphasises casual TTRPG designers. The older brief had a wider audience including players, GMs, educators, and developers; the extraction retains them as supporting audiences without letting developer tooling become the product’s purpose.

## Product choices still open

| Topic | What the sources establish | What remains to decide |
|---|---|---|
| Sharing/export | CLI HTML rendering and readable, shareable analyses. | PDF-like output, one-click publishing, a browser export workflow, and any hosted sharing service. |
| Images | Markdown image references and a longer-term desire for explanatory images. | Uploads versus generated assets, where assets live, and how portable exports package them. |
| Workspace | A multi-file workspace exists and can remain during the transition. | Its longer-term role for snippets/libraries versus a more single-document-focused interface. |
| Editing layout | Source editing and a rendered report, with whole-file Run. | A unified visual editor, split view, source/preview toggle, and showing or collapsing executable source. |
| Success measures | Useful insight, repeat use, and shared recipes were suggested. | Actual metrics, targets, research evidence, and whether any instrumentation is wanted. |
| Positioning/business | MIT/open-source, no lock-in premise. | Growth, monetisation, and claims about competing tools were not validated or decided. |

These are not hidden requirements for collaboration, paid tiers, native mobile, or a general statistics platform. Those subjects remain outside the recorded scope unless deliberately revisited.

## Document rules

These are real ambiguities or incomplete contracts in the source set:

| Question | Evidence and consequence |
|---|---|
| **Fence whitespace and indentation** | The detailed format says to trim spaces only, its regex includes tabs, and the shorter contract invokes CommonMark while examples are flush-left. Confirm leading indentation, tab treatment, and any related line/column offsets. Do not infer all CommonMark fence forms are executable. |
| **Exact separator bytes when tangling** | The short specification says a single newline between bodies; the detailed algorithm adds one only when a body does not already end in one. Confirm the detailed rule or another explicit rule. The shared requirement—ordered code in one module—is not in doubt. |
| **Which fence owns a runtime output?** | The format permits positional provenance or a sequential strategy. Static source-call counts do not settle loops, conditionals, repeated calls, or a function declared in one fence and invoked in another. Specify these cases and test the chosen strategy. |
| **Duplicate output names** | “Every output appears once” coexists with “last wins for named binding”. Name-keyed chart hydration can pair the wrong chart with an earlier table. Decide whether names must be unique, produce warnings/errors, or require a separate stable identity. No new policy is imposed here. |
| **Source locations through desugaring** | The contract calls for original document lines and columns; the implementation must account for both tangling and notation expansion. The old plan leaves detailed LSP mapping work for later. |

The later bare-fence decision **does** settle one earlier question: both unlabelled fences and exact lowercase `dice` fences execute. That is not left open merely because the older brief discussed only `dice`-labelled examples.

## Result presentation

| Question | Recorded position |
|---|---|
| **Default tab for legacy outputs-only HTML** | Full literate `report_html` defaults to Report; text/JSON are fallbacks. Legacy HTML is supported, but the source algorithm names only `report_html` and also calls text the legacy default. Confirm whether legacy HTML changes that default. |
| **Empty graph tabs** | The design says to omit/disable unavailable tabs, yet also describes an empty graph message. Choose a consistent small UI rule. |
| **Probability columns** | Teaching reports default to multiple representations. A selected single-column format is designed, but the flag/control and its interaction with default `ProbFormat` are not fully specified. |
| **Chart threshold** | Default maximum of 32 rows for probability-table charts. Whether users/options can change it is open. |
| **Graph gallery after inline charts** | Retain it for now in the extracted design. Hiding it, showing only unmatched outputs, or renaming it “All charts” is later work. |
| **Report representation** | The later chart design uses HTML placeholders plus a parallel structured output list, without inline JSON. A structured segment API is only a deferred alternative. |

The main settled correction is the **unified Output panel**. It supersedes the older plan that separated literate Report/Graph regions from legacy text/JSON/graph tabs.

## Safety and operational targets

- **Meaning of exact:** the approved wording distinguishes deterministic finite calculation, `f64` precision, reconstructed display fractions, and capped infinite mechanics. The old marketing shorthand alone is not a mathematical guarantee.
- **Latency:** “milliseconds” has no workload, hardware, percentile, or accepted threshold. Preserve it as a responsiveness goal until a measurable target exists.
- **Bundle size:** release measurement is required by the architecture, but there is no approved final WASM budget. Numbers and optional thresholds in existing bundle guidance are not newly accepted limits.
- **Resource bounds:** source size and an output-count limit are established, as is a selected-path enumeration guard. A tangled-code limit, global instruction/memory budget, and consistent guard coverage across entry points are not fully specified or proved by the current code.
- **Sanitization:** engine-side sanitization and rejection of executable markup are required. The complete allowed-HTML, URL-scheme, external-image, and asset policy needs an explicit definition if richer content is offered. Allowing the chart’s structural attributes is a narrow decision, not permission for arbitrary HTML or inline styles.
- **WASM evidence:** the old spike records successful compilation. It does not prove a later dependency version’s browser behaviour or the final download size; check the current build when implementation work is undertaken.

## Deferred extensions, not initial-format requirements

The source material explicitly defers or merely suggests:

- prose output placeholders, with `{{output "name"}}` reserved for a later format;
- YAML front matter, per-fence echo/hidden metadata, callouts, math, footnotes, and a full Pandoc/GFM feature set beyond the explicitly adopted tables;
- image creation/upload/storage workflows and PDF-like export;
- markdown CLI output or replacing the plain-text formatter;
- two-dimensional pivot tables and optional HTML row caps for very large tables;
- arrow-key tab navigation, tab persistence across sessions, and per-tab copy buttons;
- static-page chart hydration or generated PNG/SVG charts; and
- structured report segments if string HTML plus chart mounting proves awkward.

Earlier research explored Noweb syntax, directive-based prose regions, prose-in-Starlark helper functions, alternate fence tags, and immediate prose placeholders. Those were alternatives, not additional capabilities to implement. The chosen path is markdown-first with legacy fallback, automatic per-fence output placement initially, and a shared Rust renderer.

## Implementation observations

These are evidence-backed observations at source baseline `2d00070`, not a comprehensive audit or a new set of accepted fixes. The technical documents give the relevant paths.

| Area | Observation | What not to claim |
|---|---|---|
| Literate corpus and static build | 13 tutorial and 10 cookbook `.dice` sources exist; the build uses `dice render`/`render-md`, not Pandoc. | The old “today we have separate markdown and scripts” baseline is still current. |
| Report and chart support | Report fields, output panel, HTML tables, chart eligibility, and report-host code exist. | An “implemented” heading or an unchecked old task list proves all acceptance criteria. |
| Output placement | The weave allocates global outputs using static source-call counts and gives leftovers to the final fence. | Arbitrary dynamic outputs have precise call-site ownership. |
| Source handoff | Static links carry individual code-block snippets, not a whole literate lesson; long-source handoff is not complete. | A lesson’s explanations and cross-fence definitions are preserved by existing links. |
| In-file copies | Some lessons have display-only code and a separate executable copy in the same source. | Corpus execution tests prove those copies agree. |
| Documentation navigation | Some guide/reference links point to markdown destinations not emitted by the site build. | Rendering smoke tests establish that all deployed links work. |
| LLM reference | `llms.txt` is manually maintained; some shorthand pool/total wording needs checking. | Generated API-reference equality also validates the LLM reference. |
| LSP | The inspected native LSP parses source as Starlark without the full literate preparation path. | Literate source mapping and highlighting are finished. |
| Limits | CLI rendering shares lower-level engine functions but bypasses the playground response wrapper; the output-count guard is after evaluation, and joint-enumeration guards are not global. | All callers have identical resource protection or every expensive computation is bounded. |
| Numerical precision | Numeric PMFs use `f64`; explosions/open-ended helpers are finite models. | Symbolic exact arithmetic or complete infinite distributions are implemented. |

The detailed source map distinguishes extracted decisions from this supplemental implementation evidence.

## Shared-lexing follow-up (after the extraction baseline)

The original implementation observations above are historical, not overwritten by later work. The highlighting-first simplification now shares native Starlark tokenization, dice/band recognition, source preparation, and mapped primary diagnostics across the public check/eval/LSP paths. The supervisor approved a narrow tangle correction: append extracted bodies once, add separators only after nonempty code lacking a newline, and keep line maps aligned with emitted text. Static rendering shares that preparation without changing report binding.

Two limitations remain deliberate:

- **Document detection precedence:** fence detection still runs before native Starlark lexing. Executable fence lines inside an otherwise valid triple-quoted legacy string can select literate mode. This is recorded by a regression, not silently changed; resolving legacy-string versus markdown-fence precedence needs a format decision.
- **LSP AST features:** transformed/invalid buffers receive mapped parse/lint diagnostics but an inert comment-only AST to invalidate the upstream last-valid-AST cache. No complete literate/shorthand hover, navigation, or contextual completion is claimed. Unchanged successfully parsed Starlark uses its original AST. Secondary locations embedded in upstream diagnostic message prose are not separately mapped.

The context-sensitive pool-versus-sum policy also remains compatibility debt, not a newly endorsed semantic design. See [the current pipeline](../design/architecture.md#shared-lexical-preparation-current-implementation).

## Approval checklist

- [x] The [product purpose](product.md) represents what Dice Playground is for and the intended casual-designer audience.
- [x] The [experience](experience.md) describes the desired workflow without promising unsettled features.
- [x] The [mathematical explanation](../design/probability-engine.md) states the supported model and its precision/approximation limits accurately.
- [x] The [document](../design/literate-documents.md), [report](../design/reports-and-visualisation.md), and [architecture](../design/architecture.md) notes preserve the important design decisions.
- [x] The [learning and LLM notes](../design/learning-and-llm.md) preserve the single-source documentation goal and distinguish external drafting from embedded AI.
- [x] Unresolved questions are intentionally retained as open.
- [x] The [source map](../design/source-map.md) accounts for all original output documents and identifies additional code/reference evidence.

## Checks performed for this extraction

- Before removal, all 13 original output files were checked against the original source baseline and were unchanged.
- Local links, heading anchors, and code-fence structure were checked across the 11 new documents.
- The mathematical examples were checked through the CLI, including independent addition versus scaling, advantage, success counts, conditioning, and a capped exploding die. The two-fence document example evaluates to 45% and renders a heading and HTML table.
- Native tests passed: 286 passed and one ignored. Clippy with warnings denied also passed.
- The extraction’s `make check` run stopped at its final formatting check because of existing Rust formatting differences. The affected implementation files had not changed from the inspection baseline; they were deliberately not reformatted.
- No deployment, full release-site build, or interactive browser validation was performed for this extraction.

## Approval and cleanup

On 2026-09-14, the project owner approved all extracted documents and requested removal of the old BMad output.

- `docs/requirements/` and `docs/design/` are now the maintained product and technical documentation.
- `_bmad-output/` and its 13 source documents have been removed. The originals remain available in Git history at commit `2d00070`, with their historical paths recorded in the source map.
- The module documentation in `src/engine/literate/mod.rs` now points to `docs/design/literate-documents.md`.
- Historical source names are retained for provenance only; there is no live application dependency on the deleted output.
- Existing project standards and unrelated files are retained. Installation and skill removals occurred separately; remaining `.github/agents/bmad-*.agent.md` wrappers are outside this output-only cleanup.
- After cleanup, all 101 local documentation links/anchors passed validation, native tests passed (286 passed, one ignored), Clippy passed, and `make check-wasm` passed. `make check` still stops only at the pre-existing Rust formatting differences.

These documents are linked from the repository README. Approval and cleanup do not automatically publish them through the tutorial-site build.
