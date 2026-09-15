//! Lossless Starlark token spans, with dice and integer-band literals folded in.
//!
//! The upstream lexer owns strings, comments, identifiers, keywords, and numbers.
//! We retain whitespace gaps and stop conservatively on lexical errors: an unfinished
//! string must never expose its contents to shorthand lowering.

use std::ops::Range;

use starlark_syntax::codemap::CodeMap;
use starlark_syntax::lexer::{LexemeError, Lexer, Token};
use starlark_syntax::{Error, ErrorKind};

use super::literals::{band_at, dice_at, Literal};
use super::playground::dice_dialect_public;

/// Presentation categories shared by script preprocessing and document highlighting.
///
/// ```
/// use dice_playground::engine::{lex_source, SourceTokenKind};
/// assert_eq!(lex_source("2d6")[0].kind, SourceTokenKind::Dice);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTokenKind {
    Plain,
    Keyword,
    String,
    Number,
    Comment,
    Dice,
    Band,
    Identifier,
    Operator,
    Markdown,
    Heading,
    Fence,
    Error,
}

/// A half-open UTF-8 byte range into the exact original source (never expanded text).
///
/// ```
/// use dice_playground::engine::lex_source;
/// let source = "output(2d6)\n";
/// let text: String = lex_source(source).iter().map(|t| &source[t.range.clone()]).collect();
/// assert_eq!(text, source);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceToken {
    /// Half-open byte offsets into the source supplied to the lexer.
    pub range: Range<usize>,
    /// Presentation category; no expansion is needed to colour this token.
    pub kind: SourceTokenKind,
    pub(crate) literal: Option<Literal>,
}

impl SourceToken {
    pub(crate) fn new(range: Range<usize>, kind: SourceTokenKind) -> Self {
        Self {
            range,
            kind,
            literal: None,
        }
    }
}

/// Tokenize a Starlark script, retaining every source byte, including incomplete input.
/// Use [`super::lex_document`] for an editor containing literate fences.
///
/// ```
/// use dice_playground::engine::{lex_source, SourceTokenKind};
/// let tokens = lex_source("lambda x: x + 1e6 # 4d6");
/// assert_eq!(tokens[0].kind, SourceTokenKind::Keyword);
/// assert_eq!(tokens.last().unwrap().kind, SourceTokenKind::Comment);
/// ```
pub fn lex_source(source: &str) -> Vec<SourceToken> {
    let native = native_tokens(source);
    let mut folded = Vec::with_capacity(native.len());
    let mut i = 0;
    while i < native.len() {
        let token = &native[i];
        let start = token.range.start;
        let eligible = matches!(
            token.kind,
            SourceTokenKind::Number | SourceTokenKind::Identifier | SourceTokenKind::Operator
        );
        let prev = source[..start].chars().next_back();
        let literal = eligible
            .then(|| dice_at(&source[start..], prev).or_else(|| band_at(&source[start..], prev)))
            .flatten();
        if let Some((literal, len)) = literal {
            let end = start + len;
            let mut j = i;
            while j < native.len() && native[j].range.end < end {
                j += 1;
            }
            // In particular, do not split 0x2d6 or a float that absorbed range dots.
            // 1..5 is two native floats (1. and .5); ..5 is Dot then Float.
            if j < native.len()
                && native[j].range.end == end
                && native[i..=j].iter().all(|t| {
                    matches!(
                        t.kind,
                        SourceTokenKind::Number
                            | SourceTokenKind::Identifier
                            | SourceTokenKind::Operator
                    )
                })
            {
                folded.push(SourceToken {
                    range: start..end,
                    kind: match literal {
                        Literal::Dice(_) => SourceTokenKind::Dice,
                        Literal::Band { .. } => SourceTokenKind::Band,
                    },
                    literal: Some(literal),
                });
                i = j + 1;
                continue;
            }
        }
        folded.push(token.clone());
        i += 1;
    }
    folded
}

