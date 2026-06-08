use std::fmt::{self, Display};

use super::super::RollPool;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueLike};

use super::dist_value::StarlarkDist;

/// Independent dice pool (not summed until `sum()`).
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkRollPool {
    #[allocative(skip)]
    pub(crate) inner: RollPool,
}

impl StarlarkRollPool {
    pub fn new(inner: RollPool) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &RollPool {
        &self.inner
    }
}

impl Display for StarlarkRollPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RollPool({} dice)", self.inner.dice().len())
    }
}

starlark_simple_value!(StarlarkRollPool);

starlark::methods_static!(
    ROLL_POOL_METHODS = |builder| {
        roll_pool_methods(builder);
    }
);

#[starlark_module]
fn roll_pool_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Sum all dice in the pool into one outcome distribution.
    fn sum(this: &StarlarkRollPool) -> anyhow::Result<StarlarkDist> {
        Ok(StarlarkDist::new(this.inner.sum()?))
    }
}

#[starlark_value(type = "RollPool")]
impl<'v> StarlarkValue<'v> for StarlarkRollPool {
    fn get_methods() -> Option<&'static starlark::environment::Methods> {
        Some(ROLL_POOL_METHODS.methods())
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        let summed = match self.inner.sum() {
            Ok(d) => d,
            Err(e) => return Some(Err(e.into())),
        };
        StarlarkDist::new(summed).add(rhs, heap)
    }

    fn sub(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let left = self.inner.sum()?;
        if let Some(other) = rhs.downcast_ref::<StarlarkRollPool>() {
            let merged = left.difference(&other.inner().sum()?)?;
            return Ok(heap.alloc(StarlarkDist::new(merged)));
        }
        StarlarkDist::new(left).sub(rhs, heap)
    }

    fn mul(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        let summed = match self.inner.sum() {
            Ok(d) => d,
            Err(e) => return Some(Err(e.into())),
        };
        StarlarkDist::new(summed).mul(rhs, heap)
    }

    fn rmul(&self, lhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        self.mul(lhs, heap)
    }

    fn floor_div(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let summed = self.inner.sum()?;
        StarlarkDist::new(summed).floor_div(rhs, heap)
    }
}
