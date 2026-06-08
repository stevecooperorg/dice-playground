//! Named outcome bands (miss / partial / hit) instead of raw numeric totals.
//!
//! PbtA moves, crit bands, and similar rules care about **ordered labels**. A [`Scale`]
//! lists those labels low → high; [`Outcomes`] is a PMF over label names tied to that order.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::int_band::IntBand;
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Scale {
    labels: Vec<String>,
    bands: Vec<IntBand>,
    /// When true, this step is considered before non-early steps when bucketing (rank unchanged).
    early: Vec<bool>,
}

impl Scale {
    fn validate_labels(labels: &[String]) -> Result<()> {
        if labels.is_empty() {
            bail!("scale requires at least one label");
        }
        let mut seen = BTreeSet::new();
        for label in labels {
            if label.is_empty() {
                bail!("scale labels must be non-empty");
            }
            if !seen.insert(label.as_str()) {
                bail!("duplicate label in scale: {label}");
            }
        }
        Ok(())
    }

    /// Empty scale for fluent construction via [`Scale::with_step`] (Starlark: `scale().step(...)`).
    pub fn empty() -> Self {
        Self {
            labels: Vec::new(),
            bands: Vec::new(),
            early: Vec::new(),
        }
    }

    /// Append one label (low → high) and band. Set `early` so this band is checked before non-early steps when bucketing.
    pub fn with_step(mut self, label: String, band: IntBand, early: bool) -> Result<Self> {
        if label.is_empty() {
            bail!("scale labels must be non-empty");
        }
        if self.labels.iter().any(|l| l == &label) {
            bail!("duplicate label in scale: {label}");
        }
        self.labels.push(label);
        self.bands.push(band);
        self.early.push(early);
        Ok(self)
    }

    /// Build a scale from an ordered list of unique non-empty labels (unbounded bands).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::Scale;
    /// let scale = Scale::new(vec!["LOW".into(), "HIGH".into()]).unwrap();
    /// assert_eq!(scale.len(), 2);
    /// assert!(scale.band_at(0).unwrap().is_unbounded());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(labels: Vec<String>) -> Result<Self> {
        Self::validate_labels(&labels)?;
        let bands = vec![IntBand::unbounded(); labels.len()];
        let early = vec![false; labels.len()];
        Ok(Self {
            labels,
            bands,
            early,
        })
    }

    /// Build a scale with one inclusive band per label (use [`IntBand::unbounded`] for label-only steps).
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{IntBand, Scale};
    /// let scale = Scale::with_bands(
    ///     vec!["FAIL".into(), "PASS".into()],
    ///     vec![IntBand::at_most(14), IntBand::at_least(15)],
    /// )
    /// .unwrap();
    /// assert!(!scale.band_at(1).unwrap().is_unbounded());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn with_bands(labels: Vec<String>, bands: Vec<IntBand>) -> Result<Self> {
        Self::validate_labels(&labels)?;
        if bands.len() != labels.len() {
            bail!(
                "scale expects {} band(s) for {} label(s), got {}",
                labels.len(),
                labels.len(),
                bands.len()
            );
        }
        let early = vec![false; labels.len()];
        Ok(Self {
            labels,
            bands,
            early,
        })
    }

    /// Inclusive bands in rank order (parallel to [`Scale::labels`]).
    pub fn bands(&self) -> &[IntBand] {
        &self.bands
    }

    /// Per-step `early` flags (parallel to [`Scale::labels`]); default false when absent in JSON.
    pub fn early_flags(&self) -> &[bool] {
        &self.early
    }

    /// Band at `rank`, if in range.
    pub fn band_at(&self, rank: usize) -> Option<IntBand> {
        self.bands.get(rank).copied()
    }

    /// Band for `label` on this scale.
    pub fn band_for(&self, label: &str) -> Result<IntBand> {
        let r = self.rank(label)?;
        self.band_at(r)
            .with_context(|| format!("band missing for label: {label}"))
    }

    /// True when every label has an unbounded band (classify-only scales).
    pub fn all_bands_unbounded(&self) -> bool {
        self.bands.iter().all(|b| b.is_unbounded())
    }

    /// True when at least one label has a numeric band.
    pub fn has_bounded_bands(&self) -> bool {
        !self.all_bands_unbounded()
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

    /// Replace bands on a scale with the same labels (keeps `early` flags).
    pub fn with_bands_replaced(mut self, bands: Vec<IntBand>) -> Result<Self> {
        if bands.len() != self.labels.len() {
            bail!(
                "scale expects {} band(s) for {} label(s), got {}",
                self.labels.len(),
                self.labels.len(),
                bands.len()
            );
        }
        self.bands = bands;
        Ok(self)
    }
}

