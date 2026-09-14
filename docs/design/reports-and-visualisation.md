# Reports, tables, and charts

**Output design — approved.** [Design index](README.md)

## Keep the data independent of its presentation

An evaluation produces structured output entries: numeric distributions, ordered outcomes, scalar probabilities, and labelled probability-table rows. A report, a text table, a JSON export, and a chart are different views of these entries, not different calculations.

This separation is important for correctness. Compressing a long table for readability must not compress the underlying distribution used by a chart or change the JSON/CSV contract.

```text
Evaluated output entries
    ├── text formatter ── terminal / text tab
    ├── structured data ─ JSON / CSV / charts
    └── markdown table formatter
              └── markdown-to-HTML + sanitize ── report / static page
```

## One Output panel for both document modes

The later output-panel design replaces separate literate Report/Graph sections and a legacy-only tabbed panel with **one shared Output panel** after diagnostics.

Tab order is **Report · text · json · graph**. Report’s internal identifier is `html`; the user-facing label is Report. The panel uses the existing result scroll anchor after Run rather than adding a separate report-only destination.

| Data | Report view |
|---|---|
| Literate `report_html` | Full weave: explanation plus fence-bound output sections. |
| Legacy `outputs_html` | Results-only HTML using the same table formatter; no invented prose or fence binding. |
| No HTML | Omit or disable Report. |

Text, JSON, and structured outputs remain populated for successful literate evaluations. The tabbed interface must not clear them simply because a full report exists.

The specified default is Report when full `report_html` is non-empty, otherwise text when non-empty, otherwise JSON. Selection persists until the next successful Run. The sources are less clear about the default when **only legacy `outputs_html`** exists; that needs confirmation rather than a silent reinterpretation of `report_html`.

