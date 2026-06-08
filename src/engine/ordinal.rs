//! Named outcome bands (miss / partial / hit) instead of raw numeric totals.
//!
//! PbtA moves, crit bands, and similar rules care about **ordered labels**. A [`Scale`]
//! lists those labels low → high; [`Outcomes`] is a PMF over label names tied to that order.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::DieRoll;

/// User-defined ordered list of outcome names (lowest rank first).
///
/// # Example
///
/// ```
/// use dice_playground::engine::Scale;
/// let scale = Scale::new(vec!["MISS".into(), "HIT".into()]).unwrap();
/// assert_eq!(scale.rank("HIT").unwrap(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scale {
    labels: Vec<String>,
}

impl Scale {
    /// Build a scale from an ordered list of unique non-empty labels.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// let scale = Scale::new(vec!["LOW".into(), "HIGH".into()]).unwrap();
    /// assert_eq!(scale.len(), 2);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(labels: Vec<String>) -> Result<Self> {
        if labels.is_empty() {
            bail!("scale requires at least one label");
        }
        let mut seen = BTreeSet::new();
        for label in &labels {
            if label.is_empty() {
                bail!("scale labels must be non-empty");
            }
            if !seen.insert(label.as_str()) {
                bail!("duplicate label in scale: {label}");
            }
        }
        Ok(Self { labels })
    }

    /// Labels in rank order (index 0 is lowest).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
    /// assert_eq!(scale.labels()[0], "A");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Zero-based rank of `label` on this scale.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
    /// assert_eq!(scale.rank("B").unwrap(), 1);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn rank(&self, label: &str) -> Result<usize> {
        self.labels
            .iter()
            .position(|l| l == label)
            .with_context(|| format!("unknown label: {label}"))
    }

    /// Label at `rank`, if in range.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// let scale = Scale::new(vec!["A".into()]).unwrap();
    /// assert_eq!(scale.label_at(0), Some("A"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn label_at(&self, rank: usize) -> Option<&str> {
        self.labels.get(rank).map(String::as_str)
    }

    /// Number of labels on the scale.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// assert_eq!(Scale::new(vec!["X".into()]).unwrap().len(), 1);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// True when the scale has no labels (only possible on failed construction paths).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// assert!(!Scale::new(vec!["X".into()]).unwrap().is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// Exact probabilities for named outcomes on a fixed [`Scale`].
///
/// # Example
///
/// ```
/// use dice_playground::engine::{DieRoll, Outcomes, Scale};
/// use std::collections::BTreeMap;
/// let scale = Scale::new(vec!["LOW".into(), "HIGH".into()]).unwrap();
/// let mut mass = BTreeMap::new();
/// mass.insert("LOW".into(), 0.25);
/// mass.insert("HIGH".into(), 0.75);
/// let o = Outcomes::from_mass(scale, mass).unwrap();
/// assert!((o.p_exact("HIGH").unwrap() - 0.75).abs() < 1e-12);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outcomes {
    scale: Scale,
    mass: BTreeMap<String, f64>,
}

