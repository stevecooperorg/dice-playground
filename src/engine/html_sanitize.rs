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
    // Preserve fence identity for static playground links: python fences in the
    // reference are signatures, while dice fences are executable examples.
    builder.add_allowed_classes(
        "code",
        &["language-dice", "language-python", "language-text"],
    );
    builder.clean(fragment).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_known_code_languages_without_allowing_arbitrary_attributes() {
        for language in ["dice", "python", "text"] {
            let raw = format!("<code class=\"language-{language} unexpected\" onclick=\"alert(1)\">example</code><script>alert(1)</script>");
            let clean = sanitize_woven_html(&raw);
            assert!(clean.contains(&format!("class=\"language-{language}\"")));
            assert!(!clean.contains("unexpected"));
            assert!(!clean.contains("onclick"));
            assert!(!clean.contains("<script"));
        }
    }

    #[test]
    fn keeps_chart_data_attributes() {
        let raw = r#"<section class="dice-output" data-dice-output-name="d6"><div class="dice-output-chart" data-dice-output="d6" data-dice-chart-kind="dieroll" role="img" aria-label="Chart for output d6"></div><p>x</p></section>"#;
        let clean = sanitize_woven_html(raw);
        assert!(clean.contains("data-dice-output=\"d6\""));
        assert!(clean.contains("data-dice-chart-kind=\"dieroll\""));
    }
}
