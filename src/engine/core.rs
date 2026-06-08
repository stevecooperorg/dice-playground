//! Re-exports and tests for the PMF engine.

pub use super::die::Die;
pub use super::enumerate::MAX_JOINT_CELLS;
pub use super::pool::{PoolOp, RollPool};

/// Alias for [`Die`] (historical name).
pub type Dist = Die;

/// Total variation distance `0.5 * sum |p - q|`.
pub fn total_variation_distance(a: &Die, b: &Die) -> f64 {
    let keys: Vec<i64> = a
        .mass
        .keys()
        .chain(b.mass.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut tv = 0.0;
    for k in keys {
        tv += (a.pmf(k) - b.pmf(k)).abs();
    }
    0.5 * tv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d6_uniform_and_mean() {
        let d6 = Die::die(6).unwrap();
        let mut sum = 0.0_f64;
        for face in 1..=6 {
            sum += d6.pmf(face);
            assert!((d6.pmf(face) - 1.0 / 6.0).abs() < 1e-12);
        }
        assert!((sum - 1.0).abs() < 1e-12);
        assert!((d6.mean() - 3.5).abs() < 1e-9);
    }

    #[test]
    fn two_d10_minus_three_d6_mean() {
        let two_d10 = Die::pool_sum(2, 10).unwrap();
        let three_d6 = Die::pool_sum(3, 6).unwrap();
        let diff = two_d10.difference(&three_d6).unwrap();
        assert!((diff.mean() - 0.5).abs() < 1e-9);
        assert_eq!(diff.min(), Some(2 - 18));
        assert_eq!(diff.max(), Some(20 - 3));
    }

    #[test]
    fn two_d6_mean_is_seven() {
        let d6 = Die::die(6).unwrap();
        let two = d6.convolve(&d6).unwrap();
        assert!((two.mean() - 7.0).abs() < 1e-9);
        assert_eq!(two.min(), Some(2));
        assert_eq!(two.max(), Some(12));
    }

    #[test]
    fn two_d10_plus_five_shift() {
        let d10 = Die::die(10).unwrap();
        let two = d10.convolve(&d10).unwrap();
        let shifted = two.shift(5).unwrap();
        assert!((shifted.p_ge(15) - two.p_ge(10)).abs() < 1e-12);
        assert_eq!(shifted.min(), Some(7));
        assert_eq!(shifted.max(), Some(25));
    }

    #[test]
    fn four_d6_drop_lowest_matches_reference_mean() {
        let dist = Die::pool_drop_lowest(4, 6, 1).unwrap();
        assert!((dist.mean() - 12.244598765432098).abs() < 1e-9);
        assert_eq!(dist.min(), Some(3));
        assert_eq!(dist.max(), Some(18));
        assert!((dist.total_mass() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn four_d6_drop_highest_one_mean() {
        let dist = Die::pool_drop_highest(4, 6, 1).unwrap();
        assert!((dist.mean() - 8.755401234567925).abs() < 1e-9);
    }

    #[test]
    fn three_d12_keep_lowest_one_mean() {
        let dist = Die::pool_keep_lowest(3, 12, 1).unwrap();
        assert!((dist.mean() - 3.5208333333333326).abs() < 1e-9);
    }

    #[test]
    fn pool_keep_highest_two_of_four_d6() {
        let dist = Die::pool_keep_highest(4, 6, 2).unwrap();
        assert_eq!(dist.min(), Some(2));
        assert_eq!(dist.max(), Some(12));
        assert!((dist.total_mass() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cdf_and_p_ge_complement_on_d6() {
        let d6 = Die::die(6).unwrap();
        for k in 1..=6 {
            assert!((d6.cdf(k) + d6.p_ge(k + 1) - 1.0).abs() < 1e-12);
        }
    }
}
