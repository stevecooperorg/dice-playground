use std::fmt::Write;

use serde::{Deserialize, Serialize};

#[cfg(feature = "cli")]
use clap::ValueEnum;

/// Show every outcome when the support is this small or smaller.
const PMF_FULL_MAX: usize = 64;
/// Tail prefix/suffix whose total mass is below this is one row (e.g. rare open-ended extremes).
const PMF_TAIL_MAX_SUM: f64 = 0.005;
/// Single outcomes below this are folded into tail bands instead of listed alone.
const PMF_INSIGNIFICANT: f64 = 0.001;
const FRACTION_MAX_DENOM: u64 = 10_000;
const PROB_MATCH_EPS: f64 = 1e-6;

/// How to print probabilities in text output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbFormat {
    /// e.g. `0.167`
    #[default]
    Decimal,
    /// e.g. `16.7%`
    Percent,
    /// e.g. `1/6` when a simple rational match exists
    Fraction,
    /// e.g. `6/36` on 2d6 — shared denominator for the whole distribution (not reduced)
    SampleSpace,
}

#[cfg(feature = "cli")]
impl ValueEnum for ProbFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Decimal,
            Self::Percent,
            Self::Fraction,
            Self::SampleSpace,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Decimal => clap::builder::PossibleValue::new("decimal"),
            Self::Percent => clap::builder::PossibleValue::new("percent"),
            Self::Fraction => clap::builder::PossibleValue::new("fraction"),
            Self::SampleSpace => clap::builder::PossibleValue::new("sample-space"),
        })
    }
}

/// Format a probability in the chosen style.
///
/// For [`ProbFormat::SampleSpace`], pass `sample_denom` from [`infer_sample_space_denominator`]
/// when formatting PMF rows; isolated prob outputs may omit it (falls back to decimal).
pub fn format_probability(p: f64, style: ProbFormat) -> String {
    format_probability_with_denom(p, style, None)
}

fn format_prob_decimal_fallback(p: f64) -> String {
    format!("{p:.3}")
}

pub fn format_probability_with_denom(
    p: f64,
    style: ProbFormat,
    sample_denom: Option<u64>,
) -> String {
    match style {
        ProbFormat::Decimal => format_prob_decimal_fallback(p),
        ProbFormat::Percent => format!("{:.1}%", p * 100.0),
        ProbFormat::Fraction => format_probability_fraction(p),
        ProbFormat::SampleSpace => format_probability_sample_space(p, sample_denom),
    }
}

/// Shared denominator from a list of probabilities (e.g. ordinal buckets summing to 1).
pub fn infer_sample_space_denominator_probs(probs: &[f64]) -> Option<u64> {
    if probs.is_empty() {
        return None;
    }
    let entries: Vec<(i64, f64)> = probs
        .iter()
        .enumerate()
        .map(|(i, &p)| (i as i64, p))
        .collect();
    infer_sample_space_denominator(&entries)
}

fn sample_space_valid(entries: &[(i64, f64)], d: u64) -> bool {
    let mut sum_n: u64 = 0;
    for &(_, p) in entries {
        if p.abs() < PROB_MATCH_EPS {
            continue;
        }
        let n = (p * d as f64).round();
        if n < 0.0 || (n / d as f64 - p).abs() > PROB_MATCH_EPS {
            return false;
        }
        sum_n += n as u64;
    }
    sum_n == d
}

/// Shared sample-space size (e.g. `6` for one die, `36` for 2d6 or bucketed 2d6 moves).
pub fn infer_sample_space_denominator(entries: &[(i64, f64)]) -> Option<u64> {
    if entries.is_empty() {
        return None;
    }
    let min_d = (1..=FRACTION_MAX_DENOM).find(|&d| sample_space_valid(entries, d))?;
    // Few outcome buckets on a dice grid (e.g. PbtA 2d6+stat) — prefer 36 when it fits.
    if entries.len() <= 5 && min_d < 36 && sample_space_valid(entries, 36) {
        return Some(36);
    }
    Some(min_d)
}

fn probability_as_simplified_fraction(p: f64) -> Option<(u64, u64)> {
    if p.abs() < PROB_MATCH_EPS {
        return Some((0, 1));
    }
    for den in 1..=FRACTION_MAX_DENOM {
        let num = (p * den as f64).round();
        if num < 0.0 {
            continue;
        }
        if (num / den as f64 - p).abs() > PROB_MATCH_EPS {
            continue;
        }
        let n = num as u64;
        let g = gcd_u64(n, den);
        return Some((n / g, den / g));
    }
    None
}

