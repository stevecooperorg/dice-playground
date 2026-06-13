//! Weave literate `.dice` into an HTML report fragment (prose + bound outputs).
//!
//! **Output binding (v1):** outputs are assigned to executable fences in **global eval order**,
//! matching each fence's `output(` call count in source (static scan of fence bodies). Remaining
//! outputs attach to the last fence.

use anyhow::{bail, Context};

use super::fence::{is_closing_fence, parse_fence_opener};
use super::parse::LiterateDocument;
use crate::engine::html_sanitize::sanitize_woven_html;
use crate::engine::markdown_to_html;
use crate::engine::output_html::format_output_section_html;
use crate::engine::starlark_guest::{EvalResult, OutputEntry, ProbFormat};
use crate::engine::starlark_guest::shared_sample_space_for_outputs;

/// Static site chrome when rendering full HTML pages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LiterateStaticLayout {
    #[default]
    Tutorial,
    Cookbook,
}

/// Options for [`weave_literate`].
#[derive(Debug, Clone, Copy)]
pub struct WeaveOptions {
    pub prob_format: ProbFormat,
    pub static_layout: LiterateStaticLayout,
}

impl Default for WeaveOptions {
    fn default() -> Self {
        Self {
            prob_format: ProbFormat::default(),
            static_layout: LiterateStaticLayout::Tutorial,
        }
    }
}

/// Build a sanitized HTML fragment from literate source and a successful eval.
///
/// # Example
///
/// ```
/// use dice_playground::engine::{
///     eval_source_with_dialect, dice_dialect_public, is_literate, parse_literate,
///     tangle_literate, weave_literate, WeaveOptions,
/// };
/// let src = "# Hi\n\n```dice\noutput(\"d6\", 1d6)\n```\n";
/// assert!(is_literate(src));
/// let doc = parse_literate(src).unwrap();
/// let tangled = tangle_literate(&doc);
/// let expanded = dice_playground::engine::desugar_if_needed("x.dice", &tangled.tangled).unwrap();
/// let eval = eval_source_with_dialect("x.dice", &expanded, &dice_dialect_public()).unwrap();
/// let html = weave_literate(src, &doc, &tangled, &eval, WeaveOptions::default()).unwrap();
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("mean"));
/// ```
pub fn weave_literate(
    source: &str,
    _doc: &LiterateDocument,
    _tangled: &super::tangle::TangleResult,
    eval: &EvalResult,
    options: WeaveOptions,
) -> anyhow::Result<String> {
    let bound = bind_outputs_to_fences(_doc, eval);
    let raw = weave_document(source, &bound, eval, options)?;
    Ok(sanitize_woven_html(&raw))
}

/// Evaluate a literate file and return a full HTML document (fragment wrapped with minimal shell).
pub fn render_literate_document(
    path: &str,
    source: &str,
    options: WeaveOptions,
) -> anyhow::Result<String> {
    use crate::engine::playground::dice_dialect_public;
    use crate::engine::{
        desugar_if_needed, eval_source_with_dialect, parse_literate, tangle_literate,
    };

    if !super::is_literate(source) {
        bail!("render requires a literate `.dice` file (executable fenced blocks)");
    }
    if source.len() > super::MAX_LITERATE_BYTES {
        bail!("source exceeds maximum literate size");
    }
    let doc = parse_literate(source).context("parse literate document")?;
    let tangled = tangle_literate(&doc);
    let expanded = desugar_if_needed(path, &tangled.tangled).context("desugar")?;
    let eval = eval_source_with_dialect(path, &expanded, &dice_dialect_public()).context("eval")?;
    let fragment = weave_literate(source, &doc, &tangled, &eval, options)?;
    let title = literate_document_title(source);
    let page = match options.static_layout {
        LiterateStaticLayout::Tutorial => wrap_static_tutorial_page(&title, &fragment),
        LiterateStaticLayout::Cookbook => wrap_static_cookbook_page(&title, &fragment),
    };
    Ok(page)
}

/// First ATX `#` heading in the source, or a fallback label.
pub fn literate_document_title(source: &str) -> String {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let rest = rest.trim_start_matches('#').trim();
            if !rest.is_empty() {
                return rest.to_owned();
            }
        }
    }
    "Dice lesson".to_owned()
}

/// Full HTML page for `dist/tutorial/` static lesson pages.
pub fn wrap_static_tutorial_page(title: &str, fragment: &str) -> String {
    let title_esc = escape_html_text(title);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="tutorial.css">
<title>{title_esc}</title>
</head>
<body class="dice-literate-report">
<header>
<a href="../docs/index.html">User guide</a>
<a href="index.html">All lessons</a>
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

/// Full HTML page for `dist/cookbook/` static recipe pages.
pub fn wrap_static_cookbook_page(title: &str, fragment: &str) -> String {
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
<body class="dice-literate-report">
<header>
<a href="../docs/index.html">User guide</a>
<a href="index.html">All recipes</a>
<a href="../tutorial/index.html">Tutorial</a>
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

fn bind_outputs_to_fences(doc: &LiterateDocument, eval: &EvalResult) -> Vec<Vec<usize>> {
    let fence_count = doc.fences.len().max(1);
    let mut per_fence: Vec<Vec<usize>> = vec![Vec::new(); fence_count];
    let mut out_idx = 0usize;
    for (fence_i, fence) in doc.fences.iter().enumerate() {
        let expect = count_output_calls_in_body(&fence.body);
        for _ in 0..expect {
            if out_idx < eval.outputs.len() {
                per_fence[fence_i].push(out_idx);
                out_idx += 1;
            }
        }
    }
    while out_idx < eval.outputs.len() {
        let last = per_fence.len() - 1;
        per_fence[last].push(out_idx);
        out_idx += 1;
    }
    per_fence
}

fn count_output_calls_in_body(body: &str) -> usize {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.contains("output(")
        })
        .count()
}

