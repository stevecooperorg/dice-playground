use std::fmt;

use super::super::Scale;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkScale {
    #[allocative(skip)]
    pub(crate) inner: Scale,
}

impl StarlarkScale {
    pub fn new(inner: Scale) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &Scale {
        &self.inner
    }
}

impl fmt::Display for StarlarkScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scale({:?})", self.inner.labels())
    }
}

starlark_simple_value!(StarlarkScale);

#[starlark_value(type = "Scale")]
impl<'v> StarlarkValue<'v> for StarlarkScale {}
