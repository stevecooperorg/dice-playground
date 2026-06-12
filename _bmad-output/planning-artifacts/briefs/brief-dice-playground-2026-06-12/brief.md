---
title: "Product Brief: Dice Playground"
status: draft
created: 2026-06-12
updated: 2026-06-12
source: inferred-from-repository
spike_refs:
  - ../implementation-artifacts/spec-wasm-markdown-html-spike.md
  - ../planning-artifacts/research/technical-prose-encoding-in-dice-files-research-2026-06-12.md
---

# Product Brief: Dice Playground

## Executive Summary

Dice Playground is a browser-first tool for tabletop gamers, GMs, and designers who need **exact** probabilities for dice mechanics—not estimates from rolling thousands of times. Users work in **`.dice` documents** that combine familiar notation (`2d6`, `4d6dl1`, advantage pools) with **Starlark** for modifiers, loops, and multiple outputs. The engine runs entirely in the browser (WASM) and computes exact distributions; tutorials and cookbooks ground the language in real games (D&D 5e, PbtA, Blades, and others).

**Today** the product feels like a small **IDE**: script files in a workspace, a code editor, and **terminal-style output** in a separate panel (text, JSON, or graph tabs). **The intended end state** is **literate programming in a single `.dice` file**: prose and code live together; **Run** evaluates the **whole document in one shot** (milliseconds—no per-cell stepping) and produces a **report** with **inline formatted output** (text, tables, charts, eventually images). Presentation is notebook-like; execution is **knit-once**, closer to rendering a Quarto/R Markdown document than stepping through Jupyter cells.

The core problem unchanged: “how likely is this?” at the table and in design. The product bet is **clarity and correctness** (exact math, tabletop-shaped language) delivered through a **narrative, shareable document** users can read top to bottom like an analysis, not only run like a script. CLI and LSP remain the power-user path; the playground becomes the place you **publish understanding**, not just execute it. Open source (MIT), static deploy, live public instance.

## The Problem

Tabletop play and design constantly require probability questions: chance to hit a DC, distribution of ability scores, partial success bands on 2d6, exploding dice, save-for-half on a fireball, grids of success rates across modifiers. Wrong intuitions change character builds, encounter difficulty, and homebrew balance.

People also struggle to **communicate** their analysis: a raw probability dump does not explain *why* the mechanic was modeled that way or *what changed* when you tweak a modifier. Spreadsheets and one-shot calculators answer a number but not a **story** GMs and designers can share with a table.

Today people cope by:

- **Mental math and folklore** — fast but often wrong on non-obvious pools and keep/drop rules.
- **Monte Carlo simulation** — easy to misuse and tedious to explore many outputs.
- **Legacy web calculators** — strong for expressions, weak for step-by-step explanation and rich reports.
- **Notebook workflows in Python/R** — flexible reports, but simulation-heavy and not built for `.dice`-style exact tabletop mechanics out of the box.

## The Solution

### North star: literate single-file `.dice`

**One file** holds everything—narrative and executable content interleaved in `.dice` syntax (not a sidecar markdown file or JSON notebook). The user edits the source view or a unified document surface; **one Run** re-evaluates the **entire** script and refreshes the **rendered report**: prose blocks stay prose; each code region’s **outputs appear inline** where the language places them (formatted text, tables, charts, images). No run-this-cell-only workflow—the engine is fast enough that whole-document execution is the only model.

### Current experience (stepping stone)

- Multi-file **workspace** with filenames, monospace **editor**, **Run** / Shift+Enter.
- **Diagnostics** and a separate **output** region with text / JSON / graph tabs—useful but **decoupled** from the narrative flow of the script.
- Tutorial and cookbook as **external** markdown with “load in playground” links.

The gap between today and north star is intentional product direction: the IDE layout validates the engine and language; the notebook/report UX is the **primary UX bet** going forward.

### Supporting pieces (retained)

- **Learning path**: tutorial and cookbook as **literate `.dice` documents**—the same artifact users read, run in the playground, and ship in CI (not parallel markdown plus copied script blocks).
- **`llms.txt`** for drafting scripts from plain-language rules, then refining in a literate document.

Outcome: not a dice roller and not a generic IDE—a **reliable odds notebook** you can hand to someone else and they understand both the math and the modeling choices.

### Literate encoding and markdown (WASM)

Prose and code share one **markdown-first `.dice` file**: narrative is CommonMark-style markdown; executable regions use fenced **`dice` code blocks**. Processing is **Rust in `src/engine/`**, shared by **WASM playground**, **CLI**, and **static docs build**—not Pandoc inside the browser.

| Phase | Role |
|-------|------|
| **Tangle** | Extract fenced Starlark, concatenate, source-map for diagnostics |
| **Eval** | Existing pipeline: desugar → Starlark → `output(...)` (one shot, milliseconds) |
| **Weave** | Markdown prose → HTML; inject formatted outputs at placeholders / fences |

