use super::super::{DieRoll, Outcomes, Scale};
use super::int_band_value::StarlarkIntBand;
use anyhow::{bail, Context, Result};
use starlark::values::list::UnpackList;
use starlark::values::{UnpackValue, Value, ValueLike};

fn expand_bucket_spec_items(items: Vec<Value<'_>>) -> Vec<Value<'_>> {
    if items.len() == 1 {
        if let Some(inner) = UnpackList::<Value<'_>>::unpack_value_opt(items[0]) {
            if inner
                .items
                .first()
                .and_then(|v| v.downcast_ref::<StarlarkIntBand>())
                .is_none()
                && inner.items.iter().all(|v| v.unpack_i32().is_some())
            {
                return inner.items;
            }
        }
    }
    items
}

/// `bucket(roll, scale, [cuts])` or `bucket(roll, scale, band, …)` after range desugar.
pub fn outcomes_from_bucket_args(
    dist: &DieRoll,
    scale: Scale,
    spec: Vec<Value<'_>>,
) -> Result<Outcomes> {
    let items = expand_bucket_spec_items(spec);
    if items.is_empty() {
        bail!("bucket requires cut list or one IntBand per label");
    }
    if items[0].downcast_ref::<StarlarkIntBand>().is_some() {
        let mut bands = Vec::with_capacity(items.len());
        for (i, v) in items.iter().enumerate() {
            let band = v
                .downcast_ref::<StarlarkIntBand>()
                .with_context(|| format!("bucket band {i}: expected IntBand, got {v}"))?;
            bands.push(band.inner());
        }
        return Outcomes::from_label_bands(dist, scale, &bands);
    }
    let cuts: Vec<i64> = items
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.unpack_i32()
                .with_context(|| format!("bucket cut {i}: expected int, got {v}"))
                .map(i64::from)
        })
        .collect::<Result<_>>()?;
    Outcomes::from_bucket(dist, scale, &cuts)
}
