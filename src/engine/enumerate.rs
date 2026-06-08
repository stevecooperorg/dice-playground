//! Joint enumeration over dice pools.

use anyhow::{bail, Context, Result};

use super::RollPool;

pub const MAX_JOINT_CELLS: usize = 1_000_000;

pub fn joint_cell_count_pool(pool: &RollPool) -> Result<usize> {
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

/// Enumerate all joint outcomes of a uniform fair `n`d`sides` pool.
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

/// Enumerate all joint outcomes of an arbitrary independent pool.
pub fn for_each_joint(pool: &RollPool, mut f: impl FnMut(&[i64], f64)) -> Result<()> {
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

/// Fast path: uniform pool with identical fair dice.
pub fn for_each_pool_joint(pool: &RollPool, f: impl FnMut(&[i64], f64)) -> Result<()> {
    if let Some((n, sides)) = pool.uniform_fair_params() {
        for_each_uniform_joint(n, sides, f)
    } else {
        for_each_joint(pool, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::RollPool;

    #[test]
    fn uniform_joint_count_3d6() {
        let mut n = 0usize;
        for_each_uniform_joint(3, 6, |_, _| n += 1).unwrap();
        assert_eq!(n, 216);
    }

    #[test]
    fn pool_joint_matches_uniform() {
        let pool = RollPool::from_count(2, 6).unwrap();
        let mut sum_prob = 0.0;
        for_each_pool_joint(&pool, |_, p| sum_prob += p).unwrap();
        assert!((sum_prob - 1.0).abs() < 1e-9);
    }
}
