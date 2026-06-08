//! Exact chances for numeric roll totals (`1d6`, `2d6`, `4d6dl1`, modifiers).
//!
//! A [`DieRoll`] stores a **probability mass function** (PMF): each possible integer
//! outcome maps to a probability, and the values sum to 1. Independent rolls add via
//! [**convolution**](https://en.wikipedia.org/wiki/Probability_distribution#Algebra_of_random_variables)
//! ([`convolve`]); flat bonuses shift every outcome ([`shift`]).

use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::int_band::IntBand;

/// Exact distribution over integer roll totals (supports modifiers and negative outcomes).
///
/// # Example
///
/// ```
/// use dice_playground::engine::DieRoll;
/// let two_d6 = DieRoll::pool_sum(2, 6).unwrap();
/// assert!((two_d6.pmf(7) - 6.0 / 36.0).abs() < 1e-12);
/// assert!((two_d6.mean() - 7.0).abs() < 1e-9);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DieRoll {
    pub(crate) mass: BTreeMap<i64, f64>,
}

impl DieRoll {
    /// Empty distribution (use [`DieRoll::die`] or [`DieRoll::constant`] to build rolls).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert_eq!(DieRoll::new().support_size(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            mass: BTreeMap::new(),
        }
    }

    /// Fair die showing `1..=sides` with equal probability (tabletop `1d{sides}`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d20 = DieRoll::die(20).unwrap();
    /// assert!((d20.pmf(20) - 0.05).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[allow(clippy::self_named_constructors)]
    pub fn die(sides: i64) -> Result<Self> {
        if sides < 1 {
            bail!("die sides must be >= 1, got {sides}");
        }
        let p = 1.0 / sides as f64;
        let mut mass = BTreeMap::new();
        for face in 1..=sides {
            mass.insert(face, p);
        }
        Ok(Self { mass })
    }

    /// Fair die from an explicit face list (repeated faces increase weight).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let fudge = DieRoll::from_faces(&[-1, 0, 1]).unwrap();
    /// assert!((fudge.pmf(0) - 1.0 / 3.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_faces(faces: &[i64]) -> Result<Self> {
        if faces.is_empty() {
            bail!("from_faces: need at least one face");
        }
        let mut mass = BTreeMap::new();
        let p = 1.0 / faces.len() as f64;
        for &f in faces {
            *mass.entry(f).or_insert(0.0) += p;
        }
        Ok(Self { mass })
    }

    /// Degenerate roll that always shows `value` (modifiers, fixed damage).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let plus_five = DieRoll::constant(5);
    /// assert_eq!(plus_five.pmf(5), 1.0);
    /// ```
    pub fn constant(value: i64) -> Self {
        let mut mass = BTreeMap::new();
        mass.insert(value, 1.0);
        Self { mass }
    }

    /// Number of outcomes with positive probability.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert_eq!(DieRoll::die(6).unwrap().support_size(), 6);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn support_size(&self) -> usize {
        self.mass.len()
    }

    /// Smallest outcome with positive probability, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert_eq!(DieRoll::die(6).unwrap().min(), Some(1));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn min(&self) -> Option<i64> {
        self.mass.keys().next().copied()
    }

    /// Largest outcome with positive probability, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert_eq!(DieRoll::die(6).unwrap().max(), Some(6));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn max(&self) -> Option<i64> {
        self.mass.keys().next_back().copied()
    }

    /// Probability of exactly `value` (0 if impossible).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d6 = DieRoll::die(6).unwrap();
    /// assert!((d6.pmf(4) - 1.0 / 6.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pmf(&self, value: i64) -> f64 {
        self.mass.get(&value).copied().unwrap_or(0.0)
    }

    /// Cumulative probability **P(X ≤ value)**.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d6 = DieRoll::die(6).unwrap();
    /// assert!((d6.cdf(3) - 0.5).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn cdf(&self, value: i64) -> f64 {
        self.mass
            .iter()
            .filter(|(k, _)| **k <= value)
            .map(|(_, p)| p)
            .sum()
    }

    /// Probability of rolling **at least** `value` (common for DC checks).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d20 = DieRoll::die(20).unwrap();
    /// assert!((d20.p_ge(15) - 6.0 / 20.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_ge(&self, value: i64) -> f64 {
        self.mass
            .iter()
            .filter(|(k, _)| **k >= value)
            .map(|(_, p)| p)
            .sum()
    }

    /// Expected value (average total over many rolls).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert!((DieRoll::die(6).unwrap().mean() - 3.5).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn mean(&self) -> f64 {
        self.mass.iter().map(|(k, p)| *k as f64 * p).sum()
    }

    /// Sum of stored probabilities (1.0 for normalized distributions).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert!((DieRoll::die(6).unwrap().total_mass() - 1.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn total_mass(&self) -> f64 {
        self.mass.values().sum()
    }

    /// `(outcome, probability)` pairs sorted by outcome.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// assert_eq!(DieRoll::constant(7).entries(), vec![(7, 1.0)]);
    /// ```
    pub fn entries(&self) -> Vec<(i64, f64)> {
        self.mass.iter().map(|(&k, &p)| (k, p)).collect()
    }

    /// Build from a raw mass map (callers should normalize when importing external data).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// use std::collections::BTreeMap;
    /// let mut mass = BTreeMap::new();
    /// mass.insert(0, 1.0);
    /// assert_eq!(DieRoll::from_mass(mass).pmf(0), 1.0);
    /// ```
    pub fn from_mass(mass: BTreeMap<i64, f64>) -> Self {
        Self { mass }
    }

    pub(crate) fn normalize_in_place(&mut self) -> Result<()> {
        let total = self.total_mass();
        if total <= 0.0 {
            bail!("distribution has zero total mass");
        }
        if (total - 1.0).abs() > 1e-9 {
            for p in self.mass.values_mut() {
                *p /= total;
            }
        }
        Ok(())
    }

    /// Independent subtraction: each pair of outcomes from `self` and `other` is differenced.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let a = DieRoll::pool_sum(2, 10).unwrap();
    /// let b = DieRoll::pool_sum(3, 6).unwrap();
    /// assert!((a.difference(&b).unwrap().mean() - 0.5).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn difference(&self, other: &Self) -> Result<Self> {
        let neg_other = other.map_outcomes(|k| -k)?;
        self.convolve(&neg_other)
    }

    /// Combine two **independent** numeric rolls by adding outcomes (convolution of PMFs).
    ///
    /// This is how `2d6` works: convolve one d6 with another. Probabilities multiply
    /// for each face pair; totals are summed.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d6 = DieRoll::die(6).unwrap();
    /// let two_d6 = d6.convolve(&d6).unwrap();
    /// assert_eq!(two_d6.min(), Some(2));
    /// assert_eq!(two_d6.max(), Some(12));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn convolve(&self, other: &Self) -> Result<Self> {
        if self.mass.is_empty() || other.mass.is_empty() {
            bail!("cannot convolve empty distribution");
        }
        let mut out = BTreeMap::new();
        for (&a, &pa) in &self.mass {
            for (&b, &pb) in &other.mass {
                *out.entry(a + b).or_insert(0.0) += pa * pb;
            }
        }
        let mut dist = Self { mass: out };
        dist.normalize_in_place()?;
        Ok(dist)
    }

    /// Add a flat modifier to every outcome (`+3` on the character sheet).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d20 = DieRoll::die(20).unwrap();
    /// let with_bonus = d20.shift(5).unwrap();
    /// assert!((with_bonus.p_ge(20) - d20.p_ge(15)).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn shift(&self, delta: i64) -> Result<Self> {
        if self.mass.is_empty() {
            bail!("cannot shift empty distribution");
        }
        let mass = self.mass.iter().map(|(&k, &p)| (k + delta, p)).collect();
        Ok(Self { mass })
    }

    /// Apply `f` to each outcome, merging masses when different faces collide.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d6 = DieRoll::die(6).unwrap();
    /// let neg = d6.map_outcomes(|x| -x).unwrap();
    /// assert_eq!(neg.pmf(-1), d6.pmf(1));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn map_outcomes(&self, mut f: impl FnMut(i64) -> i64) -> Result<Self> {
        if self.mass.is_empty() {
            bail!("cannot map empty distribution");
        }
        let mut out = BTreeMap::new();
        for (&k, &p) in &self.mass {
            *out.entry(f(k)).or_insert(0.0) += p;
        }
        let mut dist = Self { mass: out };
        dist.normalize_in_place()?;
        Ok(dist)
    }

    /// Clamp every outcome to `[min, max]`, merging probability when values collide.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let boosted = DieRoll::pool_sum(3, 6).unwrap().shift(5).unwrap();
    /// let capped = boosted.clamp(3, 18).unwrap();
    /// assert_eq!(capped.min(), Some(8));
    /// assert_eq!(capped.max(), Some(18));
    /// assert!(capped.pmf(18) > boosted.pmf(18));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn clamp(&self, min: i64, max: i64) -> Result<Self> {
        if min > max {
            bail!("clamp min must be <= max, got {min}..{max}");
        }
        self.map_outcomes(|k| k.clamp(min, max))
    }

    /// Keep only outcomes that satisfy `predicate`, then renormalize.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let high_faces = DieRoll::die(6).unwrap().keep_ge(5).unwrap();
    /// assert_eq!(high_faces.support_size(), 2);
    /// assert!((high_faces.pmf(5) - 0.5).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn keep_faces(&self, mut predicate: impl FnMut(i64) -> bool) -> Result<Self> {
        if self.mass.is_empty() {
            bail!("cannot keep faces on empty distribution");
        }
        let mut out = BTreeMap::new();
        for (&k, &p) in &self.mass {
            if predicate(k) {
                *out.entry(k).or_insert(0.0) += p;
            }
        }
        if out.is_empty() {
            bail!("keep_faces: no outcomes remain");
        }
        let mut dist = Self { mass: out };
        dist.normalize_in_place()?;
        Ok(dist)
    }

    /// Drop outcomes that satisfy `predicate`, then renormalize.
    pub fn remove_faces(&self, mut predicate: impl FnMut(i64) -> bool) -> Result<Self> {
        self.keep_faces(|k| !predicate(k))
    }

    /// Keep outcomes with value ≥ `threshold` (inclusive).
    pub fn keep_ge(&self, threshold: i64) -> Result<Self> {
        self.keep_faces(|k| k >= threshold)
    }

    /// Keep outcomes with value > `threshold`.
    pub fn keep_gt(&self, threshold: i64) -> Result<Self> {
        self.keep_faces(|k| k > threshold)
    }

    /// Keep outcomes with value ≤ `threshold` (inclusive).
    pub fn keep_le(&self, threshold: i64) -> Result<Self> {
        self.keep_faces(|k| k <= threshold)
    }

    /// Keep outcomes with value < `threshold`.
    pub fn keep_lt(&self, threshold: i64) -> Result<Self> {
        self.keep_faces(|k| k < threshold)
    }

    /// Keep outcomes in the inclusive closed interval `[lo, hi]`.
    pub fn keep_in_range(&self, lo: i64, hi: i64) -> Result<Self> {
        IntBand::through(lo, hi)?;
        self.keep_faces(|k| (lo..=hi).contains(&k))
    }

    /// Keep outcomes whose value appears in `values` (duplicates in the list are ignored).
    pub fn keep_in_set(&self, values: &[i64]) -> Result<Self> {
        if values.is_empty() {
            bail!("keep_in_set: need at least one face value");
        }
        self.keep_faces(|k| values.contains(&k))
    }

    /// Keep outcomes that fall in `band`.
    pub fn keep_in_band(&self, band: IntBand) -> Result<Self> {
        self.keep_faces(|k| band.contains(k))
    }

    /// Keep only faces matching a script [`FaceSpec`](crate::engine::FaceSpec).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, FaceSpec, IntBand};
    /// let high = DieRoll::die(6).unwrap().keep_faces_spec(FaceSpec::Band(IntBand::at_least(5)))?;
    /// assert_eq!(high.min(), Some(5));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn keep_faces_spec(&self, spec: super::face_spec::FaceSpec) -> Result<Self> {
        spec.keep_die_roll(self)
    }

    /// Drop faces matching `spec`, then renormalize.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, FaceSpec, IntBand};
    /// let d6 = DieRoll::die(6).unwrap();
    /// let removed = d6.remove_faces_spec(FaceSpec::Band(IntBand::through(1, 4)?))?;
    /// let kept = d6.keep_faces_spec(FaceSpec::Band(IntBand::at_least(5)))?;
    /// assert_eq!(removed.entries(), kept.entries());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn remove_faces_spec(&self, spec: super::face_spec::FaceSpec) -> Result<Self> {
        self.remove_faces(|k| spec.matches(k))
    }

    /// Remap matching faces to `to`; other faces unchanged (masses merged when outcomes collide).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, FaceSpec, IntBand};
    /// let ignored = DieRoll::die(6).unwrap()
    ///     .convert_faces_spec(FaceSpec::Band(IntBand::through(1, 4)?), 0)?;
    /// assert!((ignored.pmf(0) - 4.0 / 6.0).abs() < 1e-12);
    /// assert!((ignored.pmf(5) - 1.0 / 6.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn convert_faces_spec(&self, spec: super::face_spec::FaceSpec, to: i64) -> Result<Self> {
        self.map_outcomes(|k| if spec.matches(k) { to } else { k })
    }

    /// Remap matching faces to 0 (`convert(spec, 0)`).
    pub fn ignore_faces_spec(&self, spec: super::face_spec::FaceSpec) -> Result<Self> {
        self.convert_faces_spec(spec, 0)
    }

    /// Multiply every outcome by `factor` (e.g. tens die reading `d4 * 10`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let scaled = DieRoll::die(4).unwrap().scale_outcomes(10).unwrap();
    /// assert_eq!(scaled.min(), Some(10));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn scale_outcomes(&self, factor: i64) -> Result<Self> {
        if factor <= 0 {
            bail!("scale factor must be positive, got {factor}");
        }
        self.map_outcomes(|k| {
            k.checked_mul(factor)
                .unwrap_or_else(|| panic!("outcome overflow scaling {k} by {factor}"))
        })
    }

    /// Floor-divide every outcome by `divisor`, matching Starlark `//` (half damage on save).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let halved = DieRoll::pool_sum(2, 6).unwrap().floor_divide_outcomes(2).unwrap();
    /// assert_eq!(halved.min(), Some(1));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn floor_divide_outcomes(&self, divisor: i64) -> Result<Self> {
        if divisor <= 0 {
            bail!("divisor must be positive, got {divisor}");
        }
        if self.mass.is_empty() {
            bail!("cannot floor-divide empty distribution");
        }
        let mut out = BTreeMap::new();
        for (&k, &p) in &self.mass {
            let v = starlark_floor_div_i64(k, divisor)?;
            *out.entry(v).or_insert(0.0) += p;
        }
        let mut dist = Self { mass: out };
        dist.normalize_in_place()?;
        Ok(dist)
    }

    /// Rolemaster **open-ended roll** on a **1–100** (`d100`) result—two d10s as 01–100, **00** = 100.
    ///
    /// Implements the full open-ended procedure: **low open-ended** on **01–05** (subtract rerolls),
    /// **high open-ended** on **96–00** (add rerolls). Rerolls chain only while they show **96–00**
    /// (faces 96–100 in this model). **06–95** on the first roll stops with no reroll.
    /// `max_chain` caps consecutive **96–00** rerolls after an open trigger (like `explode` depth).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let oe = DieRoll::open_ended_d100(4).unwrap();
    /// assert!((oe.pmf(50) - 0.01).abs() < 1e-12);
    /// assert!(oe.min().unwrap() < 0);
    /// assert!(oe.max().unwrap() > 100);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn open_ended_d100(max_chain: u32) -> Result<Self> {
        let chain = reroll_sum_chain(max_chain)?;
        let mut mass = BTreeMap::new();
        let p_each = 0.01;
        for first in 1..=100 {
            if (6..=95).contains(&first) {
                *mass.entry(first).or_insert(0.0) += p_each;
            } else if (1..=5).contains(&first) {
                for (t, pt) in chain.entries() {
                    *mass.entry(first - t).or_insert(0.0) += p_each * pt;
                }
            } else {
                for (t, pt) in chain.entries() {
                    *mass.entry(first + t).or_insert(0.0) += p_each * pt;
                }
            }
        }
        let mut out = Self { mass };
        out.normalize_in_place()?;
        Ok(out)
    }

    /// Exploding die: on max face, reroll and add (up to `max_depth` extra explosions).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let d6 = DieRoll::die(6).unwrap();
    /// let exploded = d6.explode(1).unwrap();
    /// assert!(exploded.max().unwrap() > 6);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn explode(&self, max_depth: u32) -> Result<Self> {
        let Some(max_face) = self.max() else {
            bail!("explode: empty die");
        };
        let mut mass = BTreeMap::new();
        for (&face, &p) in &self.mass {
            if face == max_face && max_depth > 0 {
                let tail = self.explode(max_depth - 1)?;
                for (t, pt) in tail.entries() {
                    *mass.entry(face + t).or_insert(0.0) += p * pt;
                }
            } else {
                *mass.entry(face).or_insert(0.0) += p;
            }
        }
        let mut out = Self { mass };
        out.normalize_in_place()?;
        Ok(out)
    }

    /// Sum of `n` independent fair `1..=sides` dice (`nd{sides}`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let two_d6 = DieRoll::pool_sum(2, 6).unwrap();
    /// assert!((two_d6.mean() - 7.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pool_sum(n: usize, sides: i64) -> Result<Self> {
        if n == 0 {
            return Ok(Self::constant(0));
        }
        let one = Self::die(sides)?;
        let mut acc = one.clone();
        for _ in 1..n {
            acc = acc.convolve(&one)?;
        }
        Ok(acc)
    }

    /// `n`d`sides`, drop the lowest `drop` dice, sum the rest (`4d6dl1`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let dist = DieRoll::pool_drop_lowest(4, 6, 1).unwrap();
    /// assert_eq!(dist.min(), Some(3));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pool_drop_lowest(n: usize, sides: i64, drop: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        DicePool::from_count(n, sides)?.apply_pool_op(drop, PoolOp::DropLowestSum)
    }

    /// `n`d`sides`, keep the highest `keep` dice and sum them (`3d6kh2`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let dist = DieRoll::pool_keep_highest(3, 6, 2).unwrap();
    /// assert_eq!(dist.max(), Some(12));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pool_keep_highest(n: usize, sides: i64, keep: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        if keep == 0 {
            return Ok(Self::constant(0));
        }
        DicePool::from_count(n, sides)?.apply_pool_op(keep, PoolOp::KeepHighestSum)
    }

    /// `n`d`sides`, drop the highest `drop` dice, sum the rest.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let dist = DieRoll::pool_drop_highest(4, 6, 1).unwrap();
    /// assert!((dist.mean() - 8.755401234567925).abs() < 1e-6);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pool_drop_highest(n: usize, sides: i64, drop: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        DicePool::from_count(n, sides)?.apply_pool_op(drop, PoolOp::DropHighestSum)
    }

    /// `n`d`sides`, keep the lowest `keep` dice and sum them.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DieRoll;
    /// let dist = DieRoll::pool_keep_lowest(3, 12, 1).unwrap();
    /// assert!((dist.mean() - 3.5208333333333326).abs() < 1e-6);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pool_keep_lowest(n: usize, sides: i64, keep: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        if keep == 0 {
            return Ok(Self::constant(0));
        }
        DicePool::from_count(n, sides)?.apply_pool_op(keep, PoolOp::KeepLowestSum)
    }

    /// Label numeric totals using the bands on `scale` ([`Outcomes::from_scale`]).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, IntBand, Scale};
    /// let scale = Scale::with_bands(
    ///     vec!["LOW".into(), "HIGH".into()],
    ///     vec![IntBand::at_most(3), IntBand::at_least(4)],
    /// )
    /// .unwrap();
    /// let o = DieRoll::die(6).unwrap().bucket(scale).unwrap();
    /// assert!((o.p_exact("HIGH").unwrap() - 0.5).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn bucket(&self, scale: super::ordinal::Scale) -> Result<super::ordinal::Outcomes> {
        super::ordinal::Outcomes::from_scale(self, scale)
    }
}