fn format_probability_sample_space(p: f64, sample_denom: Option<u64>) -> String {
    let Some(d) = sample_denom else {
        return format_prob_decimal_fallback(p);
    };
    if p.abs() < PROB_MATCH_EPS {
        return format!("0/{d}");
    }
    let n = (p * d as f64).round() as u64;
    format!("{n}/{d}")
}

/// Plain percent for PMF tables (no `%` suffix); at least four characters (e.g. `16.5`, `3.45`).
fn format_probability_percent_plain(p: f64) -> String {
    let v = p * 100.0;
    for decimals in 1..=4 {
        let s = match decimals {
            1 => format!("{v:.1}"),
            2 => format!("{v:.2}"),
            3 => format!("{v:.3}"),
            _ => format!("{v:.4}"),
        };
        if s.len() >= 4 {
            return s;
        }
    }
    format!("{v:.4}")
}

fn rel_column_header(sample_denom: Option<u64>) -> String {
    match sample_denom {
        Some(d) => format!("X/{d}"),
        None => "X".to_owned(),
    }
}

fn format_sample_space_numerator(p: f64, sample_denom: Option<u64>) -> String {
    let Some(d) = sample_denom else {
        return format_prob_decimal_fallback(p);
    };
    if p.abs() < PROB_MATCH_EPS {
        return "0".to_owned();
    }
    ((p * d as f64).round() as u64).to_string()
}

fn format_probability_fraction(p: f64) -> String {
    if (p - 1.0).abs() < PROB_MATCH_EPS {
        return "1".to_owned();
    }
    let Some((n, d)) = probability_as_simplified_fraction(p) else {
        return format_prob_decimal_fallback(p);
    };
    if n == 0 {
        return "0".to_owned();
    }
    format!("{n}/{d}")
}

fn outcome_is_numeric_label(label: &str) -> bool {
    label.parse::<i64>().is_ok()
        || label.starts_with('<')
        || label.starts_with('>')
        || label.contains("..")
}

fn format_value_band(lo: i64, hi: i64) -> String {
    if lo == hi {
        lo.to_string()
    } else {
        format!("{lo}..{hi}")
    }
}

fn probs_match(a: f64, b: f64) -> bool {
    (a - b).abs() <= PROB_MATCH_EPS
}

