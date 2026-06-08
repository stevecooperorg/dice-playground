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
    /// Chance of landing on **exactly** this named outcome (one band on the ladder).
    fn pmf(this: &StarlarkOutcomes, label: &str) -> anyhow::Result<f64> {
        this.inner.pmf(label)
    }

    /// Chance of this outcome **or any better one** on the scale—e.g. “partial success or full success”.
    fn p_at_least(this: &StarlarkOutcomes, label: &str) -> anyhow::Result<f64> {
        this.inner.p_at_least(label)
    }

    /// Chance of this outcome **or any worse one**—e.g. “failure or partial failure”.
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
