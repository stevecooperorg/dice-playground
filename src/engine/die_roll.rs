//! Exact chances for numeric roll totals (`1d6`, `2d6`, `4d6dl1`, modifiers).

use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Sparse PMF over signed integer outcomes (supports shifted rolls and modifiers).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DieRoll {
    pub(crate) mass: BTreeMap<i64, f64>,
}

impl DieRoll {
    pub fn new() -> Self {
        Self {
            mass: BTreeMap::new(),
        }
    }

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

    /// Fair die from an explicit face list (multiplicity = weight).
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

    pub fn constant(value: i64) -> Self {
        let mut mass = BTreeMap::new();
        mass.insert(value, 1.0);
        Self { mass }
    }

    pub fn support_size(&self) -> usize {
        self.mass.len()
    }

    pub fn min(&self) -> Option<i64> {
        self.mass.keys().next().copied()
    }

    pub fn max(&self) -> Option<i64> {
        self.mass.keys().next_back().copied()
    }

    pub fn pmf(&self, value: i64) -> f64 {
        self.mass.get(&value).copied().unwrap_or(0.0)
    }

    pub fn cdf(&self, value: i64) -> f64 {
        self.mass
            .iter()
            .filter(|(k, _)| **k <= value)
            .map(|(_, p)| p)
            .sum()
    }

    pub fn p_ge(&self, value: i64) -> f64 {
        self.mass
            .iter()
            .filter(|(k, _)| **k >= value)
            .map(|(_, p)| p)
            .sum()
    }

    pub fn mean(&self) -> f64 {
        self.mass.iter().map(|(k, p)| *k as f64 * p).sum()
    }

    pub fn total_mass(&self) -> f64 {
        self.mass.values().sum()
    }

    pub fn entries(&self) -> Vec<(i64, f64)> {
        self.mass.iter().map(|(&k, &p)| (k, p)).collect()
    }

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

    pub fn difference(&self, other: &Self) -> Result<Self> {
        let neg_other = other.map_outcomes(|k| -k)?;
        self.convolve(&neg_other)
    }

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

    pub fn shift(&self, delta: i64) -> Result<Self> {
        if self.mass.is_empty() {
            bail!("cannot shift empty distribution");
        }
        let mass = self.mass.iter().map(|(&k, &p)| (k + delta, p)).collect();
        Ok(Self { mass })
    }

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

    /// Multiply every outcome by `factor` (e.g. `d(4) * 10` for ten d4 pips).
    pub fn scale_outcomes(&self, factor: i64) -> Result<Self> {
        if factor <= 0 {
            bail!("scale factor must be positive, got {factor}");
        }
        self.map_outcomes(|k| {
            k.checked_mul(factor)
                .unwrap_or_else(|| panic!("outcome overflow scaling {k} by {factor}"))
        })
    }

    /// Floor-divide every outcome by `divisor`, matching Starlark `//` on integers.
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

    /// Exploding die: reroll and add while face equals `max_face`, up to `max_depth` extra rolls.
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

    pub fn pool_drop_lowest(n: usize, sides: i64, drop: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        DicePool::from_count(n, sides)?.apply_pool_op(drop, PoolOp::DropLowestSum)
    }

    pub fn pool_keep_highest(n: usize, sides: i64, keep: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        if keep == 0 {
            return Ok(Self::constant(0));
        }
        DicePool::from_count(n, sides)?.apply_pool_op(keep, PoolOp::KeepHighestSum)
    }

    pub fn pool_drop_highest(n: usize, sides: i64, drop: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        DicePool::from_count(n, sides)?.apply_pool_op(drop, PoolOp::DropHighestSum)
    }

    pub fn pool_keep_lowest(n: usize, sides: i64, keep: usize) -> Result<Self> {
        use super::dice_pool::DicePool;
        use super::dice_pool::PoolOp;
        if keep == 0 {
            return Ok(Self::constant(0));
        }
        DicePool::from_count(n, sides)?.apply_pool_op(keep, PoolOp::KeepLowestSum)
    }
}

impl Default for DieRoll {
    fn default() -> Self {
        Self::new()
    }
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
}
