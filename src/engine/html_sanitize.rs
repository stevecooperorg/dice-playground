//! Sanitize HTML fragments from markdown weave and output sections.

/// Sanitize woven HTML before it crosses to UI or CLI files (format v1 §6.5).
pub fn sanitize_woven_html(fragment: &str) -> String {
    let mut builder = ammonia::Builder::default();
    builder.add_tag_attributes(
        "div",
        &[
            "class",
            "data-dice-output",
            "data-dice-chart-kind",
            "role",
            "aria-label",
        ],
    );
    builder.add_tag_attributes("section", &["class", "data-dice-output-name"]);
    builder.clean(fragment).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_chart_data_attributes() {
        let raw = r#"<section class="dice-output" data-dice-output-name="d6"><div class="dice-output-chart" data-dice-output="d6" data-dice-chart-kind="dieroll" role="img" aria-label="Chart for output d6"></div><p>x</p></section>"#;
        let clean = sanitize_woven_html(raw);
        assert!(clean.contains("data-dice-output=\"d6\""));
        assert!(clean.contains("data-dice-chart-kind=\"dieroll\""));
    }
}
