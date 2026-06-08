use std::fmt;

use super::super::IntBand;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

#[derive(Debug, Clone, Copy, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkIntBand {
    #[allocative(skip)]
    pub(crate) inner: IntBand,
}

impl StarlarkIntBand {
    pub fn new(inner: IntBand) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> IntBand {
        self.inner
    }
}

impl fmt::Display for StarlarkIntBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.inner.min, self.inner.max) {
            (Some(lo), Some(hi)) => write!(f, "IntBand({lo}..{hi})"),
            (None, Some(hi)) => write!(f, "IntBand(..{hi})"),
            (Some(lo), None) => write!(f, "IntBand({lo}..)"),
            (None, None) => write!(f, "IntBand(..)"),
        }
    }
}

starlark_simple_value!(StarlarkIntBand);

#[starlark_value(type = "IntBand")]
impl<'v> StarlarkValue<'v> for StarlarkIntBand {}
