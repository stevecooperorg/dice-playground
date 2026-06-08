//! Inclusive integer bands for face restriction and labeled bucketing.
//!
//! Bands use **inclusive** endpoints. Open ends use `None` for the missing bound
//! (`..6` is “at most 6”; `10..` is “at least 10”).

use anyhow::{bail, Result};

/// Inclusive integer interval, optionally open on one or both ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    /// True if `x` lies in this band (inclusive bounds where present).
    pub fn contains(&self, x: i64) -> bool {
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
}
