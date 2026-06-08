use super::int_band_value::StarlarkIntBand;
use crate::engine::{FaceSpec, OptionalFaceSpec};
use anyhow::{bail, Result};
use starlark::values::list::UnpackList;
use starlark::values::{UnpackValue, Value, ValueLike};

/// Parse a required FaceSpec argument for `keep` / `count` / face filters.
pub fn face_spec_from_value(v: Value<'_>) -> Result<FaceSpec> {
    face_spec_from_one(v)
}

/// Parse 0–1 FaceSpec arguments for `p_any` / `p_none` / `p_at_least`.
pub fn optional_face_spec_from_values(items: &[Value<'_>]) -> Result<OptionalFaceSpec> {
    match items.len() {
        0 => Ok(OptionalFaceSpec::LengthOnly),
        1 => Ok(OptionalFaceSpec::Spec(face_spec_from_one(items[0])?)),
        n => bail!("expected 0 or 1 face spec argument, got {n}"),
    }
}

fn face_spec_from_one(v: Value<'_>) -> Result<FaceSpec> {
    if let Some(band) = v.downcast_ref::<StarlarkIntBand>() {
        return Ok(FaceSpec::Band(band.inner()));
    }
    if let Some(i) = v.unpack_i32() {
        return Ok(FaceSpec::Faces(vec![i64::from(i)]));
    }
    if let Some(list) = UnpackList::<Value<'_>>::unpack_value_opt(v) {
        if list.items.is_empty() {
            bail!("face spec list must not be empty");
        }
        if list
            .items
            .first()
            .and_then(|x| x.downcast_ref::<StarlarkIntBand>())
            .is_some()
        {
            bail!("face spec: expected list of ints, got IntBand elements");
        }
        let mut faces = Vec::with_capacity(list.items.len());
        for (i, x) in list.items.iter().enumerate() {
            let face = x
                .unpack_i32()
                .ok_or_else(|| anyhow::anyhow!("face spec face {i}: expected int, got {x}"))?;
            faces.push(i64::from(face));
        }
        return Ok(FaceSpec::Faces(faces));
    }
    bail!("face spec: expected int, IntBand, or list of ints, got {v}")
}