impl Default for DieRoll {
    fn default() -> Self {
        Self::new()
    }
}

/// Sum of d100 rerolls after a low/high open trigger; further rolls while the face is **96–00**.
fn reroll_sum_chain(max_chain: u32) -> Result<DieRoll> {
    let mut g = DieRoll::die(100)?;
    for _ in 0..max_chain {
        let tail = g.clone();
        let mut mass = BTreeMap::new();
        let p_each = 0.01;
        for r in 1..=95 {
            *mass.entry(r).or_insert(0.0) += p_each;
        }
        for r in 96..=100 {
            for (t, pt) in tail.entries() {
                *mass.entry(r + t).or_insert(0.0) += p_each * pt;
            }
        }
        g = DieRoll { mass };
        g.normalize_in_place()?;
    }
    Ok(g)
}

/// Starlark-compatible `a // b` for signed integers.
pub(crate) fn starlark_floor_div_i64(a: i64, b: i64) -> Result<i64> {
    if b == 0 {
        bail!("division by zero");
    }
    let sig = b.signum() * a.signum();
    let offset = i64::from(sig < 0 && a % b != 0);
    Ok(a / b - offset)
}

#[cfg(test)]
mod floor_div_tests {
    use super::*;

    #[test]
    fn starlark_floor_div_examples() {
        assert_eq!(starlark_floor_div_i64(7, 2).unwrap(), 3);
        assert_eq!(starlark_floor_div_i64(-7, -2).unwrap(), 3);
        assert_eq!(starlark_floor_div_i64(7, -2).unwrap(), -4);
        assert_eq!(starlark_floor_div_i64(-7, 2).unwrap(), -4);
    }

