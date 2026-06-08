//! Exact dice probability distributions with a Starlark scripting surface.

mod core;
mod die;
mod enumerate;
mod ordinal;
mod poly_explode;
mod pool;
mod sugar;

mod playground;
mod starlark_guest;

#[cfg(feature = "lsp")]
pub mod lsp;

pub use core::{total_variation_distance, Die, Dist, PoolOp, RollPool, MAX_JOINT_CELLS};
pub use ordinal::{LabelDist, ResultScale};
pub use playground::{
    check_source, dice_dialect_public, eval_program, CheckResult, EvalProgramOptions,
    EvalProgramResponse, SourceDiagnostic, MAX_OUTPUT_COUNT, MAX_SOURCE_BYTES,
};
pub use poly_explode::{successes_dist, Counterbalance};
pub use starlark_guest::{
    dice_dialect, dice_globals, dice_stdlib_docs, dist_type_docs, eval_source,
    eval_source_with_dialect, format_eval_result_text, format_probability,
    format_probability_with_denom, full_environment_docs, infer_sample_space_denominator,
    label_dist_type_docs, render_stdlib_reference_markdown, EvalResult, OutputEntry, OutputStore,
    ProbFormat, StarlarkDist, StarlarkLabelDist, StarlarkResultScale, StarlarkRollPool,
};
pub use sugar::{desugar, desugar_if_needed, dice_literal_len_at};
