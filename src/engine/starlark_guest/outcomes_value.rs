use std::fmt;

use super::super::Outcomes;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkOutcomes {
    #[allocative(skip)]
    pub(crate) inner: Outcomes,
}

impl StarlarkOutcomes {
    pub fn new(inner: Outcomes) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &Outcomes {
        &self.inner
    }
}

impl fmt::Display for StarlarkOutcomes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Outcomes({:?})", self.inner.entries_ordered())
    }
}

starlark_simple_value!(StarlarkOutcomes);

starlark::methods_static!(
    OUTCOMES_METHODS = |builder| {
        starlark_outcomes_methods(builder);
    }
);

#[starlark_module]
fn starlark_outcomes_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Find the chance of exactly one named result.
    ///
    /// `.pmf("Partial")` counts only partial successes, not full hits as well.
    /// The result is a probability from 0 to 1. `pmf` is the mathematical name
    /// for the chance of one exact result.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
    /// move = bucket(2d6, results)
    /// output("Exactly a partial success", move.pmf("Partial"))
    /// ```
    ///
    /// # Arguments
    /// * `label`: A label on your scale, in quotes. Spelling and capital letters must match. An unknown label is an error; a known but impossible result has chance 0.
    fn pmf(this: &StarlarkOutcomes, label: &str) -> anyhow::Result<f64> {
        this.inner.pmf(label)
    }

    /// Find the chance of this result or any higher step on your ladder.
    ///
    /// If the scale runs from miss to partial to hit, “at least partial” includes
    /// both partial and hit. Higher means added later with `.step(...)`, not
    /// alphabetical order. Put your labels from worst to best when building the scale.
    /// The result is a probability from 0 to 1.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
    /// move = bucket(2d6, results)
    /// output("Partial success or better", move.p_at_least("Partial"))
    /// ```
    ///
    /// # Arguments
    /// * `label`: Lowest step to include, spelled exactly as on your scale. An unknown label is an error.
    fn p_at_least(this: &StarlarkOutcomes, label: &str) -> anyhow::Result<f64> {
        this.inner.p_at_least(label)
    }

    /// Find the chance of this result or any lower step on your ladder.
    ///
    /// On a miss, partial, hit scale, “at most partial” includes miss and partial.
    /// Lower means added earlier with `.step(...)`. The chosen label itself is included.
    /// The result is a probability from 0 to 1.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
    /// move = bucket(2d6, results)
    /// output("Partial success or worse", move.p_at_most("Partial"))
    /// ```
    ///
    /// # Arguments
    /// * `label`: Highest step to include, spelled exactly as on your scale. An unknown label is an error.
    fn p_at_most(this: &StarlarkOutcomes, label: &str) -> anyhow::Result<f64> {
        this.inner.p_at_most(label)
    }
}

#[starlark_value(type = "Outcomes")]
impl<'v> StarlarkValue<'v> for StarlarkOutcomes {
    fn get_methods() -> Option<&'static Methods> {
        Some(OUTCOMES_METHODS.methods())
    }
}