    #[test]
    fn d4_times_ten() {
        let d4 = DieRoll::die(4).unwrap();
        let scaled = d4.scale_outcomes(10).unwrap();
        assert_eq!(scaled.min(), Some(10));
        assert_eq!(scaled.max(), Some(40));
        assert!((scaled.pmf(10) - 0.25).abs() < 1e-12);
        assert!((scaled.mean() - 25.0).abs() < 1e-9);
    }

    #[test]
    fn eight_d6_halved() {
        let full = DieRoll::pool_sum(8, 6).unwrap();
        let half = full.floor_divide_outcomes(2).unwrap();
        assert_eq!(half.min(), Some(4));
        assert_eq!(half.max(), Some(24));
        let p14 = full.pmf(28) + full.pmf(29);
        assert!((half.pmf(14) - p14).abs() < 1e-12);
    }

    #[test]
    fn clamp_3d6_plus_five() {
        let roll = DieRoll::pool_sum(3, 6).unwrap().shift(5).unwrap();
        let capped = roll.clamp(3, 18).unwrap();
        assert_eq!(capped.min(), Some(8));
        assert_eq!(capped.max(), Some(18));
        assert!((capped.pmf(10) - roll.pmf(10)).abs() < 1e-12);
        let tail: f64 = (19..=23).map(|k| roll.pmf(k)).sum();
        assert!((capped.pmf(18) - roll.pmf(18) - tail).abs() < 1e-12);
    }

