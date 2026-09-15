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

    for fence in &doc.fences {
        if fence.body.is_empty() {
            fence_tangled_lines.push((0, 0));
            continue;
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
    append_code_body(tangled, &fence.body);
    for (offset, _) in fence.body.split_inclusive('\n').enumerate() {
        line_map.push(open_content_line + offset as u32);
    }
}

/// Append one extracted body, returning its byte range (excluding any separator).
/// Highlighting uses this same join rule for its virtual executable module.
pub(crate) fn append_code_body(tangled: &mut String, body: &str) -> std::ops::Range<usize> {
    if !body.is_empty() && !tangled.is_empty() && !tangled.ends_with('\n') {
        tangled.push('\n');
    }
    let start = tangled.len();
    tangled.push_str(body);
    start..tangled.len()
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
    fn trailing_blank_lines_and_empty_fences_do_not_shift_next_body() {
        let source = "```dice\n```\n```dice\na = 1\n\n\n```\n```dice\n```\n```dice\nb = a + 1\n```\n```dice\n```\n";
        let t = tangle(&parse(source).unwrap());
        assert_eq!(t.tangled, "a = 1\n\nb = a + 1");
        assert_eq!(t.line_map.lines, [4, 5, 11]);
        assert_eq!(
            t.fence_tangled_lines,
            [(0, 0), (1, 2), (0, 0), (3, 3), (0, 0)]
        );

        // CR characters belong to the body and are preserved, not normalized.
        let t = tangle(&parse(&source.replace('\n', "\r\n")).unwrap());
        assert_eq!(t.tangled, "a = 1\r\n\r\n\r\nb = a + 1\r");
        assert_eq!(t.line_map.lines, [4, 5, 6, 11]);
        assert_eq!(
            t.fence_tangled_lines,
            [(0, 0), (1, 3), (0, 0), (4, 4), (0, 0)]
        );
    }

    #[test]
    fn line_map_points_at_source_lines() {
        let src = "# title\n\n```dice\nbad(\n```\n";
        let doc = parse(src).unwrap();
        let t = tangle(&doc);
        assert_eq!(t.line_map.lines, vec![4]);
    }
}
