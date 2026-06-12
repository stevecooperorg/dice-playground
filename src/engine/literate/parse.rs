use anyhow::{anyhow, Context};

use super::fence::{is_closing_fence, parse_fence_opener};

/// One executable fence extracted from a literate document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceMeta {
    /// Zero-based index in tangle order.
    pub index: usize,
    /// 1-based line of the opening fence in the source file.
    pub source_open_line: u32,
    /// 1-based line of the closing fence in the source file.
    pub source_close_line: u32,
    /// Starlark body (lines between fences, no fence markers).
    pub body: String,
}

/// Parsed literate document (executable fences only; prose is not stored separately in v1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiterateDocument {
    pub fences: Vec<FenceMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: Option<u32>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {line}: {}", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// Scan for at least one executable fence without full parse.
pub fn contains_executable_fence(source: &str) -> bool {
    scan_fences(source).is_ok_and(|fences| {
        fences
            .iter()
            .any(|f| f.executable && matches!(f.kind, ScanKind::Closed { .. }))
    })
}

/// Parse literate structure; errors on unclosed fences or invalid UTF-8 (caller holds `&str`).
pub fn parse(source: &str) -> anyhow::Result<LiterateDocument> {
    let scanned = scan_fences(source).context("scan literate fences")?;
    let mut fences = Vec::new();
    let mut index = 0usize;
    for block in scanned {
        match block {
            ScannedFence {
                executable: true,
                kind:
                    ScanKind::Closed {
                        body,
                        open_line,
                        close_line,
                    },
                ..
            } => {
                fences.push(FenceMeta {
                    index,
                    source_open_line: open_line,
                    source_close_line: close_line,
                    body,
                });
                index += 1;
            }
            ScannedFence {
                executable: true,
                kind: ScanKind::Unclosed { open_line },
                ..
            } => {
                return Err(anyhow!(ParseError {
                    message: "unclosed fenced code block".into(),
                    line: Some(open_line),
                }));
            }
            _ => {}
        }
    }
    if fences.is_empty() {
        return Err(anyhow!(ParseError {
            message: "no executable fences found".into(),
            line: None,
        }));
    }
    Ok(LiterateDocument { fences })
}

#[derive(Debug, Clone)]
struct ScannedFence {
    executable: bool,
    kind: ScanKind,
}

#[derive(Debug, Clone)]
enum ScanKind {
    Closed {
        open_line: u32,
        close_line: u32,
        body: String,
    },
    Unclosed {
        open_line: u32,
    },
}

fn scan_fences(source: &str) -> anyhow::Result<Vec<ScannedFence>> {
    let mut out = Vec::new();
    let lines: Vec<&str> = source.split('\n').collect();
    let mut i = 0usize;
    while i < lines.len() {
        if let Some(open) = parse_fence_opener(lines[i]) {
            let open_line = (i + 1) as u32;
            let mut j = i + 1;
            let mut closed = false;
            while j < lines.len() {
                if is_closing_fence(lines[j], open.tick_count) {
                    let close_line = (j + 1) as u32;
                    let body_lines = &lines[(i + 1)..j];
                    let body = if body_lines.is_empty() {
                        String::new()
                    } else {
                        body_lines.join("\n")
                    };
                    out.push(ScannedFence {
                        executable: open.executable,
                        kind: ScanKind::Closed {
                            open_line,
                            close_line,
                            body,
                        },
                    });
                    i = j + 1;
                    closed = true;
                    break;
                }
                j += 1;
            }
            if !closed {
                out.push(ScannedFence {
                    executable: open.executable,
                    kind: ScanKind::Unclosed { open_line },
                });
                break;
            }
        } else {
            i += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_script_not_literate() {
        assert!(!contains_executable_fence("output(\"one_d6\", 1d6)\n"));
    }

    #[test]
    fn dice_fence_triggers_literate() {
        let src = "# One die\n\n```dice\noutput(\"d6\", d(6))\n```\n";
        assert!(contains_executable_fence(src));
        let doc = parse(src).expect("parse");
        assert_eq!(doc.fences.len(), 1);
        assert_eq!(doc.fences[0].body, "output(\"d6\", d(6))");
    }

    #[test]
    fn bare_fence_opener_is_executable() {
        let src = "# Roll\n\n```\noutput(\"d6\", d(6))\n```\n";
        assert!(contains_executable_fence(src));
    }

    #[test]
    fn rust_fence_does_not_trigger_literate() {
        assert!(!contains_executable_fence("```rust\nfn main() {}\n```\n"));
    }

    #[test]
    fn two_fences_parsed_in_order() {
        let src =
            "Intro.\n\n```dice\nbonus = 3\n```\n\n```dice\noutput(\"check\", 1d20 + bonus)\n```\n";
        let doc = parse(src).expect("parse");
        assert_eq!(doc.fences.len(), 2);
        assert_eq!(doc.fences[0].body, "bonus = 3");
        assert_eq!(doc.fences[1].body, "output(\"check\", 1d20 + bonus)");
    }

    #[test]
    fn unclosed_fence_errors() {
        let err = parse("```dice\nx = 1\n").unwrap_err();
        assert!(err.to_string().contains("unclosed"));
    }
}
