use std::fmt;

use super::super::{IntBand, Scale};
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

fn format_band(b: IntBand) -> String {
    match (b.min, b.max) {
        (Some(lo), Some(hi)) => format!("{lo}..{hi}"),
        (None, Some(hi)) => format!("..{hi}"),
        (Some(lo), None) => format!("{lo}.."),
        (None, None) => "..".to_owned(),
    }
}

impl fmt::Display for StarlarkScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let labels = self.inner.labels();
        let bands = self.inner.bands();
        if self.inner.has_bounded_bands() {
            write!(f, "Scale([")?;
            for (i, label) in labels.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "({label:?}, {})", format_band(bands[i]))?;
            }
            write!(f, "])")
        } else {
            write!(f, "Scale({labels:?})")
        }
    }
}

starlark_simple_value!(StarlarkScale);

#[starlark_value(type = "Scale")]
impl<'v> StarlarkValue<'v> for StarlarkScale {}
