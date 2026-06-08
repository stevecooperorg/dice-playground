//! Ordered enumerated result scales and label distributions.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::Dist;

/// User-defined ordered scale (low → high rank).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultScale {
    labels: Vec<String>,
}

impl ResultScale {
    /// Build a scale from an ordered list of unique non-empty labels.
    pub fn new(labels: Vec<String>) -> Result<Self> {
        if labels.is_empty() {
            bail!("result_type requires at least one label");
        }
        let mut seen = BTreeSet::new();
        for label in &labels {
            if label.is_empty() {
                bail!("result_type labels must be non-empty");
            }
            if !seen.insert(label.as_str()) {
                bail!("duplicate label in result_type: {label}");
            }
        }
        Ok(Self { labels })
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn rank(&self, label: &str) -> Result<usize> {
        self.labels
            .iter()
            .position(|l| l == label)
            .with_context(|| format!("unknown label: {label}"))
    }

    pub fn label_at(&self, rank: usize) -> Option<&str> {
        self.labels.get(rank).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// PMF over labels bound to a [`ResultScale`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabelDist {
    scale: ResultScale,
    mass: BTreeMap<String, f64>,
}

impl LabelDist {
    pub fn scale(&self) -> &ResultScale {
        &self.scale
    }

    /// Build from label masses (validated against scale, normalized to sum 1).
    pub fn from_mass(scale: ResultScale, mass: BTreeMap<String, f64>) -> Result<Self> {
        Self::validate_and_normalize(scale, mass)
    }

    fn validate_and_normalize(scale: ResultScale, mass: BTreeMap<String, f64>) -> Result<Self> {
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

    /// Map numeric outcomes into ordered bands.
    ///
    /// `cuts` has length `scale.len() - 1`, strictly increasing.
    /// Band 0: `x <= cuts[0]`; band `i` for `0 < i < n-1`: `cuts[i-1]+1 <= x <= cuts[i]`;
    /// top band: `x >= cuts[n-2]+1`.
    pub fn from_bucket(dist: &Dist, scale: ResultScale, cuts: &[i64]) -> Result<Self> {
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
            let label = scale.label_at(idx).ok_or_else(|| {
                anyhow::anyhow!("bucket_index {idx} out of range for scale len {}", scale.len())
            })?.to_owned();
            *mass.entry(label).or_insert(0.0) += p;
        }
        Self::validate_and_normalize(scale, mass)
    }

    /// Map each numeric outcome through `classify(x) -> label` and accumulate mass.
    pub fn from_classify<F>(dist: &Dist, scale: ResultScale, classify: F) -> Result<Self>
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

    /// Independent pair `(d1, d2)` classified by `classify(w, b) -> label`.
    pub fn from_joint<F>(d1: &Dist, d2: &Dist, scale: ResultScale, classify: F) -> Result<Self>
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

    pub fn pmf(&self, label: &str) -> Result<f64> {
        self.scale.rank(label)?;
        Ok(*self.mass.get(label).unwrap_or(&0.0))
    }

    pub fn p_exact(&self, label: &str) -> Result<f64> {
        self.pmf(label)
    }

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
        let scale = ResultScale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
        let d6 = Dist::die(6).unwrap();
        let ld = LabelDist::from_bucket(&d6, scale, &[2, 4]).unwrap();
        assert!((ld.p_exact("LOW").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_exact("MID").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_exact("HIGH").unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ld.p_at_least("MID").unwrap() - 4.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn classify_natural_d20_crit_bands() {
        let scale = ResultScale::new(vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ])
        .unwrap();
        let d20 = Dist::die(20).unwrap();
        let dc = 15_i64;
        let ld = LabelDist::from_classify(&d20, scale, |n| {
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
        let scale = ResultScale::new(vec!["A".into(), "B".into()]).unwrap();
        let d2 = Dist::die(2).unwrap();
        let ld = LabelDist::from_joint(&d2, &d2, scale, |w, b| {
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
