# Tabular eval output: GFM tables (design)

**Status:** implemented (HTML weave / static render)
**Related:** `literate-dice-format.md` §6, `output_format.rs`, `weave.rs`, `OutputEntry`

## Problem

Today, distribution and grid results are rendered as **monospace ASCII pipe tables** with hand-padded columns (`render_multi_format_table` in `output_format.rs`). That format is:

- Duplicated conceptually with **GFM pipe tables** we already parse for docs prose.
- Shown inside **`<pre>`** in woven reports, so pipes never become real HTML tables in the playground or static lessons.
- Awkward in narrow viewports (fixed padding, no semantic `<table>`).
- **`ProbFormat` is ignored** for PMF/table rows (`_prob` parameters); CLI always emits `%`, `frac`, and sample-space columns together.

Authors already learn GFM tables in the user guide (`docs/README.md`). Tabular **eval output** should use the same primitive in HTML contexts.

## Goals

1. **HTML paths** (literate weave, static `dice render`, playground report): tabular data is **GFM markdown → `markdown_to_html` → sanitize**, producing `<table>` like prose.
2. **One table builder** in the engine shared by DieRoll PMF, Outcomes, `prob_table`, and scalar `Prob` when shown as a row.
3. **Structured data unchanged:** `OutputEntry` JSON, CSV export, and charts remain the source of truth for tools.
4. **PMF compression unchanged:** tail folding and band labels stay in row preparation; only the **renderer** changes.

## Non-goals (v1 of this design)

- Replacing JSON/graph tabs in the legacy playground UI.
- 2D pivot layouts for modifier grids (still one row per `(label, p)` in `prob_table`).
- Full GFM in CLI plain-text mode (see “Surfaces” below).

## Surfaces

| Surface | Current | Target |
|--------|---------|--------|
| Literate weave / report HTML | Escaped text in `<pre>` | Markdown fragment with GFM tables → HTML in `<section class="dice-output">` |
| CLI `dice eval --format text` | ASCII pipe table | **Keep ASCII** for terminal copy-paste, or add `--format markdown` later |
| CLI JSON / CSV | Structured | No change |
| Legacy `.dice` text tab | `format_eval_result_text` | Same as CLI text until UI adopts HTML report for legacy |
| Graph tab | `OutputEntry` | No change |

**Principle:** **HTML consumers get GFM; plain-text consumers keep a text formatter** until we explicitly unify them.

## Architecture

```
EvalResult (OutputEntry[])
        │
        ├─► Structured (JSON, CSV, graphs)     [unchanged]
        │
        ├─► format_eval_result_text()        [ASCII, CLI / legacy text tab]
        │
        └─► format_eval_result_markdown()    [NEW: GFM fragments per output]
                 │
                 └─► markdown_to_html + sanitize_woven_html
                          │
                          └─► weave HTML, optional future “markdown eval” CLI
```

Add in `output_format.rs` (or sibling `output_markdown.rs`):

- `format_eval_result_markdown(result, prob_format) -> String` — concatenates per-output fragments.
- `format_distribution_gfm(...)`, `format_prob_table_gfm(...)`, etc. — build **unpadded** GFM pipes.
- Shared helper: `gfm_table(header: &[&str], rows: &[Vec<String>]) -> String` with **GFM escaping** for `\|` in cells.

Weave change (`append_fence_outputs`):

1. Call `format_eval_result_markdown` instead of `format_eval_result_text`.
2. Run `markdown_to_html` on the fragment (tables already enabled via `markdown_options()`).
3. Wrap with `<section class="dice-output">` + sanitized HTML (**no** `<pre>` for tables).

Optional caption line before each table (markdown, not inside table):

```markdown
**success_grid** · Table
```

DieRoll mean stays in caption: `**d6** · DieRoll · mean 3.500`.

## Column policy

Two modes (weave options / `ProbFormat`):

### A. Multi-column (default for lessons, matches today’s teaching)

Header row:

```markdown
| outcome | % | frac | 6/36 |
|---------|---|------|------|
```

- Sample-space header uses inferred denominator (`6`, `36`, or `count` label from `rel_column_header`).
- Same columns for DieRoll PMF, Outcomes, and `prob_table` rows.
- Numeric outcome labels right-aligned in HTML via CSS (`td.num`), not ASCII padding.

### B. Single-column (`ProbFormat` respected)

When the user selects decimal / percent / fraction / sample-space (playground setting or CLI flag), **one** probability column:

```markdown
| outcome | p |
|---------|---|
| 7 | 16.7% |
```

Reduces noise in reports when authors do not need all four representations.

**Decision:** Implement **A** as default for woven/static HTML (parity with current lessons); wire **B** when `WeaveOptions.prob_format` ≠ default or a new `WeaveOptions.table_columns: Multi | Selected` flag is set.

## Mapping from `OutputEntry`

| Kind | GFM shape |
|------|-----------|
| `DieRoll` | Caption + table of compressed PMF rows; first column face/band label, prob columns per policy |
| `Outcomes` | Same as distribution table on ordered labels |
| `Table` (`prob_table`) | Caption + table; first column row label |
| `Prob` | One-row table **or** caption + `p` line (prefer one-row table for consistency) |

Row labels that contain `|` MUST be escaped per GFM (`\|`).

## HTML / CSS

- Playground: extend `.literate-report-body table` (mirror `tutorial-static/tutorial.css` rules).
- Static pages: woven output already uses literate report classes on tutorial/cookbook pages; ensure table borders readable in dark theme.
- `enhance-static-site` only augments `<pre><code>` script blocks — **no change** required for output tables.

## Security

Same as prose: **ammonia** after `markdown_to_html`. Do not inject raw HTML from Starlark labels into markdown (labels are table cell text only; escape `|`, `<`, `&` in GFM builder).

## Migration / compatibility

1. Ship weave HTML tables behind no author-facing flag (output is strictly better in report view).
2. Update lesson copy that says “one block with rows like `modifier -2, target 0`” — still true; mention HTML table in UI if desired.
3. Snapshot tests: GFM strings for `10-tables.dice` eval; weave integration asserts `<table>` and no raw `| % | frac |` inside `<pre>`.
4. Deprecate `render_multi_format_table` for HTML paths only; keep for `format_eval_result_text` until CLI markdown mode exists.

## Open questions

1. **Scalar `Prob`:** one-row GFM table vs bold line — table wins for uniform styling.
2. **CLI default long-term:** emit GFM in text mode (readable in GitHub) vs keep ASCII — defer; document both formatters.
3. **Very large `prob_table`:** optional row cap in markdown with “… N more rows (see JSON)” — only if perf/UX bites; JSON tab already exists.

## Implementation checklist

- [ ] `gfm_table` + cell escaping + unit tests
- [ ] `format_*_gfm` for each `OutputEntry` variant
- [ ] `format_eval_result_markdown`
- [ ] Weave: markdown path + CSS
- [ ] `WeaveOptions`: document column policy vs `prob_format`
- [ ] Integration: `docs/tutorial/10-tables.dice`, literate weave tests
- [ ] Optional: `dice eval --format markdown`

## References

- Current ASCII: `render_multi_format_table`, `append_distribution_table`
- Weave embedding: `append_fence_outputs` in `weave.rs`
- Docs prose tables: `markdown_html.rs` (`ENABLE_TABLES`)
