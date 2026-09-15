//! Chart view of `output()` PMFs from eval results (full data; no display compression).

use crate::engine::{output_entry_supports_chart, OutputEntry};
use leptos::prelude::*;
use leptos_chartistry::*;

/// Full width of the parent output panel; height follows width / ratio.
const CHART_ASPECT: AspectRatio = AspectRatio::from_env_width_apply_ratio(2.0);

fn bottom_axis_ticks() -> TickLabels<f64> {
    TickLabels::aligned_floats().with_min_chars(3)
}

fn chart_axes_only() -> [InnerLayout<f64, f64>; 2] {
    [
        AxisMarker::left_edge().into_inner(),
        AxisMarker::bottom_edge().into_inner(),
    ]
}

#[derive(Clone, Debug, PartialEq)]
struct BarRow {
    label: String,
    prob: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct LineDatum {
    x: f64,
    prob: f64,
}

#[derive(Clone, Debug, PartialEq)]
enum OutputChart {
    DieRollLine {
        title: String,
        entries: Vec<(i64, f64)>,
    },
    OrdinalBar {
        title: String,
        rows: Vec<BarRow>,
    },
    ProbBar {
        title: String,
        rows: Vec<BarRow>,
    },
}

fn rows_from_ordinal_entries(entries: &[(String, f64)]) -> Vec<BarRow> {
    entries
        .iter()
        .map(|(label, p)| BarRow {
            label: label.clone(),
            prob: *p,
        })
        .collect()
}

fn rows_from_prob(value: f64) -> Vec<BarRow> {
    vec![BarRow {
        label: format_prob_label(value),
        prob: value,
    }]
}

fn format_prob_pct(p: f64) -> String {
    if p.is_finite() && (0.0..=1.0).contains(&p) {
        format!("{:.2}%", p * 100.0)
    } else {
        format!("{p}")
    }
}

fn charts_from_outputs(outputs: &[OutputEntry]) -> Vec<OutputChart> {
    outputs
        .iter()
        .filter(|entry| output_entry_supports_chart(entry))
        .map(|entry| match entry {
            OutputEntry::DieRoll { name, entries, .. } => OutputChart::DieRollLine {
                title: name.clone(),
                entries: entries.clone(),
            },
            OutputEntry::Outcomes { name, entries, .. } | OutputEntry::Table { name, entries } => {
                let rows = rows_from_ordinal_entries(entries);
                OutputChart::OrdinalBar {
                    title: name.clone(),
                    rows,
                }
            }
            OutputEntry::Prob { name, value } => OutputChart::ProbBar {
                title: name.clone(),
                rows: rows_from_prob(*value),
            },
        })
        .collect()
}

/// Chart for one output (inline report hydration).
#[component]
pub fn OutputEntryChart(entry: OutputEntry) -> AnyView {
    match charts_from_outputs(&[entry]).into_iter().next() {
        Some(OutputChart::DieRollLine { title, entries }) => {
            view! { <DieRollLineChart title=title entries=entries /> }.into_any()
        }
        Some(OutputChart::OrdinalBar { title, rows } | OutputChart::ProbBar { title, rows }) => {
            view! { <OrdinalBarChart title=title rows=rows /> }.into_any()
        }
        None => ().into_any(),
    }
}

fn format_prob_label(p: f64) -> String {
    if p.is_finite() && (0.0..=1.0).contains(&p) {
        format!("{:.4}", p)
    } else {
        format!("{p}")
    }
}

fn line_data_from_entries(entries: &[(i64, f64)]) -> Vec<LineDatum> {
    let mut data: Vec<LineDatum> = entries
        .iter()
        .map(|(k, p)| LineDatum {
            x: *k as f64,
            prob: *p,
        })
        .collect();
    data.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    data
}

fn max_prob_f64(values: impl Iterator<Item = f64>) -> f64 {
    values
        .filter(|p| p.is_finite() && *p > 0.0)
        .fold(0.0_f64, f64::max)
}

fn y_axis_max_prob(max: f64) -> f64 {
    if max.is_finite() && max > 0.0 {
        max
    } else {
        1.0
    }
}

/// Keep labels compact without rounding tiny, nonzero chances to zero.
fn compact_prob_pct(p: f64) -> String {
    if p > 0.0 && p < 0.0001 {
        return "<0.01%".into();
    }
    if p > 0.9999 && p < 1.0 {
        return ">99.99%".into();
    }
    let formatted = format_prob_pct(p);
    match formatted.strip_suffix('%') {
        Some(number) => format!("{}%", number.trim_end_matches('0').trim_end_matches('.')),
        None => formatted,
    }
}

/// Every track uses the same 0–100% scale, including single-outcome charts.
fn bar_width_pct(p: f64) -> f64 {
    if p.is_finite() {
        p.clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    }
}

#[component]
fn DieRollLineChart(title: String, entries: Vec<(i64, f64)>) -> impl IntoView {
    let data_vec = line_data_from_entries(&entries);
    let y_max = y_axis_max_prob(max_prob_f64(data_vec.iter().map(|d| d.prob)));
    let (data, _) = signal(data_vec);
    let debug = Signal::from(false);

    let left = TickLabels::aligned_floats()
        .with_min_chars(5)
        .with_format(|v: &f64, _| format_prob_pct(*v));
    let tooltip = Tooltip::left_cursor().show_x_ticks(false);

    let line_colour = Colour::from_rgb(0x05, 0x96, 0x69);
    let series = Series::new(|d: &LineDatum| d.x)
        .line(
            Line::new(|d: &LineDatum| d.prob)
                .with_name("P")
                .with_colour(line_colour),
        )
        .with_y_range(0.0, y_max);

    view! {
        <div class="w-full min-w-0 dice-chartistry">
            <h3 class="text-slate-300 font-semibold text-sm mb-2 m-0">{title}</h3>
            <div class="w-full min-w-0">
            <Chart
                aspect_ratio=CHART_ASPECT
                debug=debug
                series=series
                data=data
                left=left
                bottom=bottom_axis_ticks()
                tooltip=tooltip
                inner=chart_axes_only()
            />
            </div>
        </div>
    }
}

#[component]
fn OrdinalBarChart(title: String, rows: Vec<BarRow>) -> impl IntoView {
    view! {
        <div class="w-full min-w-0">
            <h3 class="text-slate-300 font-semibold text-sm mb-2 m-0">{title.clone()}</h3>
            <ul class="list-none p-0 m-0 space-y-3" aria-label=title>
                {rows.into_iter().map(|row| {
                    let chance = compact_prob_pct(row.prob);
                    let width = format!("width: {}%;", bar_width_pct(row.prob));
                    view! {
                        <li>
                            <div class="text-sm text-slate-200 mb-1" style="overflow-wrap: anywhere;">
                                <span>{row.label}</span>
                                <span class="text-slate-400">" · "</span>
                                <span class="font-mono whitespace-nowrap">{chance}</span>
                            </div>
                            <div class="w-full h-2 rounded bg-slate-700 overflow-hidden" aria-hidden="true">
                                <div class="h-full rounded bg-emerald-600" style=width></div>
                            </div>
                        </li>
                    }
                }).collect_view()}
            </ul>
            <p class="text-xs text-slate-400 mt-2 mb-0">"Bar scale: 0–100%"</p>
        </div>
    }
}

#[component]
pub fn OutputGraphView(outputs: Vec<OutputEntry>) -> AnyView {
    let charts = charts_from_outputs(&outputs);
    if charts.is_empty() {
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
        <div class="w-full min-w-0 space-y-6 font-sans">
            {charts
                .into_iter()
                .map(|chart| match chart {
                    OutputChart::DieRollLine { title, entries } => {
                        view! { <DieRollLineChart title=title entries=entries /> }.into_any()
                    }
                    OutputChart::OrdinalBar { title, rows }
                    | OutputChart::ProbBar { title, rows } => {
                        view! { <OrdinalBarChart title=title rows=rows /> }.into_any()
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
    fn die_roll_chart_uses_full_entry_count() {
        let outputs = vec![OutputEntry::DieRoll {
            name: "d6".into(),
            entries: (1..=6).map(|i| (i, 1.0 / 6.0)).collect(),
            mean: 3.5,
        }];
        let charts = charts_from_outputs(&outputs);
        assert_eq!(charts.len(), 1);
        match &charts[0] {
            OutputChart::DieRollLine { entries, .. } => assert_eq!(entries.len(), 6),
            _ => panic!("expected line chart for dieroll"),
        }
    }

    #[test]
    fn outcomes_use_bar_chart() {
        let outputs = vec![OutputEntry::Outcomes {
            name: "check".into(),
            scale: vec!["fail".into(), "ok".into()],
            entries: vec![("fail".into(), 0.4), ("ok".into(), 0.6)],
        }];
        let charts = charts_from_outputs(&outputs);
        match &charts[0] {
            OutputChart::OrdinalBar { rows, .. } => assert_eq!(rows.len(), 2),
            _ => panic!("expected bar chart for outcomes"),
        }
    }

    #[test]
    fn compact_percentages_preserve_useful_precision() {
        for (prob, expected) in [
            (0.41, "41%"),
            (0.23, "23%"),
            (0.125, "12.5%"),
            (1.0 / 6.0, "16.67%"),
            (0.0, "0%"),
            (1.0, "100%"),
            (0.00001, "<0.01%"),
            (0.99999, ">99.99%"),
            (-1.0, "-1"),
        ] {
            assert_eq!(compact_prob_pct(prob), expected);
        }
        assert_eq!(compact_prob_pct(f64::NAN), "NaN");
    }

    #[test]
    fn bars_use_absolute_probability_and_safe_widths() {
        assert_eq!(bar_width_pct(0.23), 23.0);
        assert_eq!(bar_width_pct(0.0), 0.0);
        assert_eq!(bar_width_pct(1.0), 100.0);
        assert_eq!(bar_width_pct(-0.5), 0.0);
        assert_eq!(bar_width_pct(2.0), 100.0);
        assert_eq!(bar_width_pct(f64::NAN), 0.0);
        assert_eq!(bar_width_pct(f64::INFINITY), 0.0);
    }

    #[test]
    fn bar_rows_preserve_scale_labels_and_order() {
        let entries = vec![("FAIL".into(), 0.41), ("3".into(), 0.23)];
        let rows = rows_from_ordinal_entries(&entries);
        assert_eq!(rows[0].label, "FAIL");
        assert_eq!(compact_prob_pct(rows[0].prob), "41%");
        assert_eq!(rows[1].label, "3");
        assert_eq!(compact_prob_pct(rows[1].prob), "23%");
        assert!(rows_from_ordinal_entries(&[]).is_empty());
    }

    #[test]
    fn line_data_sorted_by_face() {
        let data = line_data_from_entries(&[(3, 0.2), (1, 0.5), (2, 0.3)]);
        assert!((data[0].x - 1.0).abs() < 1e-9);
        assert!((data[2].x - 3.0).abs() < 1e-9);
    }
}
