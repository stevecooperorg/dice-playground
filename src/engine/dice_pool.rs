//! Several independent dice rolled together before summing or keep/drop rules.
//!
//! Tabletop notation like `4d6dl1` is implemented by enumerating all `4d6` face tuples,
//! dropping the lowest die, then summing the rest. [`DicePool`] is the intermediate object
//! when you need per-die information (highest die, success counts, order statistics).

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::die_roll::DieRoll;
use super::enumerate::for_each_pool_joint;
use super::face_spec::{FaceSpec, OptionalFaceSpec};
use super::int_band::IntBand;

/// Independent dice that have not yet been combined into a single [`DieRoll`].
///
/// # Example
///
/// ```
/// use dice_playground::engine::DicePool;
/// let pool = DicePool::from_count(4, 6).unwrap();
/// let four_d6_drop_lowest = pool.apply_pool_op(1, dice_playground::engine::PoolOp::DropLowestSum).unwrap();
/// assert!((four_d6_drop_lowest.mean() - 12.244598765432098).abs() < 1e-6);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DicePool {
    dice: Vec<DieRoll>,
}

/// How to collapse a pool to one total after sorting faces (drop/keep lowest or highest).
#[derive(Clone, Copy, Debug)]
pub enum PoolOp {
    /// Sort ascending, drop the lowest `param` faces, sum the rest (`4d6dl1`).
    DropLowestSum,
    /// Sort ascending, drop the highest `param` faces, sum the rest.
    DropHighestSum,
    /// Sort descending, keep the highest `param` faces, sum them (`3d6kh2`).
    KeepHighestSum,
    /// Sort ascending, keep the lowest `param` faces, sum them.
    KeepLowestSum,
}

