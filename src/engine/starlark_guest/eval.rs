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
    format_dist_pmf_gfm, format_dist_pmf_text, format_ordinal_pmf_gfm, format_ordinal_pmf_text,
    format_prob_gfm, format_prob_multi_column, format_prob_table_gfm, format_prob_table_text,
    infer_sample_space_denominator, infer_sample_space_denominator_probs, ProbFormat,
};
use super::prob_table_value::StarlarkProbTable;
use super::scale_value::StarlarkScale;

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

/// Collector populated by `output()` during evaluation.
#[derive(Debug, Default, ProvidesStaticType)]
pub struct OutputStore(pub RefCell<Vec<OutputEntry>>);

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

/// GFM markdown for eval outputs (woven report HTML).
pub fn format_eval_result_markdown(result: &EvalResult, prob_format: ProbFormat) -> String {
    let shared_sample_denom = shared_sample_space_for_outputs(&result.outputs);
    let mut out = String::new();
    if result.return_value != "None" {
        let _ = writeln!(out, "return: {}\n", result.return_value);
    }
    for entry in &result.outputs {
        let block = format_output_entry_markdown(entry, prob_format, shared_sample_denom);
        if !block.is_empty() {
            out.push_str(&block);
            if !block.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
    }
    out
}

/// GFM markdown for a single `output()` block.
pub fn format_output_entry_markdown(
    entry: &OutputEntry,
    prob_format: ProbFormat,
    shared_sample_denom: Option<u64>,
) -> String {
    match entry {
        OutputEntry::DieRoll {
            name,
            entries,
            mean,
        } => format_dist_pmf_gfm(name, entries, *mean, prob_format, shared_sample_denom),
        OutputEntry::Prob { name, value } => {
            format_prob_gfm(name, *value, prob_format, shared_sample_denom)
        }
        OutputEntry::Outcomes { name, entries, .. } => {
            format_ordinal_pmf_gfm(name, entries, prob_format, shared_sample_denom)
        }
        OutputEntry::Table { name, entries } => {
            format_prob_table_gfm(name, entries, prob_format, shared_sample_denom)
        }
    }
}

/// Shared sample-space denominator for a batch of outputs (PMF / ordinal / table).
pub fn shared_sample_space_for_outputs(outputs: &[OutputEntry]) -> Option<u64> {
    sample_space_denom_for_eval(&EvalResult {
        return_value: "None".into(),
        outputs: outputs.to_vec(),
    })
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
    /// Make one fair die: every face has the same chance.
    ///
    /// `d(6)` means a six-sided die, just like `1d6` in a `.dice` script.
    /// It returns a `DieRoll`: all possible results and their chances, not one random roll.
    /// Give that result to `output` to see its table.
    ///
    /// ```dice
    /// output("One six-sided die", d(6))
    /// ```
    ///
    /// # Arguments
    /// * `sides`: Number of faces, from 1 upwards. Faces are numbered 1 through this number.
    #[starlark(as_type = StarlarkDieRoll)]
    fn d(sides: i32) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(DieRoll::die(i64::from(sides))?))
    }

    /// Make a die with numbers of your choosing on its faces.
    ///
    /// Write the numbers inside square brackets, separated by commas. This is a **list**.
    /// Each entry is one equally likely face, so repeating a number makes it more likely.
    /// The example has four faces: 2 is twice as likely as either 1 or 3.
    /// The result is a `DieRoll` you can add to other rolls or show with `output`.
    ///
    /// ```dice
    /// output("Custom die", die_faces([1, 2, 2, 3]))
    /// ```
    ///
    /// # Arguments
    /// * `faces`: A non-empty list of whole numbers. Zero, negative numbers, and repeats are allowed.
    #[starlark(as_type = StarlarkDieRoll)]
    fn die_faces(faces: UnpackList<i32>) -> anyhow::Result<StarlarkDieRoll> {
        let f: Vec<i64> = faces.items.into_iter().map(i64::from).collect();
        Ok(StarlarkDieRoll::new(DieRoll::from_faces(&f)?))
    }

    /// Roll again on the highest result and add the extra roll to the total.
    ///
    /// This is an **exploding die**. On a d6, a 6 earns another d6; another 6 can
    /// earn another roll. The result is a `DieRoll` of totals, including those above 6.
    /// To keep the calculation finite, extra rolls stop at `max_depth` even if the last die is a 6.
    /// This models a capped rule, not an unlimited chain.
    ///
    /// ```dice
    /// output("Exploding d6, at most two extra rolls", explode(d(6), max_depth=2))
    /// ```
    ///
    /// # Arguments
    /// * `dist`: The roll to repeat. Usually one die, such as `d(6)`. For a total, only its highest possible total triggers another roll of that whole total.
    /// * `max_depth`: Maximum extra rolls, 0 or more. Defaults to 2 when omitted; 0 leaves the roll unchanged.
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

    /// Make a Rolemaster-style d100 roll that can finish below 1 or above 100.
    ///
    /// A first roll of 6–95 stands as it is. On 1–5, roll again and subtract;
    /// on 96–100, roll again and add. Further rolls continue only on 96–100,
    /// keeping the same subtracting or adding direction. A printed 00 means 100 here.
    /// The result is a `DieRoll` of final totals.
    ///
    /// ```dice
    /// output("Open-ended d100", open_ended_d100())
    /// ```
    ///
    /// # Arguments
    /// * `max_chain`: Maximum additional high-result continuations after the first extra roll. Defaults to 8; must be 0 or more. Even 0 allows the first extra roll. The cap makes this a finite version of the rule.
    #[starlark(as_type = StarlarkDieRoll)]
    fn open_ended_d100(#[starlark(default = 8)] max_chain: i32) -> anyhow::Result<StarlarkDieRoll> {
        if max_chain < 0 {
            anyhow::bail!("max_chain must be >= 0");
        }
        Ok(StarlarkDieRoll::new(DieRoll::open_ended_d100(
            u32::try_from(max_chain).context("max_chain")?,
        )?))
    }

    /// Make several dice that you can examine separately before adding them.
    ///
    /// The result is a `DicePool`. Use a pool when your rule counts successes or
    /// chooses the highest die. The dice have not been added together yet.
    /// Write `.sum()` after the pool to turn it into a `DieRoll` that `output` can show.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6)
    /// output("Total of three dice", pool.sum())
    /// output("Number of sixes", pool.count(6))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Number of dice, at least 1.
    /// * `sides`: Number of faces on each die, at least 1. Every die has equally likely faces numbered 1 through this number.
    #[starlark(as_type = StarlarkDicePool)]
    fn dice_pool(count: i32, sides: i32) -> anyhow::Result<StarlarkDicePool> {
        let n = usize::try_from(count).context("dice_pool count must be non-negative")?;
        Ok(StarlarkDicePool::new(DicePool::from_count(
            n,
            i64::from(sides),
        )?))
    }

    /// Add the dice in a pool to get the chances for each possible total.
    ///
    /// The result is a `DieRoll`; for four d6, the totals run from 4 to 24.
    /// You can also write `pool.sum()`: the dot means “use this operation on this pool”.
    /// This dice helper does not add an ordinary list of numbers.
    ///
    /// ```dice
    /// pool = dice_pool(4, 6)
    /// output("Four dice added together", sum(pool))
    /// ```
    ///
    /// # Arguments
    /// * `value`: A `DicePool` to total, or a `DieRoll` to return unchanged.
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

    /// Count how many dice show a matching face, rather than adding their values.
    ///
    /// If each 5 or 6 is a success, three dice can give 0, 1, 2, or 3 successes.
    /// The returned `DieRoll` gives the chance of each count. `pool.count(spec)`
    /// is another way to write the same operation.
    ///
    /// ```dice
    /// pool = dice_pool(3, 6)
    /// output("Successes on 5 or 6", count(pool, [5, 6]))
    /// ```
    ///
    /// # Arguments
    /// * `pool`: Dice made with `dice_pool`.
    /// * `spec`: Faces that count: one number such as `6`, a non-empty list such as `[5, 6]`, or a range such as `at_least(5)` (also written `5..` in `.dice` scripts).
    #[starlark(as_type = StarlarkDieRoll)]
    fn count(pool: &StarlarkDicePool, spec: Value<'_>) -> anyhow::Result<StarlarkDieRoll> {
        let parsed = super::face_spec::face_spec_from_value(spec)?;
        Ok(StarlarkDieRoll::new(pool.inner().count_faces(parsed)?))
    }

    /// Choose one die by its position from highest to lowest.
    ///
    /// Use 1 for the highest die, 2 for the second-highest, and so on.
    /// Ties still occupy separate positions: in 6, 6, 2 the second-highest is 6.
    /// The result is a `DieRoll` of the chosen value, not a sum. You can also write `pool.order_stat(k)`.
    ///
    /// ```dice
    /// output("Highest of three d6", order_stat(dice_pool(3, 6), 1))
    /// ```
    ///
    /// # Arguments
    /// * `pool`: The dice to compare.
    /// * `k`: Position counting from 1 at the top; cannot exceed the number of dice.
    #[starlark(as_type = StarlarkDieRoll)]
    fn order_stat(pool: &StarlarkDicePool, k: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(k).context("k")?;
        Ok(StarlarkDieRoll::new(pool.inner().order_stat(k)?))
    }

    /// Add the middle dice, leaving out the lowest and highest results.
    ///
    /// With three dice and `keep=1`, this chooses the middle die. With five dice
    /// and `keep=3`, it adds the middle three. The result is a `DieRoll` of totals.
    /// If the number left out is odd, one more die is removed from the high end
    /// than from the low end. You can also write `pool.middle_of(keep)`.
    ///
    /// ```dice
    /// output("Middle of three d6", middle_of(dice_pool(3, 6), 1))
    /// ```
    ///
    /// # Arguments
    /// * `pool`: The dice to sort and choose from.
    /// * `keep`: How many middle dice to add, from 1 to the number of dice.
    #[starlark(as_type = StarlarkDieRoll)]
    fn middle_of(pool: &StarlarkDicePool, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(pool.inner().middle_of(k)?))
    }

    /// Work out the chances for a custom rule that looks at all the dice together.
    ///
    /// Try helpers such as `count` and `order_stat` first. When they do not fit,
    /// write a small function: `def` names your rule, and `return` supplies its answer.
    /// The engine calls it for every possible combination and collects the answers
    /// into a `DieRoll`. Large pools can take a long time because every combination is checked.
    ///
    /// Here doubles score 2 and everything else scores 0. Square brackets pick a
    /// die from the list: `[0]` is the first, `[1]` the second. `==` asks whether they are equal.
    /// Keep the indentation shown: it marks which lines belong to the rule and the `if`.
    ///
    /// ```dice
    /// def score_doubles(faces):
    ///     if faces[0] == faces[1]:
    ///         return 2
    ///     return 0
    ///
    /// output("Doubles score", pool_map(dice_pool(2, 6), score_doubles))
    /// ```
    ///
    /// # Arguments
    /// * `pool`: The dice your rule examines.
    /// * `map_fn`: Your function's name, without calling it with parentheses. It receives a list of faces in pool order (not sorted) and must return a whole number.
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

    /// Count successes for a specific rule: even faces and the highest face succeed; highest faces also roll again.
    ///
    /// On a d6, 2, 4, and 6 each earn one success, and each 6 adds another d6.
    /// On a d5, 2, 4, and 5 succeed, and each 5 adds another d5.
    /// The result is a `DieRoll` of success counts, never below 0.
    /// This is not a general target-number system: for “5 or higher succeeds”, use `count` instead.
    ///
    /// | Mode | What happens to rolled 1s? |
    /// |------|---------------------------|
    /// | `"baseline"` | Nothing extra. |
    /// | `"ones_cancel"` | Any 1 stops all extra dice earned in that round; successes still count. |
    /// | `"ones_remove"` | Each 1 subtracts one success from the final count. |
    /// | `"implode"` | Each 1 earns a penalty roll of the same die; an odd result subtracts one success. Penalty rolls do not trigger more rolls. |
    ///
    /// Extra dice are processed in rounds, capped at 48 rounds including the first.
    /// This is a finite approximation to unlimited explosions. Start with small pools.
    ///
    /// ```dice
    /// output("Even faces succeed, sixes explode", success_pool(1, 6))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Starting number of dice, 0 or more.
    /// * `sides`: Number of faces on each die, at least 1.
    /// * `mode`: One of the quoted names in the table. Defaults to `"baseline"`.
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

    /// Leave out the lowest dice, then add the rest.
    ///
    /// For the familiar ability-score rule, roll four d6 and leave out the lowest one.
    /// The result is a `DieRoll` of totals from 3 to 18, just like `4d6dl1`.
    /// Tied low dice are still separate dice; dropping one removes only one of them.
    ///
    /// ```dice
    /// output("Ability score", drop_lowest(4, 6, 1))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Number of dice to roll, at least 1.
    /// * `sides`: Faces per die, at least 1.
    /// * `drop`: Number of lowest dice to leave out, 0 or more. A non-empty pool always keeps at least one die, even if you ask to drop them all.
    fn drop_lowest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_drop_lowest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Choose the highest dice and add them, leaving the others out.
    ///
    /// Keeping three of four d6 is the same as dropping the lowest one.
    /// The result is a `DieRoll` of totals, also written `4d6kh3` in dice notation.
    /// Keeping just one of two d20 models advantage.
    ///
    /// ```dice
    /// output("Advantage", keep_highest(2, 20, 1))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Number of dice to roll; use at least 1.
    /// * `sides`: Faces per die, at least 1.
    /// * `keep`: Number of highest dice to add, 0 or more. Keeping none gives 0; asking for more than `count` keeps all the dice.
    fn keep_highest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_keep_highest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Leave out the highest dice, then add the rest.
    ///
    /// For example, roll four d6, remove the highest one, and add the other three.
    /// The result is a `DieRoll` of totals, also written `4d6dh1`.
    /// If several dice tie for highest, each still counts as a separate die.
    ///
    /// ```dice
    /// output("Four d6, leave out the highest", drop_highest(4, 6, 1))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Number of dice to roll, at least 1.
    /// * `sides`: Faces per die, at least 1.
    /// * `drop`: Number of highest dice to leave out, 0 or more. A non-empty pool always keeps at least one die, even if you ask to drop them all.
    fn drop_highest(count: i32, sides: i32, drop: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let d = usize::try_from(drop).context("drop")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_drop_highest(
            n,
            i64::from(sides),
            d,
        )?))
    }

    /// Choose the lowest dice and add them, leaving the others out.
    ///
    /// Keeping one of two d20 models disadvantage. The result is a `DieRoll`
    /// of the kept total. In dice notation, this example is `2d20kl1`.
    ///
    /// ```dice
    /// output("Disadvantage", keep_lowest(2, 20, 1))
    /// ```
    ///
    /// # Arguments
    /// * `count`: Number of dice to roll; use at least 1.
    /// * `sides`: Faces per die, at least 1.
    /// * `keep`: Number of lowest dice to add, 0 or more. Keeping none gives 0; asking for more than `count` keeps all the dice.
    fn keep_lowest(count: i32, sides: i32, keep: i32) -> anyhow::Result<StarlarkDieRoll> {
        let n = usize::try_from(count).context("count")?;
        let k = usize::try_from(keep).context("keep")?;
        Ok(StarlarkDieRoll::new(DieRoll::pool_keep_lowest(
            n,
            i64::from(sides),
            k,
        )?))
    }

    /// Add the same bonus or penalty to every possible total.
    ///
    /// The returned `DieRoll` keeps the same chances but moves the numbers.
    /// Adding 3 to a d20 changes its results from 1–20 to 4–23.
    /// Usually `roll + 3` is the simplest way to write this; `shift(roll, 3)` does the same thing.
    ///
    /// ```dice
    /// output("d20 with a +3 bonus", shift(d(20), 3))
    /// ```
    ///
    /// # Arguments
    /// * `dist`: A numeric roll or total.
    /// * `delta`: Whole-number modifier. Use a negative number for a penalty.
    fn shift(dist: &StarlarkDieRoll, delta: i32) -> anyhow::Result<StarlarkDieRoll> {
        Ok(StarlarkDieRoll::new(dist.inner.shift(i64::from(delta))?))
    }

    /// Start a ladder of named results, such as miss, partial success, and full success.
    ///
    /// `scale()` takes no arguments and returns an empty `Scale`. Add labels from
    /// worst to best with `.step(...)`. Each step returns the longer ladder, so
    /// you can write one step after another. Quotes mark labels as text.
    /// A scale describes the rule; use `bucket` to find its chances for a roll.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
    /// output("Move result", bucket(2d6, results))
    /// ```
    ///
    /// You can omit the number ranges when using `classify` to choose labels yourself,
    /// or when supplying the ranges separately to `bucket`.
    #[starlark(as_type = StarlarkScale)]
    fn scale() -> anyhow::Result<StarlarkScale> {
        Ok(StarlarkScale::new(Scale::empty()))
    }

    /// Describe a range of whole numbers, including both ends.
    ///
    /// `through(5, 6)` means “5 or 6”, also written `5..6` in a `.dice` script.
    /// It returns an `IntBand`, a range for matching faces or defining outcome bands.
    /// It does not roll a die or calculate a chance on its own.
    ///
    /// ```dice
    /// output("Number of 5s and 6s", dice_pool(3, 6).count(through(5, 6)))
    /// ```
    ///
    /// # Arguments
    /// * `lo`: Lowest number to include.
    /// * `hi`: Highest number to include; must be at least `lo`.
    #[starlark(as_type = StarlarkIntBand)]
    fn through(lo: i32, hi: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::through(
            i64::from(lo),
            i64::from(hi),
        )?))
    }

    /// Describe “this number or lower”, including the number itself.
    ///
    /// `at_most(2)` matches 2, 1, 0, and every lower whole number.
    /// It returns an `IntBand` range, also written `..2` in a `.dice` script.
    /// Use it to match faces or define outcome bands, not to ask for a probability directly.
    ///
    /// ```dice
    /// output("Number of low dice", dice_pool(3, 6).count(at_most(2)))
    /// ```
    ///
    /// # Arguments
    /// * `hi`: Highest number to include.
    #[starlark(as_type = StarlarkIntBand)]
    fn at_most(hi: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::at_most(i64::from(hi))))
    }

    /// Describe “this number or higher”, including the number itself.
    ///
    /// `at_least(5)` matches 5, 6, 7, and every higher whole number.
    /// It returns an `IntBand` range, also written `5..` in a `.dice` script.
    /// This describes which faces match; `count` or a probability method works out their chances.
    ///
    /// ```dice
    /// output("Number of successes on 5+", dice_pool(3, 6).count(at_least(5)))
    /// ```
    ///
    /// # Arguments
    /// * `lo`: Lowest number to include.
    #[starlark(as_type = StarlarkIntBand)]
    fn at_least(lo: i32) -> anyhow::Result<StarlarkIntBand> {
        Ok(StarlarkIntBand::new(IntBand::at_least(i64::from(lo))))
    }

    /// Group numeric totals into named results, such as miss, partial success, and hit.
    ///
    /// First build a scale with a number range for each label. `bucket` then returns
    /// `Outcomes`: the chance of each label, rather than each individual total.
    /// You can also write `roll.bucket(results)`.
    ///
    /// ```dice
    /// results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
    /// output("Move result", bucket(2d6, results))
    /// ```
    ///
    /// Every possible total must match a band, or the script reports an error.
    /// If bands overlap, steps marked `early=True` are checked first, then the others;
    /// within each group, the first matching step wins.
    ///
    /// You can instead supply cut points: for three labels, `[6, 9]` means 6 or less,
    /// 7–9, and 10 or more. These replace any ranges already on the scale.
    ///
    /// ```dice
    /// results = scale().step("Miss").step("Partial").step("Hit")
    /// output("Move result using cut points", bucket(2d6, results, [6, 9]))
    /// ```
    ///
    /// # Arguments
    /// * `dist`: The numeric roll or total to label.
    /// * `scale`: Your ladder of labels, made with `scale()` and `.step(...)`.
    /// * `spec`: Optional replacement ranges. Supply one fewer cut point than labels, in increasing order (as a list or separate numbers), or one range per label as separate arguments, such as `at_most(6), through(7, 9), at_least(10)`.
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

    /// Choose a label for each possible roll using a rule you write yourself.
    ///
    /// Use `bucket` for simple number bands. Use `classify` when a special result,
    /// such as a natural 20, needs its own rule. The returned `Outcomes` contains
    /// the chance of each label. Your rule sees the value of the roll you pass in:
    /// pass the unmodified die if natural faces matter, not a total with a bonus already added.
    ///
    /// `def` starts your rule, `if` tests a condition, and `return` gives its label.
    /// `==` means “is equal to”. Keep the indentation shown.
    ///
    /// ```dice
    /// def name_roll(face):
    ///     if face == 20:
    ///         return "Critical"
    ///     if face >= 10:
    ///         return "Hit"
    ///     return "Miss"
    ///
    /// results = scale().step("Miss").step("Hit").step("Critical")
    /// output("d20 check", classify(d(20), results, name_roll))
    /// ```
    ///
    /// # Arguments
    /// * `dist`: The numeric roll your rule examines.
    /// * `scale`: All allowed labels, in order from worst to best.
    /// * `classify`: Your function's name, without parentheses. It receives one whole number and must return a label on the scale, spelled exactly the same way.
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

    /// Choose a named result by looking at two independent rolls together.
    ///
    /// For an opposed check, compare one player's roll with another's. Your function
    /// receives both numbers and returns a label; the engine checks every possible pair.
    /// The result is `Outcomes`, with a chance for each label.
    /// Even if you pass the same roll variable twice, the two rolls are independent,
    /// not two references to the same rolled face.
    ///
    /// ```dice
    /// def compare(left, right):
    ///     if left > right:
    ///         return "Win"
    ///     if left == right:
    ///         return "Tie"
    ///     return "Lose"
    ///
    /// results = scale().step("Lose").step("Tie").step("Win")
    /// output("Opposed d6 rolls", joint_classify(d(6), d(6), results, compare))
    /// ```
    ///
    /// `def` names the rule, and `return` supplies its answer. The indented lines
    /// belong to the rule; `>` means “greater than” and `==` means “equal to”.
    ///
    /// # Arguments
    /// * `d1`: First roll, passed to the first argument of your function.
    /// * `d2`: Second, independent roll, passed to the second argument.
    /// * `scale`: All allowed labels in order from worst to best.
    /// * `classify`: Your function's name, without parentheses. It must accept two whole numbers and return one of the scale's labels.
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

    /// Put several probability questions into one table for comparison.
    ///
    /// Each row has a text label and a probability. Rows answer separate questions;
    /// they may overlap and do not have to add to 100%. The result is a `ProbTable`;
    /// pass it to `output` to show it.
    ///
    /// Square brackets make the list of rows. Parentheses group each label with its chance.
    /// Here `.p_ge(...)` asks for the chance of meeting or beating a target.
    ///
    /// ```dice
    /// roll = dice_pool(2, 6).sum()
    /// rows = [("Need 7+", roll.p_ge(7)), ("Need 10+", roll.p_ge(10))]
    /// output("Compare targets", prob_table(rows))
    /// ```
    ///
    /// # Arguments
    /// * `rows`: A list of `(label, probability)` pairs. Labels are quoted text; probabilities are numbers from 0 to 1, such as 0.5 for 50%, not 50.
    #[starlark(as_type = StarlarkProbTable)]
    fn prob_table(rows: UnpackList<Value<'_>>) -> anyhow::Result<StarlarkProbTable> {
        Ok(StarlarkProbTable::new(parse_prob_table_rows(rows)?))
    }

    /// Show a result in the playground report.
    ///
    /// Call `output` at least once so you have a result to read. Put a helpful title
    /// in quotes, followed by the result. You can show a full numeric roll (`DieRoll`),
    /// named results (`Outcomes`), one probability, or a table made with `prob_table`.
    /// A `DicePool` must first become a result, for example with `.sum()` or `.count(6)`.
    ///
    /// ```dice
    /// roll = 2d6 + 3
    /// output("Total with bonus", roll)
    /// output("Chance of 10 or more", roll.p_ge(10))
    /// ```
    ///
    /// `roll = ...` gives the calculation a name so you can use it on later lines.
    /// A probability is a number from 0 to 1: 0.5 means 50%. Do not multiply it
    /// by 100 before passing it to `output`. The call records the result and returns
    /// `None` (no value); do not use it to build another roll.
    ///
    /// # Arguments
    /// * `args`: Usually two arguments: `output("Title", result)`. With just `output(result)`, a title is supplied automatically.
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
    eval_ast(ast)
        .map_err(starlark_err)
        .with_context(|| format!("eval {path}"))
}

