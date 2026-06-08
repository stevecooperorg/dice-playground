//! Stdio LSP server for `.dice` / Starlark (native only).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use super::{dice_dialect, full_environment_docs};
use starlark::analysis::AstModuleLint;
use starlark::docs::DocModule;
use starlark::errors::EvalMessage;
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
        match AstModule::parse(&path.to_string_lossy(), content, &dice_dialect()) {
            Ok(ast) => {
                let diagnostics = ast
                    .lint(None)
                    .into_iter()
                    .map(|l| eval_message_to_lsp_diagnostic(EvalMessage::from(l)))
                    .collect();
                LspEvalResult {
                    diagnostics,
                    ast: Some(ast),
                }
            }
            Err(e) => LspEvalResult {
                diagnostics: vec![eval_message_to_lsp_diagnostic(EvalMessage::from_error(
                    &path, &e,
                ))],
                ast: None,
            },
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
