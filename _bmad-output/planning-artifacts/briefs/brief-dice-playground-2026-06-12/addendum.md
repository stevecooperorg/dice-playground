# Addendum — Dice Playground

Detail for PRD, UX, and architecture passes.

## Decided: document and execution model

| Decision | Choice |
|----------|--------|
| **Artifact** | Everything in a **single `.dice` file** (prose + executable content; no sidecar, no `.dicenb`) |
| **Execution** | **One-shot** — Run evaluates the **entire** file; no per-cell or partial run UI |
| **Rationale** | Exact eval is **fast enough** (milliseconds) that stepping through cells adds UX complexity without benefit |
| **Analogy** | **Report rendering** (Quarto / knit HTML) for layout and inline output; **not** Jupyter’s interactive cell loop |

Implication: the playground optimizes for **edit → Run → read report**, not **edit cell → run cell → iterate**.

## Decided: tutorial and cookbook are literate `.dice`

| Decision | Choice |
|----------|--------|
| **Source of truth** | Tutorial and cookbook lessons/recipes are **`.dice` files in literate mode** (prose + fenced code), not parallel `docs/**/*.md` plus copied scripts in `examples/` |
| **Execution** | Each lesson/recipe must **`dice eval` / playground Run as-is**—same file users read on the docs site (woven HTML is a **render**, not a second format) |
| **CI** | Tutorial/cookbook corpus is part of eval smoke tests (today: `examples/tutorial/*.dice`; target: literate paths under `docs/` or consolidated tree) |
| **Today vs target** | Repo still has `docs/tutorial/*.md` + `examples/tutorial/*.dice`; migration removes drift between prose and runnable script |

Implication for architecture: literate parser/weave is **required for the docs pipeline**, not only the playground UI (`bin/build-tutorial-site.sh` eventually renders from `.dice`, not Pandoc-on-markdown-only).

## Current UI (as of repo)

| Element | Behavior |
|---------|----------|
| Header | Menu, GitHub/docs links, **Files** drawer for multi-file workspace |
| Editor | Full-height `HighlightedEditor`; one active `.dice` file; Shift+Enter runs |
| Diagnostics | Below editor; scroll-into-view on run |
| Output | Separate section below; tabs **text** / **json** / **graph**; `pre` for text/json |

Implemented in `src/ui/app.rs` — vertically stacked **code-first IDE**, not literate single-file report view yet.

## North star UX references

| Reference | Take |
|-----------|------|
| Quarto / R Markdown | Single source, knit to report, figures inline |
| Jupyter | **Layout** inspiration only (narrative + code + output interleaved)—**not** execution model |
| Observable | Out of scope (reactive cells conflict with one-shot decision) |

## Technical form factor

| Surface | Role |
|---------|------|
| Web playground (Leptos CSR WASM) | Literate `.dice` editor + rendered report; one-shot eval |
| `dice` CLI | `eval` whole file; potential future `render` for static export |
| Static site + docs | **Literate `.dice`** tutorial/cookbook; site = woven HTML + nav index |
| `llms.txt` | Drafting full `.dice` documents |

No runtime application server today.

## Architecture constraints

- Engine/UI separation preserved.
- Likely needs: literate **parse** (prose vs code regions), **unchanged whole-file eval** (existing playground eval), **output-to-region binding** for inline renderers (tables, charts, images).
- Per-cell eval API **not** required by product.

## Open product / design questions

- **Literate syntax inside `.dice`:** markdown fences, `@` directives, string literals, or new lexer rules—PRD/language design.
- **Export:** static HTML from WASM vs. CLI `dice render` for CI/GitHub Pages.
- **Images:** uploads vs. generated-only; asset storage on static deploy.
- **Multi-file workspace:** keep for snippets/libraries or collapse to single-file-only UX over time.
- Growth loop, monetization, AnyDice positioning — still open.

## Mechanic coverage (docs)

Tutorial through D&D 5e d20 and PbtA 2d6; cookbook spans Pool, exploding dice, 4d6dl1, fireball, Blades, Brindlewood Bay, Cairn, Rolemaster, Fudge 4dF.
