use std::fmt;

use super::super::{IntBand, Scale};
use super::int_band_value::StarlarkIntBand;
use allocative::Allocative;
use anyhow::Context;
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{NoSerialize, StarlarkValue, Value, ValueLike};

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
        (Some(lo), Some(hi)) if lo == hi => format!("{lo}..{hi}"),
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
        if labels.is_empty() {
            return write!(f, "scale()");
        }
        write!(f, "scale()")?;
        let early_flags = self.inner.early_flags();
        for (i, (label, band)) in labels.iter().zip(bands.iter()).enumerate() {
            if band.is_unbounded() {
                write!(f, ".step({label:?}")?;
            } else {
                write!(f, ".step({label:?}, {}", format_band(*band))?;
            }
            if early_flags.get(i).copied().unwrap_or(false) {
                write!(f, ", early=True")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

starlark_simple_value!(StarlarkScale);

starlark::methods_static!(
    SCALE_METHODS = |builder| {
        starlark_scale_methods(builder);
    }
);

#[starlark_module]
fn starlark_scale_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Append one outcome label (low → high). With no band, the step is for `classify` only.
    ///
    /// With a band (`IntBand` or desugared `..6`, `7..9`, `10..`), the step buckets numeric totals.
    /// Bands may overlap: **early** steps (see `early=True`) are checked first, then other steps, each in declaration order.
    /// Declaration order still defines ladder rank for `p_at_least` / `p_at_most`.
    fn step(
        this: &StarlarkScale,
        label: &str,
        #[starlark(args)] band: UnpackTuple<Value<'_>>,
        #[starlark(default = false)] early: bool,
    ) -> anyhow::Result<StarlarkScale> {
        let band = match band.items.len() {
            0 => IntBand::unbounded(),
            1 => band.items[0]
                .downcast_ref::<StarlarkIntBand>()
                .with_context(|| {
                    format!("scale.step band: expected IntBand, got {}", band.items[0])
                })?
                .inner(),
            n => anyhow::bail!("scale.step expects at most one band, got {n} extra argument(s)"),
        };
        Ok(StarlarkScale::new(this.inner.clone().with_step(
            label.to_owned(),
            band,
            early,
        )?))
    }
}

#[starlark_value(type = "Scale")]
impl<'v> StarlarkValue<'v> for StarlarkScale {
    fn get_methods() -> Option<&'static Methods> {
        Some(SCALE_METHODS.methods())
    }
}