fn band_index_for_outcome(bands: &[IntBand], early: &[bool], x: i64) -> Option<usize> {
    for (i, band) in bands.iter().enumerate() {
        if !early.get(i).copied().unwrap_or(false) || band.is_unbounded() {
            continue;
        }
        if band.contains(x) {
            return Some(i);
        }
    }
    for (i, band) in bands.iter().enumerate() {
        if early.get(i).copied().unwrap_or(false) || band.is_unbounded() {
            continue;
        }
        if band.contains(x) {
            return Some(i);
        }
    }
    None
}

impl<'de> Deserialize<'de> for Scale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ScaleRaw {
            labels: Vec<String>,
            #[serde(default)]
            bands: Vec<IntBand>,
            #[serde(default)]
            early: Vec<bool>,
        }
        let raw = ScaleRaw::deserialize(deserializer)?;
        let mut scale = if raw.bands.is_empty() {
            Scale::new(raw.labels).map_err(serde::de::Error::custom)?
        } else {
            Scale::with_bands(raw.labels, raw.bands).map_err(serde::de::Error::custom)?
        };
        if !raw.early.is_empty() {
            if raw.early.len() != scale.labels.len() {
                return Err(serde::de::Error::custom(format!(
                    "scale early flags length {} does not match {} label(s)",
                    raw.early.len(),
                    scale.labels.len()
                )));
            }
            scale.early = raw.early;
        }
        Ok(scale)
    }
}

