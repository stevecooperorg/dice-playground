//! Stdio LSP server for `.dice` / Starlark (native only).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use super::source::{preparation_error_message, prepare_source};
use super::{dice_dialect_public, full_environment_docs};
use starlark::docs::DocModule;
use starlark::syntax::AstModule;
use starlark_lsp::error::eval_message_to_lsp_diagnostic;
use starlark_lsp::server::{stdio_server, LspContext, LspEvalResult, LspUri, StringLiteralResult};

#[derive(Default)]
struct DiceLspContext {
    files: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl LspContext for DiceLspContext {
    fn parse_file_with_contents(&self, uri: &LspUri, content: String) -> LspEvalResult {
        let path = uri_path(uri);
        let path = path.to_string_lossy();
        let (ast, messages) = match prepare_source(&path, &content) {
            Ok(prepared) => {
                let (ast, messages) = prepared.check(&path);
                // The upstream backend retains its previous AST on None. Replace it,
                // don't return an expanded AST with falsely original coordinates.
                let ast = if prepared.transformed { None } else { ast };
                (ast, messages)
            }
            Err(e) => (None, vec![preparation_error_message(&path, &e)]),
        };
        let diagnostics = messages
            .into_iter()
            .map(|message| {
                let mut diagnostic = eval_message_to_lsp_diagnostic(message);
                // Starlark columns count Unicode scalar values; LSP uses UTF-16 units.
                diagnostic.range.start.character = utf16_column(
                    &content,
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                );
                diagnostic.range.end.character = utf16_column(
                    &content,
                    diagnostic.range.end.line,
                    diagnostic.range.end.character,
                );
                diagnostic
            })
            .collect();
        LspEvalResult {
            diagnostics,
            ast: ast.or_else(|| inert_ast(&path, &content)),
        }
    }

    fn resolve_load(
        &self,
        path: &str,
        current_file: &LspUri,
        _workspace_root: Option<&Path>,
    ) -> Result<LspUri, String> {
        let current = uri_path(current_file);
        let joined = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            current
                .parent()
                .map(|p| p.join(path))
                .unwrap_or_else(|| PathBuf::from(path))
        };
        Ok(LspUri::Starlark(joined))
    }

    fn render_as_load(
        &self,
        target: &LspUri,
        current_file: &LspUri,
        _workspace_root: Option<&Path>,
    ) -> Result<String, String> {
        let target = uri_path(target);
        let current = uri_path(current_file);
        if let Some(parent) = current.parent() {
            if let Ok(rel) = target.strip_prefix(parent) {
                return Ok(rel.to_string_lossy().into_owned());
            }
        }
        Ok(target.to_string_lossy().into_owned())
    }

    fn resolve_string_literal(
        &self,
        _literal: &str,
        _current_file: &LspUri,
        _workspace_root: Option<&Path>,
    ) -> Result<Option<StringLiteralResult>, String> {
        Ok(None)
    }

    fn get_load_contents(&self, uri: &LspUri) -> Result<Option<String>, String> {
        let path = uri_path(uri);
        if let Ok(map) = self.files.read() {
            if let Some(s) = map.get(&path) {
                return Ok(Some(s.clone()));
            }
        }
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| e.to_string())
    }

    fn get_environment(&self, _uri: &LspUri) -> DocModule {
        full_environment_docs()
    }

    fn get_uri_for_global_symbol(
        &self,
        _current_file: &LspUri,
        _symbol: &str,
    ) -> Result<Option<LspUri>, String> {
        Ok(None)
    }
}

/// Replace all non-newline bytes with comments. The resulting AST has no symbols,
/// strings, loads, or expression nodes, but covers the editor's line/byte extents.
/// This invalidates starlark_lsp's last-valid-parse cache even after parse failures.
fn inert_ast(path: &str, content: &str) -> Option<AstModule> {
    let inert: String = content
        .bytes()
        .map(|b| if b == b'\n' { '\n' } else { '#' })
        .collect();
    AstModule::parse(path, inert, &dice_dialect_public()).ok()
}

fn utf16_column(source: &str, line: u32, scalar_column: u32) -> u32 {
    source
        .split('\n')
        .nth(line as usize)
        .unwrap_or_default()
        .trim_end_matches('\r')
        .chars()
        .take(scalar_column as usize)
        .map(|c| c.len_utf16() as u32)
        .sum()
}

fn uri_path(uri: &LspUri) -> PathBuf {
    match uri {
        LspUri::File(p) | LspUri::Starlark(p) => p.clone(),
        _ => PathBuf::from("unknown.dice"),
    }
}

