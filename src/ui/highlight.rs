//! Presentation only: engine token spans determine all language/document categories.

pub use crate::engine::SourceTokenKind as TokenKind;
use crate::engine::{lex_document, lex_source, SourceToken};

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
            TokenKind::Band => "tok-band",
            TokenKind::Identifier => "tok-id",
            TokenKind::Operator => "tok-op",
            TokenKind::Markdown => "tok-md",
            TokenKind::Heading => "tok-heading",
            TokenKind::Fence => "tok-fence",
            TokenKind::Error => "tok-error",
        }
    }
}

fn colored(source: &str, tokens: Vec<SourceToken>) -> Vec<ColoredSpan> {
    let mut spans: Vec<ColoredSpan> = Vec::new();
    for token in tokens {
        let text = &source[token.range];
        if let Some(last) = spans.last_mut().filter(|last| last.kind == token.kind) {
            last.text.push_str(text);
        } else {
            spans.push(ColoredSpan {
                text: text.into(),
                kind: token.kind,
            });
        }
    }
    spans
}

/// Highlight a whole document, preserving embedded/trailing newlines exactly.
///
/// ```
/// use dice_playground::ui::highlight::highlight_document;
/// let source = "```dice\n2d6\n```\n";
/// assert_eq!(highlight_document(source).iter().map(|s| s.text.as_str()).collect::<String>(), source);
/// ```
pub fn highlight_document(source: &str) -> Vec<ColoredSpan> {
    colored(source, lex_document(source))
}

/// Compatibility helper for script snippets. The editor uses document-level state.
///
/// ```
/// use dice_playground::ui::highlight::{highlight_line, TokenKind};
/// assert_eq!(highlight_line("d6")[0].kind, TokenKind::Dice);
/// ```
pub fn highlight_line(line: &str) -> Vec<ColoredSpan> {
    colored(line, lex_source(line))
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
    fn document_spans_preserve_backdrop_text_and_multiline_state() {
        let source = "# Title\r\n```dice\r\nx = '''4d6\r\n..5'''\r\noutput(2d6.keep(5..))\r\n```\r\n```text\r\n4d6\r\n```\r\n\r\n";
        let spans = highlight_document(source);
        assert_eq!(
            spans.iter().map(|s| s.text.as_str()).collect::<String>(),
            source
        );
        assert_eq!(text_for_kind(&spans, TokenKind::Dice), "2d6");
        assert_eq!(text_for_kind(&spans, TokenKind::Band), "5..");
        assert!(has_kind(&spans, TokenKind::Heading));
        assert!(has_kind(&spans, TokenKind::Markdown));
        assert_eq!(
            spans
                .iter()
                .find(|s| s.kind == TokenKind::Band)
                .unwrap()
                .class_name(),
            "tok-band"
        );
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
