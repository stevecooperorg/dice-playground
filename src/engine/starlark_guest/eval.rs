use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

use super::super::{Counterbalance, Die, Dist, LabelDist, ResultScale, RollPool, successes_dist};
use anyhow::Context;
use starlark::values::list::AllocList;
use serde::Serialize;
use starlark::any::ProvidesStaticType;
use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect, DialectTypes};
use starlark::values::float::StarlarkFloat;
use starlark::values::none::NoneType;
use starlark::values::list::UnpackList;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{UnpackValue, Value, ValueLike};

use super::dist_value::StarlarkDist;
use super::label_value::StarlarkLabelDist;
use super::pool_value::StarlarkRollPool;
use super::prob_table_value::StarlarkProbTable;
use super::output_format::{
    format_dist_pmf_text, format_ordinal_pmf_text, format_prob_multi_column, format_prob_table_text,
    infer_sample_space_denominator, infer_sample_space_denominator_probs, ProbFormat,
};
use super::scale_value::StarlarkResultScale;

/// Collector populated by `output()` during evaluation.
#[derive(Debug, Default, ProvidesStaticType)]
pub struct OutputStore(pub RefCell<Vec<OutputEntry>>);

/// One recorded `output()` call.
#[derive(Clone, Debug, Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum OutputEntry {
    #[serde(rename = "dist")]
    Dist {
        name: String,
        entries: Vec<(i64, f64)>,
        mean: f64,
    },
    #[serde(rename = "prob")]
    Prob { name: String, value: f64 },
    #[serde(rename = "ordinal")]
    Ordinal {
        name: String,
        scale: Vec<String>,
        entries: Vec<(String, f64)>,
    },
    #[serde(rename = "table")]
    Table {
        name: String,
        entries: Vec<(String, f64)>,
    },
}

impl OutputStore {
    fn push_dist(&self, name: String, dist: &Dist) {
        self.0.borrow_mut().push(OutputEntry::Dist {
            name,
            entries: dist.entries(),
            mean: dist.mean(),
        });
    }

    fn push_prob(&self, name: String, value: f64) {
        self.0.borrow_mut().push(OutputEntry::Prob { name, value });
    }

    fn push_ordinal(&self, name: String, dist: &LabelDist) {
        self.0.borrow_mut().push(OutputEntry::Ordinal {
            name,
            scale: dist.scale().labels().to_vec(),
            entries: dist.entries_ordered(),
        });
    }

    fn push_table(&self, name: String, rows: &[(String, f64)]) {
        self.0.borrow_mut().push(OutputEntry::Table {
            name,
            entries: rows.to_vec(),
        });
    }
}

/// Result of evaluating a script.
#[derive(Debug)]
pub struct EvalResult {
    pub return_value: String,
    pub outputs: Vec<OutputEntry>,
}

impl fmt::Display for EvalResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_eval_result_text(self, ProbFormat::Decimal))
    }
}

/// Format eval result for human-readable text (PMF tables show %, fraction, and X/denom count columns).
pub fn format_eval_result_text(result: &EvalResult, _prob: ProbFormat) -> String {
    let shared_sample_denom = sample_space_denom_for_eval(result);
    let mut out = String::new();
    let _ = writeln!(out, "return: {}", result.return_value);
    for entry in &result.outputs {
        match entry {
            OutputEntry::Dist {
                name,
                entries,
                mean,
            } => {
                let _ = write!(
                    out,
                    "{}",
                    format_dist_pmf_text(name, entries, *mean, _prob, shared_sample_denom)
                );
            }
            OutputEntry::Prob { name, value } => {
                let _ = write!(
                    out,
                    "{}",
                    format_prob_multi_column(name, *value, shared_sample_denom)
                );
            }
            OutputEntry::Ordinal { name, entries, .. } => {
                let _ = write!(
                    out,
                    "{}",
                    format_ordinal_pmf_text(name, entries, _prob, shared_sample_denom)
                );
            }
            OutputEntry::Table { name, entries } => {
                let _ = write!(
                    out,
                    "{}",
                    format_prob_table_text(name, entries, _prob, shared_sample_denom)
                );
            }
        }
    }
    out
}