fn weave_document(
    source: &str,
    bound: &[Vec<usize>],
    eval: &EvalResult,
    options: WeaveOptions,
) -> anyhow::Result<String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut html = String::new();
    let mut prose: Vec<&str> = Vec::new();
    let mut fence_index = 0usize;
    let mut i = 0usize;

    while i < lines.len() {
        if let Some(open) = parse_fence_opener(lines[i]) {
            if open.executable {
                flush_prose(&mut prose, &mut html);
                let mut j = i + 1;
                while j < lines.len() {
                    if is_closing_fence(lines[j], open.tick_count) {
                        append_fence_outputs(
                            &mut html,
                            bound,
                            fence_index,
                            eval,
                            options.prob_format,
                        );
                        fence_index += 1;
                        i = j + 1;
                        break;
                    }
                    j += 1;
                }
                if j >= lines.len() {
                    bail!("unclosed executable fence while weaving");
                }
            } else {
                prose.push(lines[i]);
                i += 1;
            }
        } else {
            prose.push(lines[i]);
            i += 1;
        }
    }
    flush_prose(&mut prose, &mut html);
    Ok(html)
}

fn flush_prose(prose: &mut Vec<&str>, html: &mut String) {
    if prose.is_empty() {
        return;
    }
    let md = prose.join("\n");
    prose.clear();
    if md.trim().is_empty() {
        return;
    }
    html.push_str(&markdown_to_html(&format!("{md}\n")));
}

fn append_fence_outputs(
    html: &mut String,
    bound: &[Vec<usize>],
    fence_index: usize,
    eval: &EvalResult,
    prob_format: ProbFormat,
) {
    let Some(indices) = bound.get(fence_index) else {
        return;
    };
    if indices.is_empty() {
        return;
    }
    let outputs: Vec<OutputEntry> = indices
        .iter()
        .filter_map(|&i| eval.outputs.get(i).cloned())
        .collect();
    if outputs.is_empty() {
        return;
    }
    let shared = shared_sample_space_for_outputs(&eval.outputs);
    for entry in &outputs {
        let section = format_output_section_html(entry, prob_format, shared);
        html.push_str(&sanitize_woven_html(&section));
    }
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
    use crate::engine::literate::parse::parse;
    use crate::engine::literate::tangle::tangle;
    use crate::engine::{
        desugar_if_needed, dice_dialect_public, eval_source_with_dialect, is_literate,
    };

    #[test]
    fn static_page_includes_tutorial_chrome() {
        let html = wrap_static_tutorial_page("One die", "<p>body</p>");
        assert!(html.contains("All lessons"));
        assert!(html.contains("<main>"));
    }

    #[test]
    fn weave_includes_heading_and_output() {
        let src = "# One die\n\nFair d6:\n\n```dice\noutput(\"one_d6\", 1d6)\n```\n";
        assert!(is_literate(src));
        let doc = parse(src).unwrap();
        let tangled = tangle(&doc);
        let expanded = desugar_if_needed("t.dice", &tangled.tangled).unwrap();
        let eval = eval_source_with_dialect("t.dice", &expanded, &dice_dialect_public()).unwrap();
        let html = weave_literate(src, &doc, &tangled, &eval, WeaveOptions::default()).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("one_d6"));
        assert!(html.contains("<table>"));
        assert!(html.contains("data-dice-output=\"one_d6\""));
        assert!(html.contains("dice-output-chart"));
    }

    #[test]
    fn sanitize_strips_script_from_prose() {
        use crate::engine::html_sanitize::sanitize_woven_html;
        let dirty = "<p>ok</p><script>alert(1)</script>";
        let clean = sanitize_woven_html(dirty);
        assert!(!clean.contains("<script>"));
        assert!(clean.contains("ok"));
    }

    #[test]
    fn two_fences_bind_outputs_by_call_site_line() {
        let src = r#"Intro.

```dice
output("first", 1d6)
```

```dice
output("second", 1d4)
```
"#;
        let doc = parse(src).unwrap();
        let tangled = tangle(&doc);
        let expanded = desugar_if_needed("t.dice", &tangled.tangled).unwrap();
        let eval = eval_source_with_dialect("t.dice", &expanded, &dice_dialect_public()).unwrap();
        assert_eq!(eval.outputs.len(), 2);
        let html = weave_literate(src, &doc, &tangled, &eval, WeaveOptions::default()).unwrap();
        let first_pos = html.find("first").expect("first output");
        let second_pos = html.find("second").expect("second output");
        assert!(first_pos < second_pos);
    }
}
