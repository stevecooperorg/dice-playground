use std::fmt::{self, Display};

use super::super::DieRoll;
use super::bucket_args::outcomes_from_bucket_args;
use super::dice_pool_value::StarlarkDicePool;
use super::face_spec::face_spec_from_value;
use super::outcomes_value::StarlarkOutcomes;
use super::scale_value::StarlarkScale;
use allocative::Allocative;
use anyhow::{anyhow, Context};
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{Heap, NoSerialize, StarlarkValue, Value, ValueError, ValueLike};

/// Exact chances for each numeric result of a roll or total (see function reference).
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkDieRoll {
    #[allocative(skip)]
    pub(crate) inner: DieRoll,
}

impl StarlarkDieRoll {
    pub fn new(inner: DieRoll) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &DieRoll {
        &self.inner
    }
}

impl Display for StarlarkDieRoll {
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
            "DieRoll(support={min}..{max}, mean={:.4})",
            self.inner.mean()
        )
    }
}

starlark_simple_value!(StarlarkDieRoll);

starlark::methods_static!(
    DIE_ROLL_METHODS = |builder| {
        starlark_die_roll_methods(builder);
    }
);

#[starlark_module]
fn starlark_die_roll_methods(builder: &mut starlark::environment::MethodsBuilder) {
    /// Find the chance of rolling exactly one number.
    ///
    /// For two d6, a total of 7 has a chance of 6 out of 36, about 16.7%.
    /// The returned probability is a number from 0 to 1, not a percentage from 0 to 100.
    /// `pmf` is short for “probability mass function”; think “chance of this exact result”.
    /// Use `.p_ge(7)` instead if you mean “7 or higher”.
    ///
    /// ```dice
    /// roll = dice_pool(2, 6).sum()
    /// output("Exactly seven", roll.pmf(7))
    /// ```
    ///
    /// # Arguments
    /// * `value`: The whole-number result you want. An impossible result has chance 0.
    fn pmf(this: &StarlarkDieRoll, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.pmf(i64::from(value)))
    }

    /// Find the chance of this total or anything lower.
    ///
    /// Use this for roll-under rules, where low numbers are good.
    /// `.cdf(8)` includes 8 itself. It returns a probability from 0 to 1.
    /// `cdf` means “cumulative distribution function”: add up the chances from the bottom.
    ///
    /// ```dice
    /// roll = dice_pool(2, 6).sum()
    /// output("Eight or less", roll.cdf(8))
    /// ```
    ///
    /// # Arguments
    /// * `value`: Highest total to include.
    fn cdf(this: &StarlarkDieRoll, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.cdf(i64::from(value)))
    }

    /// Find the chance of meeting or beating a target number.
    ///
    /// Use this for “need 15 or more” checks. The letters `ge` mean “greater than
    /// or equal to”, so the target itself counts. The result is a probability from 0 to 1.
    /// This checks the finished total, not each separate die.
    ///
    /// ```dice
    /// roll = 2d10 + 3
    /// output("Meet a target of 15", roll.p_ge(15))
    /// ```
    ///
    /// # Arguments
    /// * `value`: Lowest successful total.
    fn p_ge(this: &StarlarkDieRoll, value: i32) -> anyhow::Result<f64> {
        Ok(this.inner.p_ge(i64::from(value)))
    }

    /// Find the average total you would get over many rolls.
    ///
    /// A d6 has a mean of 3.5 even though no face shows 3.5. This is an average,
    /// not the most likely result and not a probability. The result is a decimal number.
    /// The report for a full roll already includes its mean; use this method when
    /// you need the number in a calculation.
    ///
    /// ```dice
    /// roll = d(6)
    /// average = roll.mean()  # 3.5; a number you can use in later calculations.
    /// output("The roll, with its mean in the report", roll)
    /// ```
    fn mean(this: &StarlarkDieRoll) -> anyhow::Result<f64> {
        Ok(this.inner.mean())
    }

    /// Count how many different numeric results are possible.
    ///
    /// Two d6 have 11 possible totals, 2 through 12, even though there are 36
    /// ways the dice can land. The returned whole number counts totals, not combinations.
    /// “Support” is the mathematical name for the set of possible results.
    ///
    /// ```dice
    /// roll = dice_pool(2, 6).sum()
    /// possible_totals = roll.support_size()  # 11, counting totals 2 through 12.
    /// output("Two dice: eleven possible totals", roll)
    /// ```
    fn support_size(this: &StarlarkDieRoll) -> anyhow::Result<i32> {
        i32::try_from(this.inner.support_size()).context("support_size fits in i32")
    }

    /// Set a floor and a ceiling for the final result.
    ///
    /// Anything below the floor becomes the floor; anything above the ceiling
    /// becomes the ceiling. Those chances are kept, not thrown away.
    /// The returned `DieRoll` models a rule such as “damage is at least 1, at most 6”.
    ///
    /// ```dice
    /// damage = d(6) + 2
    /// output("Damage capped at six", damage.clamp(1, 6))
    /// ```
    ///
    /// # Arguments
    /// * `min`: Lowest result allowed.
    /// * `max`: Highest result allowed; must be at least `min`.
    fn clamp(this: &StarlarkDieRoll, min: i32, max: i32) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(
            this.inner.clamp(i64::from(min), i64::from(max))?,
        ))
    }

    /// Keep only matching results and recalculate their chances to add up to 100%.
    ///
    /// On a d6, keeping 5 and 6 makes each of them 50% likely. This describes
    /// a roll restricted to those results, not the chance of rolling them on an ordinary d6.
    /// Use `.p_ge(5)` for that probability. The returned `DieRoll` leaves the original unchanged.
    /// Keeping no possible results is an error.
    ///
    /// ```dice
    /// output("Only fives and sixes", d(6).keep([5, 6]))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: One number, a non-empty list of numbers, or a range such as `at_least(5)`. After `.sum()`, this matches whole totals, not the individual dice.
    fn keep(this: &StarlarkDieRoll, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(this.inner.keep_faces_spec(parsed)?))
    }

    /// Remove matching results and share all the probability among those left.
    ///
    /// Removing 1 from a fair d6 leaves 2–6, each with a 20% chance.
    /// This is different from `.ignore(1)`, which keeps the chance of rolling 1
    /// but makes that result worth 0. The result is a new `DieRoll`;
    /// removing every possible result is an error.
    ///
    /// ```dice
    /// output("A die without ones", d(6).remove(1))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Results to remove: one number, a non-empty list, or a range such as `at_most(2)`. For a summed roll, these are totals.
    fn remove(this: &StarlarkDieRoll, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(this.inner.remove_faces_spec(parsed)?))
    }

    /// Change matching results to a new number without changing how often they happen.
    ///
    /// For a “sixes count double” rule, turn 6 into 12. Other results stay as they are.
    /// If several results become the same number, their chances add together.
    /// The result is a new `DieRoll`; the original is unchanged.
    ///
    /// ```dice
    /// output("Sixes count as twelve", d(6).convert(6, 12))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Results to change: one number, a non-empty list, or a range such as `at_least(5)`. On a summed roll this matches totals.
    /// * `to`: New whole-number value for every match.
    fn convert(
        this: &StarlarkDieRoll,
        spec: Value<'_>,
        to: i32,
    ) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(
            this.inner.convert_faces_spec(parsed, i64::from(to))?,
        ))
    }

    /// Make matching results worth zero, while keeping their chance of happening.
    ///
    /// On a d6, ignoring 1–4 gives a 4-in-6 chance of 0, plus the usual chances
    /// of 5 and 6. Unlike `.remove(...)`, it does not rule out those rolls.
    /// It returns a new `DieRoll` and is shorthand for `.convert(spec, 0)`.
    ///
    /// ```dice
    /// output("Only high faces add points", d(6).ignore(through(1, 4)))
    /// ```
    ///
    /// # Arguments
    /// * `spec`: Results to turn into 0: one number, a non-empty list, or a range. On a summed roll this matches whole totals.
    fn ignore(this: &StarlarkDieRoll, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(this.inner.ignore_faces_spec(parsed)?))
    }

    /// Turn this roll's totals into named results using your scale.
    ///
    /// This is the same as `bucket(roll, results)`. It returns `Outcomes`,
    /// which you can show as a table or ask about with `.p_at_least("Hit")`.
    /// Each possible total must be covered by a band.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Hit", at_least(7))
    /// roll = dice_pool(2, 6).sum()
    /// output("Move result", roll.bucket(results))
    /// ```
    ///
    /// # Arguments
    /// * `scale`: Your ordered labels and their number ranges.
    /// * `bands`: Optional replacement cut points or ranges, following the same rules as the `bucket` function. For two labels, `[6]` means 6 or less, then 7 or more.
    fn bucket(
        this: &StarlarkDieRoll,
        scale: &StarlarkScale,
        #[starlark(args)] bands: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<StarlarkOutcomes> {
        Ok(StarlarkOutcomes::new(outcomes_from_bucket_args(
            this.inner(),
            scale.inner().clone(),
            bands.items,
        )?))
    }
}