fn sample_space_denom_for_eval(result: &EvalResult) -> Option<u64> {
    for entry in &result.outputs {
        match entry {
            OutputEntry::Dist { entries, .. } => {
                if let Some(d) = infer_sample_space_denominator(entries) {
                    return Some(d);
                }
            }
            OutputEntry::Ordinal { entries, .. } | OutputEntry::Table { entries, .. } => {
                let probs: Vec<f64> = entries.iter().map(|(_, p)| *p).collect();
                if let Some(d) = infer_sample_space_denominator_probs(&probs) {
                    return Some(d);
                }
            }
            OutputEntry::Prob { .. } => {}
        }
    }
    None
}

fn prob_from_value(v: Value<'_>) -> anyhow::Result<f64> {
    if let Some(f) = v.downcast_ref::<StarlarkFloat>() {
        return Ok(f.0);
    }
    if let Some(p) = v.unpack_i32() {
        return Ok(f64::from(p));
    }
    anyhow::bail!("expected float or int probability, got {v}")
}

fn parse_prob_table_rows(rows: UnpackList<Value<'_>>) -> anyhow::Result<Vec<(String, f64)>> {
    let mut out = Vec::with_capacity(rows.items.len());
    for (i, item) in rows.items.into_iter().enumerate() {
        let pair = UnpackTuple::<Value<'_>>::unpack_value(item)
            .map_err(starlark_err)?
            .with_context(|| format!("prob_table row {i}: expected (label, probability) tuple"))?;
        if pair.items.len() != 2 {
            anyhow::bail!(
                "prob_table row {i}: expected (label, probability) pair, got {} value(s)",
                pair.items.len()
            );
        }
        let label = pair.items[0]
            .unpack_str()
            .with_context(|| format!("prob_table row {i}: label must be a string"))?
            .to_owned();
        let p = prob_from_value(pair.items[1])?;
        out.push((label, p));
    }
    if out.is_empty() {
        anyhow::bail!("prob_table requires at least one row");
    }
    Ok(out)
}

/// Starlark dialect with type annotations enabled.
pub fn dice_dialect() -> Dialect {
    Dialect {
        enable_types: DialectTypes::Enable,
        enable_top_level_stmt: true,
        ..Dialect::Standard
    }
}

fn starlark_err(err: starlark::Error) -> anyhow::Error {
    anyhow::anyhow!("{err}")
}

