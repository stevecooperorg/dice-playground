//! Check and evaluate `.dice` programs for the web playground.
//!
//! Applies dice notation desugar, parses with a restricted Starlark dialect (`load` disabled),
//! and returns structured diagnostics plus formatted output tables.

use std::path::Path;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use starlark::analysis::{AstModuleLint, EvalMessage, EvalSeverity};
use starlark::syntax::AstModule;

use super::desugar_if_needed;
use super::literate::MAX_LITERATE_BYTES;
use super::literate::{
    is_literate, parse_literate, source_line_for_tangled, tangle_literate, weave_literate,
    LiterateDocument, TangleResult, WeaveOptions,
};
use super::output_html::format_eval_outputs_html_sections;
use super::{eval_source_with_dialect, format_eval_result_text, OutputEntry, ProbFormat};

/// Maximum script size accepted from the public playground API.
pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// Maximum `output()` lines allowed per evaluation (DoS guardrail).
pub const MAX_OUTPUT_COUNT: usize = 500;

/// Dialect for untrusted public evaluation (`load` disabled).
///
/// # Example
///
/// ```
/// use dice_playground::engine::dice_dialect_public;
/// assert!(!dice_dialect_public().enable_load);
/// ```
pub fn dice_dialect_public() -> starlark::syntax::Dialect {
    starlark::syntax::Dialect {
        enable_load: false,
        enable_types: starlark::syntax::DialectTypes::Enable,
        enable_top_level_stmt: true,
        ..starlark::syntax::Dialect::Standard
    }
}

/// Single parse, lint, or runtime diagnostic with 1-based line/column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDiagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: String,
}

/// Result of [`check_source`] (parse + lint only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub diagnostics: Vec<SourceDiagnostic>,
}

impl CheckResult {
    /// True when any diagnostic has severity `"error"`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::check_source;
    /// let r = check_source("ok.dice", "output(d(6))").unwrap();
    /// assert!(!r.has_errors());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == "error")
    }
}

/// Options for [`eval_program`] (probability display format, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvalProgramOptions {
    #[serde(default)]
    pub prob_format: ProbFormat,
}

/// Structured and rendered text from a successful [`eval_program`] call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalProgramResponse {
    pub return_value: String,
    pub outputs: Vec<OutputEntry>,
    pub text: String,
    /// Sanitized HTML report fragment when the source is literate; empty for legacy scripts.
    #[serde(default)]
    pub report_html: String,
    /// Sanitized HTML for output blocks only (legacy scripts); empty when literate weave supplies `report_html`.
    #[serde(default)]
    pub outputs_html: String,
}

/// Parse and lint after desugar; does not evaluate.
///
/// # Example
///
/// ```
/// use dice_playground::engine::check_source;
/// let r = check_source("test.dice", "output(d(6))").unwrap();
/// assert!(!r.has_errors());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn check_source(path: &str, source: &str) -> anyhow::Result<CheckResult> {
    let prepared = starlark_input(path, source)?;
    let expanded = desugar_if_needed(path, &prepared.starlark)?;
    let dialect = dice_dialect_public();
    let diagnostics = match AstModule::parse(path, expanded, &dialect) {
        Ok(ast) => ast
            .lint(None)
            .into_iter()
            .map(EvalMessage::from)
            .map(|msg| {
                remap_diagnostic(
                    diagnostic_from_eval_message(msg),
                    prepared.line_map.as_ref(),
                )
            })
            .collect(),
        Err(e) => {
            vec![remap_diagnostic(
                diagnostic_from_eval_message(EvalMessage::from_error(Path::new(path), &e)),
                prepared.line_map.as_ref(),
            )]
        }
    };
    Ok(CheckResult { diagnostics })
}

/// Check, then evaluate with guardrails (size limits, no `load`, output cap).
///
/// # Example
///
/// ```
/// use dice_playground::engine::{eval_program, EvalProgramOptions};
/// let r = eval_program("test.dice", "output(d(6))", EvalProgramOptions::default()).unwrap();
/// assert_eq!(r.outputs.len(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn eval_program(
    path: &str,
    source: &str,
    options: EvalProgramOptions,
) -> anyhow::Result<EvalProgramResponse> {
    let prepared = starlark_input(path, source)?;
    let check = check_source(path, source)?;
    if check.has_errors() {
        bail!("fix parse/lint errors before running");
    }
    let expanded = desugar_if_needed(path, &prepared.starlark)?;
    let result =
        eval_source_with_dialect(path, &expanded, &dice_dialect_public()).context("evaluate")?;
    if result.outputs.len() > MAX_OUTPUT_COUNT {
        bail!("too many output() calls (max {MAX_OUTPUT_COUNT})");
    }
    let text = format_eval_result_text(&result, options.prob_format);
    let report_html = if let Some((doc, tangled)) = prepared.literate.as_ref() {
        weave_literate(
            source,
            doc,
            tangled,
            &result,
            WeaveOptions {
                prob_format: options.prob_format,
                ..Default::default()
            },
        )
        .context("weave literate report")?
    } else {
        String::new()
    };
    let outputs_html = if report_html.is_empty() {
        format_eval_outputs_html_sections(&result, options.prob_format)
    } else {
        String::new()
    };
    Ok(EvalProgramResponse {
        return_value: result.return_value,
        outputs: result.outputs,
        text,
        report_html,
        outputs_html,
    })
}

