//! Face matchers for `keep`, `count`, and pool `p_*` helpers in scripts.

use super::int_band::IntBand;

/// Which die faces match (int, face list, or inclusive band / range sugar).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaceSpec {
    /// Face lies in this inclusive band (`through`, `at_least`, desugared `5..`, etc.).
    Band(IntBand),
    /// Face is one of these values (a single int in scripts becomes one element).
    Faces(Vec<i64>),
}

/// Optional matcher for pool probability helpers when the spec may be omitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionalFaceSpec {
    /// No face filter: check pool size only (deterministic).
    LengthOnly,
    Spec(FaceSpec),
}

impl FaceSpec {
    /// True if face value `k` matches this spec.
    pub fn matches(&self, k: i64) -> bool {
        match self {
            FaceSpec::Band(band) => band.contains(k),
            FaceSpec::Faces(values) => values.contains(&k),
        }
    }

    /// Keep only matching faces on a die, then renormalize.
    pub fn keep_die_roll(&self, die: &super::DieRoll) -> anyhow::Result<super::DieRoll> {
        match self {
            FaceSpec::Band(band) => die.keep_in_band(*band),
            FaceSpec::Faces(values) => die.keep_in_set(values),
        }
    }

    /// Drop matching faces on a die, then renormalize.
    pub fn remove_die_roll(&self, die: &super::DieRoll) -> anyhow::Result<super::DieRoll> {
        die.remove_faces_spec(self.clone())
    }

    /// Remap matching faces to `to` on a die (other faces unchanged).
    pub fn convert_die_roll(
        &self,
        die: &super::DieRoll,
        to: i64,
    ) -> anyhow::Result<super::DieRoll> {
        die.convert_faces_spec(self.clone(), to)
    }

    /// Remap matching faces to 0 (`convert(spec, 0)`).
    pub fn ignore_die_roll(&self, die: &super::DieRoll) -> anyhow::Result<super::DieRoll> {
        die.ignore_faces_spec(self.clone())
    }
}
