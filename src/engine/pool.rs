//! Independent dice pools (not summed until collapsed).

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::die::Die;
use super::enumerate::for_each_pool_joint;

#[derive(Clone, Debug, PartialEq)]
pub struct RollPool {
    dice: Vec<Die>,
}

#[derive(Clone, Copy, Debug)]
pub enum PoolOp {
    DropLowestSum,
    DropHighestSum,
    KeepHighestSum,
    KeepLowestSum,
}

impl RollPool {
    pub fn from_dice(dice: Vec<Die>) -> Result<Self> {
        if dice.is_empty() {
            bail!("roll pool must contain at least one die");
        }
        Ok(Self { dice })
    }

    pub fn from_count(count: usize, sides: i64) -> Result<Self> {
        if count == 0 {
            bail!("roll pool count must be >= 1");
        }
        let one = Die::die(sides)?;
        Ok(Self {
            dice: vec![one; count],
        })
    }

    pub fn dice(&self) -> &[Die] {
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

    pub fn sum(&self) -> Result<Die> {
        if let Some((n, sides)) = self.uniform_fair_params() {
            return Die::pool_sum(n, sides);
        }
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let total: i64 = faces.iter().sum();
            *mass.entry(total).or_insert(0.0) += p;
        })?;
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    pub fn apply_pool_op(&self, param: usize, op: PoolOp) -> Result<Die> {
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
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Distribution of how many faces are `>= threshold`.
    pub fn count_ge(&self, threshold: i64) -> Result<Die> {
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
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// Distribution of how many faces appear in `values`.
    pub fn count_in(&self, values: &[i64]) -> Result<Die> {
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
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    /// `k=1` is the highest face in the sorted pool.
    pub fn order_stat(&self, k: usize) -> Result<Die> {
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
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    pub fn map_joint(&self, mut f: impl FnMut(&[i64]) -> i64) -> Result<Die> {
        let mut mass = BTreeMap::new();
        for_each_pool_joint(self, |faces, p| {
            let out = f(faces);
            *mass.entry(out).or_insert(0.0) += p;
        })?;
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }

    pub fn middle_of(&self, keep: usize) -> Result<Die> {
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
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(die)
    }
}

/// Binomial: number of successes in `n` i.i.d. trials with probability `p` each.
fn binomial_success_count(n: usize, p: f64) -> Result<Die> {
    if n == 0 {
        return Ok(Die::constant(0));
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
    Ok(Die::from_mass(mass))
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
        let pool = RollPool::from_count(3, 6).unwrap();
        let total = pool.sum().unwrap();
        assert!((total.mean() - 10.5).abs() < 1e-9);
    }

    #[test]
    fn count_ge_3d6_above_4() {
        let pool = RollPool::from_count(3, 6).unwrap();
        let c = pool.count_ge(5).unwrap();
        assert!((c.pmf(0) - 8.0 / 27.0).abs() < 1e-9);
        assert!((c.pmf(1) - 4.0 / 9.0).abs() < 1e-9);
        assert!((c.pmf(3) - 1.0 / 27.0).abs() < 1e-9);
    }

    #[test]
    fn order_stat_highest_is_keep_one() {
        let pool = RollPool::from_count(3, 6).unwrap();
        let hi = pool.order_stat(1).unwrap();
        let keep = Die::pool_keep_highest(3, 6, 1).unwrap();
        for k in 1..=6 {
            assert!((hi.pmf(k) - keep.pmf(k)).abs() < 1e-9);
        }
    }
}