/// Evaluate an already parsed module without discarding structured error locations.
pub(crate) fn eval_ast(ast: AstModule) -> Result<EvalResult, starlark::Error> {
    let globals = dice_globals();
    let store = OutputStore::default();
    let return_value = Module::with_temp_heap(|module| -> Result<String, starlark::Error> {
        let mut eval = Evaluator::new(&module);
        eval.extra = Some(&store);
        let res: Value = eval.eval_module(ast, &globals)?;
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
    fn eval_mixed_pool_addition() {
        let src = r#"output("hi", (dice_pool(1, 12) + dice_pool(2, 6)).order_stat(1))"#;
        let res = eval_source("test.dice", src).expect("eval");
        match &res.outputs[0] {
            OutputEntry::DieRoll { mean, .. } => {
                let mixed = DicePool::from_count(1, 12)
                    .unwrap()
                    .join(&DicePool::from_count(2, 6).unwrap())
                    .unwrap();
                let expected = mixed.order_stat(1).unwrap().mean();
                assert!((*mean - expected).abs() < 1e-9);
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
Scale = scale().step("FAIL").step("SUCCESS")
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
T = 15
Scale = (
    scale()
    .step("CRITICAL_FAIL", 1..1, early=True)
    .step("FAIL", at_most(T - 1))
    .step("SUCCESS", at_least(T))
    .step("CRITICAL_SUCCESS", 20..20, early=True)
)
out = d(20).bucket(Scale)
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
Scale = scale().step("FAILURE").step("MIXED").step("SUCCESS")
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
Scale = scale().step("A").step("B").step("C")
out = bucket(d(6), Scale, [3])
"#;
        let err = eval_source("test.star", src).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("bucket expects"));
    }

    #[test]
    fn eval_joint_classify_rejects_unknown_label() {
        let src = r#"
Scale = scale().step("A").step("B")
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
Scale = scale().step("MISS", ..6).step("PARTIAL", 7..9).step("FULL", 10..)
ScalePlain = scale().step("MISS").step("PARTIAL").step("FULL")
STAT = 2
roll = sum(dice_pool(2, 6)) + STAT
by_on_scale = roll.bucket(Scale)
by_cuts = bucket(roll, ScalePlain, [6, 9])
by_bands = roll.bucket(ScalePlain, at_most(6), through(7, 9), at_least(10))
output("on_scale", by_on_scale)
output("cuts", by_cuts)
output("bands", by_bands)
"#;
        let res = eval_source("test.dice", src).expect("eval");
        let on_scale = match &res.outputs[0] {
            OutputEntry::Outcomes { entries, .. } => entries.clone(),
            other => panic!("expected outcomes, got {other:?}"),
        };
        let cuts = match &res.outputs[1] {
            OutputEntry::Outcomes { entries, .. } => entries.clone(),
            other => panic!("expected outcomes, got {other:?}"),
        };
        let bands = match &res.outputs[2] {
            OutputEntry::Outcomes { entries, .. } => entries.clone(),
            other => panic!("expected outcomes, got {other:?}"),
        };
        assert_eq!(on_scale, cuts);
        assert_eq!(cuts, bands);
    }

    #[test]
    fn eval_range_desugar_in_bucket() {
        let src = r#"
Scale = scale().step("LOW").step("HIGH")
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
