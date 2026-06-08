//! Exact PMF for polyhedral explode pools (even or max = success; max explodes).

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::Dist;

const MAX_WAVES: usize = 48;

/// How rolls of **1** interact with total successes and exploding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Counterbalance {
    Baseline,
    OnesRemoveSuccess,
    OnesCancelExplosions,
    OnesImplode,
}

fn die_success(face: i64, sides: i64) -> bool {
    face % 2 == 0 || face == sides
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

/// Exact distribution of total successes (clamped at 0).
///
/// Exploding chains are truncated after [`MAX_WAVES`] waves; remaining probability
/// mass is finalized at the current success tally (negligible tail for typical dice).
pub fn successes_dist(sides: i64, n_dice: usize, mode: Counterbalance) -> Result<Dist> {
    if sides < 1 {
        bail!("sides must be >= 1");
    }
    if n_dice == 0 {
        return Ok(Dist::constant(0));
    }

    type Key = (usize, i32, usize, usize);
    let mut frontier: BTreeMap<Key, f64> = BTreeMap::new();
    frontier.insert((n_dice, 0, 0, 0), 1.0);
    let mut outcomes: BTreeMap<i32, f64> = BTreeMap::new();

    while let Some((key, prob)) = frontier.iter().next().map(|(k, v)| (*k, *v)) {
        frontier.remove(&key);
        let (pool, successes, ones_total, wave) = key;
        if pool == 0 || wave >= MAX_WAVES {
            finalize(mode, sides, successes, ones_total, prob, &mut outcomes);
            continue;
        }

        let p_each = prob / (sides as f64).powi(pool as i32);
        let mut faces = vec![1i64; pool];
        loop {
            let mut wave_success = 0i32;
            let mut wave_ones = 0usize;
            let mut max_faces = 0usize;
            for &r in &faces {
                if r == 1 {
                    wave_ones += 1;
                }
                if die_success(r, sides) {
                    wave_success += 1;
                }
                if r == sides {
                    max_faces += 1;
                }
            }
            let any_one = wave_ones > 0;
            let explode = match mode {
                Counterbalance::OnesCancelExplosions if any_one => 0,
                _ => max_faces,
            };
            let next_success = successes + wave_success;
            let next_ones = ones_total + wave_ones;
            if explode == 0 {
                finalize(mode, sides, next_success, next_ones, p_each, &mut outcomes);
            } else {
                *frontier
                    .entry((explode, next_success, next_ones, wave + 1))
                    .or_insert(0.0) += p_each;
            }
            if !bump_faces(&mut faces, sides) {
                break;
            }
        }
    }

    let mut mass: BTreeMap<i64, f64> = outcomes
        .into_iter()
        .map(|(k, p)| (i64::from(k), p))
        .collect();
    let total: f64 = mass.values().sum();
    if total <= 0.0 {
        bail!("poly_explode produced empty distribution");
    }
    if (total - 1.0).abs() > 1e-5 {
        for p in mass.values_mut() {
            *p /= total;
        }
    }
    Ok(Dist::from_mass(mass))
}

fn finalize(
    mode: Counterbalance,
    sides: i64,
    successes: i32,
    ones_total: usize,
    prob: f64,
    outcomes: &mut BTreeMap<i32, f64>,
) {
    match mode {
        Counterbalance::Baseline | Counterbalance::OnesCancelExplosions => {
            *outcomes.entry(successes.max(0)).or_insert(0.0) += prob;
        }
        Counterbalance::OnesRemoveSuccess => {
            let total = (successes - i32::try_from(ones_total).unwrap_or(i32::MAX)).max(0);
            *outcomes.entry(total).or_insert(0.0) += prob;
        }
        Counterbalance::OnesImplode => {
            apply_implode(sides, successes, ones_total, prob, outcomes);
        }
    }
}

fn apply_implode(
    sides: i64,
    successes: i32,
    ones_total: usize,
    prob: f64,
    outcomes: &mut BTreeMap<i32, f64>,
) {
    if ones_total == 0 {
        *outcomes.entry(successes.max(0)).or_insert(0.0) += prob;
        return;
    }
    let p_face = 1.0 / sides as f64;
    let mut pen_mass: BTreeMap<i32, f64> = BTreeMap::from([(0, 1.0)]);
    for _ in 0..ones_total {
        let mut next = BTreeMap::new();
        for (&pen, &p) in &pen_mass {
            for face in 1..=sides {
                let add = if face % 2 == 0 { 0 } else { 1 };
                *next.entry(pen + add).or_insert(0.0) += p * p_face;
            }
        }
        pen_mass = next;
    }
    for (&pen, &p) in &pen_mass {
        let total = (successes - pen).max(0);
        *outcomes.entry(total).or_insert(0.0) += prob * p;
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn baseline_two_d6_finite_mass() {
        let dist = successes_dist(6, 2, Counterbalance::Baseline).unwrap();
        assert!((dist.total_mass() - 1.0).abs() < 1e-5);
        assert!(dist.mean() > 0.5 && dist.mean() < 2.5);
    }

    #[test]
    fn baseline_mean_matches_monte_carlo_seed_42() {
        let exact = successes_dist(6, 2, Counterbalance::Baseline).unwrap();
        let exact_mean = exact.mean();
        let mut rng = StdRng::seed_from_u64(42);
        let trials = 10_000usize;
        let mut sum = 0i32;
        for _ in 0..trials {
            sum += mc_trial(6, 2, Counterbalance::Baseline, &mut rng);
        }
        let mc_mean = sum as f64 / trials as f64;
        assert!((exact_mean - mc_mean).abs() < 0.05);
    }

    fn mc_trial(sides: i64, n_dice: usize, mode: Counterbalance, rng: &mut StdRng) -> i32 {
        let mut pool = n_dice;
        let mut wave_successes = 0i32;
        let mut ones_total = 0usize;
        while pool > 0 {
            let mut wave_ones = 0usize;
            let mut max_faces = 0usize;
            for _ in 0..pool {
                let r = rng.random_range(1..=sides);
                if r == 1 {
                    wave_ones += 1;
                }
                if die_success(r, sides) {
                    wave_successes += 1;
                }
                if r == sides {
                    max_faces += 1;
                }
            }
            ones_total += wave_ones;
            let any_one = wave_ones > 0;
            let explode = match mode {
                Counterbalance::OnesCancelExplosions if any_one => 0,
                _ => max_faces,
            };
            pool = explode;
        }
        match mode {
            Counterbalance::Baseline | Counterbalance::OnesCancelExplosions => wave_successes,
            Counterbalance::OnesRemoveSuccess => (wave_successes - ones_total as i32).max(0),
            Counterbalance::OnesImplode => {
                let mut total = wave_successes;
                for _ in 0..ones_total {
                    let r = rng.random_range(1..=sides);
                    if r % 2 != 0 {
                        total -= 1;
                    }
                }
                total.max(0)
            }
        }
    }
}
