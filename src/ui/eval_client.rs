//! Check and eval for the playground UI (in-browser WASM; no HTTP server).

use crate::engine::{
    check_source as lang_check, eval_program as lang_eval, CheckResult, EvalProgramOptions,
    ProbFormat, SourceDiagnostic,
};

use super::models::UiDiagnostic;

pub struct EvalResponse {
    pub return_value: String,
    pub text: String,
    pub outputs: Vec<crate::engine::OutputEntry>,
    /// Non-empty when the active file is literate and Run succeeded.
    pub report_html: String,
}

pub async fn check_source(path: &str, source: &str) -> Result<Vec<UiDiagnostic>, String> {
    run_check(path, source)
}

pub async fn eval_program(
    path: &str,
    source: &str,
    prob_format: &str,
) -> Result<EvalResponse, String> {
    run_eval(path, source, prob_format)
}

fn run_check(path: &str, source: &str) -> Result<Vec<UiDiagnostic>, String> {
    let body = lang_check(path, source).map_err(|e| format!("{e:#}"))?;
    Ok(map_diagnostics(body))
}

fn run_eval(path: &str, source: &str, prob_format: &str) -> Result<EvalResponse, String> {
    let prob_format = parse_prob_format(prob_format);
    let body = lang_eval(path, source, EvalProgramOptions { prob_format })
        .map_err(|e| format!("{e:#}"))?;
    Ok(EvalResponse {
        return_value: body.return_value,
        text: body.text,
        outputs: body.outputs,
        report_html: body.report_html,
    })
}

fn map_diagnostics(body: CheckResult) -> Vec<UiDiagnostic> {
    body.diagnostics
        .into_iter()
        .map(|d: SourceDiagnostic| UiDiagnostic {
            line: d.line,
            column: d.column,
            message: d.message,
            severity: d.severity,
        })
        .collect()
}

fn parse_prob_format(s: &str) -> ProbFormat {
    match s {
        "percent" => ProbFormat::Percent,
        "fraction" => ProbFormat::Fraction,
        "sample-space" => ProbFormat::SampleSpace,
        _ => ProbFormat::Decimal,
    }
}
