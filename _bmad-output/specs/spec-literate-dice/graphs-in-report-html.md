# Charts in Report HTML (design)

**Status:** design  
**Related:** architecture **D12**, `output_graph.rs`, `weave.rs`, `format_eval_result_markdown`, `output-panel-html-tab.md`

## Problem

After GFM tables and the unified **Report** tab, numeric results appear twice in different shapes:

- **Report tab:** prose + **HTML tables** (engine weave).
- **Graph tab:** **Chartistry** charts built from the same `OutputEntry` list.

Readers must switch tabs to connect a distribution to its picture. Static `dice render` pages never show charts at all. Architecture already reserved **placeholders in weave HTML** with chart rendering staying in the UI (D12).

## Goal

In the **Report** HTML fragment, for each chartable `output(name, …)`, render:

1. **Chart mount point** (empty placeholder, keyed by `name`).
2. **Table block** (existing GFM → HTML caption + `<table>`).

Order: **chart above table**, one `<section class="dice-output">` per logical output (not one blob per fence).

The **graph** tab can remain as an “all charts” gallery until inline charts ship; long term it may become redundant or show only outputs without placeholders.

## Non-goals (v1)

- Server-side PNG/SVG generation for static `dice render` (placeholders stay empty on CDN).
- Charts inside markdown prose via `{{output "name"}}` (v1.1 placeholders).
- Embedding Chartistry or wasm chart code in `src/engine/`.

## Chart eligibility (match `charts_from_outputs`)

| `OutputEntry` | Chart type | Placeholder? |
|---------------|------------|--------------|
| `DieRoll` | Line (full PMF, not display-compressed) | Yes, if `entries` non-empty |
| `Outcomes` | Ordinal bar | Yes, if rows non-empty |
| `Prob` | Single bar | Yes |
| `Table` (`prob_table`) | Ordinal bar today | **Only if row count ≤ N** (default **32**); skip for modifier grids (e.g. lesson 10, 100+ rows) |

Engine function **`output_entry_supports_chart(entry) -> bool`** shared with UI (move `charts_from_outputs` logic to engine or a tiny `output_chart.rs` both UI and weave call).

## HTML shape (engine)

Per output block:

```html
<section class="dice-output" data-dice-output-name="one_d6">
  <div
    class="dice-output-chart"
    data-dice-output="one_d6"
    data-dice-chart-kind="dieroll"
    role="img"
    aria-label="Chart for output one_d6"
  ></div>
  <!-- sanitized markdown fragment: caption + table -->
  <p><strong>one_d6</strong> · DieRoll · mean 3.500</p>
  <table>...</table>
</section>
```

- **`data-dice-output`:** stable bind key (output name string from Starlark).
- **`data-dice-chart-kind`:** `dieroll` \| `outcomes` \| `prob` \| `table` (UI picks component).
- No inline JSON in HTML (avoids huge attributes and escaping); hydration uses **`Vec<OutputEntry>`** already on the client, keyed by name.

If not chartable, omit the inner `<div class="dice-output-chart">` (table only).

### Ammonia

Default ammonia may strip unknown `data-*` on `div`. Configure weave sanitizer (or dedicated pass) to allow:

- Tags: `section`, `div`, `p`, `strong`, `table`, … (existing markdown output).
- Attributes: `class`, `data-dice-output`, `data-dice-chart-kind`, `data-dice-output-name`, `role`, `aria-label`.

Still no `script`, `on*`, or `style`.

## Weave pipeline change

Today `append_fence_outputs` batches all outputs from one fence into **one** markdown string → one `<section>`. Change to **per output entry**:

```text
for each OutputEntry bound to this fence (in eval order):
  push chart placeholder div (if eligible)
  push format_single_output_markdown_html(entry)  // caption + GFM table only
  wrap in <section class="dice-output" …>
```

Refactor `format_eval_result_markdown` into **`format_output_entry_markdown(entry, …) -> String`** (already implied by GFM helpers per kind).

Same structure for **legacy `outputs_html`** (no prose, only output sections).

Literate **fence with multiple `output()` calls** → multiple sections in document order (chart/table, chart/table, …).

## Playground UI hydration

**Constraint:** Report tab uses `inner_html` on a single container; Leptos cannot nest `OutputGraphView` inside that string.

**Pattern: mount-after-render**

1. **`ReportHtmlHost`** component replaces bare `inner_html` in `OutputPanelView` when on Report tab.
2. Props: `html: String`, `outputs: Vec<OutputEntry>`.
3. On mount / when `html` or `outputs` change:
   - Set `inner_html` on a host `div` (ref).
   - `querySelectorAll(".dice-output-chart[data-dice-output]")`.
   - For each node, find `OutputEntry` with matching `name` (DieRoll/Outcomes/Table/Prob field).
   - **`leptos::mount::mount_to(placeholder, chart_view)`** with the same chart components as `OutputGraphView` (extract `SingleOutputChart { entry }` component).

4. **Cleanup:** on next Run, unmount previous chart roots (store `Vec<UnmountHandle>` or replace host innerHTML and drop handles).

**Duplicate names:** last `output()` wins for binding today; chart hydration uses same rule.

### Graph tab (interim)

- Keep **graph** tab as all chartable outputs in one scroll (current behavior).
- Optional later: hide graph tab when every chartable output has an inline placeholder, or label it “All charts”.

## Static site (`dice render`)

- HTML includes placeholders + tables; **no JS chart bundle** in v1.
- CSS: `.dice-output-chart:empty { display: none; }` or leave minimal height 0 so static pages look like today (tables only).
- Future: optional Trunk/chart hydration script for static preview—not required for this design.

## API (optional structured path)

Long-term alternative to string HTML + hydration:

```rust
enum ReportSegment {
  ProseHtml(String),
  OutputBlock {
    name: String,
    chart_kind: Option<ChartKind>,
    table_html: String,
  },
}
```

WASM returns `segments` + UI maps to Leptos tree (no `inner_html`). **Defer** until placeholder hydration proves awkward (e.g. a11y, unmount bugs).

## Testing

- Engine: woven HTML contains `data-dice-output="one_d6"` before `<table>` for `1d6` lesson; lesson 10 grid has **no** chart div when row count > 32.
- Engine: ammonia preserves allowed `data-*` attributes.
- UI (wasm test or headless): placeholder count matches chartable outputs (hard in unit test; manual checkpoint or `web_sys` in wasm test).
- Regression: `charts_from_outputs` and eligibility stay in sync.

## Implementation phases

| Phase | Work |
|-------|------|
| **1** | `output_entry_supports_chart` + `chart_kind_for_entry`; refactor single-output markdown HTML builder |
| **2** | Weave / `outputs_html`: per-output `<section>` with placeholder + table; ammonia config |
| **3** | `ReportHtmlHost` + mount chart per placeholder; wire Report tab |
| **4** | CSS (playground + `tutorial.css`); static empty placeholder styling |
| **5** | Decide graph tab UX; docs |

## Open questions

1. **Threshold for `prob_table` charts** — 32 rows default; expose via `WeaveOptions`?
2. **Outcomes vs Table** — same bar chart UI; kind attribute still useful for axis formatting.
3. **Compressed PMF in table vs full data in chart** — charts must use raw `OutputEntry` entries (already true in graph tab), not compressed GFM rows.

## References

- `src/ui/output_graph.rs` — `charts_from_outputs`, chart components  
- `src/engine/literate/weave.rs` — `append_fence_outputs`  
- `_bmad-output/planning-artifacts/architecture.md` — D12  
