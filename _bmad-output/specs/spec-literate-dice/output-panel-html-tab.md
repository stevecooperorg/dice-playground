# Output panel: HTML tab (design)

**Status:** implemented
**Related:** `src/ui/app.rs`, `report_view.rs`, `eval_client.rs`, `EvalProgramResponse`, `architecture.md` (Frontend architecture)

## Problem

The playground **splits results by mode**:

| Mode | What the user sees after Run |
|------|------------------------------|
| **Literate** | Always-on **Report** block (`LiterateReportView` + `report_html`) and a separate **Graph** block. No text/json tabs. |
| **Legacy** | Tabbed **Output**: `text` \| `json` \| `graph`. No HTML. |

That diverges from the product story (one document, woven report) and from static docs (HTML report is the primary reading experience). It also duplicates chrome: literate users get two stacked sections (Report + Graph) while legacy users get one tabbed panel.

Authors and players expect **one Output area** with the same tab names everywhere, where **HTML** is the default view for literate lessons and **text** remains the CLI-friendly view.

## Goals

1. **Single Output panel** below the editor for every successful Run (after diagnostics).
2. Add an **`html` tab** (label **“report”** or **“html”** in UI—see §Naming) that shows engine-sanitized weave HTML.
3. Keep **text**, **json**, and **graph** tabs for all modes where data exists.
4. **Default tab:** `html` when `report_html` is non-empty; otherwise `text` (legacy).
5. No second-class graph: **graph stays a tab**, not a separate section under Report.

## Non-goals

- Embedding Chartistry inside weave HTML (still parallel `outputs` + graph tab).
- iframe sandbox for report (engine ammonia pass remains the trust boundary).
- Persisting last-selected tab across sessions (optional later).
- Raw unsanitized HTML or author-supplied HTML in markdown.

## UX specification

### Layout