impl Outcomes {
    /// The ordered label list this distribution respects.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{Outcomes, Scale};
    /// use std::collections::BTreeMap;
    /// let scale = Scale::new(vec!["A".into()]).unwrap();
    /// let o = Outcomes::from_mass(scale.clone(), BTreeMap::from([("A".into(), 1.0)])).unwrap();
    /// assert_eq!(o.scale().labels(), scale.labels());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn scale(&self) -> &Scale {
        &self.scale
    }

    /// Build from label masses (validated against scale, normalized to sum 1).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{Outcomes, Scale};
    /// use std::collections::BTreeMap;
    /// let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
    /// let o = Outcomes::from_mass(scale, BTreeMap::from([("A".into(), 1.0)])).unwrap();
    /// assert!((o.p_exact("A").unwrap() - 1.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_mass(scale: Scale, mass: BTreeMap<String, f64>) -> Result<Self> {
        Self::validate_and_normalize(scale, mass)
    }

    fn validate_and_normalize(scale: Scale, mass: BTreeMap<String, f64>) -> Result<Self> {
        for key in mass.keys() {
            scale.rank(key)?;
        }
        let total: f64 = mass.values().sum();
        if total <= 0.0 {
            bail!("label distribution has no mass");
        }
        let mass = mass.into_iter().map(|(k, p)| (k, p / total)).collect();
        Ok(Self { scale, mass })
    }

    /// Bucket a numeric [`DieRoll`] into ordered bands using cut points.
    ///
    /// `cuts` has length `scale.len() - 1` and must be strictly increasing.
    /// Lowest band: totals `≤ cuts[0]`; middle bands between cuts; top band above the last cut.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
    /// let d6 = DieRoll::die(6).unwrap();
    /// let o = Outcomes::from_bucket(&d6, scale, &[2, 4]).unwrap();
    /// assert!((o.p_exact("MID").unwrap() - 2.0 / 6.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_bucket(dist: &DieRoll, scale: Scale, cuts: &[i64]) -> Result<Self> {
        let n = scale.len();
        if cuts.len() != n.saturating_sub(1) {
            bail!(
                "bucket expects {} cut(s) for {} label(s), got {}",
                n.saturating_sub(1),
                n,
                cuts.len()
            );
        }
        for w in cuts.windows(2) {
            if w[0] >= w[1] {
                bail!("bucket cuts must be strictly increasing");
            }
        }
        let mut mass = BTreeMap::new();
        for (x, p) in dist.entries() {
            if p <= 0.0 {
                continue;
            }
            let idx = bucket_index(x, cuts, n);
            let label = scale
                .label_at(idx)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bucket_index {idx} out of range for scale len {}",
                        scale.len()
                    )
                })?
                .to_owned();
            *mass.entry(label).or_insert(0.0) += p;
        }
        Self::validate_and_normalize(scale, mass)
    }

    /// Map each numeric outcome through `classify(x) -> label` and accumulate mass.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["EVEN".into(), "ODD".into()]).unwrap();
    /// let d6 = DieRoll::die(6).unwrap();
    /// let o = Outcomes::from_classify(&d6, scale, |n| {
    ///     if n % 2 == 0 { "EVEN".into() } else { "ODD".into() }
    /// }).unwrap();
    /// assert!((o.p_exact("EVEN").unwrap() - 0.5).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_classify<F>(dist: &DieRoll, scale: Scale, classify: F) -> Result<Self>
    where
        F: Fn(i64) -> String,
    {
        let mut mass = BTreeMap::new();
        for (x, p) in dist.entries() {
            if p <= 0.0 {
                continue;
            }
            let label = classify(x);
            scale.rank(&label)?;
            *mass.entry(label).or_insert(0.0) += p;
        }
        Self::validate_and_normalize(scale, mass)
    }

    /// Classify independent rolls `(d1, d2)` with `classify(a, b) -> label`.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
    /// let d2 = DieRoll::die(2).unwrap();
    /// let o = Outcomes::from_joint(&d2, &d2, scale, |a, b| {
    ///     if a + b >= 3 { "B".into() } else { "A".into() }
    /// }).unwrap();
    /// assert!((o.p_exact("B").unwrap() - 0.75).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_joint<F>(d1: &DieRoll, d2: &DieRoll, scale: Scale, classify: F) -> Result<Self>
    where
        F: Fn(i64, i64) -> String,
    {
        let mut mass = BTreeMap::new();
        for (w, pw) in d1.entries() {
            for (b, pb) in d2.entries() {
                let p = pw * pb;
                if p <= 0.0 {
                    continue;
                }
                let label = classify(w, b);
                scale.rank(&label)?;
                *mass.entry(label).or_insert(0.0) += p;
            }
        }
        Self::validate_and_normalize(scale, mass)
    }

    /// Probability of exactly this label (0 if never assigned).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["LOW".into(), "HIGH".into()]).unwrap();
    /// let d6 = DieRoll::die(6).unwrap();
    /// let o = Outcomes::from_bucket(&d6, scale, &[3]).unwrap();
    /// assert!((o.pmf("LOW").unwrap() - 0.5).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn pmf(&self, label: &str) -> Result<f64> {
        self.scale.rank(label)?;
        Ok(*self.mass.get(label).unwrap_or(&0.0))
    }

    /// Alias for [`Outcomes::pmf`] (exact label probability).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{Outcomes, Scale};
    /// use std::collections::BTreeMap;
    /// let scale = Scale::new(vec!["A".into()]).unwrap();
    /// let o = Outcomes::from_mass(scale, BTreeMap::from([("A".into(), 1.0)])).unwrap();
    /// assert_eq!(o.p_exact("A").unwrap(), 1.0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_exact(&self, label: &str) -> Result<f64> {
        self.pmf(label)
    }

    /// Probability of this label **or any higher** rank on the scale.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
    /// let o = Outcomes::from_bucket(&DieRoll::die(6).unwrap(), scale, &[2, 4]).unwrap();
    /// assert!((o.p_at_least("MID").unwrap() - 4.0 / 6.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_at_least(&self, label: &str) -> Result<f64> {
        let r = self.scale.rank(label)?;
        Ok(self
            .scale
            .labels()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= r)
            .map(|(_, l)| self.mass.get(l).copied().unwrap_or(0.0))
            .sum())
    }

    /// Probability of this label **or any lower** rank on the scale.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, Outcomes, Scale};
    /// let scale = Scale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
    /// let o = Outcomes::from_bucket(&DieRoll::die(6).unwrap(), scale, &[2, 4]).unwrap();
    /// assert!((o.p_at_most("MID").unwrap() - 4.0 / 6.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn p_at_most(&self, label: &str) -> Result<f64> {
        let r = self.scale.rank(label)?;
        Ok(self
            .scale
            .labels()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i <= r)
            .map(|(_, l)| self.mass.get(l).copied().unwrap_or(0.0))
            .sum())
    }

    /// `(label, probability)` pairs in scale order (missing labels show 0).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{Outcomes, Scale};
    /// use std::collections::BTreeMap;
    /// let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
    /// let o = Outcomes::from_mass(scale, BTreeMap::from([("B".into(), 1.0)])).unwrap();
    /// assert_eq!(o.entries_ordered()[0].0, "A");
    /// assert_eq!(o.entries_ordered()[0].1, 0.0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn entries_ordered(&self) -> Vec<(String, f64)> {
        self.scale
            .labels()
            .iter()
            .map(|l| (l.clone(), self.mass.get(l).copied().unwrap_or(0.0)))
            .collect()
    }
}