fn native_tokens(source: &str) -> Vec<SourceToken> {
    let codemap = CodeMap::new("<editor>".into(), source.into());
    let mut out = Vec::new();
    let mut cursor = 0;
    let mut fstring_depth = 0usize;
    for lexeme in Lexer::new(source, &dice_dialect_public(), codemap) {
        let lexeme = lexeme.map_err(|e| e.into_error()).or_else(|error| {
            if fstring_depth == 0 {
                if let Some(count) = leading_zero_dice_count(source, &error) {
                    return Ok(count);
                }
            }
            Err(error)
        });
        let Ok((start, token, end)) = lexeme else {
            // Some upstream string errors point inside an escape, not at the quote.
            // Keep the entire unconsumed tail opaque rather than guessing recovery.
            if cursor < source.len() {
                out.push(SourceToken::new(
                    cursor..source.len(),
                    SourceTokenKind::Error,
                ));
            }
            return out;
        };
        if end <= cursor || start == end {
            continue; // synthetic EOF/newline/dedent tokens have no source text
        }
        if start > cursor {
            out.push(SourceToken::new(cursor..start, SourceTokenKind::Plain));
        }
        if matches!(token, Token::FStringStart(_)) {
            fstring_depth += 1;
        }
        let kind = if fstring_depth > 0 {
            SourceTokenKind::String
        } else {
            native_kind(&token)
        };
        if matches!(token, Token::FStringEnd) {
            fstring_depth = fstring_depth.saturating_sub(1);
        }
        out.push(SourceToken::new(start.max(cursor)..end, kind));
        cursor = end;
    }
    if cursor < source.len() {
        out.push(SourceToken::new(
            cursor..source.len(),
            SourceTokenKind::Plain,
        ));
    }
    out
}

/// Starlark rejects `02` before seeing the `d6` suffix. Retain that prefix
/// only for a complete dice literal, so folding can combine the native spans.
/// Every other lexical error, including ordinary `02` and string escapes, stays opaque.
fn leading_zero_dice_count(source: &str, error: &Error) -> Option<(usize, Token, usize)> {
    let ErrorKind::Parser(cause) = error.kind() else {
        return None;
    };
    if !matches!(
        cause.downcast_ref::<LexemeError>(),
        Some(LexemeError::StartsZero(_))
    ) {
        return None;
    }
    let span = error.span()?.span;
    let start = span.begin().get() as usize;
    let prev = source.get(..start)?.chars().next_back();
    dice_at(source.get(start..)?, prev)?;
    Some((start, Token::RawDecInt, span.end().get() as usize))
}