Failed Runs show diagnostics without a new successful Output panel. A successful literate document with no outputs can still show prose. Legacy no-output scripts can retain a return-value text view and JSON `[]`. The design mentions both hiding empty graph tabs and displaying an empty-state message; that minor choice is [open](../requirements/review-notes.md#result-presentation).

The minimum accessibility contract is focusable tab buttons with `role="tablist"`, `role="tab"`, `aria-selected`, and a `role="tabpanel"` content region. Arrow-key navigation, per-tab copy buttons, and remembering the selected tab across sessions were deferred.

## Real tables in HTML, plain text in terminals

The HTML path uses GFM pipe-table markdown, the shared markdown renderer, and sanitization to produce semantic `<table>` elements. It does not wrap a padded ASCII table in `<pre>` and call that a report.

One engine-side table-building approach serves the output variants:

| Output | Table representation |
|---|---|
| `DieRoll` | Numeric face or compressed band label, followed by probability columns; a caption may include the mean. |
| `Outcomes` | Labels in scale order, followed by probability columns. |
| `Table` / `prob_table` | Row descriptions and their separate probabilities; rows are not required to sum to one. |
| `Prob` | Prefer a one-row table for consistent styling. |

A caption such as **d6 · DieRoll · mean 3.500** identifies the result. Long-distribution tail folding and band preparation remain separate from the renderer. Numeric columns should be aligned with HTML/CSS, not padded with spaces. Table borders and typography must remain readable in the playground and static dark-themed pages, including narrower layouts.

### Probability columns

The recorded table design has two modes:

- **Teaching/default mode:** show outcome, percentage, fraction, and a sample-space/count column. The denominator label is inferred by the formatter, not necessarily the original number of elementary trials.
- **Selected-format mode:** show outcome and one probability column in the chosen decimal, percent, fraction, or sample-space format.

The decision was to keep teaching mode as the default for woven/static lessons and support selected mode through a non-default `ProbFormat` or an explicit table-column option. The exact option/control is not settled. A formatting parameter appearing in an API must not be treated as proof that both modes are implemented.

The CLI’s plain-text output keeps the legacy padded pipe formatter. JSON, CSV, and chart inputs remain unchanged. A future `--format markdown`, changing CLI text to GFM, two-dimensional pivot layouts, and report-only caps on very large tables were not part of the initial commitment.

### Escaping is part of formatting

Output names and row labels are text, not trusted HTML. The table builder must escape pipe characters so they do not become extra columns, and handle `<` and `&` so label text does not become executable markup. HTML attribute values need their own escaping.

All generated fragments then pass through the engine sanitizer. Rich table rendering must not weaken the security boundary used for prose.

## Inline charts: chart first, table second

The chart companion is explicitly labelled **design**, not completed implementation. It extends the report so readers see a result’s shape and precise values together:

1. a chart placeholder, when the output is suitable for a chart;
2. the caption and existing HTML table; and
3. one enclosing section for that **individual output**, not one combined blob for an entire code fence.

Several outputs from one fence produce several sections in evaluation order. Legacy outputs-only HTML uses the same per-output structure.

### Which outputs get a chart?

| Output | Chart | Eligibility |
|---|---|---|
| `DieRoll` | Line | Non-empty entries; use the full PMF, not table-compressed rows. |
| `Outcomes` | Ordered bar | Non-empty rows in scale order. |
| `Prob` | Single bar | A scalar probability. |
| `Table` | Ordered bar | Small tables; the recorded default maximum is **32 rows**. Large modifier grids remain table-only. |

Eligibility should be defined once in engine-side, UI-independent helpers and shared with the chart gallery and report weaver. That prevents one view claiming a result is chartable while another omits it unexpectedly. Whether the 32-row threshold becomes configurable is still open.

### Engine output and browser responsibility

The engine emits a safe mount point, not a Leptos component or a chart image. The intended shape is:

```html
<section class="dice-output" data-dice-output-name="one_d6">
  <div class="dice-output-chart"
       data-dice-output="one_d6"
       data-dice-chart-kind="dieroll"
       role="img"
       aria-label="Chart for output one_d6"></div>
  <!-- Caption and sanitized table follow here. -->
</section>
```

`data-dice-output` is the named binding key. Chart kinds are `dieroll`, `outcomes`, `prob`, and `table`. The later design chooses a **parallel structured output list**, not JSON embedded in HTML attributes; the latter would add large attributes and escaping problems.

The sanitizer needs deliberate allowances for structural/table tags and for `class`, the three `data-dice-*` attributes shown above, `role`, and `aria-label`. Scripts, `on*` event handlers, and inline `style` remain excluded from this chart markup policy.

In the playground, a report host inserts sanitized HTML, finds its chart mount points, and mounts Chartistry/Leptos chart views using the corresponding structured output. A single-output chart component should be reusable by the Report and graph tabs.

On another Run or a changed report, old chart roots must be unmounted and their handles released before replacement. This is UI lifecycle work and must not leak into the engine.

The duplicate-name rule proposed in the original format is “last output wins for named binding”. Matching a later chart to an earlier same-named table would be misleading, so [output identity](../requirements/review-notes.md#document-rules) needs a consistent decision before this is treated as complete behaviour.

### Static pages remain useful without charts

The initial static renderer includes the same placeholders and tables, but does **not** ship a JavaScript chart bundle or generate PNG/SVG charts on the server. Empty chart placeholders should collapse or be hidden, for example with `.dice-output-chart:empty { display: none; }`. The table remains the useful fallback.

The graph tab can remain an all-charts gallery. Hiding it once every chart is inline, renaming it “All charts”, static chart hydration, and returning structured report segments instead of one HTML string are later alternatives—not requirements to implement as part of this extraction.

## Shared appearance and tests

Playground report typography uses the same concepts as the static page’s main content: `literate-report-body`, `.dice-output`, readable tables, and shared styling where practical. Matching structure matters more than copying two unrelated sets of styles that drift.

Important checks include:

- table markdown snapshots and escaping tests, including row names containing `|`, `<`, and `&`;
- a woven lesson containing real `<table>` output, not raw table pipes inside `<pre>`;
- preservation of full structured output when table rows are compressed;
- a d6 chart placeholder before its table;
- no inline chart for a large modifier grid above the row threshold;
- sanitizer preservation of required chart attributes and rejection of dangerous markup;
- browser mounting and cleanup across repeated Runs; and
- Report/text/JSON/graph availability and default-selection behaviour for both modes.

The original companion files contain “implemented” headings alongside unchecked task lists, and the chart plan has code counterparts. Those labels alone are not test evidence. Current implementation observations and remaining gaps are recorded in the [review notes](../requirements/review-notes.md#implementation-observations).

---

**Basis:** [source map](source-map.md), S07, S09–S12. The later table, unified-panel, and inline-chart designs refine the earlier architecture; their decisions have been kept without treating every old checklist as a current delivery status.
