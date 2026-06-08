use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

use super::super::{successes_dist, Counterbalance, DicePool, DieRoll, IntBand, Outcomes, Scale};
use super::bucket_args::outcomes_from_bucket_args;
use anyhow::Context;
use serde::Serialize;
use starlark::any::ProvidesStaticType;
use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect, DialectTypes};
use starlark::values::float::StarlarkFloat;
use starlark::values::list::AllocList;
use starlark::values::list::UnpackList;
use starlark::values::none::NoneType;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{UnpackValue, Value, ValueLike};

use super::dice_pool_value::StarlarkDicePool;
use super::die_roll_value::StarlarkDieRoll;
use super::int_band_value::StarlarkIntBand;
use super::outcomes_value::StarlarkOutcomes;
use super::output_format::{
    format_dist_pmf_text, format_ordinal_pmf_text, format_prob_multi_column,
    format_prob_table_text, infer_sample_space_denominator, infer_sample_space_denominator_probs,
    ProbFormat,
};
use super::prob_table_value::StarlarkProbTable;
use super::scale_value::StarlarkScale;

/// Collector populated by `output()` during evaluation.
#[derive(Debug, Default, ProvidesStaticType)]
pub struct OutputStore(pub RefCell<Vec<OutputEntry>>);

/// One recorded `output()` call.
#[derive(Clone, Debug, Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum OutputEntry {
    #[serde(rename = "dieroll")]
    DieRoll {
        name: String,
        entries: Vec<(i64, f64)>,
        mean: f64,
    },
    #[serde(rename = "prob")]
    Prob { name: String, value: f64 },
    #[serde(rename = "outcomes")]
    Outcomes {
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
    fn push_die_roll(&self, name: String, dist: &DieRoll) {
        self.0.borrow_mut().push(OutputEntry::DieRoll {
            name,
            entries: dist.entries(),
            mean: dist.mean(),
        });
    }

    fn push_prob(&self, name: String, value: f64) {
        self.0.borrow_mut().push(OutputEntry::Prob { name, value });
    }

    fn push_outcomes(&self, name: String, dist: &Outcomes) {
        self.0.borrow_mut().push(OutputEntry::Outcomes {
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
    if result.return_value != "None" {
        let _ = writeln!(out, "return: {}", result.return_value);
    }
    for entry in &result.outputs {
        match entry {
            OutputEntry::DieRoll {
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
            OutputEntry::Outcomes { name, entries, .. } => {
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
            OutputEntry::DieRoll { entries, .. } => {
                if let Some(d) = infer_sample_space_denominator(entries) {
                    return Some(d);
                }
            }
            OutputEntry::Outcomes { entries, .. } | OutputEntry::Table { entries, .. } => {
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

/// Dice probability builtins (documented in the generated function reference).
#[starlark_module]
pub(crate) fn dice_module(builder: &mut GlobalsBuilder) {
    /// One fair die with faces 1 through `sides`, each equally likely.
    ///
    /// Same idea as `1d6` or `1d20` in dice notation. Example: `d(6)` for a d6, `d(20)` for a d20.
    ///
    /// # Arguments
    /// * `sides`: Number of faces (must be at least 1).
    #[starlark(as_type = StarlarkDieRoll)]
    fn d(sides: i32) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(DieRoll::die(i64::from(sides))?))
    }

    /// A die with custom face values (listed in order; duplicates count as extra weight).
    ///
    /// Use for dice that are not uniform—`die_faces([1, 2, 2, 3])` is twice as likely to show 2 as 1 or 3.
    ///
    /// # Arguments
    /// * `faces`: List of integer face values.
    #[starlark(as_type = StarlarkDieRoll)]
    fn die_faces(faces: UnpackList<i32>) -> anyhow::Result<StarlarkDieRoll> {
        let f: Vec<i64> = faces.items.into_iter().map(i64::from).collect();
        Ok(StarlarkDieRoll::new(DieRoll::from_faces(&f)?))
    }

    /// Exploding die: on the highest face, roll again and add, up to `max_depth` extra rolls (default 2).
    ///
    /// Common in games where max on a die triggers another die (Savage Worlds–style). Example:
    /// `explode(d(4))` for one exploding d4.
    ///
    /// # Arguments
    /// * `dist`: Usually a single die from `d(...)`.
    /// * `max_depth`: Cap on how many times the die can explode (0 = no explode).
    #[starlark(as_type = StarlarkDieRoll)]
    fn explode(
        dist: &StarlarkDieRoll,
        #[starlark(default = 2)] max_depth: i32,
    ) -> anyhow::Result<StarlarkDieRoll> {
        if max_depth < 0 {
            anyhow::bail!("max_depth must be >= 0");
        }
        Ok(StarlarkDieRoll::new(
            dist.inner()
                .explode(u32::try_from(max_depth).context("max_depth")?)?,
        ))
    }

    /// Rolemaster **open-ended roll** on **1–100** (d100): low open on **01–05**, high open on **96–00**;
    /// rerolls chain on **96–00** only. `max_chain` caps consecutive **96–00** rerolls (default 8).
    #[starlark(as_type = StarlarkDieRoll)]
    fn open_ended_d100(#[starlark(default = 8)] max_chain: i32) -> anyhow::Result<StarlarkDieRoll> {
        if max_chain < 0 {
            anyhow::bail!("max_chain must be >= 0");
        }
        Ok(StarlarkDieRoll::new(DieRoll::open_ended_d100(
            u32::try_from(max_chain).context("max_chain")?,
        )?))
    }

    /// Roll `count` separate fair dice—**not** added together yet.
    ///
    /// Use when the rule looks at individual results (highest die, count 10s, etc.). Add with
    /// `.sum()` or the `sum(...)` function when you only need the total. Example: `dice_pool(4, 6)` for four d6s.
    ///
    /// # Arguments
    /// * `count`: How many dice.
    /// * `sides`: Faces per die (each die is 1..=sides).
    #[starlark(as_type = StarlarkDicePool)]
    fn dice_pool(count: i32, sides: i32) -> anyhow::Result<StarlarkDicePool> {
        let n = usize::try_from(count).context("dice_pool count must be non-negative")?;
        Ok(StarlarkDicePool::new(DicePool::from_count(
            n,
            i64::from(sides),
        )?))
    }

    /// Total a dice pool, or leave a `DieRoll` unchanged.
    ///
    /// `sum(dice_pool(4, 6))` is the distribution of 4d6 summed—equivalent to `4d6` notation.
    /// If you already have a `DieRoll`, `sum` returns it as-is.
    #[starlark(as_type = StarlarkDieRoll)]
    fn sum(value: Value) -> anyhow::Result<StarlarkDieRoll> {
        if let Some(pool) = value.downcast_ref::<StarlarkDicePool>() {
            return Ok(StarlarkDieRoll::new(pool.inner().sum()?));
        }
        if let Some(dist) = value.downcast_ref::<StarlarkDieRoll>() {
            return Ok(dist.clone());
        }
        anyhow::bail!("sum: expected DicePool or DieRoll, got {value}")
    }

    /// How many dice in the pool match a face spec?
    ///
    /// The result is a `DieRoll` over counts (0, 1, 2, …). Same as `pool.count(spec)`.
    ///
    /// # Arguments
    /// * `pool`: From `dice_pool`.
    /// * `spec`: int face, list of ints, or `IntBand` / desugared range (e.g. `5..` for 5+).
    #[starlark(as_type = StarlarkDieRoll)]
    fn count(pool: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = super::face_spec::face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(pool.inner().count_faces(parsed)?))
    }

    /// The **k**th highest die in the pool (`k = 1` is the highest, `2` is second-highest, …).
    ///
    /// Blades in the Dark and similar games use the highest die; some rules use second-highest.
    ///
    /// # Arguments
    /// * `k`: Rank from the top (1 = best die).
    #[starlark(as_type = StarlarkDieRoll)]
    fn order_stat(pool: &StarlarkDicePool, k: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(k).context("k")?;
        Ok(StarlarkDieRoll::new(pool.inner().order_stat(k)?))
    }

    /// Sum the middle `keep` dice after sorting the pool low to high.
    ///
    /// Niche rules that drop extremes from both ends; less common than keep-highest / drop-lowest.
    ///
    /// # Arguments
    /// * `keep`: How many dice in the middle to sum.
    #[starlark(as_type = StarlarkDieRoll)]
    fn middle_of(pool: &StarlarkDicePool, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(pool.inner().middle_of(k)?))
    }

    /// Custom rule: for every way the pool can land, run your function on the list of faces and use its integer result.
    ///
    /// Advanced—use when no built-in pool helper fits (e.g. “sum only dice that matched another die”).
    /// The function receives one argument: the list of rolled values, sorted as the engine stores them.
    ///
    /// # Arguments
    /// * `map_fn`: Starlark function `(faces) -> int`.
    #[starlark(as_type = StarlarkDieRoll)]
    fn pool_map<'v>(
        pool: &StarlarkDicePool,
        map_fn: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkDieRoll> {
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
                    *err.borrow_mut() = Some(anyhow::anyhow!("pool_map: function must return int"));
                    return;
                }
            };
            *mass.entry(v).or_insert(0.0) += p;
        });
        if let Some(e) = err.into_inner() {
            return Err(e);
        }
        let mut die = DieRoll::from_mass(mass);
        die.normalize_in_place()?;
        Ok(StarlarkDieRoll::new(die))
    }

    /// Count **successes** on a dice pool (Storyteller / WoD-style d10 pools and variants).
    ///
    /// Returns a `DieRoll` over how many successes you rolled. `mode` controls 1s and 10s:
    /// `"baseline"` (default), `"ones_cancel"`, `"ones_remove"`, or `"implode"`.
    ///
    /// # Arguments
    /// * `count`: Dice in the pool.
    /// * `sides`: Usually 10 for classic WoD.
    /// * `mode`: How ones and explosions interact—match your table’s house rules.
    #[starlark(as_type = StarlarkDieRoll)]
    fn success_pool(
        count: i32,
        sides: i32,
        #[starlark(default = "baseline")] mode: &str,
    ) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let cb = match mode {
            "baseline" => Counterbalance::Baseline,
            "ones_cancel" => Counterbalance::OnesCancelExplosions,
            "ones_remove" => Counterbalance::OnesRemoveSuccess,
            "implode" => Counterbalance::OnesImplode,
            other => anyhow::bail!("success_pool: unknown mode {other:?}"),
        };
        Ok(StarlarkDieRoll::new(successes_dist(
            i64::from(sides),
            n,
            cb,
        )?))
    }

    /// Roll several dice, drop the lowest results, sum the rest—**4d6 drop lowest 1** is `drop_lowest(4, 6, 1)`.
    ///
    /// Same as `4d6dl1` in dice notation.
    ///
    /// # Arguments
    /// * `count`: Dice rolled.
    /// * `sides`: Faces per die.
    /// * `drop`: How many lowest dice to remove before summing.
    fn drop_lowest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_drop_lowest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Roll dice, keep only the highest few, sum those—**4d6 keep highest 3** is `keep_highest(4, 6, 3)` (`4d6kh3`).
    ///
    /// # Arguments
    /// * `count`: Dice rolled.
    /// * `sides`: Faces per die.
    /// * `keep`: How many highest dice to sum.
    fn keep_highest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_keep_highest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Roll dice, drop the highest results, sum the rest (`4d6dh1` notation).
    ///
    /// # Arguments
    /// * `count`: Dice rolled.
    /// * `sides`: Faces per die.
    /// * `drop`: How many highest dice to remove before summing.
    fn drop_highest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_drop_highest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Roll dice, keep only the lowest few, sum those (`4d6kl3` notation).
    ///
    /// # Arguments
    /// * `count`: Dice rolled.
    /// * `sides`: Faces per die.
    /// * `keep`: How many lowest dice to sum.
    fn keep_lowest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_keep_lowest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Add a flat modifier to every outcome—**+3 to the roll** without rolling another die.
    ///
    /// Same effect as `roll + 3` when `roll` is a `DieRoll`. Prefer `roll + 3` in scripts when it reads clearer.
    ///
    /// # Arguments
    /// * `dist`: The roll (e.g. `2d10` as a `DieRoll`).
    /// * `delta`: Modifier to add (can be negative).
    fn shift(dist: &StarlarkDieRoll, delta: i32) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(dist.inner.shift(i64::from(delta))?))
    }

    /// Name your outcome steps from worst to best (or low to high).
    ///
    /// Used with `bucket` or `classify`. Example: `scale(["MISS", "PARTIAL", "FULL"])`.
    ///
    /// # Arguments
    /// * `labels`: Unique non-empty strings, first = lowest rank, last = highest.
    #[starlark(as_type = StarlarkScale)]
    fn scale(labels: UnpackList<String>) -> anyhow::Result<StarlarkScale> {
        Ok(StarlarkScale::new(Scale::new(labels.items)?))
    }

    /// Inclusive closed integer interval (same as desugared `6..94`).
    #[starlark(as_type = StarlarkIntBand)]
    fn through(lo: i32, hi: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::through(
            i64::from(lo),
            i64::from(hi),
        )?))
    }

    /// All integers at or below `hi` (desugared `..hi`).
    #[starlark(as_type = StarlarkIntBand)]
    fn at_most(hi: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::at_most(i64::from(hi))))
    }

    /// All integers at or above `lo` (desugared `lo..`).
    #[starlark(as_type = StarlarkIntBand)]
    fn at_least(lo: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::at_least(i64::from(lo))))
    }

    /// Split a numeric total into named bands using DC-style cut points.
    ///
    /// With 4 labels you pass **3** cut numbers. Totals at or below the first cut get the first label;
    /// between cuts get middle labels; above the last cut get the top label. PbtA 2d6+stat moves often use this.
    ///
    /// # Arguments
    /// * `dist`: Numeric roll (e.g. `2d6 + stat`).
    /// * `scale`: From `scale(...)`.
    /// * `cuts`: Increasing thresholds between labels.
    #[starlark(as_type = StarlarkOutcomes)]
    fn bucket(
        dist: &StarlarkDieRoll,
        scale: &StarlarkScale,
        #[starlark(args)] spec: UnpackTuple<Value<'_>>,
    ) -> anyhow::Result<StarlarkOutcomes> {
        Ok(StarlarkOutcomes::new(outcomes_from_bucket_args(
            dist.inner(),
            scale.inner().clone(),
            spec.items,
        )?))
    }

    /// Label each **exact** roll value with your own rule—natural 1s, natural 20s, custom crits.
    ///
    /// Your function takes the numeric result and returns one of the strings on `scale`.
    /// Example: map only 1 and 20 to special labels, bucket everything else by total.
    ///
    /// # Arguments
    /// * `classify`: Starlark function `(value) -> str`.
    #[starlark(as_type = StarlarkOutcomes)]
    fn classify<'v>(
        dist: &StarlarkDieRoll,
        scale: &StarlarkScale,
        classify: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkOutcomes> {
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
        Ok(StarlarkOutcomes::new(Outcomes::from_mass(
            scale_inner,
            mass,
        )?))
    }

    /// Label outcomes that depend on **two** dice together—advantage, disadvantage, or paired rolls.
    ///
    /// Every combination of `d1` and `d2` is classified by your `(left, right) -> str` function.
    ///
    /// # Arguments
    /// * `d1`, `d2`: Independent rolls (e.g. two d20s for advantage).
    /// * `classify`: Starlark function `(w, b) -> str` returning a label on `scale`.
    #[starlark(as_type = StarlarkOutcomes)]
    fn joint_classify<'v>(
        d1: &StarlarkDieRoll,
        d2: &StarlarkDieRoll,
        scale: &StarlarkScale,
        classify: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<StarlarkOutcomes> {
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
        Ok(StarlarkOutcomes::new(Outcomes::from_mass(
            scale_inner,
            mass,
        )?))
    }

    /// One table of labeled probabilities—grids of “chance to hit DC X at modifier Y”.
    ///
    /// Each row is `(description, probability)`. Rows are **independent** (they do not have to add to 100%).
    /// Build a list in a loop, then pass it here once: `output("grid", prob_table(rows))`.
    ///
    /// # Arguments
    /// * `rows`: List of `(string, float)` pairs.
    #[starlark(as_type = StarlarkProbTable)]
    fn prob_table(rows: UnpackList<Value<'_>>) -> anyhow::Result<StarlarkProbTable> {
        Ok(StarlarkProbTable::new(parse_prob_table_rows(rows)?))
    }

    /// Send a result to the playground **Output** panel (text, json, and graph tabs).
    ///
    /// Almost every script should call this at least once. Pass a name and a value:
    /// a full distribution (`DieRoll`), named outcomes (`Outcomes`), a probability (`float`),
    /// or a table (`prob_table(...)`). One argument works but naming outputs helps you read results.
    ///
    /// # Arguments
    /// * `output("label", value)` — recommended.
    /// * `output(value)` — auto-generated name.
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
    if value.downcast_ref::<StarlarkDicePool>().is_some() {
        anyhow::bail!("output: expected DieRoll or Outcomes; got DicePool (call sum() first)");
    }
    if let Some(dist) = value.downcast_ref::<StarlarkDieRoll>() {
        store.push_die_roll(name, dist.inner());
        return Ok(());
    }
    if let Some(ld) = value.downcast_ref::<StarlarkOutcomes>() {
        store.push_outcomes(name, ld.inner());
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
    anyhow::bail!("output: expected DieRoll, Outcomes, ProbTable, float, or int, got {value}");
}

