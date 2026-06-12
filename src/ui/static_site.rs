//! Build-time HTML helpers for tutorial/cookbook static pages.

use anyhow::Context;

use super::playground_handoff::{looks_like_dice_script, playground_open_href, LOAD_QUERY_PARAM};

/// Inject \"load in playground\" links into fenced `<pre><code>` blocks inside static HTML.
pub fn inject_playground_load_links(html: &str) -> anyhow::Result<String> {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;
    while let Some(pre_idx) = rest.find("<pre") {
        out.push_str(&rest[..pre_idx]);
        rest = &rest[pre_idx..];
        let Some(close_pre) = rest.find("</pre>") else {
            out.push_str(rest);
            return Ok(out);
        };
        let block = &rest[..close_pre + "</pre>".len()];
        out.push_str(&enhance_pre_block(block)?);
        rest = &rest[close_pre + "</pre>".len()..];
    }
    out.push_str(rest);
    Ok(out)
}

fn enhance_pre_block(block: &str) -> anyhow::Result<String> {
    let Some(code_open) = block.find("<code") else {
        return Ok(block.to_owned());
    };
    let code_close = block
        .find("</code>")
        .context("pre block without closing code")?;
    if code_close <= code_open {
        anyhow::bail!("malformed pre/code");
    }
    let code_inner_start = block[code_open..]
        .find('>')
        .map(|i| code_open + i + 1)
        .context("code opening tag")?;
    let raw_inner = &block[code_inner_start..code_close];
    let decoded = decode_html_entities(raw_inner);
    if !looks_like_dice_script(&decoded) {
        return Ok(block.to_owned());
    }
    if block.contains("load-in-playground") {
        return Ok(block.to_owned());
    }

    let href = playground_open_href(&decoded, None)?;
    if !href.contains(LOAD_QUERY_PARAM) {
        return Ok(block.to_owned());
    }
    let href_attr = html_escape_attr(&href);
    let link = format!(
        r#"<a class="load-in-playground" href="{href_attr}" target="_blank" rel="noopener noreferrer" title="Load in playground" aria-label="Load in playground">↗</a>"#
    );

    let pre_tag_end = block.find('>').context("pre tag")? + 1;
    let mut enhanced = String::with_capacity(block.len() + link.len() + 32);
    enhanced.push_str(&inject_pre_class(&block[..pre_tag_end]));
    enhanced.push('\n');
    enhanced.push_str(&link);
    enhanced.push_str(&block[pre_tag_end..code_open]);
    enhanced.push_str(&block[code_open..]);
    Ok(enhanced)
}

fn inject_pre_class(pre_open_through_gt: &str) -> String {
    if pre_open_through_gt.contains("code-with-playground") {
        return pre_open_through_gt.to_owned();
    }
    if let Some(class_idx) = pre_open_through_gt.find("class=\"") {
        let insert_at = class_idx + "class=\"".len();
        let mut s = pre_open_through_gt.to_owned();
        s.insert_str(insert_at, "code-with-playground ");
        return s;
    }
    if pre_open_through_gt.ends_with('>') {
        let mut s = pre_open_through_gt.to_owned();
        s.truncate(s.len() - 1);
        s.push_str(" class=\"code-with-playground\">");
        return s;
    }
    pre_open_through_gt.to_owned()
}

fn html_escape_attr(s: &str) -> String {
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

fn decode_html_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut entity = String::from("&");
        while let Some(&next) = chars.peek() {
            entity.push(next);
            chars.next();
            if next == ';' {
                break;
            }
            if entity.len() > 16 {
                break;
            }
        }
        if let Some(decoded) = decode_entity(&entity) {
            out.push(decoded);
        } else {
            out.push_str(&entity);
        }
    }
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "&quot;" => Some('"'),
        "&amp;" => Some('&'),
        "&lt;" => Some('<'),
        "&gt;" => Some('>'),
        "&#39;" | "&apos;" => Some('\''),
        _ => {
            if let Some(num) = entity.strip_prefix("&#x").and_then(|s| s.strip_suffix(';')) {
                u32::from_str_radix(num, 16).ok().and_then(char::from_u32)
            } else if let Some(num) = entity.strip_prefix("&#").and_then(|s| s.strip_suffix(';')) {
                num.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                None
            }
        }
    }
}

/// Walk `root` and enhance HTML under tutorial/, cookbook/, docs/, references/.
pub fn enhance_static_site_tree(root: &std::path::Path) -> anyhow::Result<usize> {
    let mut count = 0usize;
    for sub in ["tutorial", "cookbook", "docs", "references"] {
        let dir = root.join(sub);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::walk(&dir)? {
            if entry.extension().is_some_and(|e| e == "html") {
                let html = std::fs::read_to_string(&entry)
                    .with_context(|| format!("read {}", entry.display()))?;
                let enhanced = inject_playground_load_links(&html)?;
                if enhanced != html {
                    std::fs::write(&entry, enhanced)
                        .with_context(|| format!("write {}", entry.display()))?;
                }
                count += 1;
            }
        }
    }
    Ok(count)
}

mod walkdir {
    use std::path::{Path, PathBuf};

    pub fn walk(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        walk_inner(dir, &mut files)?;
        Ok(files)
    }

    fn walk_inner(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_inner(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_link_into_fenced_pre() {
        let html =
            r#"<main><pre class="text"><code>output(&quot;one_d6&quot;, 1d6)</code></pre></main>"#;
        let out = inject_playground_load_links(html).expect("inject");
        assert!(out.contains("load-in-playground"));
        assert!(out.contains("dice_playground_load="));
        assert!(out.contains("code-with-playground"));
    }

    #[test]
    fn skips_non_dice_blocks() {
        let html = r#"<pre class="text"><code>Write a .dice script for fun</code></pre>"#;
        let out = inject_playground_load_links(html).expect("inject");
        assert!(!out.contains("load-in-playground"));
    }

    #[test]
    fn pre_without_code_is_unchanged() {
        let html = r#"<section class="dice-output"><pre>output one_d6</pre></section>"#;
        let out = inject_playground_load_links(html).expect("inject");
        assert_eq!(out, html);
    }
}
