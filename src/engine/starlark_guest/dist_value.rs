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

/// A discrete distribution exposed to Starlark (`+` / `-` on independent rolls).
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
    /// Probability mass at an exact outcome: P(X = value).
    ///
    /// # Arguments
    /// * `value`: Outcome to query.
    fn pmf(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.pmf(i64::from(value)))
    }

    /// Cumulative distribution: P(X <= value).
    ///
    /// # Arguments
    /// * `value`: Upper bound (inclusive).
    fn cdf(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.cdf(i64::from(value)))
    }

    /// Probability of meeting or beating a target: P(X >= value).
    ///
    /// # Arguments
    /// * `value`: Target outcome (inclusive).
    fn p_ge(this: &StarlarkDist, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.p_ge(i64::from(value)))
    }

    /// Expected value (mean) of the distribution.
    fn mean(this: &StarlarkDist) -> anyhow::Result<f64> {
        Ok(this.inner.mean())
    }

    /// Number of outcomes with non-zero probability.
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