/// Parse and evaluate Starlark source with the dice standard library.
pub fn eval_source(path: &str, content: &str) -> anyhow::Result<EvalResult> {
    let expanded = super::super::desugar_if_needed(path, content)?;
    eval_source_with_dialect(path, &expanded, &dice_dialect())
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
            OutputEntry::DieRoll { name, mean, .. } => {
                assert_eq!(name, "two_d6");
                assert!((*mean - 7.0).abs() < 1e-9);
            }
            other => panic!("expected dieroll output, got {other:?}"),
        }
    }

    #[test]
    fn eval_dist_subtraction() {
        let src = r#"output("diff", dice_pool(2, 10) - dice_pool(3, 6))"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::DieRoll { mean, .. } => assert!((*mean - 0.5).abs() < 1e-9),
            other => panic!("expected dieroll output, got {other:?}"),
        }
    }

    #[test]
    fn eval_bucket_and_ordinal_output() {
        let src = r#"
Scale = scale(["FAIL", "SUCCESS"])
roll = d(6)
out = bucket(roll, Scale, [3])
output("bands", out)
output("p_success", out.p_at_least("SUCCESS"))
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 2);
        match &res.outputs[0] {
            OutputEntry::Outcomes {
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
            other => panic!("expected outcomes output, got {other:?}"),
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
Scale = scale(["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"])
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
out = classify(d(20), Scale, label)
output("check", out)
"#;
        let res = eval_source("test.star", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Outcomes { entries, .. } => {
                let p_crit_succ = entries
                    .iter()
                    .find(|(l, _)| l == "CRITICAL_SUCCESS")
                    .unwrap()
                    .1;
                assert!((p_crit_succ - 0.05).abs() < 1e-9);
            }
            other => panic!("expected outcomes, got {other:?}"),
        }
    }

    #[test]
    fn eval_joint_classify_two_d6() {
        let src = r#"
Scale = scale(["FAILURE", "MIXED", "SUCCESS"])
def label(w, b):
    if w >= 4 and b >= 4:
        return "SUCCESS"
    if w >= 4 and b <= 2:
        return "MIXED"
    return "FAILURE"
out = joint_classify(d(6), d(6), Scale, label)
output("pbtA", out)
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::Outcomes { name, entries, .. } => {
                assert_eq!(name, "pbtA");
                assert_eq!(entries.len(), 3);
                let sum: f64 = entries.iter().map(|(_, p)| p).sum();
                assert!((sum - 1.0).abs() < 1e-9);
            }
            other => panic!("expected outcomes, got {other:?}"),
        }
    }

    #[test]
    fn eval_bucket_rejects_wrong_cut_count() {
        let src = r#"
Scale = scale(["A", "B", "C"])
out = bucket(d(6), Scale, [3])
"#;
        let err = eval_source("test.star", src).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("bucket expects"));
    }

    #[test]
    fn eval_joint_classify_rejects_unknown_label() {
        let src = r#"
Scale = scale(["A", "B"])
def bad(w, b):
    return "Z"
joint_classify(d(2), d(2), Scale, bad)
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
    fn eval_clamp_on_shifted_pool() {
        let src = r#"
roll = (sum(dice_pool(3, 6)) + 5).clamp(3, 18)
output("capped", roll)
"#;
        let res = eval_source("test.star", src).expect("eval");
        assert_eq!(res.outputs.len(), 1);
        match &res.outputs[0] {
            OutputEntry::DieRoll { mean, .. } => {
                assert!(*mean > 8.0 && *mean < 18.0);
            }
            other => panic!("expected dist, got {other:?}"),
        }
    }

    #[test]
    fn eval_pool_map_count_high_faces() {
        let src = r#"
def count_high(faces):
    return len([f for f in faces if f > 4])

output("counts", pool_map(dice_pool(3, 6), count_high))
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::DieRoll { entries, .. } => {
                let p3: f64 = entries
                    .iter()
                    .find(|(k, _)| *k == 3)
                    .map(|(_, p)| *p)
                    .unwrap_or(0.0);
                assert!((p3 - 1.0 / 27.0).abs() < 1e-9);
            }
            other => panic!("expected dist, got {other:?}"),
        }
    }

    #[test]
    fn eval_pool_keep_faces_sum() {
        let src = r#"
total = dice_pool(3, 6).keep(5..).sum()
output("high_sum", total)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::DieRoll { entries, .. } => {
                assert!(entries.iter().all(|(k, _)| *k >= 15));
            }
            other => panic!("expected dist, got {other:?}"),
        }
    }

    #[test]
    fn eval_pool_ignore_faces_sum_includes_zero() {
        let src = r#"
total = dice_pool(3, 6).ignore(1..4).sum()
output("ignored_sum", total)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::DieRoll { entries, mean, .. } => {
                assert!(entries.iter().any(|(k, p)| *k == 0 && *p > 0.0));
                assert!((*mean - 5.5).abs() < 1e-9);
            }
            other => panic!("expected dist, got {other:?}"),
        }
    }

    #[test]
    fn eval_pbta_bucket_bands_matches_cuts() {
        let src = r#"
Scale = scale(["MISS", "PARTIAL", "FULL"])
STAT = 2
roll = sum(dice_pool(2, 6)) + STAT
by_cuts = bucket(roll, Scale, [6, 9])
by_bands = roll.bucket(Scale, at_most(6), through(7, 9), at_least(10))
output("cuts", by_cuts)
output("bands", by_bands)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        let cuts = match &res.outputs[0] {
            OutputEntry::Outcomes { entries, .. } => entries.clone(),
            other => panic!("expected outcomes, got {other:?}"),
        };
        let bands = match &res.outputs[1] {
            OutputEntry::Outcomes { entries, .. } => entries.clone(),
            other => panic!("expected outcomes, got {other:?}"),
        };
        assert_eq!(cuts, bands);
    }

    #[test]
    fn eval_range_desugar_in_bucket() {
        let src = r#"
Scale = scale(["LOW", "HIGH"])
out = bucket(d(6) + 3, Scale, ..5, 6..)
output("x", out)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Outcomes { entries, .. } => {
                let p_low = entries.iter().find(|(l, _)| l == "LOW").unwrap().1;
                assert!((p_low - 2.0 / 6.0).abs() < 1e-9);
            }
            other => panic!("expected outcomes, got {other:?}"),
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

    #[test]
    fn eval_pool_any_natural_one() {
        let src = r#"
p = dice_pool(2, 6).p_any(1)
output("any_one", p)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::Prob { value, .. } => {
                assert!((*value - 11.0 / 36.0).abs() < 1e-9);
            }
            other => panic!("expected prob output, got {other:?}"),
        }
    }
}
