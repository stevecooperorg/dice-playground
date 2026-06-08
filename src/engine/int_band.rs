//! Inclusive integer bands for face restriction and labeled bucketing.
//!
//! Bands use **inclusive** endpoints. Open ends use `None` for the missing bound
//! (`..6` is “at most 6”; `10..` is “at least 10”).

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Inclusive integer interval, optionally open on one or both ends.
///
/// [`IntBand::unbounded`] is a sentinel: a scale label with no numeric band
/// (use `classify` for naturals and similar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntBand {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl IntBand {
    /// Closed inclusive interval `[lo, hi]`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::IntBand;
    /// let b = IntBand::through(6, 94).unwrap();
    /// assert!(b.contains(6));
    /// assert!(b.contains(94));
    /// assert!(!b.contains(5));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn through(lo: i64, hi: i64) -> Result<Self> {
        if lo > hi {
            bail!("through: lower bound must be <= upper bound, got {lo}..{hi}");
        }
        Ok(Self {
            min: Some(lo),
            max: Some(hi),
        })
    }

    /// All integers at or below `hi` (inclusive).
    pub fn at_most(hi: i64) -> Self {
        Self {
            min: None,
            max: Some(hi),
        }
    }

    /// All integers at or above `lo` (inclusive).
    pub fn at_least(lo: i64) -> Self {
        Self {
            min: Some(lo),
            max: None,
        }
    }

    /// No numeric band (label-only on a scale; never matches a total in `bucket`).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::IntBand;
    /// let u = IntBand::unbounded();
    /// assert!(u.is_unbounded());
    /// assert!(!u.contains(1));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn unbounded() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    /// True when this band does not restrict numeric totals.
    pub fn is_unbounded(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    /// True if `x` lies in this band (inclusive bounds where present).
    pub fn contains(&self, x: i64) -> bool {
        if self.is_unbounded() {
            return false;
        }
        if let Some(lo) = self.min {
            if x < lo {
                return false;
            }
        }
        if let Some(hi) = self.max {
            if x > hi {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn through_and_contains() {
        let b = IntBand::through(7, 9).unwrap();
        assert!(b.contains(7));
        assert!(b.contains(9));
        assert!(!b.contains(6));
        assert!(!b.contains(10));
    }

    #[test]
    fn open_ends() {
        assert!(IntBand::at_most(6).contains(-100));
        assert!(IntBand::at_most(6).contains(6));
        assert!(!IntBand::at_most(6).contains(7));
        assert!(IntBand::at_least(10).contains(10));
        assert!(!IntBand::at_least(10).contains(9));
    }

    #[test]
    fn unbounded_never_contains() {
        let u = IntBand::unbounded();
        assert!(!u.contains(0));
        assert!(!u.contains(20));
    }

    #[test]
    fn unbounded_serde_round_trip() {
        let u = IntBand::unbounded();
        let json = serde_json::to_string(&u).unwrap();
        let back: IntBand = serde_json::from_str(&json).unwrap();
        assert_eq!(back, u);
    }
}
