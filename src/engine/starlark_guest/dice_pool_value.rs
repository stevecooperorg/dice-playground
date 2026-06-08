use std::fmt::{self, Display};

use super::super::DicePool;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueLike};

use super::die_roll_value::StarlarkDieRoll;
use super::face_spec::{face_spec_from_value, optional_face_spec_from_values};
use starlark::values::tuple::UnpackTuple;

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

    /// Keep only matching faces on every die; drop others and renormalize each die.
    fn keep(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.keep_faces_spec(parsed)?))
    }

    /// Drop matching faces on every die; renormalize each die.
    fn remove(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.remove_faces_spec(parsed)?))
    }

    /// Remap matching faces to `to` on every die.
    fn convert(
        this: &StarlarkDicePool,
        spec: Value<'_>,
        to: i32,
    ) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(
            this.inner.convert_faces_spec(parsed, i64::from(to))?,
        ))
    }

    /// Remap matching faces to 0 on every die.
    fn ignore(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.ignore_faces_spec(parsed)?))
    }

    /// Distribution of how many dice match `spec`.
    fn count(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(this.inner.count_faces(parsed)?))
    }

    fn order_stat(this: &StarlarkDicePool, k: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k =
            usize::try_from(k).map_err(|_| anyhow::anyhow!("order_stat: k must be positive"))?;
        Ok(StarlarkDieRoll::new(this.inner.order_stat(k)?))
    }

    fn middle_of(this: &StarlarkDicePool, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(keep)
            .map_err(|_| anyhow::anyhow!("middle_of: keep must be positive"))?;
        Ok(StarlarkDieRoll::new(this.inner.middle_of(k)?))
    }

    /// P(at least one die matches the optional face spec).
    fn p_any(
        this: &StarlarkDicePool,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<f64> {
        let parsed = optional_face_spec_from_values(&spec.items)?;
        this.inner.p_any(parsed)
    }

    /// P(no die matches the optional face spec).
    fn p_none(
        this: &StarlarkDicePool,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<f64> {
        let parsed = optional_face_spec_from_values(&spec.items)?;
        this.inner.p_none(parsed)
    }

    /// P(at least `k` dice match the optional face spec).
    fn p_at_least(
        this: &StarlarkDicePool,
        k: i32,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<f64> {
        let k = usize::try_from(k)
            .map_err(|_| anyhow::anyhow!("p_at_least: k must be non-negative"))?;
        let parsed = optional_face_spec_from_values(&spec.items)?;
        this.inner.p_at_least(k, parsed)
    }
}

#[starlark_value(type = "DicePool")]
impl<'v> StarlarkValue<'v> for StarlarkDicePool {
    fn get_methods() -> Option<&'static starlark::environment::Methods> {
        Some(DICE_POOL_METHODS.methods())
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        if let Some(other) = rhs.downcast_ref::<StarlarkDicePool>() {
            let merged = match self.inner.join(other.inner()) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDicePool::new(merged))));
        }
        if let Some(roll) = rhs.downcast_ref::<StarlarkDieRoll>() {
            let merged = match self.inner.push_die(roll.inner().clone()) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDicePool::new(merged))));
        }
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
