//! HTML sections for eval outputs (chart placeholder + GFM table).

use crate::engine::html_sanitize::sanitize_woven_html;
use crate::engine::markdown_to_html;
use crate::engine::output_chart::{chart_kind_for_entry, output_entry_name};
use crate::engine::starlark_guest::{
    format_output_entry_markdown, shared_sample_space_for_outputs, EvalResult, OutputEntry,
    ProbFormat,
};

fn escape_html_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(c),
        }
    }
    out
}

/// One output block: optional chart mount point, then sanitized table HTML.
pub fn format_output_section_html(
    entry: &OutputEntry,
    prob_format: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    let name = output_entry_name(entry);
    let name_esc = escape_html_attr(name);
    let md = format_output_entry_markdown(entry, prob_format, shared_sample_denom);
    let table_html = sanitize_woven_html(&markdown_to_html(&md));

    let mut section = format!(r#"<section class="dice-output" data-dice-output-name="{name_esc}">"#);
    if let Some(kind) = chart_kind_for_entry(entry) {
        section.push_str(&format!(
            r#"<div class="dice-output-chart" data-dice-output="{name_esc}" data-dice-chart-kind="{}" role="img" aria-label="Chart for output {name_esc}"></div>"#,
            kind.as_str()
        ));
    }
    section.push_str(&table_html);
    section.push_str("</section>\n");
    section
}

/// Concatenated output sections for legacy `outputs_html` (no literate prose).
pub fn format_eval_outputs_html_sections(result: &EvalResult, prob_format: ProbFormat) -> String {
    let shared = shared_sample_space_for_outputs(&result.outputs);
    result
        .outputs
        .iter()
        .map(|entry| format_output_section_html(entry, prob_format, shared))
        .collect()
}
