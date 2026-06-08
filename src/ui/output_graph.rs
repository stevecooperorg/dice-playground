//! Bar-chart view of `output()` PMFs from eval results.

use crate::engine::OutputEntry;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct BarRow {
    label: String,
    prob: f64,
    /// Share of total probability mass in this section (0–100).
    pct_of_total: f64,
    /// P(outcome is at least this row's value / rank).
    p_at_least: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct ChartSection {
    title: String,
    rows: Vec<BarRow>,
}

fn rows_from_dist_entries(entries: &[(i64, f64)]) -> Vec<BarRow> {
    let total: f64 = entries.iter().map(|(_, p)| *p).sum();
    entries
        .iter()
        .map(|(v, p)| {
            let p_at_least: f64 = entries
                .iter()
                .filter(|(k, _)| *k >= *v)
                .map(|(_, prob)| *prob)
                .sum();
            BarRow {
                label: v.to_string(),
                prob: *p,
                pct_of_total: share_percent(*p, total),
                p_at_least,
            }
        })
        .collect()
}

fn rows_from_ordinal_entries(entries: &[(String, f64)]) -> Vec<BarRow> {
    let total: f64 = entries.iter().map(|(_, p)| *p).sum();
    entries
        .iter()
        .enumerate()
        .map(|(i, (label, p))| {
            let p_at_least: f64 = entries[i..].iter().map(|(_, prob)| *prob).sum();
            BarRow {
                label: label.clone(),
                prob: *p,
                pct_of_total: share_percent(*p, total),
                p_at_least,
            }
        })
        .collect()
}

fn rows_from_prob(value: f64) -> Vec<BarRow> {
    vec![BarRow {
        label: format_prob_label(value),
        prob: value,
        pct_of_total: 100.0,
        p_at_least: value,
    }]
}

fn share_percent(prob: f64, total: f64) -> f64 {
    if total.is_finite() && total > 0.0 && prob.is_finite() {
        (prob / total) * 100.0
    } else {
        0.0
    }
}

fn format_tooltip(row: &BarRow) -> String {
    format!(
        "{} of total · P(≥ {}) = {}",
        format_share(row.pct_of_total),
        row.label,
        format_prob_pct(row.p_at_least)
    )
}

fn format_share(pct: f64) -> String {
    if pct.is_finite() {
        format!("{pct:.2}%")
    } else {
        "—".to_string()
    }
}

fn format_prob_pct(p: f64) -> String {
    if p.is_finite() && (0.0..=1.0).contains(&p) {
        format!("{:.2}%", p * 100.0)
    } else {
        format!("{p}")
    }
}

fn sections_from_outputs(outputs: &[OutputEntry]) -> Vec<ChartSection> {
    outputs
        .iter()
        .filter_map(|entry| {
            let (title, rows) = match entry {
                OutputEntry::Dist { name, entries, .. } => {
                    (name.clone(), rows_from_dist_entries(entries))
                }
                OutputEntry::Ordinal { name, entries, .. }
                | OutputEntry::Table { name, entries } => {
                    (name.clone(), rows_from_ordinal_entries(entries))
                }
                OutputEntry::Prob { name, value } => (name.clone(), rows_from_prob(*value)),
            };
            if rows.is_empty() {
                return None;
            }
            Some(ChartSection { title, rows })
        })
        .collect()
}

fn format_prob_label(p: f64) -> String {
    if p.is_finite() && (0.0..=1.0).contains(&p) {
        format!("{:.4}", p)
    } else {
        format!("{p}")
    }
}

fn max_prob(rows: &[BarRow]) -> f64 {
    rows.iter()
        .map(|r| r.prob)
        .filter(|p| p.is_finite() && *p > 0.0)
        .fold(0.0_f64, f64::max)
}

fn bar_width_percent(prob: f64, max: f64) -> f64 {
    if !prob.is_finite() || prob <= 0.0 {
        return 0.0;
    }
    if max <= 0.0 || !max.is_finite() {
        return 0.0;
    }
    (prob / max).clamp(0.0, 1.0) * 100.0
}

#[component]
pub fn OutputGraphView(outputs: Vec<OutputEntry>) -> AnyView {
    let sections = sections_from_outputs(&outputs);
    if sections.is_empty() {
        return view! {
            <p class="text-slate-400 font-sans text-sm m-0">
                "No distribution outputs to chart. Call "
                <code class="text-slate-300">"output()"</code>
                " in your script."
            </p>
        }
        .into_any();
    }

    view! {
        <div class="space-y-6 font-sans">
            {sections
                .into_iter()
                .map(|section| {
                    let max = max_prob(&section.rows);
                    view! {
                        <div>
                            <h3 class="text-slate-300 font-semibold text-sm mb-2 m-0">
                                {section.title.clone()}
                            </h3>
                            <div class="grid grid-cols-[minmax(3rem,auto)_1fr] gap-x-3 gap-y-1.5 items-center">
                                {section
                                    .rows
                                    .into_iter()
                                    .map(|row| {
                                        let pct = bar_width_percent(row.prob, max);
                                        let width_style = format!("width: {pct:.2}%");
                                        let tooltip = format_tooltip(&row);
                                        view! {
                                            <span
                                                class="text-slate-200 text-right tabular-nums shrink-0"
                                                title=tooltip.clone()
                                            >
                                                {row.label.clone()}
                                            </span>
                                            <div
                                                class="h-6 min-w-0 rounded bg-slate-800/80 overflow-hidden"
                                                title=tooltip
                                            >
                                                <div
                                                    class="h-full rounded bg-emerald-600/90 min-w-[2px]"
                                                    style=width_style
                                                ></div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_width_scales_to_max() {
        assert!((bar_width_percent(0.5, 1.0) - 50.0).abs() < 1e-9);
        assert!((bar_width_percent(1.0, 1.0) - 100.0).abs() < 1e-9);
        assert_eq!(bar_width_percent(0.0, 1.0), 0.0);
    }

    #[test]
    fn dist_row_stats_share_and_at_least() {
        let rows = rows_from_dist_entries(&[(1, 0.1), (2, 0.2), (3, 0.7)]);
        assert!((rows[0].pct_of_total - 10.0).abs() < 1e-9);
        assert!((rows[2].pct_of_total - 70.0).abs() < 1e-9);
        assert!((rows[0].p_at_least - 1.0).abs() < 1e-9);
        assert!((rows[1].p_at_least - 0.9).abs() < 1e-9);
        assert!((rows[2].p_at_least - 0.7).abs() < 1e-9);
    }

    #[test]
    fn ordinal_p_at_least_follows_row_order() {
        let entries = [
            ("fail".into(), 0.25),
            ("ok".into(), 0.35),
            ("crit".into(), 0.40),
        ];
        let rows = rows_from_ordinal_entries(&entries);
        assert!((rows[1].p_at_least - 0.75).abs() < 1e-9);
    }

    #[test]
    fn dist_sections_have_six_rows() {
        let outputs = vec![OutputEntry::Dist {
            name: "d6".into(),
            entries: (1..=6).map(|i| (i, 1.0 / 6.0)).collect(),
            mean: 3.5,
        }];
        let sections = sections_from_outputs(&outputs);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].rows.len(), 6);
    }
}