    #[test]
    fn clamp_rejects_inverted_bounds() {
        let d6 = DieRoll::die(6).unwrap();
        assert!(d6.clamp(10, 3).is_err());
    }

    #[test]
    #[ignore = "manual benchmark for open-ended chain cost"]
    fn bench_open_ended_chain_sizes() {
        for k in 1..=8 {
            let t = std::time::Instant::now();
            let oe = DieRoll::open_ended_d100(k).unwrap();
            eprintln!(
                "max_chain={k} support={} ms={}",
                oe.support_size(),
                t.elapsed().as_millis()
            );
        }
    }

    #[test]
    fn open_ended_d100_flat_midrange() {
        let oe = DieRoll::open_ended_d100(4).unwrap();
        assert!((oe.pmf(50) - 0.01).abs() < 1e-12);
        assert!((oe.pmf(95) - 0.01).abs() < 1e-12);
    }

    #[test]
    fn open_ended_d100_rmss_low_example_path() {
        let oe = DieRoll::open_ended_d100(4).unwrap();
        let one_path = 1.0 / 1_000_000.0;
        assert!(
            oe.pmf(-96) >= one_path,
            "RMSS low open example 04−97−03 should contribute mass at −96"
        );
    }

    #[test]
    fn open_ended_d100_rmss_high_example_path() {
        let oe = DieRoll::open_ended_d100(4).unwrap();
        let one_path = 1.0 / 1_000_000.0;
        assert!(
            oe.pmf(199) >= one_path,
            "RMSS high open example 99+96+04 should contribute mass at 199"
        );
    }