/// How many leading entries to fold into one negligible tail row (total mass &lt; 0.5%).
fn take_low_tail(entries: &[(i64, f64)]) -> (usize, Option<(String, f64)>) {
    let n = entries.len();
    let mut i = 0;
    let mut sum = 0.0;
    while i < n {
        let p = entries[i].1;
        if sum + p < PMF_TAIL_MAX_SUM {
            sum += p;
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 {
        return (0, None);
    }
    let lo = entries[0].0;
    let hi = entries[i - 1].0;
    let label = if i < n && hi + 1 == entries[i].0 {
        format!("<{}", entries[i].0)
    } else {
        format_value_band(lo, hi)
    };
    (i, Some((label, sum)))
}

/// How many trailing entries to fold into one negligible tail row (total mass &lt; 0.5%).
fn take_high_tail(entries: &[(i64, f64)]) -> (usize, Option<(String, f64)>) {
    let n = entries.len();
    let mut i = 0;
    let mut sum = 0.0;
    while i < n {
        let p = entries[n - 1 - i].1;
        if sum + p < PMF_TAIL_MAX_SUM {
            sum += p;
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 {
        return (0, None);
    }
    let hi = entries[n - 1].0;
    let lo = entries[n - i].0;
    let label = if i < n && lo == entries[n - i - 1].0 + 1 {
        format!(">{}", entries[n - i - 1].0)
    } else {
        format_value_band(lo, hi)
    };
    (i, Some((label, sum)))
}

fn parse_band_bounds(label: &str) -> (Option<i64>, Option<i64>) {
    if let Ok(v) = label.parse::<i64>() {
        return (Some(v), Some(v));
    }
    if let Some(rest) = label.strip_prefix('<') {
        let bound: Option<i64> = rest.parse().ok();
        return (None, bound.map(|b| b - 1));
    }
    if let Some(rest) = label.strip_prefix('>') {
        let bound: Option<i64> = rest.parse().ok();
        return (bound.map(|b| b + 1), None);
    }
    if let Some((lo, hi)) = label.split_once("..") {
        return (lo.parse().ok(), hi.parse().ok());
    }
    (None, None)
}

/// Open-ended labels on the first/last row when a gap separates them from the neighbor band.
fn polish_adjacent_band_labels(rows: &mut [(String, f64)]) {
    if rows.len() < 2 {
        return;
    }
    let (_, hi_0) = parse_band_bounds(&rows[0].0);
    let (lo_1, _) = parse_band_bounds(&rows[1].0);
    if let (Some(hi), Some(next_lo)) = (hi_0, lo_1) {
        if hi + 1 < next_lo {
            rows[0].0 = format!("<{next_lo}");
        }
    }
    let last = rows.len() - 1;
    let (_, hi_prev) = parse_band_bounds(&rows[last - 1].0);
    let (lo_last, _) = parse_band_bounds(&rows[last].0);
    if let (Some(prev_hi), Some(lo)) = (hi_prev, lo_last) {
        if prev_hi + 1 < lo {
            rows[last].0 = format!(">{}", lo - 1);
        }
    }
}

/// Chunk consecutive low-mass outcomes so each row is at most [`PMF_TAIL_MAX_SUM`].
fn push_insignificant_chunk(
    out: &mut Vec<(String, f64)>,
    mid: &[(i64, f64)],
    start: usize,
) -> usize {
    let lo = mid[start].0;
    let mut sum = 0.0;
    let mut hi = lo;
    let mut j = start;
    while j < mid.len() && mid[j].1 < PMF_INSIGNIFICANT {
        let p = mid[j].1;
        if sum + p >= PMF_TAIL_MAX_SUM && sum > 0.0 {
            break;
        }
        sum += p;
        hi = mid[j].0;
        j += 1;
    }
    if j > start {
        out.push((format_value_band(lo, hi), sum));
    }
    j
}

/// Equal-probability bands for the midrange; negligible outcomes in capped chunks.
fn compress_mid_bands(mid: &[(i64, f64)]) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < mid.len() {
        let (v0, p0) = mid[i];
        if p0 < PMF_INSIGNIFICANT {
            i = push_insignificant_chunk(&mut out, mid, i);
            continue;
        }
        let mut hi = v0;
        let mut sum = p0;
        let mut j = i + 1;
        while j < mid.len() {
            let (v, p) = mid[j];
            if p < PMF_INSIGNIFICANT {
                break;
            }
            if v == hi + 1 && probs_match(p, p0) {
                hi = v;
                sum += p;
                j += 1;
            } else {
                break;
            }
        }
        out.push((format_value_band(v0, hi), sum));
        i = j;
    }
    out
}

/// Compress a numeric PMF for tables and charts: insignificant tails, then equal-probability bands.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DieRoll;
/// use dice_playground::engine::compress_pmf_for_display;
/// let oe = DieRoll::open_ended_d100(8).unwrap();
/// let rows = compress_pmf_for_display(&oe.entries());
/// assert!(rows.iter().any(|(label, _)| label == "6..95"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn compress_pmf_for_display(entries: &[(i64, f64)]) -> Vec<(String, f64)> {
    if entries.is_empty() {
        return Vec::new();
    }
    if entries.len() <= PMF_FULL_MAX {
        return entries.iter().map(|&(v, p)| (v.to_string(), p)).collect();
    }
    let (lo_skip, lo_row) = take_low_tail(entries);
    let (hi_skip, hi_row) = take_high_tail(entries);
    let mid_end = entries.len().saturating_sub(hi_skip);
    let mid = if lo_skip < mid_end {
        &entries[lo_skip..mid_end]
    } else {
        &[][..]
    };
    let mut rows = Vec::new();
    if let Some(r) = lo_row {
        rows.push(r);
    }
    rows.extend(compress_mid_bands(mid));
    if let Some(r) = hi_row {
        rows.push(r);
    }
    polish_adjacent_band_labels(&mut rows);
    rows
}

fn pad_left(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_owned()
    } else {
        format!("{:>width$}", s, width = width)
    }
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_owned()
    } else {
        format!("{:<width$}", s, width = width)
    }
}

