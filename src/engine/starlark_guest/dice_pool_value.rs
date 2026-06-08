use std::fmt::{self, Display};

use super::super::DicePool;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueLike};

use super::die_roll_value::StarlarkDieRoll;

/// Several dice still treated separately until you call `.sum()` (see function reference).
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkDicePool {
    #[allocative(skip)]
    pub(crate) inner: DicePool,
}

impl StarlarkDicePool {
    pub fn new(inner: DicePool) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &DicePool {
        &self.inner
    }
}

impl Display for StarlarkDicePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DicePool({} dice)", self.inner.dice().len())
    }
}

starlark_simple_value!(StarlarkDicePool);

starlark::methods_static!(
    DICE_POOL_METHODS = |builder| {
        dice_pool_methods(builder);
    }
);

#[starlark_module]
fn dice_pool_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Add every die in the pool into one total—turns `dice_pool(4, 6)` into the same idea as `4d6`.
    fn sum(this: &StarlarkDicePool) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(this.inner.sum()?))
    }
}

#[starlark_value(type = "DicePool")]
impl<'v> StarlarkValue<'v> for StarlarkDicePool {
    fn get_methods() -> Option<&'static starlark::environment::Methods> {
        Some(DICE_POOL_METHODS.methods())
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        let summed = match self.inner.sum() {
            Ok(d) => d,
            Err(e) => return Some(Err(e.into())),
        };
        StarlarkDieRoll::new(summed).add(rhs, heap)
    }

    fn sub(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let left = self.inner.sum()?;
        if let Some(other) = rhs.downcast_ref::<StarlarkDicePool>() {
            let merged = left.difference(&other.inner().sum()?)?;
            return Ok(heap.alloc(StarlarkDieRoll::new(merged)));
        }
        StarlarkDieRoll::new(left).sub(rhs, heap)
    }

    fn mul(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        let summed = match self.inner.sum() {
            Ok(d) => d,
            Err(e) => return Some(Err(e.into())),
        };
        StarlarkDieRoll::new(summed).mul(rhs, heap)
    }

    fn rmul(&self, lhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        self.mul(lhs, heap)
    }

    fn floor_div(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let summed = self.inner.sum()?;
        StarlarkDieRoll::new(summed).floor_div(rhs, heap)
    }
}