    #[test]
    fn remove_faces_matches_keep_complement_on_d6() {
        use crate::engine::{FaceSpec, IntBand};
        let d6 = DieRoll::die(6).unwrap();
        let band_low = FaceSpec::Band(IntBand::through(1, 4).unwrap());
        let removed = d6.remove_faces_spec(band_low).unwrap();
        let kept = d6
            .keep_faces_spec(FaceSpec::Band(IntBand::at_least(5)))
            .unwrap();
        assert_eq!(removed.entries(), kept.entries());
    }

    #[test]
    fn ignore_low_faces_on_d6() {
        use crate::engine::{FaceSpec, IntBand};
        let ignored = DieRoll::die(6)
            .unwrap()
            .ignore_faces_spec(FaceSpec::Band(IntBand::through(1, 4).unwrap()))
            .unwrap();
        assert!((ignored.pmf(0) - 4.0 / 6.0).abs() < 1e-12);
        assert!((ignored.pmf(5) - 1.0 / 6.0).abs() < 1e-12);
        assert!((ignored.pmf(6) - 1.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn pool_ignore_low_faces_sum_has_zero() {
        use crate::engine::{DicePool, FaceSpec, IntBand};
        let total = DicePool::from_count(3, 6)
            .unwrap()
            .ignore_faces_spec(FaceSpec::Band(IntBand::through(1, 4).unwrap()))
            .unwrap()
            .sum()
            .unwrap();
        assert!(total.pmf(0) > 0.0);
        assert!((total.mean() - 5.5).abs() < 1e-9);
    }
}