/// Dice probability builtins for Starlark scripts.
///
/// Use `+` on `Dist` values to convolve (sum of independent outcomes).
/// Use `-` for independent differences (e.g. `2d10 - 3d6`). Use `shift` for flat modifiers.
/// Use `*` to scale every outcome (`1d4 * 10`). Use `//` for per-outcome floor division (`8d6 // 2`).
#[starlark_module]
pub(crate) fn dice_module(builder: &mut GlobalsBuilder) {
    /// Build a fair die with the given number of faces (1..=sides, uniform).
    ///
    /// # Arguments
    /// * `sides`: Number of faces (must be positive).
    ///
    /// # Returns
    /// A `Dist` for one die roll.
    #[starlark(as_type = StarlarkDist)]
    fn d(sides: i32) -> anyhow::Result<StarlarkDist> {
        Ok(StarlarkDist::new(Die::die(i64::from(sides))?))
    }

    /// Build a die from explicit face values (multiplicity = weight).
    #[starlark(as_type = StarlarkDist)]
    fn die_faces(faces: UnpackList<i32>) -> anyhow::Result<StarlarkDist> {
        let f: Vec<i64> = faces.items.into_iter().map(i64::from).collect();
        Ok(StarlarkDist::new(Die::from_faces(&f)?))
    }

    /// Explode on maximum face, summing rerolls up to `max_depth` times (default 2).
    #[starlark(as_type = StarlarkDist)]
    fn explode(dist: &StarlarkDist, #[starlark(default = 2)] max_depth: i32) -> anyhow::Result<StarlarkDist> {
        if max_depth < 0 {
            anyhow::bail!("max_depth must be >= 0");
        }
        Ok(StarlarkDist::new(
            dist.inner().explode(u32::try_from(max_depth).context("max_depth")?)?,
        ))
    }

    /// Independent fair dice pool (not summed).
    #[starlark(as_type = StarlarkRollPool)]
    fn roll_pool(count: i32, sides: i32) -> anyhow::Result<StarlarkRollPool> {
        let n = usize::try_from(count).context("roll_pool count must be non-negative")?;
        Ok(StarlarkRollPool::new(RollPool::from_count(n, i64::from(sides))?))
    }

    /// Alias for [`roll_pool`].
    #[starlark(as_type = StarlarkRollPool)]
    fn pool(count: i32, sides: i32) -> anyhow::Result<StarlarkRollPool> {
        let n = usize::try_from(count).context("roll_pool count must be non-negative")?;
        Ok(StarlarkRollPool::new(RollPool::from_count(n, i64::from(sides))?))
    }

    /// Collapse a pool to a total, or pass through a `Dist`.
    #[starlark(as_type = StarlarkDist)]
    fn sum(value: Value) -> anyhow::Result<StarlarkDist> {
        if let Some(pool) = value.downcast_ref::<StarlarkRollPool>() {
            return Ok(StarlarkDist::new(pool.inner().sum()?));
        }
        if let Some(dist) = value.downcast_ref::<StarlarkDist>() {
            return Ok(dist.clone());
        }
        anyhow::bail!("sum: expected RollPool or Dist, got {value}")
    }

    /// Distribution of how many dice in the pool are `>= threshold`.
    #[starlark(as_type = StarlarkDist)]
    fn count_ge(pool: &StarlarkRollPool, threshold: i32) -> anyhow::Result<StarlarkDist> {
        Ok(StarlarkDist::new(
            pool.inner().count_ge(i64::from(threshold))?,
        ))
    }

    /// Distribution of how many rolled faces appear in `values`.
    #[starlark(as_type = StarlarkDist)]
    fn count_in(pool: &StarlarkRollPool, values: UnpackList<i32>) -> anyhow::Result<StarlarkDist> {
        let vals: Vec<i64> = values.items.into_iter().map(i64::from).collect();
        Ok(StarlarkDist::new(pool.inner().count_in(&vals)?))
    }

    /// Distribution of the `k`th highest die (`k=1` is highest).
    #[starlark(as_type = StarlarkDist)]
    fn order_stat(pool: &StarlarkRollPool, k: i32) -> anyhow::Result<StarlarkDist> {
        let k = usize::try_from(k).context("k")?;
        Ok(StarlarkDist::new(pool.inner().order_stat(k)?))
    }

    /// Sum of the middle `keep` dice after sorting ascending.
    #[starlark(as_type = StarlarkDist)]
    fn middle_of(pool: &StarlarkRollPool, keep: i32) -> anyhow::Result<StarlarkDist> {
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDist::new(pool.inner().middle_of(k)?))
    }

    /// Map each joint pool outcome through `fn(faces: list[int]) -> int`.
    #[starlark(as_type = StarlarkDist)]
    fn pool_map<'v>(
        pool: &StarlarkRollPool,
        map_fn: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkDist> {
        use std::cell::RefCell;
        let heap = eval.heap();
        let mut mass = BTreeMap::new();
        let err = RefCell::new(None);
        let _ = super::super::enumerate::for_each_pool_joint(pool.inner(), |faces, p| {
            if err.borrow().is_some() {
                return;
            }
            let list_items: Vec<Value> = faces
                .iter()
                .filter_map(|&f| i32::try_from(f).ok().map(|x| heap.alloc(x)))
                .collect();
            if list_items.len() != faces.len() {
                *err.borrow_mut() = Some(anyhow::anyhow!("pool_map: face out of i32 range"));
                return;
            }
            let list = heap.alloc(AllocList(list_items));
            let out = match eval.eval_function(map_fn, &[list], &[]) {
                Ok(v) => v,
                Err(e) => {
                    *err.borrow_mut() = Some(starlark_err(e));
                    return;
                }
            };
            let v = match out.unpack_i32() {
                Some(x) => i64::from(x),
                None => {
                    *err.borrow_mut() =
                        Some(anyhow::anyhow!("pool_map: function must return int"));
                    return;
                }
            };
            *mass.entry(v).or_insert(0.0) += p;
        });
        if let Some(e) = err.into_inner() {
            return Err(e);
        }
        let mut die = Die::from_mass(mass);
        die.normalize_in_place()?;
        Ok(StarlarkDist::new(die))
    }

    /// WoD-style success count pool (even or max on d10, optional explode rules).
    #[starlark(as_type = StarlarkDist)]
    fn success_pool(
        count: i32,
        sides: i32,
        #[starlark(default = "baseline")] mode: &str,
    ) -> anyhow::Result<StarlarkDist> {
        let n = usize::try_from(count).context("count")?;
        let cb = match mode {
            "baseline" => Counterbalance::Baseline,
            "ones_cancel" => Counterbalance::OnesCancelExplosions,
            "ones_remove" => Counterbalance::OnesRemoveSuccess,
            "implode" => Counterbalance::OnesImplode,
            other => anyhow::bail!("success_pool: unknown mode {other:?}"),
        };
        Ok(StarlarkDist::new(successes_dist(i64::from(sides), n, cb)?))
    }

    /// Roll `count` dice, drop the `drop` lowest, sum the rest (e.g. 4d6 drop lowest 1).
    ///
    /// # Arguments
    /// * `count`: Dice in the pool.
    /// * `sides`: Faces per die.
    /// * `drop`: How many lowest results to remove before summing.
    fn drop_lowest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDist> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDist::new(Die::pool_drop_lowest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Roll `count` dice, keep the `keep` highest, sum them.
    ///
    /// # Arguments
    /// * `count`: Dice in the pool.
    /// * `sides`: Faces per die.
    /// * `keep`: How many highest results to sum.
    fn keep_highest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDist> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDist::new(Die::pool_keep_highest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Roll `count` dice, drop the `drop` highest, sum the rest.
    ///
    /// # Arguments
    /// * `count`: Dice in the pool.
    /// * `sides`: Faces per die.
    /// * `drop`: How many highest results to remove before summing.
    fn drop_highest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDist> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDist::new(Die::pool_drop_highest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Roll `count` dice, keep the `keep` lowest, sum them.
    ///
    /// # Arguments
    /// * `count`: Dice in the pool.
    /// * `sides`: Faces per die.
    /// * `keep`: How many lowest results to sum.
    fn keep_lowest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDist> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDist::new(Die::pool_keep_lowest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Add a constant to every outcome (modifier), without convolving with another die.
    ///
    /// # Arguments
    /// * `dist`: Input distribution.
    /// * `delta`: Amount to add to each outcome.
    fn shift(dist: &StarlarkDist, delta: i32) -> anyhow::Result<StarlarkDist> {
        Ok(StarlarkDist::new(dist.inner.shift(i64::from(delta))?))
    }

    /// Define an ordered enumerated result scale (low → high rank).
    ///
    /// # Arguments
    /// * `labels`: Unique non-empty strings in order from worst to best (or low to high).
    #[starlark(as_type = StarlarkResultScale)]
    fn result_type(labels: UnpackList<String>) -> anyhow::Result<StarlarkResultScale> {
        Ok(StarlarkResultScale::new(ResultScale::new(labels.items)?))
    }

    /// Partition a numeric `Dist` into ordered labels using upper-bound cuts.
    ///
    /// For `n` labels, pass `n - 1` strictly increasing cut values.
    /// Band 0: outcomes `<= cuts[0]`; middle bands between cuts; top band: outcomes above the last cut.
    #[starlark(as_type = StarlarkLabelDist)]
    fn bucket(
        dist: &StarlarkDist,
        scale: &StarlarkResultScale,
        cuts: UnpackList<i32>,
    ) -> anyhow::Result<StarlarkLabelDist> {
        let cuts: Vec<i64> = cuts.items.into_iter().map(i64::from).collect();
        Ok(StarlarkLabelDist::new(LabelDist::from_bucket(
            dist.inner(),
            scale.inner().clone(),
            &cuts,
        )?))
    }

    /// Map each outcome of `dist` through a Starlark function `(value) -> str`.
    ///
    /// Use for rules that depend on the **natural** roll (e.g. D&D nat 1 / nat 20) before or
    /// instead of simple numeric cuts. The function must return a label present on `scale`.
    #[starlark(as_type = StarlarkLabelDist)]
    fn classify<'v>(
        dist: &StarlarkDist,
        scale: &StarlarkResultScale,
        classify: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkLabelDist> {
        let scale_inner = scale.inner().clone();
        let mut mass = BTreeMap::new();
        let heap = eval.heap();
        for (x, p) in dist.inner().entries() {
            if p <= 0.0 {
                continue;
            }
            let x_val = heap.alloc(i32::try_from(x).context("outcome out of i32 range")?);
            let out = eval
                .eval_function(classify, &[x_val], &[])
                .map_err(starlark_err)?;
            let label = out
                .unpack_str()
                .context("classify: function must return string label")?
                .to_owned();
            scale_inner.rank(&label)?;
            *mass.entry(label).or_insert(0.0) += p;
        }
        Ok(StarlarkLabelDist::new(LabelDist::from_mass(scale_inner, mass)?))
    }

    /// Classify independent rolls `(d1, d2)` with a Starlark function `(w, b) -> str`.
    ///
    /// The function must return a label present on `scale`.
    #[starlark(as_type = StarlarkLabelDist)]
    fn joint_classify<'v>(
        d1: &StarlarkDist,
        d2: &StarlarkDist,
        scale: &StarlarkResultScale,
        classify: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkLabelDist> {
        let scale_inner = scale.inner().clone();
        let mut mass = BTreeMap::new();
        let pairs: Vec<(i64, i64, f64)> = d1
            .inner()
            .entries()
            .into_iter()
            .flat_map(|(w, pw)| {
                d2.inner()
                    .entries()
                    .into_iter()
                    .map(move |(b, pb)| (w, b, pw * pb))
            })
            .filter(|(_, _, p)| *p > 0.0)
            .collect();
        let heap = eval.heap();
        for (w, b, p) in pairs {
            let w_val = heap.alloc(i32::try_from(w).context("w out of range")?);
            let b_val = heap.alloc(i32::try_from(b).context("b out of range")?);
            let out = eval
                .eval_function(classify, &[w_val, b_val], &[])
                .map_err(starlark_err)?;
            let label = out
                .unpack_str()
                .context("joint_classify: function must return string label")?
                .to_owned();
            scale_inner.rank(&label)?;
            *mass.entry(label).or_insert(0.0) += p;
        }
        Ok(StarlarkLabelDist::new(LabelDist::from_mass(scale_inner, mass)?))
    }

    /// Build a multi-row probability table from `(label, probability)` tuples.
    ///
    /// Probabilities are independent (they need not sum to 1). Typical pattern: accumulate
    /// rows in a loop, then `output("name", prob_table(rows))` once.
    #[starlark(as_type = StarlarkProbTable)]
    fn prob_table(rows: UnpackList<Value<'_>>) -> anyhow::Result<StarlarkProbTable> {
        Ok(StarlarkProbTable::new(parse_prob_table_rows(rows)?))
    }

    /// Record a distribution or probability for playground output (text, json, graph).
    ///
    /// # Arguments
    /// * Optional name (`str`) and value: a `Dist`, `LabelDist`, `ProbTable`, `float`, or `int`.
    /// One argument records an anonymous name; two arguments are `output(name, value)`.
    fn output(
        #[starlark(args)] args: UnpackTuple<Value>,
        eval: &mut Evaluator,
    ) -> anyhow::Result<NoneType> {
        let store = eval
            .extra
            .and_then(|e| e.downcast_ref::<OutputStore>())
            .context("output store missing from evaluator")?;
        let items: Vec<Value<'_>> = args.items;
        let (name, value) = match items.len() {
            1 => {
                let v = items[0];
                (next_anon_name(store), v)
            }
            2 => {
                let v = items[1];
                let n = items[0]
                    .unpack_str()
                    .context("output name must be string")?
                    .to_owned();
                (n, v)
            }
            n => anyhow::bail!("output expects 1 or 2 arguments, got {n}"),
        };
        record_output(store, name, value)?;
        Ok(NoneType)
    }
}

/// Globals for dice scripts (standard Starlark + dice stdlib).
pub fn dice_globals() -> Globals {
    GlobalsBuilder::standard().with(dice_module).build()
}

fn next_anon_name(store: &OutputStore) -> String {
    let idx = store.0.borrow().len();
    format!("output_{idx}")
}

fn record_output(store: &OutputStore, name: String, value: Value<'_>) -> anyhow::Result<()> {
    if value.downcast_ref::<StarlarkRollPool>().is_some() {
        anyhow::bail!("output: expected Dist or LabelDist; got RollPool (call sum() first)");
    }
    if let Some(dist) = value.downcast_ref::<StarlarkDist>() {
        store.push_dist(name, dist.inner());
        return Ok(());
    }
    if let Some(ld) = value.downcast_ref::<StarlarkLabelDist>() {
        store.push_ordinal(name, ld.inner());
        return Ok(());
    }
    if let Some(table) = value.downcast_ref::<StarlarkProbTable>() {
        store.push_table(name, table.rows());
        return Ok(());
    }
    if let Some(f) = value.downcast_ref::<StarlarkFloat>() {
        store.push_prob(name, f.0);
        return Ok(());
    }
    if let Some(p) = value.unpack_i32() {
        store.push_prob(name, f64::from(p));
        return Ok(());
    }
    anyhow::bail!("output: expected Dist, LabelDist, ProbTable, float, or int, got {value}");
}

/// Parse and evaluate Starlark source with the dice standard library.
pub fn eval_source(path: &str, content: &str) -> anyhow::Result<EvalResult> {
    eval_source_with_dialect(path, content, &dice_dialect())
}

/// Parse and evaluate with a specific dialect (e.g. public playground without `load`).
pub fn eval_source_with_dialect(
    path: &str,
    content: &str,
    dialect: &Dialect,
) -> anyhow::Result<EvalResult> {
    let ast = AstModule::parse(path, content.to_owned(), dialect)
        .map_err(starlark_err)
        .with_context(|| format!("parse {path}"))?;
    let globals = dice_globals();
    let store = OutputStore::default();
    let return_value = Module::with_temp_heap(|module| -> anyhow::Result<String> {
        let mut eval = Evaluator::new(&module);
        eval.extra = Some(&store);
        let res: Value = eval
            .eval_module(ast, &globals)
            .map_err(starlark_err)
            .with_context(|| format!("eval {path}"))?;
        Ok(res.to_string())
    })?;
    let outputs = store.0.into_inner();
    Ok(EvalResult {
        return_value,
        outputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_two_d6_output() {
        let src = r#"
output("two_d6", d(6) + d(6))
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::Dist { name, mean, .. } => {
                assert_eq!(name, "two_d6");
                assert!((*mean - 7.0).abs() < 1e-9);
            }
            other => panic!("expected dist output, got {other:?}"),
        }
    }

    #[test]
    fn eval_dist_subtraction() {
        let src = r#"output("diff", pool(2, 10) - pool(3, 6))"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Dist { mean, .. } => assert!((*mean - 0.5).abs() < 1e-9),
            other => panic!("expected dist output, got {other:?}"),
        }
    }

    #[test]
    fn eval_bucket_and_ordinal_output() {
        let src = r#"
Outcome = result_type(["FAIL", "SUCCESS"])
roll = d(6)
out = bucket(roll, Outcome, [3])
output("bands", out)
output("p_success", out.p_at_least("SUCCESS"))
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 2);
        match &res.outputs[0] {
            OutputEntry::Ordinal {
                name,
                scale,
                entries,
            } => {
                assert_eq!(name, "bands");
                assert_eq!(scale, &["FAIL", "SUCCESS"]);
                assert_eq!(entries.len(), 2);
                let p_fail: f64 = entries.iter().find(|(l, _)| l == "FAIL").unwrap().1;
                assert!((p_fail - 0.5).abs() < 1e-9);
            }
            other => panic!("expected ordinal output, got {other:?}"),
        }
        match &res.outputs[1] {
            OutputEntry::Prob { name, value } => {
                assert_eq!(name, "p_success");
                assert!((*value - 0.5).abs() < 1e-9);
            }
            other => panic!("expected prob, got {other:?}"),
        }
    }

    #[test]
    fn eval_classify_d20_crit_bands() {
        let src = r#"
Outcome = result_type(["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"])
DC = 15
MOD = 0
def label(n):
    if n == 1:
        return "CRITICAL_FAIL"
    if n == 20:
        return "CRITICAL_SUCCESS"
    if n + MOD >= DC:
        return "SUCCESS"
    return "FAIL"
out = classify(d(20), Outcome, label)
output("check", out)
"#;
        let res = eval_source("test.star", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Ordinal { entries, .. } => {
                let p_crit_succ = entries
                    .iter()
                    .find(|(l, _)| l == "CRITICAL_SUCCESS")
                    .unwrap()
                    .1;
                assert!((p_crit_succ - 0.05).abs() < 1e-9);
            }
            other => panic!("expected ordinal, got {other:?}"),
        }
    }

    #[test]
    fn eval_joint_classify_two_d6() {
        let src = r#"
Outcome = result_type(["FAILURE", "MIXED", "SUCCESS"])
def label(w, b):
    if w >= 4 and b >= 4:
        return "SUCCESS"
    if w >= 4 and b <= 2:
        return "MIXED"
    return "FAILURE"
out = joint_classify(d(6), d(6), Outcome, label)
output("pbtA", out)
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::Ordinal { name, entries, .. } => {
                assert_eq!(name, "pbtA");
                assert_eq!(entries.len(), 3);
                let sum: f64 = entries.iter().map(|(_, p)| p).sum();
                assert!((sum - 1.0).abs() < 1e-9);
            }
            other => panic!("expected ordinal, got {other:?}"),
        }
    }

    #[test]
    fn eval_bucket_rejects_wrong_cut_count() {
        let src = r#"
Outcome = result_type(["A", "B", "C"])
out = bucket(d(6), Outcome, [3])
"#;
        let err = eval_source("test.star", src).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("bucket expects"));
    }

    #[test]
    fn eval_joint_classify_rejects_unknown_label() {
        let src = r#"
Outcome = result_type(["A", "B"])
def bad(w, b):
    return "Z"
joint_classify(d(2), d(2), Outcome, bad)
"#;
        let err = eval_source("test.star", src).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("unknown label"));
    }

    #[test]
    fn eval_drop_lowest_and_p_ge() {
        let src = r#"
roll = drop_lowest(4, 6, 1)
output("p18", roll.p_ge(18))
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::Prob { name, value } => {
                assert_eq!(name, "p18");
                assert!(*value > 0.0 && *value < 1.0);
            }
            other => panic!("expected prob, got {other:?}"),
        }
    }

    #[test]
    fn eval_pool_map_count_high_faces() {
        let src = r#"
def count_high(faces):
    return len([f for f in faces if f > 4])

output("counts", pool_map(roll_pool(3, 6), count_high))
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Dist { entries, .. } => {
                let p3: f64 = entries.iter().find(|(k, _)| *k == 3).map(|(_, p)| *p).unwrap_or(0.0);
                assert!((p3 - 1.0 / 27.0).abs() < 1e-9);
            }
            other => panic!("expected dist, got {other:?}"),
        }
    }

    #[test]
    fn eval_prob_table_from_loop_pattern() {
        let src = r#"
rows = []
for x in range(2):
    rows = rows + [("t{}".format(x), 0.5)]
output("grid", prob_table(rows))
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::Table { name, entries } => {
                assert_eq!(name, "grid");
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "t0");
                assert!((entries[0].1 - 0.5).abs() < 1e-9);
            }
            other => panic!("expected table output, got {other:?}"),
        }
    }
}