fn bucket_index(x: i64, cuts: &[i64], n_labels: usize) -> usize {
    if n_labels <= 1 {
        return 0;
    }
    if x <= cuts[0] {
        return 0;
    }
    for (i, cut) in cuts.iter().enumerate().take(n_labels - 1).skip(1) {
        if x <= *cut {
            return i;
        }
    }
    n_labels - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_three_bands() {
        let scale = Scale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
        let d6 = DieRoll::die(6).unwrap();
        let ld = Outcomes::from_bucket(&d6, scale, &[2, 4]).unwrap();
        assert!((ld.p_exact("LOW").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_exact("MID").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_exact("HIGH").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_at_least("MID").unwrap() - 4.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn classify_natural_d20_crit_bands() {
        let scale = Scale::new(vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ])
        .unwrap();
        let d20 = DieRoll::die(20).unwrap();
        let dc = 15_i64;
        let ld = Outcomes::from_classify(&d20, scale, |n| {
            if n == 1 {
                return "CRITICAL_FAIL".into();
            }
            if n == 20 {
                return "CRITICAL_SUCCESS".into();
            }
            if n >= dc {
                "SUCCESS".into()
            } else {
                "FAIL".into()
            }
        })
        .unwrap();
        assert!((ld.p_exact("CRITICAL_FAIL").unwrap() - 1.0 / 20.0).abs() < 1e-12);
        assert!((ld.p_exact("CRITICAL_SUCCESS").unwrap() - 1.0 / 20.0).abs() < 1e-12);
        assert!((ld.p_exact("SUCCESS").unwrap() - 5.0 / 20.0).abs() < 1e-12);
    }

    #[test]
    fn joint_classify_toy() {
        let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
        let d2 = DieRoll::die(2).unwrap();
        let ld = Outcomes::from_joint(&d2, &d2, scale, |w, b| {
            if w + b >= 3 {
                "B".into()
            } else {
                "A".into()
            }
        })
        .unwrap();
        assert!((ld.p_exact("A").unwrap() - 0.25).abs() < 1e-12);
        assert!((ld.p_exact("B").unwrap() - 0.75).abs() < 1e-12);
    }
}