/// Run the Dice language server on stdio (for `dice lsp`).
pub fn run_stdio() -> anyhow::Result<()> {
    stdio_server(DiceLspContext::default()).map_err(|e| anyhow::anyhow!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark_syntax::syntax::module::AstModuleFields;

    fn parse(content: &str) -> LspEvalResult {
        DiceLspContext::default()
            .parse_file_with_contents(&LspUri::File(PathBuf::from("test.dice")), content.into())
    }

    #[test]
    fn public_dialect_and_literate_diagnostics_match_check() {
        for source in [
            "output(2d6)",
            "f = lambda x: x + 1",
            "for x in [1]:\n    output(x)",
            "x: int = 1",
            "load('x', 'y')",
            "while True:\n    pass",
            "```dice\noutput(2d6)\n```\n",
            "```dice\nx = 1\n```\n\n```dice\nx = 2d6; @\n```\n",
        ] {
            let checked = super::super::check_source("test.dice", source).unwrap();
            let lsp = parse(source);
            assert_eq!(lsp.diagnostics.len(), checked.diagnostics.len(), "{source}");
            for (lsp, checked) in lsp.diagnostics.iter().zip(&checked.diagnostics) {
                assert_eq!(lsp.range.start.line + 1, checked.line, "{source}");
                assert_eq!(lsp.range.start.character + 1, checked.column, "{source}");
                assert!(checked.message.ends_with(&lsp.message), "{source}");
            }
        }
    }

    #[test]
    fn original_document_locations_use_utf16_in_lsp() {
        let line = "x = [\"é😀\", 2d6, 1..5]; @";
        let source = format!("# Header\n```dice\nx = 1\n```\n\n```dice\n{line}\n```\n");
        let result = parse(&source);
        assert_eq!(result.diagnostics.len(), 1);
        let pos = result.diagnostics[0].range.start;
        assert_eq!(pos.line, 6);
        assert_eq!(
            pos.character as usize,
            line[..line.find('@').unwrap()].encode_utf16().count()
        );
    }

    #[test]
    fn unfinished_multiline_string_ranges_end_at_the_original_body_boundary() {
        for source in ["```dice\nx = '''\n\n```\n", "```dice\nx = '''é😀\n\n```\n"] {
            for source in [source.to_owned(), source.replace('\n', "\r\n")] {
                let result = parse(&source);
                assert_eq!(result.diagnostics.len(), 1);
                let range = result.diagnostics[0].range;
                assert_eq!((range.start.line, range.start.character), (1, 4));
                assert_eq!((range.end.line, range.end.character), (2, 0));
                assert!(range.start <= range.end);
            }
        }
    }

    #[test]
    fn unclosed_later_fence_reports_its_document_opener() {
        for newline in ["\n", "\r\n"] {
            let source = "# Header\n```dice\nx = 1\n```\n\n```dice\ny = 2\n".replace('\n', newline);
            let result = parse(&source);
            assert_eq!(result.diagnostics.len(), 1);
            let diagnostic = &result.diagnostics[0];
            assert!(diagnostic.message.contains("unclosed"));
            assert_eq!(
                (
                    diagnostic.range.start.line,
                    diagnostic.range.start.character
                ),
                (5, 0)
            );
            assert!(diagnostic.range.start <= diagnostic.range.end);
            assert!(
                result.ast.is_some(),
                "preparation errors must also clear the cached AST"
            );
        }
    }

    #[test]
    fn transformed_and_invalid_buffers_replace_previous_ast_cache() {
        // Model Backend::validate: it updates its last_valid_parse ONLY on Some.
        let mut cached = parse("old_symbol = d(6)").ast.unwrap();
        assert!(cached.codemap().source().contains("old_symbol"));
        for source in [
            "new_symbol = 2d6",
            "```dice\nnew_symbol = d(6)\n```",
            "output(",
            "load('x', 'y')",
        ] {
            let result = parse(source);
            if let Some(ast) = result.ast {
                cached = ast;
            }
            let cached_source = cached.codemap().source();
            assert_eq!(cached_source.len(), source.len());
            assert_eq!(cached_source.lines().count(), source.lines().count());
            assert!(cached_source.chars().all(|c| c == '#' || c == '\n'));
            assert!(!cached_source.contains("old_symbol"));
            assert!(!cached_source.contains("dice_pool"));
        }
        let unchanged = "new_symbol = d(8)";
        if let Some(ast) = parse(unchanged).ast {
            cached = ast;
        }
        assert_eq!(cached.codemap().source(), unchanged);
    }
}