impl DicePool {
    /// Build a pool from one [`DieRoll`] per die (each may differ).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, DieRoll};
    /// let pool = DicePool::from_dice(vec![DieRoll::die(6).unwrap(), DieRoll::die(8).unwrap()]).unwrap();
    /// assert_eq!(pool.dice().len(), 2);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_dice(dice: Vec<DieRoll>) -> Result<Self> {
        if dice.is_empty() {
            bail!("roll pool must contain at least one die");
        }
        Ok(Self { dice })
    }

    /// `count` copies of a fair `1..=sides` die (tabletop `count`d`sides` before modifiers).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let three_d6 = DicePool::from_count(3, 6).unwrap();
    /// assert_eq!(three_d6.dice().len(), 3);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_count(count: usize, sides: i64) -> Result<Self> {
        if count == 0 {
            bail!("roll pool count must be >= 1");
        }
        let one = DieRoll::die(sides)?;
        Ok(Self {
            dice: vec![one; count],
        })
    }

    /// Roll two pools together: every die from `self`, then every die from `other`, still independent.
    ///
    /// Use this for mixed pools (for example 1d12 with 2d6) before `order_stat`, `count`, or `.sum()`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let mixed = DicePool::from_count(1, 12).unwrap()
    ///     .join(&DicePool::from_count(2, 6).unwrap()).unwrap();
    /// assert_eq!(mixed.dice().len(), 3);
    /// let hi = mixed.order_stat(1).unwrap();
    /// assert!((hi.pmf(12) - 1.0 / 12.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn join(&self, other: &Self) -> Result<Self> {
        let mut dice = self.dice.clone();
        dice.extend_from_slice(other.dice());
        Self::from_dice(dice)
    }

    /// Append one independent die to the pool.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, DieRoll};
    /// let pool = DicePool::from_count(2, 6).unwrap()
    ///     .push_die(DieRoll::die(12).unwrap()).unwrap();
    /// assert_eq!(pool.dice().len(), 3);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn push_die(&self, die: DieRoll) -> Result<Self> {
        let mut dice = self.dice.clone();
        dice.push(die);
        Self::from_dice(dice)
    }

    /// Slice of per-die distributions in roll order.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let pool = DicePool::from_count(2, 6).unwrap();
    /// assert_eq!(pool.dice().len(), 2);
    /// ```
    pub fn dice(&self) -> &[DieRoll] {
        &self.dice
    }

    /// Apply `f` to each die in the pool independently.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, IntBand};
    /// let pool = DicePool::from_count(3, 6).unwrap()
    ///     .keep_faces_spec(FaceSpec::Band(IntBand::at_least(5))).unwrap();
    /// let total = pool.sum().unwrap();
    /// assert_eq!(total.min(), Some(15));
    /// assert_eq!(total.max(), Some(18));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn map_dice(&self, mut f: impl FnMut(DieRoll) -> Result<DieRoll>) -> Result<Self> {
        let dice: Vec<DieRoll> = self
            .dice
            .iter()
            .cloned()
            .map(&mut f)
            .collect::<Result<_>>()?;
        Self::from_dice(dice)
    }

    /// Keep only faces ≥ `threshold` on every die.
    pub fn keep_ge(&self, threshold: i64) -> Result<Self> {
        self.map_dice(|d| d.keep_ge(threshold))
    }

    pub fn keep_gt(&self, threshold: i64) -> Result<Self> {
        self.map_dice(|d| d.keep_gt(threshold))
    }

    pub fn keep_le(&self, threshold: i64) -> Result<Self> {
        self.map_dice(|d| d.keep_le(threshold))
    }

    pub fn keep_lt(&self, threshold: i64) -> Result<Self> {
        self.map_dice(|d| d.keep_lt(threshold))
    }

    pub fn keep_in_range(&self, lo: i64, hi: i64) -> Result<Self> {
        self.map_dice(|d| d.keep_in_range(lo, hi))
    }

    pub fn keep_in_set(&self, values: &[i64]) -> Result<Self> {
        self.map_dice(|d| d.keep_in_set(values))
    }

    pub fn keep_in_band(&self, band: IntBand) -> Result<Self> {
        self.map_dice(|d| d.keep_in_band(band))
    }

    /// Keep only matching faces on every die (see [`FaceSpec`]).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, IntBand};
    /// let pool = DicePool::from_count(2, 6).unwrap()
    ///     .keep_faces_spec(FaceSpec::Band(IntBand::at_least(5))).unwrap();
    /// assert_eq!(pool.sum().unwrap().min(), Some(10));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn keep_faces_spec(&self, spec: FaceSpec) -> Result<Self> {
        self.map_dice(|d| d.keep_faces_spec(spec.clone()))
    }

    /// Drop matching faces on every die, then renormalize each die.
    pub fn remove_faces_spec(&self, spec: FaceSpec) -> Result<Self> {
        self.map_dice(|d| d.remove_faces_spec(spec.clone()))
    }

    /// Remap matching faces to `to` on every die.
    pub fn convert_faces_spec(&self, spec: FaceSpec, to: i64) -> Result<Self> {
        self.map_dice(|d| d.convert_faces_spec(spec.clone(), to))
    }

    /// Remap matching faces to 0 on every die.
    pub fn ignore_faces_spec(&self, spec: FaceSpec) -> Result<Self> {
        self.map_dice(|d| d.ignore_faces_spec(spec.clone()))
    }

    /// Distribution of how many dice in the pool match `spec`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, IntBand};
    /// let c = DicePool::from_count(3, 6).unwrap()
    ///     .count_faces(FaceSpec::Band(IntBand::at_least(5))).unwrap();
    /// assert!((c.pmf(0) - 8.0 / 27.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn count_faces(&self, spec: FaceSpec) -> Result<DieRoll> {
        match spec {
            FaceSpec::Band(band) => self.count_in_band(band),
            FaceSpec::Faces(ref values) => self.count_in(values),
        }
    }

    /// If every die is the same fair `1..=sides`, return `(count, sides)`.
    pub fn uniform_fair_params(&self) -> Option<(usize, i64)> {
        if self.dice.is_empty() {
            return None;
        }
        let first = &self.dice[0];
        let entries = first.entries();
        let sides = first.max()?;
        if sides < 1 {
            return None;
        }
        let expected = sides as usize;
        if entries.len() != expected {
            return None;
        }
        for (i, &(face, p)) in entries.iter().enumerate() {
            if face != i as i64 + 1 || (p - 1.0 / sides as f64).abs() > 1e-9 {
                return None;
            }
        }
        if !self.dice.iter().all(|d| d.entries() == entries) {
            return None;
        }
        Some((self.dice.len(), sides))
    }

    /// Distribution of the **sum** of all dice (independent convolution), e.g. `3d6` total.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let total = DicePool::from_count(2, 6).unwrap().sum().unwrap();
    /// assert!((total.mean() - 7.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn sum(&self) -> Result<DieRoll> {
        if let Some((n, sides)) = self.uniform_fair_params() {
            return DieRoll::pool_sum(n, sides);
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let total: i64 = faces.iter().sum();
            *mass.entry(total).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Apply keep/drop-then-sum rules via [`PoolOp`] (see `drop_lowest`, `keep_highest`, etc. in Starlark).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, PoolOp};
    /// let dist = DicePool::from_count(4, 6).unwrap()
    ///     .apply_pool_op(1, PoolOp::DropLowestSum).unwrap();
    /// assert_eq!(dist.min(), Some(3));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn apply_pool_op(&self, param: usize, op: PoolOp) -> Result<DieRoll> {
        let n = self.dice.len();
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let mut sorted = faces.to_vec();
            sorted.sort_unstable();
            let value = match op {
                PoolOp::DropLowestSum => {
                    let drop = param.min(n.saturating_sub(1));
                    sorted.iter().skip(drop).sum()
                }
                PoolOp::DropHighestSum => {
                    let drop = param.min(n.saturating_sub(1));
                    let keep_count = n.saturating_sub(drop);
                    sorted.iter().take(keep_count).sum()
                }
                PoolOp::KeepHighestSum => {
                    let keep = param.min(n);
                    sorted.sort_by(|a, b| b.cmp(a));
                    sorted.iter().take(keep).sum()
                }
                PoolOp::KeepLowestSum => {
                    let keep = param.min(n);
                    sorted.iter().take(keep).sum()
                }
            };
            *mass.entry(value).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Distribution of how many dice show **at least** `threshold` (success-count pools).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let hits = DicePool::from_count(3, 6).unwrap().count_ge(5).unwrap();
    /// assert!((hits.pmf(3) - 1.0 / 27.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn count_ge(&self, threshold: i64) -> Result<DieRoll> {
        if let Some((n, sides)) = self.uniform_fair_params() {
            let hits = (1..=sides).filter(|&f| f >= threshold).count();
            let p = hits as f64 / sides as f64;
            return binomial_success_count(n, p);
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let c = faces.iter().filter(|&&f| f >= threshold).count() as i64;
            *mass.entry(c).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Distribution of how many dice land on a face in `values`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let evens = DicePool::from_count(2, 6).unwrap().count_in(&[2, 4, 6]).unwrap();
    /// assert!((evens.pmf(2) - 0.25).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn count_in(&self, values: &[i64]) -> Result<DieRoll> {
        if let Some((n, sides)) = self.uniform_fair_params() {
            let hits = (1..=sides).filter(|&f| values.contains(&f)).count();
            let p = hits as f64 / sides as f64;
            return binomial_success_count(n, p);
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let c = faces.iter().filter(|f| values.contains(f)).count() as i64;
            *mass.entry(c).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// How many dice show a face in `band`?
    pub fn count_in_band(&self, band: IntBand) -> Result<DieRoll> {
        if let Some((n, sides)) = self.uniform_fair_params() {
            let hits = (1..=sides).filter(|&f| band.contains(f)).count();
            let p = hits as f64 / sides as f64;
            return binomial_success_count(n, p);
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let c = faces.iter().filter(|&&f| band.contains(f)).count() as i64;
            *mass.entry(c).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Distribution of the `k`th highest face (`k = 1` is the maximum die).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, DieRoll};
    /// let hi = DicePool::from_count(3, 6).unwrap().order_stat(1).unwrap();
    /// let keep = DieRoll::pool_keep_highest(3, 6, 1).unwrap();
    /// assert!((hi.pmf(6) - keep.pmf(6)).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn order_stat(&self, k: usize) -> Result<DieRoll> {
        let n = self.dice.len();
        if k == 0 || k > n {
            bail!("order_stat: k must be 1..={n}, got {k}");
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let mut sorted = faces.to_vec();
            sorted.sort_by(|a, b| b.cmp(a));
            let face = sorted[k - 1];
            *mass.entry(face).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Map each joint face tuple through `f` and accumulate a PMF over the resulting integers.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let max_face = DicePool::from_count(2, 6).unwrap()
    ///     .map_joint(|faces| *faces.iter().max().unwrap()).unwrap();
    /// assert!((max_face.pmf(6) - 11.0 / 36.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn map_joint(&self, mut f: impl FnMut(&[i64]) -> i64) -> Result<DieRoll> {
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let out = f(faces);
            *mass.entry(out).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Sum the middle `keep` faces after sorting (used by some house rules).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::DicePool;
    /// let mid = DicePool::from_count(3, 6).unwrap().middle_of(1).unwrap();
    /// assert_eq!(mid.min(), Some(1));
    /// assert_eq!(mid.max(), Some(6));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn middle_of(&self, keep: usize) -> Result<DieRoll> {
        let n = self.dice.len();
        if keep == 0 || keep > n {
            bail!("middle_of: keep must be 1..={n}, got {keep}");
        }
        let start = (n - keep) / 2;
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let mut sorted = faces.to_vec();
            sorted.sort_unstable();
            let value: i64 = sorted.iter().skip(start).take(keep).sum();
            *mass.entry(value).or_insert(0.0) += p;
        })?;
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    fn count_for_optional(&self, spec: &OptionalFaceSpec) -> Result<DieRoll> {
        match spec {
            OptionalFaceSpec::LengthOnly => bail!("count: face spec required"),
            OptionalFaceSpec::Spec(face) => self.count_faces(face.clone()),
        }
    }

    /// **P(at least one die matches `spec`)**, or **P(pool is non-empty)** when `spec` is [`OptionalFaceSpec::LengthOnly`].
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, OptionalFaceSpec};
    /// let pool = DicePool::from_count(2, 6).unwrap();
    /// let p = pool.p_any(OptionalFaceSpec::Spec(FaceSpec::Faces(vec![1])))?;
    /// assert!((p - 11.0 / 36.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_any(&self, spec: OptionalFaceSpec) -> Result<f64> {
        if matches!(spec, OptionalFaceSpec::LengthOnly) {
            return Ok(if self.dice.is_empty() { 0.0 } else { 1.0 });
        }
        Ok(self.count_for_optional(&spec)?.p_ge(1))
    }

    /// **P(no die matches `spec`)**, or **P(pool is empty)** when `spec` is [`OptionalFaceSpec::LengthOnly`].
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, OptionalFaceSpec};
    /// let pool = DicePool::from_count(1, 6).unwrap();
    /// let p = pool.p_none(OptionalFaceSpec::Spec(FaceSpec::Faces(vec![1])))?;
    /// assert!((p - 5.0 / 6.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_none(&self, spec: OptionalFaceSpec) -> Result<f64> {
        if matches!(spec, OptionalFaceSpec::LengthOnly) {
            return Ok(if self.dice.is_empty() { 1.0 } else { 0.0 });
        }
        Ok(self.count_for_optional(&spec)?.pmf(0))
    }

    /// **P(at least `k` dice match `spec`)**, or **P(pool has ≥ `k` dice)** when `spec` is [`OptionalFaceSpec::LengthOnly`].
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DicePool, FaceSpec, IntBand, OptionalFaceSpec};
    /// let pool = DicePool::from_count(3, 6).unwrap();
    /// let band = IntBand::at_least(5);
    /// let p = pool.p_at_least(2, OptionalFaceSpec::Spec(FaceSpec::Band(band)))?;
    /// assert!((p - 7.0 / 27.0).abs() < 1e-9);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_at_least(&self, k: usize, spec: OptionalFaceSpec) -> Result<f64> {
        if matches!(spec, OptionalFaceSpec::LengthOnly) {
            return Ok(if self.dice.len() >= k { 1.0 } else { 0.0 });
        }
        let k_i64 = i64::try_from(k).map_err(|_| anyhow::anyhow!("p_at_least: k out of range"))?;
        Ok(self.count_for_optional(&spec)?.p_ge(k_i64))
    }
}

