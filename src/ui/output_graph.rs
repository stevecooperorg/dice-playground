//! Chart view of `output()` PMFs from eval results (full data; no display compression).

use crate::engine::OutputEntry;
use leptos::prelude::*;
use leptos_chartistry::*;

/// Full width of the parent output panel; height follows width / ratio.
const CHART_ASPECT: AspectRatio = AspectRatio::from_env_width_apply_ratio(2.0);

fn bottom_axis_ticks() -> TickLabels<f64> {
    TickLabels::aligned_floats().with_min_chars(3)
}

/// Bottom axis for categorical bars: outcome names at bar centers only.
///
/// Chartistry's `aligned_floats` also places "nice" ticks between categories (e.g. `1.5`)
/// because we extend the x range for bar padding. Those in-between ticks are left blank so
/// only outcome labels show, not stray numbers.
fn ordinal_bottom_ticks(rows: &[BarRow]) -> TickLabels<f64> {
    let labels: Vec<String> = rows.iter().map(|r| r.label.clone()).collect();
    let min_chars = labels
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(3)
        .clamp(3, 32);
    TickLabels::aligned_floats()
        .with_min_chars(min_chars)
        .with_format(move |v: &f64, _fmt| {
            if labels.len() == 1 {
                return labels[0].clone();
            }
            let mut best_idx = 0usize;
            let mut best_dist = f64::INFINITY;
            for (i, _) in labels.iter().enumerate() {
                let centre = (i + 1) as f64;
                let dist = (v - centre).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = i;
                }
            }
            // Label ticks at category centers; hide midway ticks (e.g. 1.5) without showing numbers.
            if best_dist < 0.4 {
                labels[best_idx].clone()
            } else {
                String::new()
            }
        })
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
        .filter_map(|entry| {
            Some(match entry {
                OutputEntry::DieRoll { name, entries, .. } => {
                    if entries.is_empty() {
                        return None;
                    }
                    OutputChart::DieRollLine {
                        title: name.clone(),
                        entries: entries.clone(),
                    }
                }
                OutputEntry::Outcomes { name, entries, .. }
                | OutputEntry::Table { name, entries } => {
                    let rows = rows_from_ordinal_entries(entries);
                    if rows.is_empty() {
                        return None;
                    }
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

#[derive(Clone, Debug, PartialEq)]
struct BarChartDatum {
    x: f64,
    label: String,
    prob: f64,
}

fn bar_chart_data_from_rows(rows: &[BarRow]) -> Vec<BarChartDatum> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| BarChartDatum {
            x: (i + 1) as f64,
            label: row.label.clone(),
            prob: row.prob,
        })
        .collect()
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
    let data_vec = bar_chart_data_from_rows(&rows);
    let y_max = y_axis_max_prob(max_prob_f64(rows.iter().map(|r| r.prob)));
    let (data, _) = signal(data_vec);
    let debug = Signal::from(false);

    let left = TickLabels::aligned_floats()
        .with_min_chars(5)
        .with_format(|v: &f64, _| format_prob_pct(*v));
    let tooltip = Tooltip::left_cursor()
        .with_sort_by(TooltipSortBy::Descending)
        .skip_missing(true);

    let bar_colour = Colour::from_rgb(0x05, 0x96, 0x69);
    let n = rows.len() as f64;
    let series = Series::new(|d: &BarChartDatum| d.x)
        .bar(
            Bar::new(|d: &BarChartDatum| d.prob)
                .with_name("P")
                .with_colour(bar_colour)
                .with_gap(0.28),
        )
        .with_x_range(0.5, n + 0.5)
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
                bottom=ordinal_bottom_ticks(&rows)
                tooltip=tooltip
                inner=chart_axes_only()
            />
            </div>
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
    fn line_data_sorted_by_face() {
        let data = line_data_from_entries(&[(3, 0.2), (1, 0.5), (2, 0.3)]);
        assert!((data[0].x - 1.0).abs() < 1e-9);
        assert!((data[2].x - 3.0).abs() < 1e-9);
    }
}