pub(crate) fn bands_from_cuts(cuts: &[i64], n_labels: usize) -> Result<Vec<IntBand>> {
    if n_labels == 0 {
        bail!("bands_from_cuts requires at least one label");
    }
    if n_labels == 1 {
        return Ok(vec![IntBand::through(i64::MIN, i64::MAX)?]);
    }
    if cuts.len() != n_labels - 1 {
        bail!(
            "bands_from_cuts expects {} cut(s) for {} label(s), got {}",
            n_labels - 1,
            n_labels,
            cuts.len()
        );
    }
    let mut bands = Vec::with_capacity(n_labels);
    bands.push(IntBand::at_most(cuts[0]));
    for w in cuts.windows(2) {
        bands.push(IntBand::through(w[0] + 1, w[1])?);
    }
    bands.push(IntBand::at_least(cuts[n_labels - 2] + 1));
    Ok(bands)
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
        let bands = bands_from_cuts(cuts, n)?;
        Self::from_scale(dist, scale.with_bands_replaced(bands)?)
    }

    /// Bucket a numeric [`DieRoll`] using the bands stored on `scale`.
    ///
    /// Unbounded bands never receive mass; every outcome must match exactly one bounded band.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, IntBand, Outcomes, Scale};
    /// let scale = Scale::with_bands(
    ///     vec!["FAIL".into(), "PASS".into()],
    ///     vec![IntBand::at_most(14), IntBand::at_least(15)],
    /// )
    /// .unwrap();
    /// let d20 = DieRoll::die(20).unwrap();
    /// let o = Outcomes::from_scale(&d20, scale).unwrap();
    /// assert!((o.p_exact("PASS").unwrap() - 6.0 / 20.0).abs() < 1e-12);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_scale(dist: &DieRoll, scale: Scale) -> Result<Self> {
        if scale.all_bands_unbounded() {
            bail!("scale has no numeric bands; use scale().step(label, band) or bucket(..., cuts)");
        }
        let bands = scale.bands();
        let early = &scale.early;
        let mut mass = BTreeMap::new();
        for (x, p) in dist.entries() {
            if p <= 0.0 {
                continue;
            }
            let idx = band_index_for_outcome(bands, early, x)
                .with_context(|| format!("outcome {x} is not covered by any band"))?;
            let label = scale
                .label_at(idx)
                .ok_or_else(|| anyhow::anyhow!("label index {idx} out of range"))?
                .to_owned();
            *mass.entry(label).or_insert(0.0) += p;
        }
        Self::validate_and_normalize(scale, mass)
    }

    /// Split a numeric total into named bands using one inclusive [`IntBand`] per scale label.
    ///
    /// Bands may overlap; **early** steps match first (in order), then other steps (in order). Gaps are errors.
    ///
    /// # Example
    ///
    /// ```
    /// use dice_playground::engine::{DieRoll, IntBand, Outcomes, Scale};
    /// let scale = Scale::new(vec!["MISS".into(), "PARTIAL".into(), "FULL".into()]).unwrap();
    /// let roll = DieRoll::pool_sum(2, 6).unwrap().shift(2).unwrap();
    /// let bands = [
    ///     IntBand::at_most(8),
    ///     IntBand::through(9, 11).unwrap(),
    ///     IntBand::at_least(12),
    /// ];
    /// let o = Outcomes::from_label_bands(&roll, scale, &bands).unwrap();
    /// assert!(o.p_exact("FULL").unwrap() > 0.0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_label_bands(dist: &DieRoll, scale: Scale, bands: &[IntBand]) -> Result<Self> {
        let n = scale.len();
        if bands.len() != n {
            bail!(
                "bucket expects {} band(s) for {} label(s), got {}",
                n,
                n,
                bands.len()
            );
        }
        Self::from_scale(dist, scale.with_bands_replaced(bands.to_vec())?)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn scale_from_d20_early(labels: &[String], dc: i64, mod_: i64) -> Result<Scale> {
        if labels.len() != 4 {
            bail!("expected 4 labels");
        }
        let t = dc - mod_;
        Scale::empty()
            .with_step(labels[0].clone(), IntBand::through(1, 1)?, true)?
            .with_step(labels[1].clone(), IntBand::at_most(t - 1), false)?
            .with_step(labels[2].clone(), IntBand::at_least(t), false)?
            .with_step(labels[3].clone(), IntBand::through(20, 20)?, true)
    }

    #[test]
    fn from_scale_early_steps_match_before_normal() {
        let scale = Scale::empty()
            .with_step("PIN".into(), IntBand::through(1, 1).unwrap(), true)
            .unwrap()
            .with_step("BROAD".into(), IntBand::through(1, 5).unwrap(), false)
            .unwrap();
        let d5 = DieRoll::die(5).unwrap();
        let o = Outcomes::from_scale(&d5, scale).unwrap();
        assert!((o.p_exact("PIN").unwrap() - 1.0 / 5.0).abs() < 1e-12);
        assert!((o.p_exact("BROAD").unwrap() - 4.0 / 5.0).abs() < 1e-12);
    }

    #[test]
    fn from_scale_early_pin_wins_despite_later_declaration() {
        let scale = Scale::empty()
            .with_step("BROAD".into(), IntBand::through(1, 5).unwrap(), false)
            .unwrap()
            .with_step("PIN".into(), IntBand::through(1, 1).unwrap(), true)
            .unwrap();
        let one = DieRoll::from_mass(BTreeMap::from([(1, 1.0)]));
        let o = Outcomes::from_scale(&one, scale).unwrap();
        assert!((o.p_exact("PIN").unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn from_scale_without_early_broad_first_mislabels_pin() {
        let scale = Scale::empty()
            .with_step("BROAD".into(), IntBand::through(1, 5).unwrap(), false)
            .unwrap()
            .with_step("PIN".into(), IntBand::through(1, 1).unwrap(), false)
            .unwrap();
        let one = DieRoll::from_mass(BTreeMap::from([(1, 1.0)]));
        let o = Outcomes::from_scale(&one, scale).unwrap();
        assert!((o.p_exact("BROAD").unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn early_d20_p_at_least_success_includes_crit_success() {
        let labels = vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ];
        let scale = scale_from_d20_early(&labels, 15, 5).unwrap();
        let o = Outcomes::from_scale(&DieRoll::die(20).unwrap(), scale).unwrap();
        let hit = o.p_exact("SUCCESS").unwrap() + o.p_exact("CRITICAL_SUCCESS").unwrap();
        assert!((o.p_at_least("SUCCESS").unwrap() - hit).abs() < 1e-12);
    }

    #[test]
    fn with_step_builds_pbta_scale() {
        let scale = Scale::empty()
            .with_step("MISS".into(), IntBand::at_most(6), false)
            .unwrap()
            .with_step("PARTIAL".into(), IntBand::through(7, 9).unwrap(), false)
            .unwrap()
            .with_step("FULL".into(), IntBand::at_least(10), false)
            .unwrap();
        assert_eq!(scale.len(), 3);
    }

    #[test]
    fn label_bands_match_cuts_for_d6() {
        let scale = Scale::new(vec!["LOW".into(), "MID".into(), "HIGH".into()]).unwrap();
        let d6 = DieRoll::die(6).unwrap();
        let by_cuts = Outcomes::from_bucket(&d6, scale.clone(), &[2, 4]).unwrap();
        let bands = [
            IntBand::at_most(2),
            IntBand::through(3, 4).unwrap(),
            IntBand::at_least(5),
        ];
        let by_bands = Outcomes::from_label_bands(&d6, scale, &bands).unwrap();
        for label in ["LOW", "MID", "HIGH"] {
            assert!(
                (by_cuts.p_exact(label).unwrap() - by_bands.p_exact(label).unwrap()).abs() < 1e-12
            );
        }
    }

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
    fn d20_adjusted_target_matches_classify_mod_zero() {
        let labels = vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ];
        let by_classify = {
            let scale = Scale::new(labels.clone()).unwrap();
            let d20 = DieRoll::die(20).unwrap();
            let dc = 15_i64;
            Outcomes::from_classify(&d20, scale, |n| {
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
            .unwrap()
        };
        let scale = scale_from_d20_early(&labels, 15, 0).unwrap();
        let by_bucket = Outcomes::from_scale(&DieRoll::die(20).unwrap(), scale).unwrap();
        for label in ["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"] {
            assert!(
                (by_classify.p_exact(label).unwrap() - by_bucket.p_exact(label).unwrap()).abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn d20_adjusted_target_when_t_at_most_two() {
        let labels = vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ];
        let scale = scale_from_d20_early(&labels, 10, 9).unwrap();
        let o = Outcomes::from_scale(&DieRoll::die(20).unwrap(), scale).unwrap();
        assert!((o.p_exact("CRITICAL_FAIL").unwrap() - 1.0 / 20.0).abs() < 1e-12);
        assert!((o.p_exact("FAIL").unwrap() - 0.0).abs() < 1e-12);
        assert!((o.p_exact("SUCCESS").unwrap() - 18.0 / 20.0).abs() < 1e-12);
    }

    #[test]
    fn d20_adjusted_target_when_t_at_least_twenty() {
        let labels = vec![
            "CRITICAL_FAIL".into(),
            "FAIL".into(),
            "SUCCESS".into(),
            "CRITICAL_SUCCESS".into(),
        ];
        let scale = scale_from_d20_early(&labels, 10, -15).unwrap();
        let o = Outcomes::from_scale(&DieRoll::die(20).unwrap(), scale).unwrap();
        assert!((o.p_exact("SUCCESS").unwrap() - 0.0).abs() < 1e-12);
        assert!((o.p_exact("FAIL").unwrap() - 18.0 / 20.0).abs() < 1e-12);
        assert!((o.p_exact("CRITICAL_SUCCESS").unwrap() - 1.0 / 20.0).abs() < 1e-12);
    }

    #[test]
    fn from_scale_matches_with_bands_on_scale() {
        let scale = Scale::with_bands(
            vec!["LOW".into(), "MID".into(), "HIGH".into()],
            vec![
                IntBand::at_most(2),
                IntBand::through(3, 4).unwrap(),
                IntBand::at_least(5),
            ],
        )
        .unwrap();
        let d6 = DieRoll::die(6).unwrap();
        let o = Outcomes::from_scale(&d6, scale).unwrap();
        assert!((o.p_exact("LOW").unwrap() - 2.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn from_scale_all_unbounded_errors() {
        let scale = Scale::new(vec!["A".into(), "B".into()]).unwrap();
        let d6 = DieRoll::die(6).unwrap();
        assert!(Outcomes::from_scale(&d6, scale).is_err());
    }

    #[test]
    fn with_bands_allows_overlap() {
        let scale = Scale::with_bands(
            vec!["A".into(), "B".into()],
            vec![
                IntBand::through(1, 5).unwrap(),
                IntBand::through(3, 6).unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(scale.len(), 2);
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