```
┌─ Editor ─────────────────────────────────────┐
│  (highlighted .dice source)                  │
└──────────────────────────────────────────────┘
┌─ Diagnostics (if any) ─────────────────────┐
└──────────────────────────────────────────────┘
┌─ Output ─────────────────────────────────────┐
│  [ report ] [ text ] [ json ] [ graph ]      │  ← tab bar
│  ┌─────────────────────────────────────────┐ │
│  │  Active tab content                     │ │
│  └─────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

- One `<section>` with shared border/padding (reuse current Output styling).
- Tab bar: same button pattern as today (`text-xs`, active = `bg-slate-700`).
- **Scroll:** panel uses `scroll-mt-36` anchor (existing `output_ref`) after Run.

### Tab availability

| Tab | Literate | Legacy |
|-----|----------|--------|
| **report/html** | Enabled; full weave fragment (prose + GFM tables) | Enabled when `outputs_html` non-empty (outputs-only HTML—§Engine); disabled or hidden if empty |
| **text** | Enabled; `format_eval_result_text` (always computed today) | Enabled |
| **json** | Enabled; pretty `outputs` | Enabled |
| **graph** | Enabled if at least one chartable output | Same as today |

**Disabled tabs:** visually muted (`opacity-50`, `cursor-not-allowed`) or omitted. Prefer **omit** tabs with no content to reduce noise (e.g. hide `html` on pure legacy scripts until outputs-only HTML exists).

### Default selection after Run

```text
if report_html.non_empty() → tab = "html"
else if text.non_empty()   → tab = "text"
else                       → tab = "json" (fallback)
```

User tab choice **persists until the next Run** (current `output_tab` signal behavior), then reset to default above on successful eval.

### Naming

- **UI label:** **Report** (user-facing, matches literate vocabulary and static “woven report”).
- **Internal id:** `html` (matches `report_html`, CLI `render`, and future `dice eval --format html`).
- Tab order: **Report · text · json · graph** (report first).

### Report tab content

- Reuse **`LiterateReportView`** body only (drop duplicate outer “Report” heading—the Output panel title is enough, or use “Output” + tab “Report”).
- Container: `div.literate-report-body` + existing `index.html` table/typography CSS.
- **`inner_html`:** only `EvalProgramResponse.report_html` (literate) or `outputs_html` (legacy).

### Empty / error states

- Run failed: no Output panel (unchanged—diagnostics only).
- Run ok, no outputs: show Report tab with prose-only weave if literate; graph tab shows existing empty message.
- Legacy script with no `output()`: text may show `return:` only; json `[]`; graph empty state.

## Frontend architecture (Leptos)

### State (`app.rs`)

Keep existing signals; unify rendering:

| Signal | Use |
|--------|-----|
| `result_report_html` | Report tab (literate full weave) |
| `result_outputs_html` | **New optional:** legacy outputs-only HTML |
| `result_text`, `result_json`, `result_outputs` | Other tabs |
| `output_tab` | `"html"` \| `"text"` \| `"json"` \| `"graph"` |

On eval success (`eval_client` / run handler):

- Always set `text`, `json`, `outputs`.
- Set `report_html` from WASM (literate).
- Set `outputs_html` from WASM when provided (legacy).
- Apply default tab rule; do not clear text/json when literate (today literate clears them—**stop clearing** so text/json tabs work).

### Components

1. **`OutputPanelView`** (new, `src/ui/output_panel.rs`)
   - Props: tab state + four content strings + `Vec<OutputEntry>`.
   - Tab bar + match on active tab.
   - Delegates graph to `OutputGraphView`, report to inner_html div (extract from `LiterateReportView` or call shared `ReportHtmlBody`).

2. **`LiterateReportView`**
   - Deprecate as standalone section; thin wrapper around `ReportHtmlBody` for tests/static reuse, or delete and use `OutputPanelView` only.

3. **`eval_client.rs`**
   - Extend mapped response with `outputs_html: String` when engine adds field.

### Remove

- Conditional `(has_literate_report() && …)` **separate Graph section**.
- Conditional `(has_output() && !has_literate_report())` **legacy-only Output section**.
- Replace with: `has_output().then(|| OutputPanelView { … })` where `has_output()` is true when any of text, json, outputs, report_html, outputs_html is non-empty.

## Engine / WASM API

### Today

```rust
pub struct EvalProgramResponse {
    pub return_value: String,
    pub outputs: Vec<OutputEntry>,
    pub text: String,
    pub report_html: String,  // literate weave only
}
```

### Proposed

```rust
pub struct EvalProgramResponse {
    // ...
    /// Full literate weave (prose + fence outputs). Empty if legacy.
    pub report_html: String,
    /// Outputs formatted as sanitized HTML fragment (GFM tables). Legacy (and optional literate fallback).
    #[serde(default)]
    pub outputs_html: String,
}
```

**Legacy `outputs_html` generation** (v1):

```text
markdown = format_eval_result_markdown(&eval_result, prob_format)
html     = sanitize(markdown_to_html(markdown))
```

No prose, no fence binding—same tables as Report output blocks. Lets legacy users open **Report** tab without authoring literate fences.

**Literate:** `report_html` from existing `weave_literate`; `outputs_html` may duplicate fence-bound sections only—**omit in v1** (Report tab uses full `report_html` only).

Optional **`mode`** field on response (`"literate" | "legacy"`) for UI telemetry only; UI can infer from `report_html.is_empty()`.

## Security

- Unchanged: all HTML fields produced only in `src/engine/`, ammonia-sanitized before serde to UI.
- Report tab must not render user-authored raw HTML from markdown until policy allows (same as weave today).

## Accessibility

- Tab bar: `role="tablist"`, tabs `role="tab"`, panel `role="tabpanel"`, `aria-selected` on active tab.
- Keyboard: arrow keys between tabs (optional v1.1); minimum: focusable buttons (already).

## Static site / parity

- Static tutorial pages are full-page HTML from `dice render`; playground Report tab should **look like** the `<main>` fragment of those pages (shared CSS classes: `literate-report-body`, `.dice-output table`).

## Migration checklist

- [ ] `EvalProgramResponse.outputs_html` + legacy generation in `eval_program`
- [ ] WASM/TS bindings if any generated types
- [ ] `OutputPanelView` + tab `html` / label Report
- [ ] Stop clearing text/json on literate success in `app.rs`
- [ ] Default tab logic on Run
- [ ] Remove duplicate Report + Graph sections
- [ ] Tests: legacy script → text + graph; literate → report contains `<table>`, tabs switch
- [ ] Update `architecture.md` frontend bullet (unified tabs, not hide legacy when literate)
- [ ] `docs/AGENT.md` one line on Output tabs

## Open questions

1. **Show text tab for literate by default?** Yes—design keeps it for CLI parity; default tab remains Report.
2. **Copy button per tab?** Defer; text/json benefit most.
3. **Prob format change:** re-Run already refreshes all tabs; no extra work.

## References

- Current UI: `src/ui/app.rs` (~437–522), `report_view.rs`
- Eval: `playground.rs` `eval_program`, `EvalProgramResponse`
- Tabular HTML in weave: `tabular-output-gfm.md`
- Architecture intent (to revise): literate “hide legacy tabs” → unified panel with Report tab
