//! Shared public-source preparation for checking, evaluation, static rendering and LSP.
//! Locations compose expansion byte offsets with the existing literate line map.

use std::path::Path;

use anyhow::{bail, Context};
use starlark::analysis::{AstModuleLint, EvalMessage};
use starlark::codemap::{CodeMap, Pos, ResolvedPos, ResolvedSpan, Span};
use starlark::syntax::AstModule;

#[cfg(feature = "lsp")]
use super::literate::LiterateParseError;
use super::literate::{
    is_literate, parse_literate, source_line_for_tangled, tangle_literate, LiterateDocument,
    TangleResult, MAX_LITERATE_BYTES,
};
use super::lowering::{lower, Expansion};
use super::playground::{dice_dialect_public, MAX_SOURCE_BYTES};
use super::starlark_guest::EvalResult;

pub(crate) struct PreparedSource {
    expansion: Expansion,
    expanded_map: CodeMap,
    input_map: CodeMap,
    pub literate: Option<(LiterateDocument, TangleResult)>,
    pub transformed: bool,
}

pub(crate) fn prepare_source(path: &str, source: &str) -> anyhow::Result<PreparedSource> {
    let literate = if is_literate(source) {
        if source.len() > MAX_LITERATE_BYTES {
            bail!("source exceeds maximum size of {MAX_LITERATE_BYTES} bytes");
        }
        let doc = parse_literate(source).context("parse literate document")?;
        let tangled = tangle_literate(&doc);
        Some((doc, tangled))
    } else {
        if source.len() > MAX_SOURCE_BYTES {
            bail!("source exceeds maximum size of {MAX_SOURCE_BYTES} bytes");
        }
        None
    };
    let input = literate
        .as_ref()
        .map_or(source, |(_, t)| t.tangled.as_str());
    let expansion = lower(input, true, true);
    let transformed = literate.is_some() || expansion.source != source;
    Ok(PreparedSource {
        expanded_map: CodeMap::new(path.into(), expansion.source.clone()),
        input_map: CodeMap::new(path.into(), input.into()),
        expansion,
        literate,
        transformed,
    })
}

/// Preparation fails before expansion, so a fence error already names an original
/// document line. Preserve that typed location rather than parsing error prose.
#[cfg(feature = "lsp")]
pub(crate) fn preparation_error_message(path: &str, error: &anyhow::Error) -> EvalMessage {
    let mut message = EvalMessage::from_any_error(Path::new(path), error);
    if let Some(line) = error
        .downcast_ref::<LiterateParseError>()
        .and_then(|e| e.line)
    {
        let position = ResolvedPos {
            line: line.saturating_sub(1) as usize,
            column: 0,
        };
        message.span = Some(ResolvedSpan {
            begin: position,
            end: position,
        });
    }
    message
}

impl PreparedSource {
    pub fn parse(&self, path: &str) -> Result<AstModule, starlark::Error> {
        AstModule::parse(path, self.expansion.source.clone(), &dice_dialect_public())
    }

    pub fn check(&self, path: &str) -> (Option<AstModule>, Vec<EvalMessage>) {
        match self.parse(path) {
            Ok(ast) => {
                let messages = ast
                    .lint(None)
                    .into_iter()
                    .map(EvalMessage::from)
                    .map(|m| self.map_message(m))
                    .collect();
                (Some(ast), messages)
            }
            Err(e) => (
                None,
                vec![self.map_message(EvalMessage::from_error(Path::new(path), &e))],
            ),
        }
    }

    pub fn eval(&self, path: &str, ast: AstModule) -> anyhow::Result<EvalResult> {
        super::starlark_guest::eval_ast(ast).map_err(|e| {
            // Keep the existing anyhow result shape, but never print expanded coordinates.
            let message = self.map_message(EvalMessage::from_error(Path::new(path), &e));
            anyhow::anyhow!("{message}")
        })
    }

    pub fn map_message(&self, mut message: EvalMessage) -> EvalMessage {
        if !self.transformed {
            return message;
        }
        if let Some(span) = message.span {
            message.span = Some(ResolvedSpan {
                begin: self.original_position(span.begin, false),
                end: self.original_position(span.end, true),
            });
            // These upstream renderings contain the intermediate text/coordinates.
            message.full_error_with_span = None;
            message.original = None;
        }
        message
    }

    fn original_position(&self, pos: ResolvedPos, end: bool) -> ResolvedPos {
        let offset = byte_offset(&self.expanded_map, pos);
        let original = self
            .expansion
            .original_offset(offset, end)
            .min(self.input_map.source().len());
        let point = Pos::new(original as u32);
        let mut pos = self.input_map.resolve_span(Span::new(point, point)).begin;
        if let Some((doc, tangled)) = &self.literate {
            // A trailing LF places EOF on a following logical line with no emitted
            // body entry. Map the whole endpoint, not column zero on the preceding line.
            if pos.line >= tangled.line_map.lines.len() {
                if let Some(fence) = doc.fences.iter().rev().find(|f| !f.body.is_empty()) {
                    return ResolvedPos {
                        line: fence.source_open_line as usize
                            + fence.body.bytes().filter(|&b| b == b'\n').count(),
                        column: fence
                            .body
                            .rsplit('\n')
                            .next()
                            .unwrap_or_default()
                            .chars()
                            .count(),
                    };
                }
            }
            pos.line = source_line_for_tangled(&tangled.line_map, (pos.line + 1) as u32)
                .saturating_sub(1) as usize;
        }
        pos
    }
}

fn byte_offset(map: &CodeMap, pos: ResolvedPos) -> usize {
    let Some(line) = map.line_span_opt(pos.line) else {
        return map.source().len();
    };
    let text = map.source_span(line);
    line.begin().get() as usize
        + text
            .char_indices()
            .nth(pos.column)
            .map_or(text.len(), |(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_unicode_and_multiple_literals_back_to_document_columns() {
        let line = "x = [\"é😀\", 2d6, 1..5]; missing(";
        let source = format!("# Title\n```dice\ny = 1\n```\n\nLater\n```dice\n{line}\n```\n");
        let prepared = prepare_source("t.dice", &source).unwrap();
        let offset = prepared.expansion.source.find("missing").unwrap();
        let point = Pos::new(offset as u32);
        let pos = prepared
            .expanded_map
            .resolve_span(Span::new(point, point))
            .begin;
        let original = prepared.original_position(pos, false);
        assert_eq!(original.line, 7);
        assert_eq!(
            original.column,
            line[..line.find("missing").unwrap()].chars().count()
        );
    }

    #[test]
    fn unchanged_scripts_keep_ast_coordinates() {
        assert!(
            !prepare_source("t.dice", "output(d(6))")
                .unwrap()
                .transformed
        );
        assert!(prepare_source("t.dice", "output(2d6)").unwrap().transformed);
        assert!(
            prepare_source("t.dice", "```dice\noutput(d(6))\n```")
                .unwrap()
                .transformed
        );
    }
}
