# Dice standard library

This reference lists everything built into `.dice` scripts beyond basic Starlark (variables, `for` loops, lists). New here? Work through the [tutorial](../README.md#tutorial) first—it introduces notation like `2d6` and `4d6dl1`, which the playground expands into the functions below.

## Core ideas

**`Dist`** — a finished numeric roll (or total) with exact chances for each possible result. Example: `2d6` is a `Dist`; so is `4d6dl1`. Use **`output("name", dist)`** to print its table in the playground.

**`RollPool`** — several dice rolled together but **not** added yet. Use this when the rule cares about *individual* faces (highest die, count successes, Blades-style pools). Call **`.sum()`** on the pool when you only need the total.

**`LabelDist`** — chances for **named** outcomes (miss / partial / hit, crit fail / success, and so on) instead of raw numbers.

## Combining rolls (operators)

| You write | Meaning at the table |
|-----------|----------------------|
| `a + b` | Two **independent** rolls added together (e.g. `1d6 + 1d6` or `2d6 + 1d4`). |
| `roll + 5` | Flat **modifier** added to every outcome of `roll` (same as `shift(roll, 5)`). |
| `roll - 3` | Subtract 3 from every outcome. |
| `a - b` | Independent rolls subtracted (less common; niche mechanics). |
| `roll * 10` | Multiply **each** outcome (e.g. tens die reading). |
| `roll // 2` | **Halve** each outcome, round down (typical “half damage on save”). |

Dice notation (`1d20`, `3d6kh2`, …) is sugar for these functions—see the [dice notation lesson](../tutorial/05-dice-notation.md).

## Builtin functions (by topic)

### Building dice and totals

Start here for ordinary dice, custom faces, and summed pools.

## d

```python
def d(sides: int) -> Dist
```

One fair die with faces 1 through `sides`, each equally likely.

#### Parameters

* `sides`: (required)

  Number of faces (must be at least 1).



#### Details

Same idea as `1d6` or `1d20` in dice notation. Example: `d(6)` for a d6, `d(20)` for a d20.

---

---

## die_faces

```python
def die_faces(faces: list[int]) -> Dist
```

A die with custom face values (listed in order; duplicates count as extra weight).

#### Parameters

* `faces`: (required)

  List of integer face values.



#### Details

Use for dice that are not uniform—`die_faces([1, 2, 2, 3])` is twice as likely to show 2 as 1 or 3.

---

## die\_faces.cdf

```python
def die_faces.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## die\_faces.mean

```python
def die_faces.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## die\_faces.p\_ge

```python
def die_faces.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## die\_faces.pmf

```python
def die_faces.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## die\_faces.support\_size

```python
def die_faces.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## roll_pool

```python
def roll_pool(count: int, sides: int) -> RollPool
```

Roll `count` separate fair dice—**not** added together yet.

#### Parameters

* `count`: (required)

  How many dice.

* `sides`: (required)

  Faces per die (each die is 1..=sides).



#### Details

Use when the rule looks at individual results (highest die, count 10s, etc.). Add with
`.sum()` or the `sum(...)` function when you only need the total. Example: `roll_pool(4, 6)` for four d6s.

---

## roll\_pool.sum

```python
def roll_pool.sum() -> Dist
```

Add every die in the pool into one total—turns `roll_pool(4, 6)` into the same idea as `4d6`.

---

## pool

```python
def pool(count: int, sides: int) -> RollPool
```

Shorthand for `roll_pool`—same arguments, same meaning.

---

---

## sum

```python
def sum(value) -> Dist
```

Total a dice pool, or leave a `Dist` unchanged.

`sum(roll_pool(4, 6))` is the distribution of 4d6 summed—equivalent to `4d6` notation.
If you already have a `Dist`, `sum` returns it as-is.

---

---

## drop_lowest

## drop\_lowest

```python
def drop_lowest(count: int, sides: int, drop: int) -> Dist
```

Roll several dice, drop the lowest results, sum the rest—**4d6 drop lowest 1** is `drop_lowest(4, 6, 1)`.

#### Parameters

* `count`: (required)

  Dice rolled.

* `sides`: (required)

  Faces per die.

* `drop`: (required)

  How many lowest dice to remove before summing.



#### Details

Same as `4d6dl1` in dice notation.

---

## drop_highest

## drop\_highest

```python
def drop_highest(count: int, sides: int, drop: int) -> Dist
```

Roll dice, drop the highest results, sum the rest (`4d6dh1` notation).

#### Parameters

* `count`: (required)

  Dice rolled.

* `sides`: (required)

  Faces per die.

* `drop`: (required)

  How many highest dice to remove before summing.

---

## keep_highest

## keep\_highest

```python
def keep_highest(count: int, sides: int, keep: int) -> Dist
```

Roll dice, keep only the highest few, sum those—**4d6 keep highest 3** is `keep_highest(4, 6, 3)` (`4d6kh3`).

#### Parameters

* `count`: (required)

  Dice rolled.

* `sides`: (required)

  Faces per die.

* `keep`: (required)

  How many highest dice to sum.

---

## keep_lowest

## keep\_lowest

```python
def keep_lowest(count: int, sides: int, keep: int) -> Dist
```

Roll dice, keep only the lowest few, sum those (`4d6kl3` notation).

#### Parameters

* `count`: (required)

  Dice rolled.

* `sides`: (required)

  Faces per die.

* `keep`: (required)

  How many lowest dice to sum.

---

## explode

```python
def explode(dist: Dist, max_depth: int = 2) -> Dist
```

Exploding die: on the highest face, roll again and add, up to `max_depth` extra rolls (default 2).

#### Parameters

* `dist`: (required)

  Usually a single die from `d(...)`.

* `max_depth`: (defaults to: `2`)

  Cap on how many times the die can explode (0 = no explode).



#### Details

Common in games where max on a die triggers another die (Savage Worlds–style). Example:
`explode(d(4))` for one exploding d4.

---

---

## shift

```python
def shift(dist: Dist, delta: int) -> Dist
```

Add a flat modifier to every outcome—**+3 to the roll** without rolling another die.

#### Parameters

* `dist`: (required)

  The roll (e.g. `2d10` as a `Dist`).

* `delta`: (required)

  Modifier to add (can be negative).



#### Details

Same effect as `roll + 3` when `roll` is a `Dist`. Prefer `roll + 3` in scripts when it reads clearer.

---

### Pool rules (faces still matter)

These need a `RollPool` from `roll_pool` / `pool` before you total the dice.

## count_ge

```python
def count_ge(pool: RollPool, threshold: int) -> Dist
```

How many dice in the pool rolled **at least** `threshold`?

#### Parameters

* `pool`: (required)

  From `roll_pool` / `pool`.

* `threshold`: (required)

  Count dice with rolled value ≥ this number.



#### Details

The result is a `Dist` over counts (0, 1, 2, …). Example: on 5d10, how many dice show 8+ for a success pool.

---

## count\_ge.cdf

```python
def count_ge.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## count\_ge.mean

```python
def count_ge.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## count\_ge.p\_ge

```python
def count_ge.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## count\_ge.pmf

```python
def count_ge.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## count\_ge.support\_size

```python
def count_ge.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## count_in

```python
def count_in(pool: RollPool, values: list[int]) -> Dist
```

How many dice show a face in your chosen list?

#### Parameters

* `values`: (required)

  Face values that count (duplicates in the list are harmless).



#### Details

Example: count how many dice rolled 1 in a pool (list `[1]`), or how many show 9 or 10 (`[9, 10]`).

---

## count\_in.cdf

```python
def count_in.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## count\_in.mean

```python
def count_in.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## count\_in.p\_ge

```python
def count_in.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## count\_in.pmf

```python
def count_in.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## count\_in.support\_size

```python
def count_in.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## order_stat

```python
def order_stat(pool: RollPool, k: int) -> Dist
```

The **k**th highest die in the pool (`k = 1` is the highest, `2` is second-highest, …).

#### Parameters

* `k`: (required)

  Rank from the top (1 = best die).



#### Details

Blades in the Dark and similar games use the highest die; some rules use second-highest.

---

## order\_stat.cdf

```python
def order_stat.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## order\_stat.mean

```python
def order_stat.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## order\_stat.p\_ge

```python
def order_stat.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## order\_stat.pmf

```python
def order_stat.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## order\_stat.support\_size

```python
def order_stat.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## middle_of

```python
def middle_of(pool: RollPool, keep: int) -> Dist
```

Sum the middle `keep` dice after sorting the pool low to high.

#### Parameters

* `keep`: (required)

  How many dice in the middle to sum.



#### Details

Niche rules that drop extremes from both ends; less common than keep-highest / drop-lowest.

---

## middle\_of.cdf

```python
def middle_of.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## middle\_of.mean

```python
def middle_of.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## middle\_of.p\_ge

```python
def middle_of.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## middle\_of.pmf

```python
def middle_of.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## middle\_of.support\_size

```python
def middle_of.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## pool_map

```python
def pool_map(pool: RollPool, map_fn) -> Dist
```

Custom rule: for every way the pool can land, run your function on the list of faces and use its integer result.

#### Parameters

* `map_fn`: (required)

  Starlark function `(faces) -> int`.



#### Details

Advanced—use when no built-in pool helper fits (e.g. “sum only dice that matched another die”).
The function receives one argument: the list of rolled values, sorted as the engine stores them.

---

## pool\_map.cdf

```python
def pool_map.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## pool\_map.mean

```python
def pool_map.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## pool\_map.p\_ge

```python
def pool_map.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## pool\_map.pmf

```python
def pool_map.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## pool\_map.support\_size

```python
def pool_map.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## success_pool

```python
def success_pool(count: int, sides: int, mode: str = "baseline") -> Dist
```

Count **successes** on a dice pool (Storyteller / WoD-style d10 pools and variants).

#### Parameters

* `count`: (required)

  Dice in the pool.

* `sides`: (required)

  Usually 10 for classic WoD.

* `mode`: (defaults to: `"baseline"`)

  How ones and explosions interact—match your table’s house rules.



#### Details

Returns a `Dist` over how many successes you rolled. `mode` controls 1s and 10s:
`"baseline"` (default), `"ones_cancel"`, `"ones_remove"`, or `"implode"`.

---

## success\_pool.cdf

```python
def success_pool.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## success\_pool.mean

```python
def success_pool.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## success\_pool.p\_ge

```python
def success_pool.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## success\_pool.pmf

```python
def success_pool.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## success\_pool.support\_size

```python
def success_pool.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

### Named outcomes

Turn numeric totals or special roll rules into labeled results.

## result_type

```python
def result_type(labels: list[str]) -> ResultScale
```

Name your outcome steps from worst to best (or low to high).

#### Parameters

* `labels`: (required)

  Unique non-empty strings, first = lowest rank, last = highest.



#### Details

Used with `bucket` or `classify`. Example: `result_type(["MISS", "PARTIAL", "FULL"])`.

---

## bucket

```python
def bucket(dist: Dist, scale: ResultScale, cuts: list[int]) -> LabelDist
```

Split a numeric total into named bands using DC-style cut points.

#### Parameters

* `dist`: (required)

  Numeric roll (e.g. `2d6 + stat`).

* `scale`: (required)

  From `result_type(...)`.

* `cuts`: (required)

  Increasing thresholds between labels.



#### Details

With 4 labels you pass **3** cut numbers. Totals at or below the first cut get the first label;
between cuts get middle labels; above the last cut get the top label. PbtA 2d6+stat moves often use this.

---

---

## classify

```python
def classify(dist: Dist, scale: ResultScale, classify) -> LabelDist
```

Label each **exact** roll value with your own rule—natural 1s, natural 20s, custom crits.

#### Parameters

* `classify`: (required)

  Starlark function `(value) -> str`.



#### Details

Your function takes the numeric result and returns one of the strings on `scale`.
Example: map only 1 and 20 to special labels, bucket everything else by total.

---

---

## joint_classify

```python
def joint_classify(
    d1: Dist,
    d2: Dist,
    scale: ResultScale,
    classify,
) -> LabelDist
```

Label outcomes that depend on **two** dice together—advantage, disadvantage, or paired rolls.

#### Parameters

* `d1`: (required)

  , `d2`: Independent rolls (e.g. two d20s for advantage).

* `classify`: (required)

  Starlark function `(w, b) -> str` returning a label on `scale`.



#### Details

Every combination of `d1` and `d2` is classified by your `(left, right) -> str` function.

---

## joint\_classify.p\_at\_least

```python
def joint_classify.p_at_least(label: str) -> float
```

Chance of this outcome **or any better one** on the scale—e.g. “partial success or full success”.

---

## joint\_classify.p\_at\_most

```python
def joint_classify.p_at_most(label: str) -> float
```

Chance of this outcome **or any worse one**—e.g. “failure or partial failure”.

---

## joint\_classify.pmf

```python
def joint_classify.pmf(label: str) -> float
```

Chance of landing on **exactly** this named outcome (one band on the ladder).

---

### Showing results

Always end scripts with `output` so the playground prints tables and charts.

## output

```python
def output(*args) -> None
```

Send a result to the playground **Output** panel (text, json, and graph tabs).

Almost every script should call this at least once. Pass a name and a value:
a full distribution (`Dist`), named outcomes (`LabelDist`), a probability (`float`),
or a table (`prob_table(...)`). One argument works but naming outputs helps you read results.

---

## prob_table

```python
def prob_table(rows: list) -> ProbTable
```

One table of labeled probabilities—grids of “chance to hit DC X at modifier Y”.

#### Parameters

* `rows`: (required)

  List of `(string, float)` pairs.



#### Details

Each row is `(description, probability)`. Rows are **independent** (they do not have to add to 100%).
Build a list in a loop, then pass it here once: `output("grid", prob_table(rows))`.

---

# Dist methods

Ask questions about a numeric `Dist` after you build it (often inside `output(..., roll.p_ge(15))`).

## mean

```python
def mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## pmf

```python
def pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## p_ge

## p\_ge

```python
def p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## cdf

```python
def cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## support_size

## support\_size

```python
def support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

# RollPool methods

Turn a pool into a single total when the rule no longer cares about separate dice.

## sum

```python
def sum() -> Dist
```

Add every die in the pool into one total—turns `roll_pool(4, 6)` into the same idea as `4d6`.

---

# LabelDist methods

Query named outcome bands (PbtA moves, graded success, etc.).

## pmf

```python
def pmf(label: str) -> float
```

Chance of landing on **exactly** this named outcome (one band on the ladder).

---

## p_at_least

## p\_at\_least

```python
def p_at_least(label: str) -> float
```

Chance of this outcome **or any better one** on the scale—e.g. “partial success or full success”.

---

## p_at_most

## p\_at\_most

```python
def p_at_most(label: str) -> float
```

Chance of this outcome **or any worse one**—e.g. “failure or partial failure”.

---

