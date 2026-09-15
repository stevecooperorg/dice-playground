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
    /// Add the next named result to your ladder, working from worst to best.
    ///
    /// Each call returns a new `Scale`; it does not change the old one.
    /// Save the result with `results = results.step(...)`, or put several `.step(...)`
    /// calls one after another. Give each label a number range when using `bucket`.
    /// You can leave ranges out for `classify`, or supply them separately to `bucket`.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6))
    /// results = results.step("Partial", through(7, 9))
    /// results = results.step("Hit", at_least(10))
    /// output("Move result", bucket(2d6, results))
    /// ```
    ///
    /// For overlapping ranges, `early=True` means “check this step before ordinary steps”.
    /// `True` is Starlark's word for yes. Early steps are checked in the order added,
    /// then ordinary steps in their order; the first match wins. This does not change
    /// the ladder order used by `.p_at_least(...)` and `.p_at_most(...)`.
    ///
    /// # Arguments
    /// * `label`: A unique text label in quotes, such as `"Hit"`.
    /// * `band`: Optional number range, such as `at_most(6)`, `through(7, 9)`, or `at_least(10)`. A step without a range does not act as a catch-all for `bucket`.
    /// * `early`: Optional named argument, `early=True` or `early=False`. Defaults to `False`.
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
