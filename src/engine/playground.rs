//! HTTP-friendly check/eval API for the web playground (no `load()` in public dialect).

use std::path::Path;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use starlark::analysis::{AstModuleLint, EvalMessage, EvalSeverity};
use starlark::syntax::AstModule;

use super::desugar_if_needed;
use super::{eval_source_with_dialect, format_eval_result_text, OutputEntry, ProbFormat};

pub const MAX_SOURCE_BYTES: usize = 64 * 1024;
pub const MAX_OUTPUT_COUNT: usize = 500;

/// Dialect for untrusted public evaluation (`load` disabled).
pub fn dice_dialect_public() -> starlark::syntax::Dialect {
    starlark::syntax::Dialect {
        enable_load: false,
        enable_types: starlark::syntax::DialectTypes::Enable,
        enable_top_level_stmt: true,
        ..starlark::syntax::Dialect::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDiagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub diagnostics: Vec<SourceDiagnostic>,
}

impl CheckResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == "error")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvalProgramOptions {
    #[serde(default)]
    pub prob_format: ProbFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalProgramResponse {
    pub return_value: String,
    pub outputs: Vec<OutputEntry>,
    pub text: String,
}

/// Parse and lint after desugar; does not evaluate.
pub fn check_source(path: &str, source: &str) -> anyhow::Result<CheckResult> {
    if source.len() > MAX_SOURCE_BYTES {
        bail!("source exceeds maximum size of {MAX_SOURCE_BYTES} bytes");
    }
    let expanded = desugar_if_needed(path, source)?;
    let dialect = dice_dialect_public();
    let diagnostics = match AstModule::parse(path, expanded, &dialect) {
        Ok(ast) => ast
            .lint(None)
            .into_iter()
            .map(EvalMessage::from)
            .map(diagnostic_from_eval_message)
            .collect(),
        Err(e) => {
            vec![diagnostic_from_eval_message(EvalMessage::from_error(
                Path::new(path),
                &e,
            ))]
        }
    };
    Ok(CheckResult { diagnostics })
}

/// Check, then evaluate with guardrails.
pub fn eval_program(
    path: &str,
    source: &str,
    options: EvalProgramOptions,
) -> anyhow::Result<EvalProgramResponse> {
    if source.len() > MAX_SOURCE_BYTES {
        bail!("source exceeds maximum size of {MAX_SOURCE_BYTES} bytes");
    }
    let check = check_source(path, source)?;
    if check.has_errors() {
        bail!("fix parse/lint errors before running");
    }
    let expanded = desugar_if_needed(path, source)?;
    let result = eval_source_with_dialect(path, &expanded, &dice_dialect_public())
        .context("evaluate")?;
    if result.outputs.len() > MAX_OUTPUT_COUNT {
        bail!("too many output() calls (max {MAX_OUTPUT_COUNT})");
    }
    let text = format_eval_result_text(&result, options.prob_format);
    Ok(EvalProgramResponse {
        return_value: result.return_value,
        outputs: result.outputs,
        text,
    })
}

fn diagnostic_from_eval_message(msg: EvalMessage) -> SourceDiagnostic {
    let (line, column) = msg
        .span
        .map(|s| (s.begin.line as u32 + 1, s.begin.column as u32 + 1))
        .unwrap_or((1, 1));
    let message = if msg.description.is_empty() {
        msg.name.clone()
    } else {
        format!("{}: {}", msg.name, msg.description)
    };
    SourceDiagnostic {
        line,
        column,
        message,
        severity: severity_str(msg.severity).to_owned(),
    }
}

fn severity_str(s: EvalSeverity) -> &'static str {
    match s {
        EvalSeverity::Error => "error",
        EvalSeverity::Warning => "warning",
        EvalSeverity::Advice => "advice",
        EvalSeverity::Disabled => "info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tutorial(rel: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
        std::fs::read_to_string(&path).expect("read tutorial")
    }

    #[test]
    fn check_tutorial_one_die_ok() {
        let src = tutorial("examples/tutorial/01-one-die.dice");
        let r = check_source("01-one-die.dice", &src).expect("check");
        assert!(!r.has_errors());
    }

    #[test]
    fn check_syntax_error() {
        let r = check_source("bad.dice", "output(\n").expect("check");
        assert!(r.has_errors());
    }

    #[test]
    fn eval_program_tutorial_two_d6() {
        let src = tutorial("examples/tutorial/02-two-d6.dice");
        let r = eval_program("02-two-d6.dice", &src, EvalProgramOptions::default()).expect("eval");
        assert_eq!(r.outputs.len(), 1);
        assert!(r.text.contains("two_d6"));
    }

    #[test]
    fn eval_program_sample_space_ordinal_and_prob() {
        let src = tutorial("examples/tutorial/09-pbta-2d6-move.dice");
        let r = eval_program(
            "09-pbta-2d6-move.dice",
            &src,
            EvalProgramOptions {
                prob_format: ProbFormat::SampleSpace,
            },
        )
        .expect("eval");
        assert!(r.text.contains("MISS") && r.text.contains("X/36"), "ordinal: {}", r.text);
        assert!(
            r.text.contains("p_full_success") && r.text.contains("X/36") && !r.text.contains("15/36"),
            "prob: {}",
            r.text
        );
    }

    #[test]
    fn public_dialect_disables_load() {
        let d = dice_dialect_public();
        assert!(!d.enable_load);
        let full = crate::engine::dice_dialect();
        assert!(full.enable_load);
    }
}
