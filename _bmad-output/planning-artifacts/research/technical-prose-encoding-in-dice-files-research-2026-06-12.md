---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - _bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/brief.md
  - _bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/addendum.md
workflowType: research
lastStep: 6
research_type: technical
research_topic: Prose encoding in single-file .dice literate documents
research_goals: Compare encoding and pipeline options for Dice Playground (one .dice file, one-shot whole-file eval, Starlark + dice sugar, WASM playground + CLI/LSP) and recommend a direction for architecture/spec.
user_name: Steve
date: 2026-06-12
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-06-12  
**Author:** Steve  
**Research Type:** technical  

---

## Research Overview

This report evaluates **how prose and executable code should coexist in a single `.dice` file** for Dice Playground’s literate, report-style UX. Product constraints are fixed: **one file**, **one Run (whole-document eval)**, exact probability via the existing engine (Starlark guest + dice/range desugar), static WASM + CLI.

The analysis compares literate-programming encodings used in Quarto/Pandoc, Noweb, and Starlark-native patterns against the **current pipeline** (`desugar_if_needed` → `AstModule::parse`). The conclusion favors a **Quarto-like markdown surface with a tangle pass**, output bound into prose via **named placeholders**, with a **compatibility mode** for today’s pure-Starlark scripts. Full synthesis and recommendations are in **§8–9**.

---

## Technical Research Scope Confirmation

**Research Topic:** Prose encoding in single-file `.dice` literate documents  
**Research Goals:** Encoding and pipeline options for architecture/spec; aligned with brief decisions (single file, one-shot eval).

**Technical Research Scope:**

- Architecture analysis — parse/tangle/weave pipelines, engine boundaries  
- Implementation approaches — Rust preprocessing, markdown renderers, LSP implications  
- Integration — CLI `eval`, playground check/eval, tutorial migration  
- Performance — whole-file eval already ms-scale; preprocessing cost negligible  

**Research Methodology:** Repository analysis (`src/engine/sugar.rs`, `playground.rs`, `eval.rs`); product brief/addendum; web-verified Quarto/Pandoc and Noweb references.

**Scope Confirmed:** 2026-06-12 (user authorized full research run)

---

## 1. Current Dice Playground execution model

| Stage | Location | Behavior |
|-------|----------|----------|
| Desugar | `range_sugar::desugar_all` → `sugar::desugar` | Rewrites `2d6`, `4d6dl1`, `6..94`, etc. to Starlark |
| Parse + eval | `eval_source` / `playground.rs` | Full source string → Starlark AST → single module eval |
| Outputs | `OutputStore` in `eval.rs` | Sequential `output(...)` calls → `Vec<OutputEntry>` |
| UI today | `src/ui/app.rs` | One editor buffer; formatted text/json/graph **below** editor |

**Implication:** Prose cannot be “valid Starlark” unless it is comments, strings, or stripped before parse. Literate encoding is necessarily a **document layer** above (or before) Starlark parsing—not a small tweak inside the Starlark grammar.

**Confidence:** High (code-backed).

---

## 2. Literate programming landscape (verified sources)

### Quarto / Pandoc markdown (report knitting)

