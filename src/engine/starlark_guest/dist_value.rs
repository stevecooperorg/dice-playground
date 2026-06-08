use std::fmt::{self, Display};

use super::pool_value::StarlarkRollPool;
use super::super::Dist;
use allocative::Allocative;
use anyhow::{anyhow, Context};
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueError, ValueLike};

/// Exact chances for each numeric result of a roll or total (see function reference).
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkDist {
    #[allocative(skip)]
    pub(crate) inner: Dist,
}

impl StarlarkDist {
    pub fn new(inner: Dist) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &Dist {
        &self.inner
    }
}

impl Display for StarlarkDist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = self
            .inner
            .min()
            .map_or_else(|| "?".to_owned(), |m| m.to_string());
        let max = self
            .inner
            .max()
            .map_or_else(|| "?".to_owned(), |m| m.to_string());
        write!(
            f,
            "Dist(support={min}..{max}, mean={:.4})",
            self.inner.mean()
        )
    }
}

starlark_simple_value!(StarlarkDist);

starlark::methods_static!(
    DIST_METHODS = |builder| {
        starlark_dist_methods(builder);
    }
);

#[starlark_module]
fn starlark_dist_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Chance of rolling **exactly** this number (one outcome, not “this or higher”).
    ///
    /// Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.
    ///
    /// # Arguments
    /// * `value`: The total you care about.
    fn pmf(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.pmf(i64::from(value)))
    }

    /// Chance the total is **this number or lower** (cumulative from the bottom).
    ///
    /// Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.
    ///
    /// # Arguments
    /// * `value`: Upper cap (inclusive).
    fn cdf(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.cdf(i64::from(value)))
    }

    /// Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.
    ///
    /// Example: `output("success", (2d10 + 3).p_ge(15))`.
    ///
    /// # Arguments
    /// * `value`: Target total (inclusive)—success if roll ≥ this.
    fn p_ge(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.p_ge(i64::from(value)))
    }

    /// Average result if you rolled this distribution many times—the **mean** on the output table.
    fn mean(this: &StarlarkDist) -> anyhow::Result<f64> {
        Ok(this.inner.mean())
    }

    /// How many different totals can occur with non-zero chance (size of the result table).
    fn support_size(this: &StarlarkDist) -> anyhow::Result<i32> {
        i32::try_from(this.inner.support_size()).context("support_size fits in i32")
    }
}

#[starlark_value(type = "Dist")]
impl<'v> StarlarkValue<'v> for StarlarkDist {
    fn get_methods() -> Option<&'static Methods> {
        Some(DIST_METHODS.methods())
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        if let Some(other) = rhs.downcast_ref::<Self>() {
            let merged = match self.inner.convolve(&other.inner) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDist::new(merged))));
        }
        if let Some(pool) = rhs.downcast_ref::<StarlarkRollPool>() {
            let summed = match pool.inner().sum() {
                Ok(d) => d,
                Err(e) => return Some(Err(e.into())),
            };
            let merged = match self.inner.convolve(&summed) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDist::new(merged))));
        }
        if let Some(delta) = rhs.unpack_i32() {
            let shifted = match self.inner.shift(i64::from(delta)) {
                Ok(s) => s,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDist::new(shifted))));
        }
        None
    }

    fn sub(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        if let Some(other) = rhs.downcast_ref::<Self>() {
            let merged = self.inner.difference(&other.inner)?;
            return Ok(heap.alloc(StarlarkDist::new(merged)));
        }
        if let Some(pool) = rhs.downcast_ref::<StarlarkRollPool>() {
            let merged = self.inner.difference(&pool.inner().sum()?)?;
            return Ok(heap.alloc(StarlarkDist::new(merged)));
        }
        if let Some(delta) = rhs.unpack_i32() {
            let shifted = self.inner.shift(-i64::from(delta))?;
            return Ok(heap.alloc(StarlarkDist::new(shifted)));
        }
        ValueError::unsupported_with(self, "-", rhs)
    }

    fn mul(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        dist_mul(self, rhs, heap)
    }

    fn rmul(&self, lhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        dist_rmul(self, lhs, heap)
    }

    fn floor_div(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        dist_floor_div(self, rhs, heap)
    }
}

fn dist_mul<'v>(
    dist: &StarlarkDist,
    rhs: Value<'v>,
    heap: Heap<'v>,
) -> Option<starlark::Result<Value<'v>>> {
    let factor = rhs.unpack_i32()?;
    if factor <= 0 {
        return Some(Err(anyhow!("scale factor must be positive, got {factor}").into()));
    }
    let scaled = match dist.inner.scale_outcomes(i64::from(factor)) {
        Ok(d) => d,
        Err(e) => return Some(Err(e.into())),
    };
    Some(Ok(heap.alloc(StarlarkDist::new(scaled))))
}

fn dist_rmul<'v>(
    dist: &StarlarkDist,
    lhs: Value<'v>,
    heap: Heap<'v>,
) -> Option<starlark::Result<Value<'v>>> {
    dist_mul(dist, lhs, heap)
}

fn dist_floor_div<'v>(
    dist: &StarlarkDist,
    rhs: Value<'v>,
    heap: Heap<'v>,
) -> starlark::Result<Value<'v>> {
    let Some(divisor) = rhs.unpack_i32() else {
        return ValueError::unsupported_with(dist, "//", rhs);
    };
    if divisor <= 0 {
        return Err(anyhow!("divisor must be positive, got {divisor}").into());
    }
    let out = dist.inner.floor_divide_outcomes(i64::from(divisor))?;
    Ok(heap.alloc(StarlarkDist::new(out)))
}
