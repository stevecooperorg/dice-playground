use super::parse::{FenceMeta, LiterateDocument};

/// Maps each 1-based line in tangled Starlark to a 1-based line in the original `.dice` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMap {
    /// `lines[i]` is the source line for tangled line `i + 1`.
    pub lines: Vec<u32>,
}

/// Concatenated Starlark plus diagnostic line map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TangleResult {
    pub tangled: String,
    pub line_map: LineMap,
    /// Inclusive 1-based tangled line range per executable fence index.
    pub fence_tangled_lines: Vec<(u32, u32)>,
}

/// Concatenate executable fence bodies in document order (format v1 §4).
///
/// # Example
///
/// ```
/// use dice_playground::engine::{parse_literate, tangle_literate};
/// let src = "```dice\na = 1\n```\n\n```dice\nb = a + 1\n```\n";
/// let doc = parse_literate(src).unwrap();
/// let t = tangle_literate(&doc);
/// assert!(t.tangled.contains("a = 1"));
/// assert!(t.tangled.contains("b = a + 1"));
/// ```
pub fn tangle(doc: &LiterateDocument) -> TangleResult {
    let mut tangled = String::new();
    let mut line_map = Vec::new();
    let mut fence_tangled_lines = Vec::new();

    for (idx, fence) in doc.fences.iter().enumerate() {
        if fence.body.is_empty() {
            fence_tangled_lines.push((0, 0));
            continue;
        }
        if idx > 0 && !tangled.ends_with('\n') {
            tangled.push('\n');
        }
        let start_line = line_map.len() as u32 + 1;
        append_body(fence, &mut tangled, &mut line_map);
        let end_line = if line_map.is_empty() {
            0
        } else {
            line_map.len() as u32
        };
        if end_line >= start_line {
            fence_tangled_lines.push((start_line, end_line));
        } else {
            fence_tangled_lines.push((0, 0));
        }
    }

    TangleResult {
        tangled,
        line_map: LineMap { lines: line_map },
        fence_tangled_lines,
    }
}

fn append_body(fence: &FenceMeta, tangled: &mut String, line_map: &mut Vec<u32>) {
    if fence.body.is_empty() {
        return;
    }
    let open_content_line = fence.source_open_line.saturating_add(1);
    let parts: Vec<&str> = fence.body.split('\n').collect();
    for (offset, part) in parts.iter().enumerate() {
        if offset > 0 {
            tangled.push('\n');
        }
        tangled.push_str(part);
        line_map.push(open_content_line + offset as u32);
    }
    if fence.body.ends_with('\n') {
        tangled.push('\n');
        let last_body_line = open_content_line + parts.len().saturating_sub(1) as u32;
        line_map.push(last_body_line);
    }
}

/// Map a 1-based tangled line to a 1-based source line; falls back to the tangled line if unknown.
pub fn source_line_for_tangled(line_map: &LineMap, tangled_line_1based: u32) -> u32 {
    if tangled_line_1based == 0 {
        return 1;
    }
    let idx = tangled_line_1based as usize - 1;
    line_map
        .lines
        .get(idx)
        .copied()
        .unwrap_or(tangled_line_1based)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::literate::parse::parse;

    #[test]
    fn two_fences_joined_with_newline() {
        let src = "```dice\nbonus = 3\n```\n\n```dice\noutput(\"x\", bonus)\n```\n";
        let doc = parse(src).unwrap();
        let t = tangle(&doc);
        assert_eq!(t.tangled, "bonus = 3\noutput(\"x\", bonus)");
        assert_eq!(t.line_map.lines.len(), 2);
    }

    #[test]
    fn line_map_points_at_source_lines() {
        let src = "# title\n\n```dice\nbad(\n```\n";
        let doc = parse(src).unwrap();
        let t = tangle(&doc);
        assert_eq!(t.line_map.lines, vec![4]);
    }
}
