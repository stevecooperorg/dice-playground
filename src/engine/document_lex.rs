//! Document-level highlighting. Fence spelling comes from the execution layer;
//! unlike execution detection, highlighting also accepts unclosed/display-only fences.

use std::ops::Range;

use super::lex::{lex_source, SourceToken, SourceTokenKind};
use super::literate::fence::{is_closing_fence, parse_fence_opener};
use super::literate::{append_code_body, is_literate};

/// Highlight a complete editor document without changing any text or newline.
/// Markdown is distinguished from executable bare/`dice` fences and display-only
/// fenced examples. Unclosed fences are tolerated while typing, not executed.
///
/// ```
/// use dice_playground::engine::{lex_document, SourceTokenKind};
/// let source = "# Roll\n```dice\n2d6\n```\n";
/// let tokens = lex_document(source);
/// assert_eq!(tokens[0].kind, SourceTokenKind::Heading);
/// assert!(tokens.iter().any(|t| t.kind == SourceTokenKind::Dice));
/// assert_eq!(tokens.iter().map(|t| &source[t.range.clone()]).collect::<String>(), source);
/// ```
pub fn lex_document(source: &str) -> Vec<SourceToken> {
    if !source
        .lines()
        .any(|line| parse_fence_opener(line).is_some())
    {
        return lex_source(source);
    }
    if !is_literate(source) {
        let native = lex_source(source);
        // Preview-only fence detection must not reinterpret a real legacy string.
        // Closed executable fences still retain the existing format precedence.
        if !has_preview_fence(source, &native) {
            return native;
        }
    }
    let (regions, program) = document_regions(source);
    let tokens = lex_source(&program);
    let mut token_index = 0;
    let mut out = Vec::new();
    for region in regions {
        match region {
            Region::Styled(token) => out.push(token),
            Region::Code { document, program } => {
                // A native token (notably a triple string) may cross fences. Clip
                // it to each body; never colour the intervening prose as code.
                while token_index < tokens.len() && tokens[token_index].range.end <= program.start {
                    token_index += 1;
                }
                while token_index < tokens.len() && tokens[token_index].range.start < program.end {
                    let token = &tokens[token_index];
                    let start =
                        token.range.start.max(program.start) - program.start + document.start;
                    let end = token.range.end.min(program.end) - program.start + document.start;
                    out.push(SourceToken::new(start..end, token.kind));
                    if token.range.end > program.end {
                        break;
                    }
                    token_index += 1;
                }
                // The extractor omits the final LF adjacent to a fence. It is
                // still present in the textarea and must appear in the backdrop.
                let mapped_end = document.start + program.len();
                if mapped_end < document.end {
                    out.push(SourceToken::new(
                        mapped_end..document.end,
                        SourceTokenKind::Plain,
                    ));
                }
            }
        }
    }
    out
}

fn has_preview_fence(source: &str, native: &[SourceToken]) -> bool {
    let mut offset = 0;
    let mut token_index = 0;
    source.split_inclusive('\n').any(|line| {
        let start = offset;
        offset += line.len();
        if parse_fence_opener(line).is_none() {
            return false;
        }
        while token_index < native.len() && native[token_index].range.end <= start {
            token_index += 1;
        }
        !native.get(token_index).is_some_and(|token| {
            token.range.start <= start
                && matches!(
                    token.kind,
                    SourceTokenKind::String | SourceTokenKind::Comment
                )
        })
    })
}

enum Region {
    Styled(SourceToken),
    Code {
        document: Range<usize>,
        program: Range<usize>,
    },
}

