---
id: SPEC-literate-dice
companions:
  - literate-dice-format.md
  - ../planning-artifacts/architecture.md
  - ../planning-artifacts/literate-dice-migration-plan-2026-06-12.md
sources:
  - ../planning-artifacts/briefs/brief-dice-playground-2026-06-12/brief.md
  - ../planning-artifacts/briefs/brief-dice-playground-2026-06-12/addendum.md
  - ../planning-artifacts/research/technical-prose-encoding-in-dice-files-research-2026-06-12.md
  - ../implementation-artifacts/spec-wasm-markdown-html-spike.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate.

# Literate `.dice` (Dice Playground)

## Why

Dice Playground must move from an IDE with split markdown/scripts to **one executable document** that teaches and computes exact tabletop odds. The force is a **vision to realize**: literate `.dice` files are the same artifact in the playground, CLI, CI, and static docs—prose and code together, one Run, inline report. Affected: players, GMs, tutorial authors, and implementing agents that must not diverge on format or pipeline.

## Capabilities

- id: CAP-1
  intent: Author and store tutorial, cookbook, and user mechanics in a **single `.dice` file** mixing markdown prose and executable Starlark.
  success: A file matching `literate-dice-format.md` parses as literate; the same bytes pass `dice eval` and playground Run without a separate script copy.

- id: CAP-2
  intent: Evaluate a literate document **in one shot**—all executable fences tangled into one Starlark module, then exact probability eval.
  success: Two fences in one file share scope; `output()` from the second fence sees bindings from the first; no per-cell run API exists.

- id: CAP-3
  intent: Produce a **woven report** (HTML fragment) from prose plus eval outputs for WASM, CLI render, and static docs.
  success: After eval, weave output includes rendered markdown prose and output blocks bound per format rules; HTML is sanitized before any UI DOM use.

- id: CAP-4
  intent: Run **legacy** pure-Starlark `.dice` files unchanged when literate markers are absent.
  success: Existing `examples/tutorial/*.dice` and scripts without executable fences (bare ` ``` ` or ` ```dice `) behave identically to today’s `eval_program`.

- id: CAP-5
  intent: Map Starlark diagnostics from tangled lines back to **source document** lines for literate files.
  success: A deliberate syntax error in fence line N reports diagnostic line/column referring to the `.dice` source, not tangled-only line.

## Constraints

- Literate detection requires at least one **executable fence**: opening line is bare ` ``` ` (empty info string) **or** info string exactly **`dice`** (case-sensitive, CommonMark rules). Empty info string **defaults to dice** for tangle and weave.
- Tangle concatenates fence bodies in **document order** with a single newline between bodies; one Starlark module per Run.
- Markdown weave uses **`pulldown-cmark` 0.13** (`html` feature) via `engine::markdown_to_html`; no Pandoc or JS markdown in the engine/UI pipeline.
- Literate document size limit **256 KiB** (UTF-8 bytes); existing output-count and eval guards remain.
- Literate logic lives under **`src/engine/literate/`**; engine must not depend on Leptos or `web-sys`.
- HTML from weave must pass **engine-side sanitization** before the playground displays it.
- Output binding **v1** only: render eval results **immediately below each executable fence** in weave order; placeholder syntax in prose is **not** v1.

## Non-goals

- Per-cell, run-to-cursor, or reactive re-execution.
- Sidecar files (`.dice` + `.md`), JSON notebook formats, or Jupyter-style cell metadata.
- YAML front matter on `.dice` in v1.
- Full Pandoc/GFM feature set (callouts, math, footnotes) unless explicitly added later.
- Server-side rendering, accounts, or collaborative editing.

## Success signal

A pilot literate lesson (e.g. tutorial lesson 1) exists only as **`docs/tutorial/…/*.dice`**: CI evals it; `dice render` emits HTML included in `dist/`; playground Run shows the same woven report from the same source. Legacy scripts without fences still pass existing CI unchanged.

## Assumptions

- Bare fenced code openers (` ``` ` with no language tag) are **dice** for detection, tangle, and output binding—authors need not write ` ```dice `.

## Open Questions

- Confirm **`ammonia`** on `wasm32-unknown-unknown` at integration time (architecture assumes yes; verify in Phase 2).
- Graph output v1: **`data-dice-output` placeholder** in weave vs parallel `outputs` map only for UI (architecture allows either; pick one in implementation).