/// Binomial: number of successes in `n` i.i.d. trials with probability `p` each.
fn binomial_success_count(n: usize, p: f64) -> Result<DieRoll> {
    if n == 0 {
        return Ok(DieRoll::constant(0));
    }
    if !(0.0..=1.0).contains(&p) {
        bail!("invalid success probability {p}");
    }
    let q = 1.0 - p;
    let mut mass = BTreeMap::new();
    for k in 0..=n {
        let c = binomial_coeff(n, k);
        let prob = c * p.powi(k as i32) * q.powi((n - k) as i32);
        if prob > 0.0 {
            mass.insert(k as i64, prob);
        }
    }
    Ok(DieRoll::from_mass(mass))
}

fn binomial_coeff(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k);
    let mut num = 1.0_f64;
    let mut den = 1.0_f64;
    for i in 0..k {
        num *= (n - i) as f64;
        den *= (i + 1) as f64;
    }
    num / den
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_pool_sum_3d6_mean() {
        let pool = DicePool::from_count(3, 6).unwrap();
        let total = pool.sum().unwrap();
        assert!((total.mean() - 10.5).abs() < 1e-9);
    }

    #[test]
    fn count_faces_3d6_at_least_5() {
        let pool = DicePool::from_count(3, 6).unwrap();
        let c = pool
            .count_faces(FaceSpec::Band(IntBand::at_least(5)))
            .unwrap();
        assert!((c.pmf(0) - 8.0 / 27.0).abs() < 1e-9);
        assert!((c.pmf(1) - 4.0 / 9.0).abs() < 1e-9);
        assert!((c.pmf(3) - 1.0 / 27.0).abs() < 1e-9);
    }

    #[test]
    fn order_stat_highest_is_keep_one() {
        let pool = DicePool::from_count(3, 6).unwrap();
        let hi = pool.order_stat(1).unwrap();
        let keep = DieRoll::pool_keep_highest(3, 6, 1).unwrap();
        for k in 1..=6 {
            assert!((hi.pmf(k) - keep.pmf(k)).abs() < 1e-9);
        }
    }

    #[test]
    fn p_any_the_pool_style() {
        for n in 1..=10 {
            let pool = DicePool::from_count(n, 6).unwrap();
            let p = pool
                .p_any(OptionalFaceSpec::Spec(FaceSpec::Faces(vec![1])))
                .unwrap();
            let expected = 1.0 - f64::powf(5.0 / 6.0, n as f64);
            assert!((p - expected).abs() < 1e-9, "n={n}");
        }
    }

    #[test]
    fn p_none_single_one() {
        let pool = DicePool::from_count(1, 6).unwrap();
        assert!(
            (pool
                .p_none(OptionalFaceSpec::Spec(FaceSpec::Faces(vec![1])))
                .unwrap()
                - 5.0 / 6.0)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn p_at_least_two_fives_on_3d6() {
        let pool = DicePool::from_count(3, 6).unwrap();
        let p = pool
            .p_at_least(
                2,
                OptionalFaceSpec::Spec(FaceSpec::Band(IntBand::at_least(5))),
            )
            .unwrap();
        assert!((p - 7.0 / 27.0).abs() < 1e-9);
    }

    #[test]
    fn join_mixed_pool_order_stat() {
        let mixed = DicePool::from_count(1, 12)
            .unwrap()
            .join(&DicePool::from_count(2, 6).unwrap())
            .unwrap();
        assert_eq!(mixed.dice().len(), 3);
        let hi = mixed.order_stat(1).unwrap();
        assert!((hi.pmf(12) - 1.0 / 12.0).abs() < 1e-9);
        assert!(hi.max() == Some(12));
    }

    #[test]
    fn length_only_p_predicates() {
        let pool = DicePool::from_count(3, 6).unwrap();
        assert!((pool.p_any(OptionalFaceSpec::LengthOnly).unwrap() - 1.0).abs() < 1e-9);
        assert!((pool.p_none(OptionalFaceSpec::LengthOnly).unwrap() - 0.0).abs() < 1e-9);
        assert!((pool.p_at_least(4, OptionalFaceSpec::LengthOnly).unwrap() - 0.0).abs() < 1e-9);
        assert!((pool.p_at_least(3, OptionalFaceSpec::LengthOnly).unwrap() - 1.0).abs() < 1e-9);
    }
}
