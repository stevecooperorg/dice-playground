use std::fmt::{self, Display};

use super::super::DicePool;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueLike};

use super::bucket_args::outcomes_from_bucket_args;
use super::die_roll_value::StarlarkDieRoll;
use super::face_spec::{face_spec_from_value, optional_face_spec_from_values};
use super::outcomes_value::StarlarkOutcomes;
use super::scale_value::StarlarkScale;
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
    /// Add all the dice in this pool to get one total.
    ///
    /// The returned `DieRoll` gives the chance of each total; for three d6, these run from 3 to 18.
    /// Use it when you no longer need to examine individual faces.
    /// The original pool is unchanged, so you can still ask it other questions.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6)
    /// output("Three dice added together", pool.sum())
    /// ```
    fn sum(this: &StarlarkDicePool) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(this.inner.sum()?))
    }

    /// Add the dice, then group the total into named results.
    ///
    /// This is shorthand for `pool.sum().bucket(results)`. It returns `Outcomes`,
    /// not a count of how many individual dice match a band. Use `.count(...)`
    /// when each die earns a success separately.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Hit", at_least(7))
    /// pool = dice_pool(2, 6)
    /// output("Result of the total", pool.bucket(results))
    /// ```
    ///
    /// # Arguments
    /// * `scale`: Your ordered labels and their number ranges. Every possible total must be covered.
    /// * `spec`: Optional replacement cut points or ranges, as for the `bucket` function. For two labels, `[6]` means 6 or less, then 7 or more.
    fn bucket(
        this: &StarlarkDicePool,
        scale: &StarlarkScale,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<StarlarkOutcomes> {
        Ok(StarlarkOutcomes::new(outcomes_from_bucket_args(
            &this.inner.sum()?,
            scale.inner().clone(),
            spec.items,
        )?))
    }

    /// Restrict every die to matching faces, then recalculate each die's chances.
    ///
    /// Keeping 5 and 6 on three d6 makes every die show either 5 or 6 with equal chances.
    /// It does not roll ordinary dice and discard the low ones: the pool still has three dice.
    /// The returned `DicePool` is new; the original pool is unchanged.
    /// It is an error if any die has no possible face left.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6).keep([5, 6])
    /// output("Only fives and sixes: total 15–18", pool.sum())
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Faces to allow on each die: one number, a non-empty list, or a range such as `at_least(5)`.
    fn keep(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.keep_faces_spec(parsed)?))
    }

    /// Remove matching faces from each die, not dice from the pool.
    ///
    /// Removing 1 leaves faces 2–6 on each d6, with equal chances.
    /// The result is a new `DicePool` with the same number of dice.
    /// Use `.ignore(1)` instead if ones should still happen but contribute 0.
    /// It is an error if any die loses all its possible faces.
    ///
    /// ```dice
    /// output("Three dice with no ones", dice_pool(3, 6).remove(1).sum())
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Faces to exclude on each die: one number, a non-empty list, or a range such as `at_most(2)`.
    fn remove(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.remove_faces_spec(parsed)?))
    }

    /// Give matching faces a different value on every die.
    ///
    /// For “sixes count double”, change each 6 to 12 before adding the dice.
    /// The faces keep their original chances. This returns a new `DicePool`
    /// and leaves the original unchanged.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6).convert(6, 12)
    /// output("Total with double sixes", pool.sum())
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Faces to change: one number, a non-empty list, or a range such as `at_least(5)`.
    /// * `to`: New whole-number value for every matching face.
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

    /// Make matching faces contribute zero when you add the dice.
    ///
    /// Low faces still happen; they just score no points. This differs from
    /// `.remove(...)`, which makes those faces impossible. The result is a new
    /// `DicePool` with the same number of dice, equivalent to `.convert(spec, 0)`.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6).ignore(through(1, 4))
    /// output("Add only fives and sixes", pool.sum())
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Faces worth zero: one number, a non-empty list, or a range.
    fn ignore(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDicePool> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDicePool::new(this.inner.ignore_faces_spec(parsed)?))
    }

    /// Count one success for each die that shows a matching face.
    ///
    /// Three dice can give 0, 1, 2, or 3 successes. The returned `DieRoll`
    /// shows how likely each count is. It counts dice, not their face values:
    /// a 5 and a 6 give two successes, not eleven points.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6)
    /// output("Successes on 5+", pool.count(at_least(5)))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Faces that succeed: one number, a non-empty list such as `[5, 6]`, or a range such as `at_least(5)`.
    fn count(this: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(this.inner.count_faces(parsed)?))
    }

    /// Choose the highest die, second-highest die, or another position from the top.
    ///
    /// `1` means highest; `2` means second-highest. Ties count separately:
    /// in 6, 6, 2 the second-highest is 6. The result is a `DieRoll` of that one value.
    /// This is the same as `order_stat(pool, k)`.
    ///
    /// ```dice
    /// output("Best of three dice", dice_pool(3, 6).order_stat(1))
    /// ```
    ///
    /// # Arguments
    /// * `k`: Position counting from 1 at the top, no greater than the number of dice.
    fn order_stat(this: &StarlarkDicePool, k: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k =
            usize::try_from(k).map_err(|_| anyhow::anyhow!("order_stat: k must be positive"))?;
        Ok(StarlarkDieRoll::new(this.inner.order_stat(k)?))
    }

    /// Add the middle dice after setting aside low and high results.
    ///
    /// With three dice, keeping one chooses the middle die. With five dice,
    /// keeping three adds the middle three. If an odd number of dice must be left out,
    /// one more is removed from the high end. Returns a `DieRoll`,
    /// just like `middle_of(pool, keep)`.
    ///
    /// ```dice
    /// output("Middle die", dice_pool(3, 6).middle_of(1))
    /// ```
    ///
    /// # Arguments
    /// * `keep`: Number of middle dice to add, from 1 to the number of dice.
    fn middle_of(this: &StarlarkDicePool, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(keep)
            .map_err(|_| anyhow::anyhow!("middle_of: keep must be positive"))?;
        Ok(StarlarkDieRoll::new(this.inner.middle_of(k)?))
    }

    /// Find the chance that at least one die shows a matching face.
    ///
    /// Use this for “any six is a success”. It returns a probability from 0 to 1;
    /// 0.5 means 50%. Two or more matching dice still count as success, not as extra probability.
    ///
    /// ```dice
    /// output("At least one six", dice_pool(3, 6).p_any(6))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Optional face match: one number, a non-empty list, or a range such as `at_least(5)`. If omitted, this only checks whether the pool has any dice. Pools created by `dice_pool` are non-empty, so the answer is 1. It does not test for non-zero faces.
    fn p_any(
        this: &StarlarkDicePool,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<f64> {
        let parsed = optional_face_spec_from_values(&spec.items)?;
        this.inner.p_any(parsed)
    }

    /// Find the chance that none of the dice show a matching face.
    ///
    /// Use this for “avoid rolling any ones”. It returns a probability from 0 to 1
    /// and is the opposite of `.p_any(...)`: their answers add up to 1.
    ///
    /// ```dice
    /// output("No ones", dice_pool(3, 6).p_none(1))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Optional face match: one number, a non-empty list, or a range. If omitted, this only checks whether the pool is empty. Pools created by `dice_pool` are non-empty, so the answer is 0.
    fn p_none(
        this: &StarlarkDicePool,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<f64> {
        let parsed = optional_face_spec_from_values(&spec.items)?;
        this.inner.p_none(parsed)
    }

    /// Find the chance of getting enough matching dice.
    ///
    /// For “at least two successes, with each 5 or 6 succeeding”, ask for two matches
    /// to `at_least(5)`. The result is a probability from 0 to 1, not a success count.
    /// Use `.count(...)` instead to see every possible count and its chance.
    ///
    /// ```dice
    /// output("Two or more successes", dice_pool(3, 6).p_at_least(2, at_least(5)))
    /// ```
    ///
    /// # Arguments
    /// * `k`: Minimum number of matching dice, 0 or more. Asking for 0 always gives 1; asking for more dice than the pool contains gives 0.
    /// * `spec`: Optional face match: one number, a non-empty list, or a range. If omitted, this only checks whether the pool contains at least `k` dice, regardless of faces.
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
