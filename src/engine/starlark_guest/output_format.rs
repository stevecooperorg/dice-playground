use std::fmt::Write;

use serde::{Deserialize, Serialize};

#[cfg(feature = "cli")]
use clap::ValueEnum;

const PMF_FULL_MAX: usize = 64;
const PMF_MIDDLE_KEEP: usize = 62;
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

pub fn format_probability_with_denom(p: f64, style: ProbFormat, sample_denom: Option<u64>) -> String {
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
    label.parse::<i64>().is_ok() || label.starts_with('<') || label.starts_with('>')
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
            pad_left(
                &format_sample_space_numerator(*p, sample_denom),
                rel_width,
            ),
        );
    }
    out
}

fn append_distribution_table(
    out: &mut String,
    rows: &[(String, f64)],
    sample_denom: Option<u64>,
) {
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
    let _ = writeln!(out, "output {name}: Dist mean={mean:.6}");
    write_pmf_body(&mut out, entries, shared_sample_denom);
    out
}

fn write_pmf_body(
    out: &mut String,
    entries: &[(i64, f64)],
    shared_sample_denom: Option<u64>,
) {
    let n = entries.len();
    if n == 0 {
        return;
    }
    let sample_denom = shared_sample_denom.or_else(|| infer_sample_space_denominator(entries));
    let mut rows: Vec<(String, f64)> = Vec::new();

    if n <= PMF_FULL_MAX {
        for &(value, p) in entries {
            rows.push((value.to_string(), p));
        }
        append_distribution_table(out, &rows, sample_denom);
        return;
    }

    let keep = PMF_MIDDLE_KEEP;
    let skip = (n - keep) / 2;
    let first_shown = entries[skip].0;
    let last_shown = entries[skip + keep - 1].0;

    let low_mass: f64 = entries[..skip].iter().map(|(_, p)| p).sum();
    if low_mass > 0.0 {
        rows.push((format!("<{first_shown}"), low_mass));
    }

    for &(value, p) in &entries[skip..skip + keep] {
        rows.push((value.to_string(), p));
    }

    let high_mass: f64 = entries[skip + keep..].iter().map(|(_, p)| p).sum();
    if high_mass > 0.0 {
        rows.push((format!(">{last_shown}"), high_mass));
    }

    append_distribution_table(out, &rows, sample_denom);
}

/// Single probability as the same multi-column layout (one data row).
pub fn format_prob_multi_column(name: &str, value: f64, sample_denom: Option<u64>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "output {name}: Prob");
    append_distribution_table(&mut out, &[("P".to_owned(), value)], sample_denom);
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
    fn small_dist_prints_all_outcomes() {
        let entries: Vec<(i64, f64)> = (3..=10).map(|k| (k, 0.125)).collect();
        let text = format_dist_pmf_text("x", &entries, 6.5, ProbFormat::Decimal, None);
        assert!(text.contains("output x: Dist mean=6.500000"));
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
        use crate::engine::Dist;
        let two_d6 = Dist::pool_sum(2, 6).expect("2d6");
        let entries = two_d6.entries();
        assert_eq!(infer_sample_space_denominator(&entries), Some(36));
        let text = format_dist_pmf_text("two_d6", &entries, two_d6.mean(), ProbFormat::SampleSpace, None);
        assert!(text.contains("X/36"), "expected X/36 header for 2d6, got:\n{text}");
        // count column is numerators only (e.g. 6 for the 7 outcome), not 6/36
        assert!(!text.contains("6/36"));
    }

    #[test]
    fn large_dist_middle_and_tails() {
        let entries = uniform(100);
        let text = format_dist_pmf_text("big", &entries, 49.5, ProbFormat::Decimal, None);
        let data_lines: Vec<_> = text
            .lines()
            .filter(|l| l.starts_with("  ") && l.contains('|') && !l.contains("X/"))
            .collect();
        assert_eq!(data_lines.len(), 64);
        assert!(text.contains('<'));
        assert!(text.contains('>'));
        let total: f64 = entries.iter().map(|(_, p)| p).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }
}