fn render_multi_format_table(rows: &[(String, f64)], sample_denom: Option<u64>) -> String {
    let rel_header = rel_column_header(sample_denom);
    let numeric_outcomes = rows
        .iter()
        .all(|(label, _)| outcome_is_numeric_label(label));
    let outcome_width = rows
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(1)
        .max(4);
    let pct_width = rows
        .iter()
        .map(|(_, p)| format_probability_percent_plain(*p).len())
        .max()
        .unwrap_or(1)
        .max(1);
    let frac_width = rows
        .iter()
        .map(|(_, p)| format_probability_fraction(*p).len())
        .max()
        .unwrap_or(1)
        .max(4);
    let rel_width = rows
        .iter()
        .map(|(_, p)| format_sample_space_numerator(*p, sample_denom).len())
        .max()
        .unwrap_or(1)
        .max(rel_header.len());

    let mut out = String::new();
    let outcome_hdr = if numeric_outcomes {
        pad_left("", outcome_width)
    } else {
        pad_right("", outcome_width)
    };
    let _ = writeln!(
        out,
        "{} | {} | {} | {}",
        outcome_hdr,
        pad_left("%", pct_width),
        pad_left("frac", frac_width),
        pad_left(&rel_header, rel_width),
    );
    for (outcome, p) in rows {
        let outcome_cell = if numeric_outcomes {
            pad_left(outcome, outcome_width)
        } else {
            pad_right(outcome, outcome_width)
        };
        let _ = writeln!(
            out,
            "{} | {} | {} | {}",
            outcome_cell,
            pad_left(&format_probability_percent_plain(*p), pct_width),
            pad_left(&format_probability_fraction(*p), frac_width),
            pad_left(&format_sample_space_numerator(*p, sample_denom), rel_width,),
        );
    }
    out
}

fn append_distribution_table(out: &mut String, rows: &[(String, f64)], sample_denom: Option<u64>) {
    if rows.is_empty() {
        return;
    }
    let table = render_multi_format_table(rows, sample_denom);
    for line in table.lines() {
        let _ = writeln!(out, "  {line}");
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Independent probability rows (e.g. a modifier × target grid); same columns as ordinal PMF tables.
pub fn format_prob_table_text(
    name: &str,
    entries: &[(String, f64)],
    _prob: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let probs: Vec<f64> = entries.iter().map(|(_, p)| *p).collect();
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator_probs(&probs));
    let mut out = String::new();
    let _ = writeln!(out, "output {name}: Table");
    let rows: Vec<(String, f64)> = entries
        .iter()
        .map(|(label, p)| (label.clone(), *p))
        .collect();
    append_distribution_table(&mut out, &rows, sample_denom);
    out
}

/// Format an ordered label distribution for human-readable CLI text.
pub fn format_ordinal_pmf_text(
    name: &str,
    entries: &[(String, f64)],
    _prob: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let probs: Vec<f64> = entries.iter().map(|(_, p)| *p).collect();
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator_probs(&probs));
    let mut out = String::new();
    let _ = writeln!(out, "output {name}: Ordinal");
    let rows: Vec<(String, f64)> = entries
        .iter()
        .map(|(label, p)| (label.clone(), *p))
        .collect();
    append_distribution_table(&mut out, &rows, sample_denom);
    out
}

/// Format a distribution for human-readable CLI text (PMF table with optional tail summaries).
pub fn format_dist_pmf_text(
    name: &str,
    entries: &[(i64, f64)],
    mean: f64,
    _prob: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "output {name}: DieRoll mean={mean:.6}");
    write_pmf_body(&mut out, entries, shared_sample_denom);
    out
}

fn write_pmf_body(out: &mut String, entries: &[(i64, f64)], shared_sample_denom: Option<u64>) {
    if entries.is_empty() {
        return;
    }
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator(entries));
    let rows = compress_pmf_for_display(entries);
    append_distribution_table(out, &rows, sample_denom);
}

/// Single probability as the same multi-column layout (one data row).
pub fn format_prob_multi_column(name: &str, value: f64, sample_denom: Option<u64>) -> String {
    let mut out = String::new();
    append_distribution_table(&mut out, &[(name.to_owned(), value)], sample_denom);
    out
}

// --- GFM tables (HTML weave / static render) ---