**Proven (spike):** **`pulldown-cmark`** (minimal features + `html`) on **`wasm32-unknown-unknown`** via `engine::markdown_to_html`. Native `dice eval` / future `dice render` and in-browser **Run** use the same weave primitive. **Still to build:** sanitization before DOM display, tangle orchestration, playground report UI.

**Docs:** Tutorial/cookbook **`.dice` files** eval as-is; lesson HTML is **woven at build time** (CLI). Reading `/docs/` is static HTML; **Run** in the playground re-weaves the same source in WASM.

## What Makes This Different

- **Exact enumeration, not sampling** — trustworthy for edge-heavy mechanics.
- **`.dice` as a tabletop-shaped language** — notation plus Starlark for pools, labeled outcomes, and tables of checks.
- **Literate analysis as the destination** — [ASSUMPTION] few tabletop odds tools treat the artifact as a **readable report**; most stop at calculator or REPL output.
- **Player-first learning** — tutorials that read like walkthroughs, eventually native to the notebook surface.
- **One engine, WASM + CLI + docs** — literate weave/eval logic lives in `src/engine/`, not duplicated in JS or host-only Pandoc for lessons.
- **Honest positioning** — differentiation is exact engine + narrative document experience + docs/recipes, not proprietary data or lock-in.

## Who This Serves

**Primary:** Tabletop **players and GMs** who want trustworthy odds **and** a clear explanation they can revisit or share—especially when mechanics are tweaked across sessions.

**Secondary:** **Game designers** documenting homebrew balance; **educators** teaching probability through games; **developers** on the OSS stack; **LLM-assisted authors** drafting chunks into a literate `.dice` flow.

Success for a primary user (north star): open one `.dice` file, Run once, read top to bottom as a **finished report** with results woven into the narrative—not a separate terminal output pane.

## Success Criteria

**Today (baseline):**

- Engine + playground deliver exact results; IDE-style run loop works; tutorial/cookbook coverage is broad; CI on examples.

**North star (product):**

- **Single `.dice` file** literate format (prose + code regions) with **one-shot Run** and **inline rendered output**.
- Full-document eval completes in **milliseconds** under normal scripts (no cell-level execution UI).
- Reports include **formatted text and visualizations**; **images** supported where they aid explanation.
- Tutorial/cookbook content **is** literate `.dice`—**executable as-is** via Run / `dice eval` (woven HTML on the docs site is a **render** of the same source, not a separate prose format).
- **Markdown → HTML weave** runs in the shared engine on **wasm32** (spike validated); release wasm size tracked per `docs/wasm-bundle-size.md` as literate features land.
- Share/export: [ASSUMPTION] static HTML or PDF-like share of a rendered report is a key success signal for designers/GMs.

**Still [ASSUMPTION] until measured:** time-to-first-insight, return visits, community-shared notebook recipes.

## Scope

**In — now:**

- WASM playground, workspace files, editor, separate output panel (text/json/graph), diagnostics, docs site, CLI/LSP.
- **Spike landed:** `engine::markdown_to_html` via **pulldown-cmark**; `make check-wasm` passes with dependency linked.

**In — direction (explicit product goal):**

- Literate **markdown-first** `.dice` (fenced `dice` blocks + prose); **tangle + weave + eval** orchestration in engine.
- **Whole-document** eval on Run (unchanged).
- Rich **report rendering** (not terminal-only); charts as first-class inline artifacts; path to **images**.
- **HTML sanitization** policy for user prose before browser display (follow-on to markdown weave spike).
- Engine/output schema that maps named outputs to **regions in the literate document** for rendering.
- **Tutorial and cookbook** authored as literate `.dice` under `docs/` (or a single tree); **no duplicate** markdown lesson + `examples/*.dice` script that can drift. Static site generation **weaves** those files; CI **evals** them unchanged.

**Out (explicit):**

- **Per-cell or partial execution** (run one chunk, run to cursor, reactive re-run on edit)—whole file only.
- Sidecar formats (`.dice` + `.md`), JSON notebook files, or split prose/code artifacts.
- **Pandoc/JS markdown in the UI** for literate weave (host Pandoc may remain temporarily for legacy `.md` until migration).

**Out (unchanged unless revisited):**

- Hosted multi-user accounts and real-time collaboration (notebook products often add this later; not stated as v1 north star).
- General statistical computing beyond tabletop dice.
- Native mobile apps.
- [ASSUMPTION] Paid hosting/tiers not stated.

## Vision

Dice Playground becomes the **default exact-odds notebook** for tabletop: documents you link in session prep, design blogs, and rule debates. **Tutorial and cookbook entries are literate `.dice` files**—run them in the playground or CLI without copy-paste. The docs site is the woven view of that same corpus. The CLI/LSP serve automation and CI; the playground serves **human-readable probability stories**. The project stays forkable and statically deployable while sharing one `.dice` language for lessons, recipes, and user scripts.
