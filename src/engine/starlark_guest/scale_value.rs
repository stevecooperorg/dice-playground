use std::fmt;

use super::super::ResultScale;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkResultScale {
    #[allocative(skip)]
    pub(crate) inner: ResultScale,
}

impl StarlarkResultScale {
    pub fn new(inner: ResultScale) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &ResultScale {
        &self.inner
    }
}

impl fmt::Display for StarlarkResultScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResultScale({:?})", self.inner.labels())
    }
}

starlark_simple_value!(StarlarkResultScale);

#[starlark_value(type = "ResultScale")]
impl<'v> StarlarkValue<'v> for StarlarkResultScale {}