#[starlark_value(type = "DieRoll")]
impl<'v> StarlarkValue<'v> for StarlarkDieRoll {
    fn get_methods() -> Option<&'static Methods> {
        Some(DIE_ROLL_METHODS.methods())
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        if let Some(other) = rhs.downcast_ref::<Self>() {
            let merged = match self.inner.convolve(&other.inner) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDieRoll::new(merged))));
        }
        if let Some(pool) = rhs.downcast_ref::<StarlarkDicePool>() {
            let merged = match pool.inner().push_die(self.inner.clone()) {
                Ok(m) => m,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDicePool::new(merged))));
        }
        if let Some(delta) = rhs.unpack_i32() {
            let shifted = match self.inner.shift(i64::from(delta)) {
                Ok(s) => s,
                Err(e) => return Some(Err(e.into())),
            };
            return Some(Ok(heap.alloc(StarlarkDieRoll::new(shifted))));
        }
        None
    }

    fn sub(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        if let Some(other) = rhs.downcast_ref::<Self>() {
            let merged = self.inner.difference(&other.inner)?;
            return Ok(heap.alloc(StarlarkDieRoll::new(merged)));
        }
        if let Some(pool) = rhs.downcast_ref::<StarlarkDicePool>() {
            let merged = self.inner.difference(&pool.inner().sum()?)?;
            return Ok(heap.alloc(StarlarkDieRoll::new(merged)));
        }
        if let Some(delta) = rhs.unpack_i32() {
            let shifted = self.inner.shift(-i64::from(delta))?;
            return Ok(heap.alloc(StarlarkDieRoll::new(shifted)));
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
    dist: &StarlarkDieRoll,
    rhs: Value<'v>,
    heap: Heap<'v>,
) -> Option<starlark::Result<Value<'v>>> {
    let factor = rhs.unpack_i32()?;
    if factor <= 0 {
        return Some(Err(
            anyhow!("scale factor must be positive, got {factor}").into()
        ));
    }
    let scaled = match dist.inner.scale_outcomes(i64::from(factor)) {
        Ok(d) => d,
        Err(e) => return Some(Err(e.into())),
    };
    Some(Ok(heap.alloc(StarlarkDieRoll::new(scaled))))
}

fn dist_rmul<'v>(
    dist: &StarlarkDieRoll,
    lhs: Value<'v>,
    heap: Heap<'v>,
) -> Option<starlark::Result<Value<'v>>> {
    dist_mul(dist, lhs, heap)
}

fn dist_floor_div<'v>(
    dist: &StarlarkDieRoll,
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
    Ok(heap.alloc(StarlarkDieRoll::new(out)))
}