Quarto documents are **mostly markdown** with **fenced code blocks**; execution is knit/render as a whole document, not cell-by-cell REPL iteration. Pandoc markdown supports prose, tables, images, divs (`:::`), and fenced code with optional language tags and attributes ([Quarto markdown basics](https://quarto.org/docs/authoring/markdown-basics.html)).

Relevant to Dice Playground:

- **Readable raw file** — matches tutorial/cookbook authors and `llms.txt` workflows.  
- **Separate weave (markdown → HTML) and tangle (extract code)** — matches **one-shot Run**.  
- **Output placement** — Quarto attaches chunk output to chunks; Dice can use **placeholders** in prose (see §4).

**Confidence:** High.

### Noweb (tangle / weave)

Noweb uses a **small set of control sequences** to interleave TeX (or HTML) prose with named code chunks; **tangle** extracts compilable source, **weave** produces human document ([Noweb home page](https://www.cs.tufts.edu/~nr/noweb/)). Language-independent and proven, but ** unfamiliar syntax** (`<<name>>=` ) and weak alignment with existing markdown docs.

**Confidence:** High.

### Jupyter / `.ipynb` (comparison only)

Notebook JSON with markdown vs code cell types is the **layout** reference from the product brief, but **not** the execution model. Adopting `.ipynb` would violate the **single `.dice` file** decision and duplicate markdown already in `docs/`.

**Confidence:** High (product decision).

---

## 3. Encoding options (comparison)

Evaluation criteria derived from the brief:

1. Single `.dice` artifact  
2. One-shot eval (tangle → one Starlark module)  
3. Backward compatible with existing `.dice` scripts (tutorial examples)  
4. Author-friendly for GMs (markdown familiarity)  
5. Engine/UI separation (document parse in UI or `engine::document`, not Leptos in engine)  
6. LSP/diagnostics: map errors to **tangled line ↔ source line**  
7. Static export path (future HTML report)

| Option | Encoding sketch | Pros | Cons | Fit |
|--------|-----------------|------|------|-----|
| **A. Markdown-first (Quarto-like)** | `.dice` = markdown body; executable in ` ```dice ` fences; optional YAML header | Matches docs site; great prose; images/tables native in weave | Needs markdown parser + tangle; Starlark LSP sees tangled buffer or dual view | **Best overall** |
| **B. Noweb in `.dice`** | `<<chunk>>=` + `@` prose | Minimal grammar; proven tangle/weave | Ugly for non-TeX users; poor match to existing markdown tutorials | Poor |
| **C. Directive lines** | `% md` / `# %% markdown` regions | Simple lexer | Nonstandard; fights `#` Starlark comments; messy spec | Weak |
| **D. Starlark-only `prose()` / `md()` builtins** | `display_md("""...""")` | No preprocessor; one parse | Awful authoring; escaping; LSP strings not markdown | Poor for primary UX |
| **E. `.dice` = pure Starlark + trailing markdown in block comments** | Not valid—Starlark has no block comments | — | — | **Invalid** |
| **F. Dual-mode file** | If no markdown fences → legacy Starlark; else literate markdown | Smooth migration | Two dialects to document | **Combine with A** |

### Output binding (report inline)

Today outputs are an **ordered list** with names (`output("ability", 4d6dl1)`). For inline reports:

| Strategy | Mechanism | Pros | Cons |
|----------|-----------|------|------|
| **F1. Placeholders in prose** | `{{output ability}}` or Pandoc-style `{{< dice-output ability >}}` in markdown | Prose controls layout; one eval | Requires weave pass after eval |
| **F2. Chunk-attached output** | Each fence runs in shared env; render output immediately under fence | Familiar to Quarto readers | Implies chunk boundaries in weave; still one eval if tangled in order |
| **F3. Order-only** | Render all outputs at end in eval order | Trivial | Fails north-star “inline report” |

**Recommendation:** **F1 + F2 together** — tangle concatenates fences in document order (shared Starlark scope); weave renders markdown and substitutes placeholders **or** auto-appends under each fence when no placeholder.

**Confidence:** Medium-high (product fit); placeholder syntax is an open UX detail for `bmad-create-architecture`.

---

## 4. Recommended architecture direction (pre-decision)

### Primary recommendation: **Markdown-first literate `.dice` (Option A + F)**

**Authoring surface**

- File is **Pandoc-flavored markdown** (subset aligned with tutorial docs: headings, lists, links, images, tables, fenced code).  
- Executable blocks use a dedicated fence tag, e.g. ` ```dice ` or ` ```{=dice}` (Pandoc raw/code convention per [Quarto source code section](https://quarto.org/docs/authoring/markdown-basics.html)).  
- Optional YAML front matter later (title, `prob_format`, export options)—not required for v1 literate.

**Pipeline (one-shot)**

```text
.dice source
  → literate::parse (prose AST + code spans + placeholder map)
  → literate::tangle → single Starlark string (+ source map)
  → desugar_if_needed → eval_source → EvalResult
  → literate::weave (markdown → HTML + inject OutputEntry renderers at placeholders / fences)
```

**Backward compatibility (Option F)**

- **Detection:** If parse finds no literate markers (no fences with `dice` language, no front matter), treat entire file as **legacy Starlark** (current behavior).  
- Short **legacy-only** scripts (e.g. tiny CI snippets) may stay pure Starlark; **tutorial/cookbook target state is literate `.dice`**, not legacy.

**Tutorial and cookbook as canonical literate sources (product decision)**

- End state: **`docs/tutorial/` and `docs/cookbook/` (or one merged tree) are `.dice` files**—prose + fences—not `.md` with embedded copy-paste blocks pointing at `examples/tutorial/*.dice`.  
- **Same file** must pass: `dice eval lesson.dice`, playground Run, and static-site **weave** (HTML for `/docs/`).  
- Today’s split (`docs/tutorial/*.md` + `examples/tutorial/*.dice`, Pandoc in `build-tutorial-site.sh`) is **migration debt**; architecture should not treat markdown-only lessons as the long-term source of truth.  
- CI (`tests/tutorial_samples.rs`, etc.) should eventually list **literate** paths only.

**Confidence:** High (explicit user decision).

**Module placement**

- New `src/engine/literate/` (or `document/`) for parse/tangle/source-map—**no** `web-sys`.  
- UI: report view + optional source view (like Quarto source/visual), not required day one.

**LSP**

- Short term: LSP on **tangled** buffer with source map for diagnostics (pattern used by many preprocessors).  
- Long term: literate-aware highlights in fenced regions.

**CLI**

- `dice eval file.dice` — tangle then eval (text/json as today).  
- Future `dice render file.dice -o report.html` — weave for static site / CI.

**Confidence:** High for pipeline shape; medium for exact fence tag and placeholder syntax.

### Runner-up: **Noweb (Option B)**

Choose only if markdown in `.dice` is rejected (e.g. must be “obviously code file”). Conflicts with docs strategy and author audience.

**Confidence:** High that it is inferior for this product.

### Deprioritized: **Starlark builtins only (Option D)**

Useful for **small** inline notes (`# comment` already exists), not for report prose.

---

## 5. Integration patterns

| Consumer | Change |
|----------|--------|
| Playground WASM | `check_source` / `eval_source` accept literate input; return **structured report** (HTML fragments + assets) not only flat `format_eval_result_text` |
| `format_eval_result_text` | Remains for legacy + CLI text mode |
| **Tutorial / cookbook** | **Source = literate `.dice`**; `dice eval` in CI; **`dice render` / weave** replaces Pandoc-on-markdown for lesson HTML (nav/index may remain generated) |
| `llms.txt` | Document literate `.dice` so models emit tutorial-shaped files, not bare scripts |

---

## 6. Performance and risks

**Performance:** Tangle + markdown parse on typical scripts (hundreds of lines) is negligible vs PMF eval. One-shot eval aligns with ms-scale engine ([brief rationale](file://brief)).

**Risks**

| Risk | Mitigation |
|------|------------|
| Markdown/starlark fence ambiguity | Strict fence tags; golden tests from tutorial |
| Diagnostic line numbers | Source map from tangle (Noweb’s `# line` directive pattern) |
| Scope creep (full Quarto) | Subset markdown + dice fences only for v1 |
| LSP drift | Tangled buffer + map until native literate LSP |

---

## 7. Technology stack notes (supporting libraries)

| Need | Typical Rust ecosystem | Notes |
|------|------------------------|-------|
| Markdown parse | `pulldown-cmark`, `comrak` | Prefer CommonMark + tables; align with Pandoc subset gradually |
| HTML sanitize | `ammonia` | If user prose is rendered in browser |
| YAML front matter | `serde_yaml` | Optional phase 2 |

No change to Starlark crate (`starlark 0.14`) for prose encoding.

**Web research note:** Brave Search rate-limited during this run; Quarto and Noweb claims verified via direct documentation fetch above.

---

## 8. Executive summary (synthesis)

Dice Playground should treat **`.dice` as a literate markdown document** with **extractable `dice` code fences**, not as raw Starlark with prose sprinkled in the grammar. **One Run** tangles all fences into **one Starlark program**, runs existing desugar + eval, then **weaves** markdown into a report with **inline output** via placeholders and/or per-fence output blocks.

This aligns with the product’s Quarto-like north star, preserves **millisecond whole-file eval**, keeps the engine pure, and allows **legacy pure-Starlark** files where literate markers are absent. **Tutorial and cookbook must be literate `.dice` and eval as-is**—the docs site is woven from that same corpus (no duplicate markdown + script).

**Top recommendations for `bmad-create-architecture`:**

1. Specify **markdown-first `.dice`** with fenced executable blocks and legacy fallback.  
2. Define **tangle + source map + weave** as the document contract between engine and UI.  
3. Define **output binding** (`output("name", …)` + prose placeholders).  
4. Cap v1 markdown subset (headings, lists, links, images, tables, code fences)—defer callouts/equations unless needed.  
5. Plan CLI parity: **`dice eval`** tangles; **`dice render`** weaves for static `/docs/`.  
6. **Docs migration plan:** replace `docs/tutorial/*.md` + `examples/tutorial/*.dice` with single literate `.dice` per lesson; update `build-tutorial-site.sh` and CI lists accordingly.

---

## 9. Next steps

| Step | Skill / action |
|------|----------------|
| Lock grammar & pipeline | `bmad-create-architecture` — input this research + brief |
| Machine contract | `bmad-spec` — companion `literate-dice-format.md` |
| Author UX | `bmad-ux` — source vs preview layout, Run affordance |
| Spike | `bmad-quick-dev` or dev story — `literature::parse` + one golden `.dice` file |

---

## 10. Sources

| Source | URL | Used for |
|--------|-----|----------|
| Quarto — Markdown Basics | https://quarto.org/docs/authoring/markdown-basics.html | Markdown/code fences, divs, knit model |
| Noweb | https://www.cs.tufts.edu/~nr/noweb/ | Tangle/weave, language-independent literate LP |
| Dice Playground brief/addendum | `_bmad-output/planning-artifacts/briefs/brief-dice-playground-2026-06-12/` | Constraints |
| Dice Playground engine | `src/engine/playground.rs`, `sugar.rs`, `starlark_guest/eval.rs` | Current pipeline |

---

**Technical Research Completion Date:** 2026-06-12  
**Technical Confidence Level:** High on constraints and pipeline; medium on exact syntax tokens (fence label, placeholder delimiters).
