//! Walk every joint outcome of an independent dice pool.
//!
//! For `3d6`, there are `6³ = 216` equally likely face tuples before keep/drop or summing.
//! Enumeration multiplies support sizes; [`MAX_JOINT_CELLS`] caps how large a pool may be
//! for exact methods (larger pools need smaller dice counts or simulation).

use anyhow::{bail, Context, Result};

use super::DicePool;

/// Maximum joint outcomes (`face₁ × face₂ × …`) allowed for exact pool enumeration.
///
/// # Example
///
/// ```
/// use dice_playground::engine::{DicePool, MAX_JOINT_CELLS};
/// let pool = DicePool::from_count(3, 6).unwrap();
/// assert!(6usize.pow(3) < MAX_JOINT_CELLS);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub const MAX_JOINT_CELLS: usize = 1_000_000;

/// Joint support size for `pool` (product of per-die support sizes), with overflow checks.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DicePool;
/// // `2d6` has 6 × 6 = 36 joint face pairs before summing.
/// let two_d6 = DicePool::from_count(2, 6).unwrap().sum().unwrap();
/// assert!((two_d6.pmf(7) - 6.0 / 36.0).abs() < 1e-12);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn joint_cell_count_pool(pool: &DicePool) -> Result<usize> {
    let mut cells = 1usize;
    for die in pool.dice() {
        let n = die.support_size();
        cells = cells.checked_mul(n).context("joint support overflow")?;
        if cells > MAX_JOINT_CELLS {
            bail!(
                "support explosion: pool has at least {cells} joint outcomes (max {MAX_JOINT_CELLS}); use a smaller pool"
            );
        }
    }
    Ok(cells)
}

/// Visit every face tuple for `n` identical fair `1..=sides` dice, each with probability `1/sides^n`.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DieRoll;
/// // Equivalent total distribution after visiting all 36 face pairs for `2d6`.
/// let two_d6 = DieRoll::pool_sum(2, 6).unwrap();
/// assert_eq!(two_d6.min(), Some(2));
/// assert!((two_d6.total_mass() - 1.0).abs() < 1e-9);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn for_each_uniform_joint(n: usize, sides: i64, mut f: impl FnMut(&[i64], f64)) -> Result<()> {
    if sides < 1 {
        bail!("sides must be >= 1");
    }
    if n == 0 {
        bail!("pool size must be >= 1");
    }
    let sides_u = usize::try_from(sides).context("sides too large for enumeration")?;
    let cells = sides_u.pow(n as u32);
    if cells > MAX_JOINT_CELLS {
        bail!(
            "support explosion: {n}d{sides} has {cells} outcomes (max {MAX_JOINT_CELLS}); use a smaller pool or simulation"
        );
    }
    let p_each = 1.0 / cells as f64;
    let mut faces = vec![1i64; n];
    loop {
        f(&faces, p_each);
        if !bump_faces(&mut faces, sides) {
            break;
        }
    }
    Ok(())
}

/// Visit every joint outcome of an independent [`DicePool`], with probability the product of per-die PMFs.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DicePool;
/// let pool = DicePool::from_count(2, 6).unwrap();
/// assert!((pool.sum().unwrap().total_mass() - 1.0_f64).abs() < 1e-9);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn for_each_joint(pool: &DicePool, mut f: impl FnMut(&[i64], f64)) -> Result<()> {
    let dice = pool.dice();
    if dice.is_empty() {
        bail!("empty pool");
    }
    joint_cell_count_pool(pool)?;

    let entries: Vec<Vec<(i64, f64)>> = dice.iter().map(|d| d.entries()).collect();
    let mut idx = vec![0usize; entries.len()];
    loop {
        let mut prob = 1.0;
        let mut faces = Vec::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            let (face, p) = e[idx[i]];
            faces.push(face);
            prob *= p;
        }
        f(&faces, prob);
        if !bump_indices(&mut idx, &entries) {
            break;
        }
    }
    Ok(())
}

fn bump_faces(faces: &mut [i64], sides: i64) -> bool {
    for f in faces.iter_mut() {
        if *f < sides {
            *f += 1;
            return true;
        }
        *f = 1;
    }
    false
}

fn bump_indices(idx: &mut [usize], entries: &[Vec<(i64, f64)>]) -> bool {
    for i in 0..idx.len() {
        idx[i] += 1;
        if idx[i] < entries[i].len() {
            return true;
        }
        idx[i] = 0;
    }
    false
}

/// Enumerate `pool`, using a fast path when every die is the same fair `nd`sides`.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DicePool;
/// // `3d6` enumeration feeds pool keep/drop helpers; sum checks normalization.
/// let three_d6 = DicePool::from_count(3, 6).unwrap().sum().unwrap();
/// assert!((three_d6.mean() - 10.5).abs() < 1e-9);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn for_each_pool_joint(pool: &DicePool, f: impl FnMut(&[i64], f64)) -> Result<()> {
    if let Some((n, sides)) = pool.uniform_fair_params() {
        for_each_uniform_joint(n, sides, f)
    } else {
        for_each_joint(pool, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::DicePool;

    #[test]
    fn uniform_joint_count_3d6() {
        let mut n = 0usize;
        for_each_uniform_joint(3, 6, |_, _| n += 1).unwrap();
        assert_eq!(n, 216);
    }

    #[test]
    fn pool_joint_matches_uniform() {
        let pool = DicePool::from_count(2, 6).unwrap();
        let mut sum_prob = 0.0;
        for_each_pool_joint(&pool, |_, p| sum_prob += p).unwrap();
        assert!((sum_prob - 1.0).abs() < 1e-9);
    }
}
