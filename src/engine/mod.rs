//! Exact dice probability (PMF) engine and Starlark scripting surface.
//!
//! At the table, a roll like `2d6` or `4d6dl1` is a finite set of outcomes each with a
//! chance. This crate represents that as a **probability mass function** (PMF): a map from
//! integer totals to probabilities that sum to 1. [`DieRoll`] is one such distribution;
//! [`DicePool`] holds several dice before you sum or apply keep/drop rules; [`Outcomes`]
//! names bands (hit/miss, crit, and so on) instead of raw numbers.
//!
//! User-facing lessons live under `docs/tutorial/`; the generated function list is
//! `docs/references/stdlib.md` (`make references`).

mod core;
mod dice_pool;
mod die_roll;
mod enumerate;
mod face_spec;
mod int_band;
mod ordinal;
mod poly_explode;
mod range_sugar;
mod sugar;

mod markdown_html;
mod playground;
mod starlark_guest;

#[cfg(feature = "lsp")]
pub mod lsp;

pub use core::{total_variation_distance, DicePool, DieRoll, PoolOp, MAX_JOINT_CELLS};
pub use face_spec::{FaceSpec, OptionalFaceSpec};
pub use int_band::IntBand;
pub use markdown_html::markdown_to_html;
pub use ordinal::{Outcomes, Scale};
pub use playground::{
    check_source, dice_dialect_public, eval_program, CheckResult, EvalProgramOptions,
    EvalProgramResponse, SourceDiagnostic, MAX_OUTPUT_COUNT, MAX_SOURCE_BYTES,
};
pub use poly_explode::{successes_dist, Counterbalance};
pub use starlark_guest::{
    compress_pmf_for_display, dice_dialect, dice_globals, dice_stdlib_docs, die_roll_type_docs,
    eval_source, eval_source_with_dialect, format_eval_result_text, format_probability,
    format_probability_with_denom, full_environment_docs, infer_sample_space_denominator,
    outcomes_type_docs, render_stdlib_reference_markdown, EvalResult, OutputEntry, OutputStore,
    ProbFormat, StarlarkDicePool, StarlarkDieRoll, StarlarkIntBand, StarlarkOutcomes,
    StarlarkScale,
};
pub use sugar::{desugar, desugar_if_needed, dice_literal_len_at};
