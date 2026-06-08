use std::fmt;

use super::super::LabelDist;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkLabelDist {
    #[allocative(skip)]
    pub(crate) inner: LabelDist,
}

impl StarlarkLabelDist {
    pub fn new(inner: LabelDist) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &LabelDist {
        &self.inner
    }
}

impl fmt::Display for StarlarkLabelDist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LabelDist({:?})", self.inner.entries_ordered())
    }
}

starlark_simple_value!(StarlarkLabelDist);

starlark::methods_static!(
    LABEL_DIST_METHODS = |builder| {
        starlark_label_dist_methods(builder);
    }
);

#[starlark_module]
fn starlark_label_dist_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Probability mass at an exact label: P(X = label).
    fn pmf(this: &StarlarkLabelDist, label: &str) -> anyhow::Result<f64> {
        Ok(this.inner.pmf(label)?)
    }

    /// Probability of this label or any higher-ranked label on the scale.
    fn p_at_least(this: &StarlarkLabelDist, label: &str) -> anyhow::Result<f64> {
        Ok(this.inner.p_at_least(label)?)
    }

    /// Probability of this label or any lower-ranked label on the scale.
    fn p_at_most(this: &StarlarkLabelDist, label: &str) -> anyhow::Result<f64> {
        Ok(this.inner.p_at_most(label)?)
    }
}

#[starlark_value(type = "LabelDist")]
impl<'v> StarlarkValue<'v> for StarlarkLabelDist {
    fn get_methods() -> Option<&'static Methods> {
        Some(LABEL_DIST_METHODS.methods())
    }
}