fn document_regions(source: &str) -> (Vec<Region>, String) {
    let mut regions = Vec::new();
    let mut program = String::new();
    let mut offset = 0;
    let mut lines = source.split_inclusive('\n');
    while let Some(line) = lines.next() {
        let end = offset + line.len();
        if let Some(open) = parse_fence_opener(line) {
            regions.push(Region::Styled(SourceToken::new(
                offset..end,
                SourceTokenKind::Fence,
            )));
            let body_start = end;
            offset = end;
            let mut close = None;
            for body_line in lines.by_ref() {
                let end = offset + body_line.len();
                if is_closing_fence(body_line, open.tick_count) {
                    close = Some(offset..end);
                    break;
                }
                offset = end;
            }
            if open.executable {
                let body = &source[body_start..offset];
                // Match parse_literate's body-lines.join("\n"), including CRs.
                let body = body.strip_suffix('\n').unwrap_or(body);
                let range = append_code_body(&mut program, body);
                regions.push(Region::Code {
                    document: body_start..offset,
                    program: range,
                });
            } else if offset > body_start {
                regions.push(Region::Styled(SourceToken::new(
                    body_start..offset,
                    SourceTokenKind::Markdown,
                )));
            }
            if let Some(close) = close {
                offset = close.end;
                regions.push(Region::Styled(SourceToken::new(
                    close,
                    SourceTokenKind::Fence,
                )));
            }
        } else {
            let kind = if line.trim_start().starts_with('#') {
                SourceTokenKind::Heading
            } else {
                SourceTokenKind::Markdown
            };
            regions.push(Region::Styled(SourceToken::new(offset..end, kind)));
            offset = end;
        }
    }
    (regions, program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_fences_inside_legacy_strings_do_not_hide_executable_code() {
        for tag in ["text", "rust", "Dice"] {
            let source = format!("s = '''\n```{tag}\nexample\n```\n'''\noutput(2d6)\n");
            assert!(!crate::engine::is_literate(&source));
            assert_eq!(lex_document(&source), lex_source(&source));
            assert_eq!(
                lex_document(&source)
                    .iter()
                    .filter(|t| t.kind == SourceTokenKind::Dice)
                    .map(|t| &source[t.range.clone()])
                    .collect::<Vec<_>>(),
                ["2d6"]
            );
        }
    }

    #[test]
    fn markdown_executable_and_non_executable_fences() {
        let source = "# Rolls 4d6\nProse 2d6\n```text\n2d6\n```\n```dice\nx = '''4d6\n..5'''\n2d6.keep(5..)\n```\n````\nd6\n````\n";
        let tokens = lex_document(source);
        let dice: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == SourceTokenKind::Dice)
            .map(|t| &source[t.range.clone()])
            .collect();
        assert_eq!(dice, ["2d6", "d6"]);
        assert!(tokens.iter().any(|t| t.kind == SourceTokenKind::Band));
        assert_eq!(
            tokens
                .iter()
                .map(|t| &source[t.range.clone()])
                .collect::<String>(),
            source
        );
    }

    #[test]
    fn cross_fence_strings_and_delimiters_share_native_state() {
        let source = "# Title\n```dice\ns = '''é\n```\nProse d6\n```\n```\n```dice\n4d6 ..5'''\nx = (\n```\nMore prose\n```dice\n2d6\n)\n```\n";
        let tokens = lex_document(source);
        let dice: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == SourceTokenKind::Dice)
            .map(|t| &source[t.range.clone()])
            .collect();
        assert_eq!(dice, ["2d6"]);
        assert!(tokens
            .iter()
            .any(|t| t.kind == SourceTokenKind::String
                && source[t.range.clone()].contains("4d6 ..5")));
        assert!(!tokens.iter().any(|t| t.kind == SourceTokenKind::Error));
        assert_eq!(
            tokens
                .iter()
                .map(|t| &source[t.range.clone()])
                .collect::<String>(),
            source
        );
    }

    #[test]
    fn virtual_highlighting_program_matches_execution_tangle() {
        for source in [
            "```dice\na = 1\n\n```\nProse\n```dice\nb = 2\n```\n",
            "```dice\n```\n```dice\na = 1\n\n\n```\n```text\n2d6\n```\n```\n```\n```dice\nb = 2\n```\n",
        ] {
            for source in [source.to_owned(), source.replace('\n', "\r\n")] {
                let (_, program) = document_regions(&source);
                let doc = crate::engine::parse_literate(&source).unwrap();
                assert_eq!(program, crate::engine::tangle_literate(&doc).tangled);
                assert_eq!(lex_document(&source).iter().map(|t| &source[t.range.clone()]).collect::<String>(), source);
            }
        }
    }

    #[test]
    fn retained_fence_detection_precedes_native_multiline_strings() {
        // Known format ambiguity: existing document detection is not a Starlark
        // scan. Changing execution precedence needs a separate format decision.
        let source = "x = '''\n```dice\n2d6\n```\n'''";
        assert!(crate::engine::is_literate(source));
        assert!(!lex_source(source)
            .iter()
            .any(|t| t.kind == SourceTokenKind::Dice));
        assert!(lex_document(source)
            .iter()
            .any(|t| t.kind == SourceTokenKind::Dice));
    }

    #[test]
    fn incomplete_and_display_only_documents_are_lossless() {
        for source in [
            "# Title\r\n```dice\r\n2d6\r\nx = '''..5\r\n",
            "```text\n2d6\n```\n",
            "```Dice\n2d6",
            "```dice\n..5",
            "```\n",
            "```rust\n```dice\n2d6\n```\n",
            "```dice\n'''first\nsecond'''\n```",
        ] {
            let tokens = lex_document(source);
            assert_eq!(
                tokens
                    .iter()
                    .map(|t| &source[t.range.clone()])
                    .collect::<String>(),
                source
            );
            if source.starts_with("```text")
                || source.starts_with("```Dice")
                || source.starts_with("```rust")
            {
                assert!(!tokens.iter().any(|t| t.kind == SourceTokenKind::Dice));
            }
        }
    }
}