/// Escape `|` and `\` for GFM pipe table cells.
pub fn escape_gfm_table_cell(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '|' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Build a GFM pipe table (header + separator + rows).
pub fn gfm_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push('|');
    for h in headers {
        out.push(' ');
        out.push_str(&escape_gfm_table_cell(h));
        out.push_str(" |");
    }
    out.push('\n');
    out.push('|');
    for _ in headers {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in rows {
        out.push('|');
        for cell in row {
            out.push(' ');
            out.push_str(&escape_gfm_table_cell(cell));
            out.push_str(" |");
        }
        out.push('\n');
    }
    out
}

fn markdown_output_caption(name: &str, kind: &str) -> String {
    let safe = escape_gfm_table_cell(name);
    format!("**{safe}** · {kind}\n\n")
}

fn distribution_gfm_table(
    rows: &[(String, f64)],
    sample_denom: Option<u64>,
    prob_format: ProbFormat,
    multi_column: bool,
) -> String {
    if rows.is_empty() {
        return String::new();
    }
    if multi_column {
        let rel = rel_column_header(sample_denom);
        let headers = ["outcome", "%", "frac", rel.as_str()];
        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|(label, p)| {
                vec![
                    label.clone(),
                    format_probability_percent_plain(*p),
                    format_probability_fraction(*p),
                    format_sample_space_numerator(*p, sample_denom),
                ]
            })
            .collect();
        gfm_table(&headers, &data)
    } else {
        let headers = ["outcome", "p"];
        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|(label, p)| {
                vec![
                    label.clone(),
                    format_probability_with_denom(*p, prob_format, sample_denom),
                ]
            })
            .collect();
        gfm_table(&headers, &data)
    }
}

/// True when woven reports show `%`, `frac`, and sample-space columns together.
pub fn woven_table_multi_column(prob_format: ProbFormat) -> bool {
    prob_format == ProbFormat::Decimal
}

/// GFM markdown for a DieRoll PMF block (caption + table).
pub fn format_dist_pmf_gfm(
    name: &str,
    entries: &[(i64, f64)],
    mean: f64,
    prob_format: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let mut out = markdown_output_caption(name, &format!("DieRoll · mean {mean:.3}"));
    if entries.is_empty() {
        return out;
    }
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator(entries));
    let rows = compress_pmf_for_display(entries);
    let multi = woven_table_multi_column(prob_format);
    out.push_str(&distribution_gfm_table(&rows, sample_denom, prob_format, multi));
    out
}

/// GFM markdown for ordered outcomes.
pub fn format_ordinal_pmf_gfm(
    name: &str,
    entries: &[(String, f64)],
    prob_format: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let mut out = markdown_output_caption(name, "Outcomes");
    if entries.is_empty() {
        return out;
    }
    let probs: Vec<f64> = entries.iter().map(|(_, p)| *p).collect();
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator_probs(&probs));
    let rows: Vec<(String, f64)> = entries
        .iter()
        .map(|(label, p)| (label.clone(), *p))
        .collect();
    let multi = woven_table_multi_column(prob_format);
    out.push_str(&distribution_gfm_table(&rows, sample_denom, prob_format, multi));
    out
}

/// GFM markdown for `prob_table` rows.
pub fn format_prob_table_gfm(
    name: &str,
    entries: &[(String, f64)],
    prob_format: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let mut out = markdown_output_caption(name, "Table");
    if entries.is_empty() {
        return out;
    }
    let probs: Vec<f64> = entries.iter().map(|(_, p)| *p).collect();
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator_probs(&probs));
    let rows: Vec<(String, f64)> = entries
        .iter()
        .map(|(label, p)| (label.clone(), *p))
        .collect();
    let multi = woven_table_multi_column(prob_format);
    out.push_str(&distribution_gfm_table(&rows, sample_denom, prob_format, multi));
    out
}

