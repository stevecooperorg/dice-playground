//! Render Markdown-only static site pages (user guide, function reference) with site chrome.

use crate::engine::literate::sanitize_woven_html;
use crate::engine::markdown_to_html;

/// Which static HTML shell and link rewrite rules to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownStaticLayout {
    Guide,
    Reference,
}

/// Build a full HTML document from Markdown (no literate fences).
pub fn render_markdown_static_page(
    title: &str,
    markdown: &str,
    layout: MarkdownStaticLayout,
) -> String {
    let rewritten = rewrite_markdown_links(markdown, layout);
    let fragment = sanitize_woven_html(&markdown_to_html(&rewritten));
    match layout {
        MarkdownStaticLayout::Guide => wrap_static_guide_page(title, &fragment),
        MarkdownStaticLayout::Reference => wrap_static_reference_page(title, &fragment),
    }
}

/// Read a `.md` file, strip optional YAML frontmatter, pick title, and render.
pub fn render_markdown_static_file(
    markdown: &str,
    layout: MarkdownStaticLayout,
) -> (String, String) {
    let (fm_title, body) = strip_yaml_frontmatter(markdown);
    let title = fm_title
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| markdown_document_title(body, layout));
    let html = render_markdown_static_page(&title, body, layout);
    (title, html)
}

fn markdown_document_title(body: &str, layout: MarkdownStaticLayout) -> String {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let rest = rest.trim_start_matches('#').trim();
            if !rest.is_empty() {
                return rest.to_owned();
            }
        }
    }
    match layout {
        MarkdownStaticLayout::Guide => "Dice user guide".to_owned(),
        MarkdownStaticLayout::Reference => "Dice standard library".to_owned(),
    }
}

/// Remove leading `---` YAML block; return `(title from frontmatter, body)`.
pub fn strip_yaml_frontmatter(source: &str) -> (Option<String>, &str) {
    let rest = source
        .strip_prefix("---")
        .map(|s| s.trim_start_matches('\n'));
    let Some(after_open) = rest else {
        return (None, source);
    };
    let Some(end) = after_open.find("\n---") else {
        return (None, source);
    };
    let front = &after_open[..end];
    let body = after_open[end + 4..].trim_start_matches('\n');
    let title = front.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix("title:")?;
        let rest = rest.trim();
        let rest = rest.strip_prefix('"').unwrap_or(rest);
        let rest = rest.strip_suffix('"').unwrap_or(rest);
        Some(rest.to_owned())
    });
    (title, body)
}

pub fn rewrite_markdown_links(markdown: &str, layout: MarkdownStaticLayout) -> String {
    let mut out = String::with_capacity(markdown.len());
    let bytes = markdown.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'(' && i > 0 && bytes[i - 1] == b']' {
            if let Some((target, end)) = parse_link_target(&markdown[i + 1..]) {
                out.push('(');
                let mapped = map_link_target(&target, layout);
                out.push_str(&mapped);
                out.push(')');
                i += 1 + end;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn parse_link_target(s: &str) -> Option<(String, usize)> {
    let mut depth = 0usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => {
                return Some((s[..idx].to_owned(), idx + 1));
            }
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn map_link_target(target: &str, layout: MarkdownStaticLayout) -> String {
    match layout {
        MarkdownStaticLayout::Guide => map_guide_link(target),
        MarkdownStaticLayout::Reference => map_reference_link(target),
    }
}

fn map_guide_link(target: &str) -> String {
    if target == "../README.md" {
        return "https://github.com/stevecooperorg/dice-playground".to_owned();
    }
    if target == "README.md" {
        return "index.html".to_owned();
    }
    if let Some(frag) = target.strip_prefix("README.md#") {
        return format!("index.html#{frag}");
    }
    if let Some(slug) = target.strip_prefix("tutorial/") {
        if let Some(html) = lesson_slug_to_html(slug) {
            return html;
        }
    }
    if target == "cookbook/README.md" {
        return "../cookbook/index.html".to_owned();
    }
    if target == "references/stdlib.md" {
        return "../references/stdlib.html".to_owned();
    }
    if let Some(frag) = target.strip_prefix("references/stdlib.md#") {
        return format!("../references/stdlib.html#{frag}");
    }
    if target == "references/README.md" {
        return "../references/index.html".to_owned();
    }
    target.to_owned()
}

fn map_reference_link(target: &str) -> String {
    if let Some(slug) = target.strip_prefix("../tutorial/") {
        if let Some(html) = lesson_slug_to_html(slug) {
            return html;
        }
    }
    target.to_owned()
}

fn lesson_slug_to_html(slug: &str) -> Option<String> {
    let path = slug
        .strip_suffix(".html")
        .or_else(|| slug.strip_suffix(".md"))
        .or_else(|| slug.strip_suffix(".dice"))?;
    if path.len() >= 3
        && path.as_bytes()[2] == b'-'
        && path[..2].chars().all(|c| c.is_ascii_digit())
    {
        return Some(format!("../tutorial/{path}.html"));
    }
    None
}

pub fn wrap_static_guide_page(title: &str, fragment: &str) -> String {
    let title_esc = escape_html_text(title);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="../tutorial/tutorial.css">
<title>{title_esc}</title>
</head>
<body>
<header>
<strong>User guide</strong>
<a href="../tutorial/index.html">Tutorial</a>
<a href="../cookbook/index.html">Cookbook</a>
<a href="../references/index.html">Function reference</a>
<a href="/">Open playground</a>
</header>
<main>
{fragment}
</main>
</body>
</html>
"#
    )
}

pub fn wrap_static_reference_page(title: &str, fragment: &str) -> String {
    let title_esc = escape_html_text(title);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="../tutorial/tutorial.css">
<title>{title_esc}</title>
</head>
<body>
<header>
<a href="../docs/index.html">User guide</a>
<a href="../tutorial/index.html">Tutorial</a>
<a href="../cookbook/index.html">Cookbook</a>
<a href="/">Open playground</a>
</header>
<main>
{fragment}
</main>
</body>
</html>
"#
    )
}

fn escape_html_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_frontmatter_title() {
        let src = "---\ntitle: \"My guide\"\n---\n\n# Body\n";
        let (title, body) = strip_yaml_frontmatter(src);
        assert_eq!(title.as_deref(), Some("My guide"));
        assert!(body.starts_with("# Body"));
    }

    #[test]
    fn guide_rewrites_tutorial_html_link() {
        let md = "[lesson](tutorial/01-one-die.html)";
        let out = rewrite_markdown_links(md, MarkdownStaticLayout::Guide);
        assert_eq!(out, "[lesson](../tutorial/01-one-die.html)");
    }

    #[test]
    fn guide_rewrites_tutorial_dice_link() {
        let md = "See [lesson](tutorial/07-mixed-dice-pools.dice).";
        let out = rewrite_markdown_links(md, MarkdownStaticLayout::Guide);
        assert!(out.contains("../tutorial/07-mixed-dice-pools.html"));
    }

    #[test]
    fn reference_rewrites_lesson_md_link() {
        let md = "[x](../tutorial/01-one-die.md)";
        let out = rewrite_markdown_links(md, MarkdownStaticLayout::Reference);
        assert_eq!(out, "[x](../tutorial/01-one-die.html)");
    }

    #[test]
    fn render_guide_includes_main_and_title() {
        let html = render_markdown_static_page("T", "# Hi\n", MarkdownStaticLayout::Guide);
        assert!(html.contains("<strong>User guide</strong>"));
        assert!(html.contains("<h1>"));
    }
}
