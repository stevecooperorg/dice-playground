//! Minimal markdown → HTML for literate `.dice` weave (playground WASM + CLI).
//!
//! Uses [pulldown-cmark](https://docs.rs/pulldown-cmark/) (CommonMark). This spike intentionally
//! does not implement a custom parser; tutorial prose can grow into a documented subset later.

use pulldown_cmark::{html, Options, Parser};

/// Markdown extensions enabled for docs weave and static pages (CommonMark + GFM tables).
pub fn markdown_options() -> Options {
    Options::ENABLE_TABLES
}

/// Convert a markdown fragment to an HTML fragment (no `<html>` document wrapper).
///
/// Suitable for embedding in the playground report view after sanitization in the UI layer
/// (or a future engine sanitize pass). Supports headings, paragraphs, lists, links,
/// emphasis, fenced code blocks, and GFM pipe tables.
///
/// # Example
///
/// ```
/// use dice_playground::engine::markdown_to_html;
/// let html = markdown_to_html("# Title\n\nA **bold** [link](https://example.com).\n");
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("<strong>"));
/// assert!(html.contains("href=\"https://example.com\""));
/// ```
pub fn markdown_to_html(markdown: &str) -> String {
    let mut out = String::new();
    let parser = Parser::new_ext(markdown, markdown_options());
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::markdown_to_html;

    #[test]
    fn empty_input_yields_empty_html() {
        assert!(markdown_to_html("").is_empty());
    }

    #[test]
    fn heading_and_paragraph() {
        let html = markdown_to_html("# Hello\n\nWorld.\n");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<p>"));
        assert!(html.contains("World"));
    }

    #[test]
    fn link_and_emphasis() {
        let html = markdown_to_html("See [docs](https://example.com/path) for *help*.\n");
        assert!(html.contains("href=\"https://example.com/path\""));
        assert!(html.contains("<em>"));
    }

    #[test]
    fn fenced_code_block() {
        let md = "Text\n\n```dice\noutput(\"x\", 2d6)\n```\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<pre>") || html.contains("<code>"));
        assert!(html.contains("2d6"));
    }

    #[test]
    fn gfm_table() {
        let md = "| A | B |\n|---|---|\n| [1](x.html) | `2d6` |\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>"));
        assert!(!html.contains("| A | B |"));
    }
}
