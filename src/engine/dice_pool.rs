//! Several independent dice rolled together before summing or keep/drop rules.
//!
//! Tabletop notation like `4d6dl1` is implemented by enumerating all `4d6` face tuples,
//! dropping the lowest die, then summing the rest. [`DicePool`] is the intermediate object
//! when you need per-die information (highest die, success counts, order statistics).

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::die_roll::DieRoll;
use super::enumerate::for_each_pool_joint;

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
    fn count_ge_3d6_above_4() {
        let pool = DicePool::from_count(3, 6).unwrap();
        let c = pool.count_ge(5).unwrap();
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
}
