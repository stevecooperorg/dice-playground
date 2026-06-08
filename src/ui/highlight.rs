//! Line-oriented syntax highlighting for `.dice` / Starlark (no external highlighter crates).

use crate::engine::dice_literal_len_at;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Plain,
    Keyword,
    String,
    Number,
    Comment,
    Dice,
    Identifier,
    Operator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColoredSpan {
    pub text: String,
    pub kind: TokenKind,
}

impl ColoredSpan {
    pub fn class_name(&self) -> &'static str {
        match self.kind {
            TokenKind::Plain => "tok-plain",
            TokenKind::Keyword => "tok-kw",
            TokenKind::String => "tok-str",
            TokenKind::Number => "tok-num",
            TokenKind::Comment => "tok-com",
            TokenKind::Dice => "tok-dice",
            TokenKind::Identifier => "tok-id",
            TokenKind::Operator => "tok-op",
        }
    }
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "and"
            | "break"
            | "continue"
            | "def"
            | "elif"
            | "else"
            | "False"
            | "for"
            | "if"
            | "in"
            | "load"
            | "None"
            | "not"
            | "or"
            | "pass"
            | "return"
            | "True"
            | "while"
    )
}

fn prev_char(line: &str, byte_index: usize) -> Option<char> {
    if byte_index == 0 {
        None
    } else {
        line.get(..byte_index).and_then(|s| s.chars().next_back())
    }
}

fn push_span(spans: &mut Vec<ColoredSpan>, text: &str, kind: TokenKind) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut() {
        if last.kind == kind {
            last.text.push_str(text);
            return;
        }
    }
    spans.push(ColoredSpan {
        text: text.to_string(),
        kind,
    });
}

fn next_char(rest: &str) -> Option<(char, usize)> {
    let ch = rest.chars().next()?;
    Some((ch, ch.len_utf8()))
}

fn highlight_code_segment(line: &str, spans: &mut Vec<ColoredSpan>) {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < line.len() {
        let prev = prev_char(line, i);
        if let Some(len) = dice_literal_len_at(&line[i..], prev) {
            push_span(spans, &line[i..i + len], TokenKind::Dice);
            i += len;
            continue;
        }

        let Some((ch, ch_len)) = next_char(&line[i..]) else {
            break;
        };

        if ch == '#' {
            push_span(spans, &line[i..], TokenKind::Comment);
            break;
        }

        if ch == '\'' || ch == '"' {
            let quote = ch;
            let start = i;
            i += ch_len;
            while i < line.len() {
                let Some((c, c_len)) = next_char(&line[i..]) else {
                    break;
                };
                i += c_len;
                if c == '\\' && i < line.len() {
                    if let Some((_, esc_len)) = next_char(&line[i..]) {
                        i += esc_len;
                    }
                    continue;
                }
                if c == quote {
                    break;
                }
            }
            push_span(spans, &line[start..i], TokenKind::String);
            continue;
        }

        if ch.is_ascii_digit() {
            let start = i;
            i += ch_len;
            while i < line.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i < line.len()
                && bytes[i] == b'.'
                && i + 1 < line.len()
                && bytes[i + 1].is_ascii_digit()
            {
                i += 1;
                while i < line.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            push_span(spans, &line[start..i], TokenKind::Number);
            continue;
        }

        if ch == '_' || ch.is_ascii_alphabetic() {
            let start = i;
            i += ch_len;
            while i < line.len() {
                let Some((c, c_len)) = next_char(&line[i..]) else {
                    break;
                };
                if c == '_' || c.is_ascii_alphanumeric() {
                    i += c_len;
                } else {
                    break;
                }
            }
            let word = &line[start..i];
            let kind = if is_keyword(word) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            };
            push_span(spans, word, kind);
            continue;
        }

        if ch.is_ascii_whitespace() {
            let start = i;
            i += ch_len;
            while i < line.len() {
                let Some((c, c_len)) = next_char(&line[i..]) else {
                    break;
                };
                if c.is_ascii_whitespace() {
                    i += c_len;
                } else {
                    break;
                }
            }
            push_span(spans, &line[start..i], TokenKind::Plain);
            continue;
        }

        push_span(spans, &line[i..i + ch_len], TokenKind::Operator);
        i += ch_len;
    }
}

/// Highlight a single line (no trailing newline).
pub fn highlight_line(line: &str) -> Vec<ColoredSpan> {
    if line.is_empty() {
        return vec![ColoredSpan {
            text: String::new(),
            kind: TokenKind::Plain,
        }];
    }

    let mut spans = Vec::new();
    highlight_code_segment(line, &mut spans);

    if spans.is_empty() {
        spans.push(ColoredSpan {
            text: String::new(),
            kind: TokenKind::Plain,
        });
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_for_kind(spans: &[ColoredSpan], kind: TokenKind) -> String {
        spans
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.text.as_str())
            .collect()
    }

    fn has_kind(spans: &[ColoredSpan], kind: TokenKind) -> bool {
        spans.iter().any(|s| s.kind == kind)
    }

    #[test]
    fn keyword_def() {
        let spans = highlight_line("def f():");
        assert!(text_for_kind(&spans, TokenKind::Keyword).contains("def"));
    }

    #[test]
    fn dice_in_output() {
        let spans = highlight_line(r#"output("x", 4d6kl4)"#);
        let dice = text_for_kind(&spans, TokenKind::Dice);
        assert!(dice.contains("4d6kl4"));
        assert!(has_kind(&spans, TokenKind::String));
    }

    #[test]
    fn dice_forms() {
        for line in ["2d10", "4d6dl1", "d20"] {
            let spans = highlight_line(line);
            assert!(
                has_kind(&spans, TokenKind::Dice),
                "expected dice token in {line:?}"
            );
        }
    }

    #[test]
    fn no_dice_inside_identifier() {
        let spans = highlight_line("foo4d6");
        assert!(!has_kind(&spans, TokenKind::Dice));
    }

    #[test]
    fn no_dice_inside_string() {
        let spans = highlight_line(r#""4d6""#);
        assert!(!has_kind(&spans, TokenKind::Dice));
        assert!(has_kind(&spans, TokenKind::String));
    }

    #[test]
    fn comment_line() {
        let spans = highlight_line("# not dice 4d6");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].kind, TokenKind::Comment);
    }
}