/// GFM markdown for a single scalar probability (one-row table).
pub fn format_prob_gfm(
    name: &str,
    value: f64,
    prob_format: ProbFormat,
    sample_denom: Option<u64>,
) -> String {
    let mut out = markdown_output_caption(name, "Prob");
    let multi = woven_table_multi_column(prob_format);
    out.push_str(&distribution_gfm_table(
        &[(name.to_owned(), value)],
        sample_denom,
        prob_format,
        multi,
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(n: usize) -> Vec<(i64, f64)> {
        let p = 1.0 / n as f64;
        (0..n as i64).map(|k| (k, p)).collect()
    }

    #[test]
    fn percent_plain_min_four_chars() {
        assert_eq!(format_probability_percent_plain(1.0 / 6.0), "16.7");
        assert_eq!(format_probability_percent_plain(0.0345), "3.45");
        assert_eq!(format_probability_percent_plain(0.035), "3.50");
        assert!(format_probability_percent_plain(0.001).len() >= 4);
    }

    #[test]
    fn format_one_sixth() {
        let p = 1.0 / 6.0;
        assert_eq!(format_probability(p, ProbFormat::Decimal), "0.167");
        assert_eq!(format_probability(p, ProbFormat::Percent), "16.7%");
        assert_eq!(format_probability(p, ProbFormat::Fraction), "1/6");
    }

    #[test]
    fn gfm_table_renders_pipe_rows() {
        let md = gfm_table(
            &["a", "b"],
            &[vec!["1".into(), "2".into()], vec!["x|y".into(), "z".into()]],
        );
        assert!(md.contains("| a | b |"));
        assert!(md.contains("x\\|y"));
        assert!(md.contains("| --- |"));
    }

    #[test]
    fn dist_pmf_gfm_includes_table_syntax() {
        let entries: Vec<(i64, f64)> = (1..=6).map(|k| (k, 1.0 / 6.0)).collect();
        let md = format_dist_pmf_gfm("d6", &entries, 3.5, ProbFormat::Decimal, None);
        assert!(md.contains("**d6**"));
        assert!(md.contains("| outcome |"));
        assert!(md.contains("1/6"));
    }

    #[test]
    fn small_dist_prints_all_outcomes() {
        let entries: Vec<(i64, f64)> = (3..=10).map(|k| (k, 0.125)).collect();
        let text = format_dist_pmf_text("x", &entries, 6.5, ProbFormat::Decimal, None);
        assert!(text.contains("output x: DieRoll mean=6.500000"));
        for k in 3..=10 {
            assert!(text.contains(&k.to_string()));
        }
        assert!(!text.contains('<'));
        assert!(!text.contains('>'));
    }

    #[test]
    fn d6_multi_column_output() {
        let entries: Vec<(i64, f64)> = (1..=6).map(|k| (k, 1.0 / 6.0)).collect();
        let text = format_dist_pmf_text("d6", &entries, 3.5, ProbFormat::Fraction, None);
        assert!(text.contains("X/6"));
        assert!(text.contains("frac"));
        assert!(!text.contains("16.7%"));
        assert_eq!(text.matches("16.7").count(), 6);
        assert_eq!(text.matches("1/6").count(), 6);
    }

    #[test]
    fn ordinal_sample_space_uses_shared_denom() {
        let entries = [
            ("MISS".to_owned(), 1.0 / 6.0),
            ("PARTIAL".to_owned(), 5.0 / 12.0),
            ("FULL_SUCCESS".to_owned(), 5.0 / 12.0),
        ];
        let text = format_ordinal_pmf_text("move", &entries, ProbFormat::SampleSpace, None);
        assert!(text.contains("MISS"));
        assert!(text.contains("X/36"));
        assert!(text.contains("PARTIAL"));
        assert!(!text.contains("15/36"));
        assert!(text.contains("FULL_SUCCESS"));
    }

    #[test]
    fn two_d6_sample_space_uses_36() {
        use crate::engine::DieRoll;
        let two_d6 = DieRoll::pool_sum(2, 6).expect("2d6");
        let entries = two_d6.entries();
        assert_eq!(infer_sample_space_denominator(&entries), Some(36));
        let text = format_dist_pmf_text(
            "two_d6",
            &entries,
            two_d6.mean(),
            ProbFormat::SampleSpace,
            None,
        );
        assert!(
            text.contains("X/36"),
            "expected X/36 header for 2d6, got:\n{text}"
        );
        // count column is numerators only (e.g. 6 for the 7 outcome), not 6/36
        assert!(!text.contains("6/36"));
    }

    #[test]
    fn large_uniform_dist_one_band() {
        let entries = uniform(100);
        let rows = compress_pmf_for_display(&entries);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "0..99");
        assert!((rows[0].1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn open_ended_d100_compresses_flat_midrange() {
        use crate::engine::DieRoll;
        let oe = DieRoll::open_ended_d100(8).unwrap();
        let rows = compress_pmf_for_display(&oe.entries());
        assert!(
            rows.iter()
                .any(|(label, p)| label == "6..95" && (*p - 0.90).abs() < 1e-9),
            "rows: {rows:?}"
        );
        for (label, p) in &rows {
            if label.starts_with('<') || label.starts_with('>') {
                assert!(
                    *p < PMF_TAIL_MAX_SUM + 1e-9,
                    "tail row {label} mass {p} should be under {PMF_TAIL_MAX_SUM}"
                );
            }
        }
        assert!(
            rows.len() < 120,
            "expected far fewer than {} rows",
            oe.support_size()
        );
    }
}