fn native_kind(token: &Token) -> SourceTokenKind {
    use SourceTokenKind as K;
    match token {
        Token::Indent | Token::Dedent | Token::Newline | Token::Tabs => K::Plain,
        Token::String(_) | Token::Bytes(_) => K::String,
        Token::Comment(_) => K::Comment,
        Token::Int(_) | Token::Float(_) | Token::RawDecInt => K::Number,
        Token::Identifier(_) => K::Identifier,
        Token::And
        | Token::Break
        | Token::Continue
        | Token::Def
        | Token::Elif
        | Token::Else
        | Token::For
        | Token::If
        | Token::In
        | Token::Lambda
        | Token::Load
        | Token::Not
        | Token::Or
        | Token::Pass
        | Token::Return => K::Keyword,
        _ => K::Operator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(source: &str, kind: SourceTokenKind) -> Vec<&str> {
        lex_source(source)
            .into_iter()
            .filter(|t| t.kind == kind)
            .map(|t| &source[t.range])
            .collect()
    }

    #[test]
    fn leading_zero_dice_counts_keep_their_literal_identity() {
        let source = "output(02d6, 04d6dl1, 0002d06)";
        assert_eq!(
            texts(source, SourceTokenKind::Dice),
            ["02d6", "04d6dl1", "0002d06"]
        );
        for source in ["02", "02d6foo", "02d6dl", "02d6dl0", "'bad\\xZZ02d6'"] {
            let tokens = lex_source(source);
            assert!(tokens.iter().any(|t| t.kind == SourceTokenKind::Error));
            assert!(!tokens.iter().any(|t| t.kind == SourceTokenKind::Dice));
            assert_eq!(
                tokens
                    .iter()
                    .map(|t| &source[t.range.clone()])
                    .collect::<String>(),
                source
            );
        }
        assert!(texts("'02d6' # 04d6dl1", SourceTokenKind::Dice).is_empty());
    }

    #[test]
    fn native_keywords_numbers_and_identifiers() {
        assert_eq!(texts("lambda x: x", SourceTokenKind::Keyword), ["lambda"]);
        assert_eq!(
            texts("while True:", SourceTokenKind::Keyword),
            Vec::<&str>::new()
        );
        assert_eq!(
            texts(
                "1e6 0xFF .5 1.5 0b101 0o77 12345678901234567890 1.e-2",
                SourceTokenKind::Number
            ),
            [
                "1e6",
                "0xFF",
                ".5",
                "1.5",
                "0b101",
                "0o77",
                "12345678901234567890",
                "1.e-2"
            ]
        );
        assert_eq!(
            texts("foo4d6 _d6 0x2d6 1e6d6 1.5d6", SourceTokenKind::Dice),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn only_complete_shorthand_tokens() {
        assert_eq!(
            texts(
                "2d6 d6 4D6DL1 4d6dl 4d6dl0 4d6abc 2..d6 1.5d6",
                SourceTokenKind::Dice
            ),
            ["2d6", "d6", "4D6DL1"]
        );
        assert_eq!(
            texts(
                "[1.5, 1..5, ..5, 5.., 1...5, 0x2..6, 1e2..6, 1.5..6]",
                SourceTokenKind::Band
            ),
            ["1..5", "..5", "5.."]
        );
    }

    #[test]
    fn native_strings_and_comments_are_opaque() {
        let source = "'4d6dl1' \"2d6\" r'..5' r\"4d6\" '''2d6\n..5''' \"\"\"4d6\n5..\"\"\" 'escaped\\\'d6' # don't lower 1..5\n2d6\n1..5";
        assert_eq!(texts(source, SourceTokenKind::Dice), ["2d6"]);
        assert_eq!(texts(source, SourceTokenKind::Band), ["1..5"]);
        assert_eq!(texts(source, SourceTokenKind::String).len(), 7);
    }

    #[test]
    fn every_edit_prefix_preserves_text_and_opaque_literals() {
        for source in [
            "output(r\"é😀 4d6dl1\", 2d6.keep(5..)) # don't lower ..5\n",
            "x = '''raw-looking r\"4d6\"\n..5'''\noutput(4D6KH2)\n",
            "x = [0x2d6, 0o777, 0b101, 1e-6, .5, 1., 1..5, ..5, 5..]\n",
            "def f(x):\n    # one\n    if x:\n        return 2d6\n    return d6\n",
        ] {
            for end in source.char_indices().map(|(i, _)| i).chain([source.len()]) {
                let prefix = &source[..end];
                let tokens = lex_source(prefix);
                assert_eq!(
                    tokens
                        .iter()
                        .map(|t| &prefix[t.range.clone()])
                        .collect::<String>(),
                    prefix
                );
            }
        }
    }

    #[test]
    fn every_byte_is_covered_once_even_in_incomplete_input() {
        for source in [
            "",
            "\n",
            " \r\n",
            "x = '''é\n2d6",
            "\"bad\\xZZ 4d6\"",
            "def f():\n    # a\n    return 2d6\n\nx = 1..5\n",
            "while True:\n  2d6",
            "output(4d6dl",
            "f\"text {2d6}\"",
            "# don't\r\n1..5\r\n",
            "é 2d6",
        ] {
            let tokens = lex_source(source);
            let mut cursor = 0;
            for t in &tokens {
                assert_eq!(t.range.start, cursor, "{source:?}");
                assert!(t.range.end > cursor);
                cursor = t.range.end;
            }
            assert_eq!(cursor, source.len(), "{source:?}");
            let joined: String = tokens.iter().map(|t| &source[t.range.clone()]).collect();
            assert_eq!(joined, source);
        }
    }
}