struct PreparedSource {
    starlark: String,
    line_map: Option<super::literate::LineMap>,
    literate: Option<(LiterateDocument, TangleResult)>,
}

fn starlark_input(_path: &str, source: &str) -> anyhow::Result<PreparedSource> {
    if is_literate(source) {
        if source.len() > MAX_LITERATE_BYTES {
            bail!("source exceeds maximum size of {MAX_LITERATE_BYTES} bytes");
        }
        let doc = parse_literate(source).context("parse literate document")?;
        let tangled = tangle_literate(&doc);
        Ok(PreparedSource {
            starlark: tangled.tangled.clone(),
            line_map: Some(tangled.line_map.clone()),
            literate: Some((doc, tangled)),
        })
    } else {
        if source.len() > MAX_SOURCE_BYTES {
            bail!("source exceeds maximum size of {MAX_SOURCE_BYTES} bytes");
        }
        Ok(PreparedSource {
            starlark: source.to_owned(),
            line_map: None,
            literate: None,
        })
    }
}

fn remap_diagnostic(
    d: SourceDiagnostic,
    line_map: Option<&super::literate::LineMap>,
) -> SourceDiagnostic {
    let Some(map) = line_map else {
        return d;
    };
    SourceDiagnostic {
        line: source_line_for_tangled(map, d.line),
        ..d
    }
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
    fn eval_literate_includes_report_html() {
        let src = "# Title\n\n```dice\noutput(\"d6\", 1d6)\n```\n";
        let r = eval_program("lit.dice", src, EvalProgramOptions::default()).expect("eval");
        assert!(!r.report_html.is_empty());
        assert!(r.report_html.contains("<h1>"));
        assert!(r.outputs_html.is_empty());
    }

    #[test]
    fn eval_legacy_includes_outputs_html() {
        let r = eval_program("legacy.dice", "output(\"d6\", 1d6)", EvalProgramOptions::default())
            .expect("eval");
        assert!(r.report_html.is_empty());
        assert!(!r.outputs_html.is_empty());
        assert!(r.outputs_html.contains("<table>"));
    }

    #[test]
    fn eval_literate_two_fences_shared_scope() {
        let src =
            "Intro.\n\n```dice\nbonus = 3\n```\n\n```dice\noutput(\"check\", 1d20 + bonus)\n```\n";
        let r = eval_program("lit.dice", src, EvalProgramOptions::default()).expect("eval");
        assert_eq!(r.outputs.len(), 1);
        assert!(r.text.contains("check"));
    }

    #[test]
    fn literate_syntax_error_maps_to_source_line() {
        let src = "# Title\n\n```dice\noutput(\n```\n";
        let r = check_source("lit.dice", src).expect("check");
        assert!(r.has_errors());
        assert_eq!(r.diagnostics[0].line, 4);
    }

    #[test]
    fn check_tutorial_two_d6_ok() {
        let src = tutorial("docs/tutorial/02-two-dice.dice");
        let r = check_source("02-two-d6.dice", &src).expect("check");
        assert!(!r.has_errors());
    }

    #[test]
    fn check_syntax_error() {
        let r = check_source("bad.dice", "output(\n").expect("check");
        assert!(r.has_errors());
    }

    #[test]
    fn eval_program_tutorial_two_d6() {
        let src = tutorial("docs/tutorial/02-two-dice.dice");
        let r =
            eval_program("02-two-dice.dice", &src, EvalProgramOptions::default()).expect("eval");
        assert_eq!(r.outputs.len(), 1);
        assert!(r.text.contains("two_d6"));
    }

    #[test]
    fn eval_program_sample_space_ordinal_and_prob() {
        let src = tutorial("docs/tutorial/13-pbta-2d6-move.dice");
        let r = eval_program(
            "13-pbta-2d6-move.dice",
            &src,
            EvalProgramOptions {
                prob_format: ProbFormat::SampleSpace,
            },
        )
        .expect("eval");
        assert!(
            r.text.contains("MISS") && r.text.contains("X/36"),
            "ordinal: {}",
            r.text
        );
        assert!(
            r.text.contains("p_full_success")
                && r.text.contains("X/36")
                && !r.text.contains("15/36"),
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
