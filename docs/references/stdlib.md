# Dice standard library

This reference lists everything built into `.dice` scripts beyond basic Starlark (variables, `for` loops, lists). New here? Work through the [tutorial](../README.md#tutorial) first—it introduces notation like `2d6` and `4d6dl1`, which the playground expands into the functions below.

**Naming:** face matching (`keep` / `remove` / `convert` / `ignore`), `count`, and pool `p_*` methods follow [API conventions](api-conventions.md).

## Core ideas

**`DieRoll`** — a finished numeric roll (or total) with exact chances for each possible result. Example: `2d6` is a `DieRoll`; so is `4d6dl1`. Use **`output("name", roll)`** to print its table in the playground.

**`DicePool`** — several dice rolled together but **not** added yet. Use this when the rule cares about *individual* faces (highest die, count successes, Blades-style pools). Call **`.sum()`** on the pool when you only need the total.

**`Outcomes`** — chances for **named** outcome bands (miss / partial / hit, crit fail / success, and so on) instead of raw numbers.

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
def d(sides: int) -> DieRoll
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
def die_faces(faces: list[int]) -> DieRoll
```

A die with custom face values (listed in order; duplicates count as extra weight).

#### Parameters

* `faces`: (required)

  List of integer face values.



#### Details

Use for dice that are not uniform—`die_faces([1, 2, 2, 3])` is twice as likely to show 2 as 1 or 3.

---

## die\_faces.bucket

```python
def die_faces.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

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

## die\_faces.clamp

```python
def die_faces.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## die\_faces.convert

```python
def die_faces.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## die\_faces.ignore

```python
def die_faces.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## die\_faces.keep

```python
def die_faces.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

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

## die\_faces.remove

```python
def die_faces.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## die\_faces.support\_size

```python
def die_faces.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## dice_pool

```python
def dice_pool(count: int, sides: int) -> DicePool
```

Roll `count` separate fair dice—**not** added together yet.

#### Parameters

* `count`: (required)

  How many dice.

* `sides`: (required)

  Faces per die (each die is 1..=sides).



#### Details

Use when the rule looks at individual results (highest die, count 10s, etc.). Add with
`.sum()` or the `sum(...)` function when you only need the total. Example: `dice_pool(4, 6)` for four d6s.

---

## dice\_pool.bucket

```python
def dice_pool.bucket(scale: Scale, *spec) -> Outcomes
```

Label the pool total using bands on `scale` (sums first).

---

## dice\_pool.convert

```python
def dice_pool.convert(spec, to: int) -> DicePool
```

Remap matching faces to `to` on every die.

---

## dice\_pool.count

```python
def dice_pool.count(spec) -> DieRoll
```

Distribution of how many dice match `spec`.

---

## dice\_pool.ignore

```python
def dice_pool.ignore(spec) -> DicePool
```

Remap matching faces to 0 on every die.

---

## dice\_pool.keep

```python
def dice_pool.keep(spec) -> DicePool
```

Keep only matching faces on every die; drop others and renormalize each die.

---

## dice\_pool.middle\_of

```python
def dice_pool.middle_of(keep: int) -> DieRoll
```

---

## dice\_pool.order\_stat

```python
def dice_pool.order_stat(k: int) -> DieRoll
```

---

## dice\_pool.p\_any

```python
def dice_pool.p_any(*spec) -> float
```

P(at least one die matches the optional face spec).

---

## dice\_pool.p\_at\_least

```python
def dice_pool.p_at_least(k: int, *spec) -> float
```

P(at least `k` dice match the optional face spec).

---

## dice\_pool.p\_none

```python
def dice_pool.p_none(*spec) -> float
```

P(no die matches the optional face spec).

---

## dice\_pool.remove

```python
def dice_pool.remove(spec) -> DicePool
```

Drop matching faces on every die; renormalize each die.

---

## dice\_pool.sum

```python
def dice_pool.sum() -> DieRoll
```

Add every die in the pool into one total—turns `dice_pool(4, 6)` into the same idea as `4d6`.

---

## sum

```python
def sum(value) -> DieRoll
```

Total a dice pool, or leave a `DieRoll` unchanged.

`sum(dice_pool(4, 6))` is the distribution of 4d6 summed—equivalent to `4d6` notation.
If you already have a `DieRoll`, `sum` returns it as-is.

---

---

## drop_lowest

## drop\_lowest

```python
def drop_lowest(count: int, sides: int, drop: int) -> DieRoll
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
def drop_highest(count: int, sides: int, drop: int) -> DieRoll
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
def keep_highest(count: int, sides: int, keep: int) -> DieRoll
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
def keep_lowest(count: int, sides: int, keep: int) -> DieRoll
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
def explode(dist: DieRoll, max_depth: int = 2) -> DieRoll
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

## open_ended_d100

```python
def open_ended_d100(max_chain: int = 8) -> DieRoll
```

Rolemaster **open-ended roll** on **1–100** (d100): low open on **01–05**, high open on **96–00**; rerolls chain on **96–00** only. `max_chain` caps consecutive **96–00** rerolls (default 8).

---

## open\_ended\_d100.bucket

```python
def open_ended_d100.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

---

## open\_ended\_d100.cdf

```python
def open_ended_d100.cdf(value: int) -> float
```

Chance the total is **this number or lower** (cumulative from the bottom).

#### Parameters

* `value`: (required)

  Upper cap (inclusive).



#### Details

Less common than `p_ge` for “beat the DC” checks; useful when rules ask “at most X”.

---

## open\_ended\_d100.clamp

```python
def open_ended_d100.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## open\_ended\_d100.convert

```python
def open_ended_d100.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## open\_ended\_d100.ignore

```python
def open_ended_d100.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## open\_ended\_d100.keep

```python
def open_ended_d100.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

---

## open\_ended\_d100.mean

```python
def open_ended_d100.mean() -> float
```

Average result if you rolled this distribution many times—the **mean** on the output table.

---

## open\_ended\_d100.p\_ge

```python
def open_ended_d100.p_ge(value: int) -> float
```

Chance of **meeting or beating** a target number—your go-to for “need 15+ on 2d10”.

#### Parameters

* `value`: (required)

  Target total (inclusive)—success if roll ≥ this.



#### Details

Example: `output("success", (2d10 + 3).p_ge(15))`.

---

## open\_ended\_d100.pmf

```python
def open_ended_d100.pmf(value: int) -> float
```

Chance of rolling **exactly** this number (one outcome, not “this or higher”).

#### Parameters

* `value`: (required)

  The total you care about.



#### Details

Example: `output("pct_seven", 2d6.pmf(7))` for the probability of a 7 on 2d6.

---

## open\_ended\_d100.remove

```python
def open_ended_d100.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## open\_ended\_d100.support\_size

```python
def open_ended_d100.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## shift

```python
def shift(dist: DieRoll, delta: int) -> DieRoll
```

Add a flat modifier to every outcome—**+3 to the roll** without rolling another die.

#### Parameters

* `dist`: (required)

  The roll (e.g. `2d10` as a `DieRoll`).

* `delta`: (required)

  Modifier to add (can be negative).



#### Details

Same effect as `roll + 3` when `roll` is a `DieRoll`. Prefer `roll + 3` in scripts when it reads clearer.

---

## through

```python
def through(lo: int, hi: int) -> IntBand
```

Inclusive closed integer interval (same as desugared `6..94`).

---

## at_most

```python
def at_most(hi: int) -> IntBand
```

All integers at or below `hi` (desugared `..hi`).

---

## at_least

```python
def at_least(lo: int) -> IntBand
```

All integers at or above `lo` (desugared `lo..`).

---

### Inclusive ranges

Integer bands for face filters and bucketing. In `.dice` scripts you can also write `6..94`, `..6`, and `10..` (inclusive endpoints).

## through

```python
def through(lo: int, hi: int) -> IntBand
```

Inclusive closed integer interval (same as desugared `6..94`).

---

## at_most

```python
def at_most(hi: int) -> IntBand
```

All integers at or below `hi` (desugared `..hi`).

---

## at_least

```python
def at_least(lo: int) -> IntBand
```

All integers at or above `lo` (desugared `lo..`).

---

### Pool rules (faces still matter)

These need a `DicePool` from `dice_pool` before you total the dice.

## count

```python
def count(pool: DicePool, spec) -> DieRoll
```

How many dice in the pool match a face spec?

#### Parameters

* `pool`: (required)

  From `dice_pool`.

* `spec`: (required)

  int face, list of ints, or `IntBand` / desugared range (e.g. `5..` for 5+).



#### Details

The result is a `DieRoll` over counts (0, 1, 2, …). Same as `pool.count(spec)`.

---

---

## order_stat

```python
def order_stat(pool: DicePool, k: int) -> DieRoll
```

The **k**th highest die in the pool (`k = 1` is the highest, `2` is second-highest, …).

#### Parameters

* `k`: (required)

  Rank from the top (1 = best die).



#### Details

Blades in the Dark and similar games use the highest die; some rules use second-highest.

---

## order\_stat.bucket

```python
def order_stat.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

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

## order\_stat.clamp

```python
def order_stat.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## order\_stat.convert

```python
def order_stat.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## order\_stat.ignore

```python
def order_stat.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## order\_stat.keep

```python
def order_stat.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

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

## order\_stat.remove

```python
def order_stat.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## order\_stat.support\_size

```python
def order_stat.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## middle_of

```python
def middle_of(pool: DicePool, keep: int) -> DieRoll
```

Sum the middle `keep` dice after sorting the pool low to high.

#### Parameters

* `keep`: (required)

  How many dice in the middle to sum.



#### Details

Niche rules that drop extremes from both ends; less common than keep-highest / drop-lowest.

---

## middle\_of.bucket

```python
def middle_of.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

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

## middle\_of.clamp

```python
def middle_of.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## middle\_of.convert

```python
def middle_of.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## middle\_of.ignore

```python
def middle_of.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## middle\_of.keep

```python
def middle_of.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

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

## middle\_of.remove

```python
def middle_of.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## middle\_of.support\_size

```python
def middle_of.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## pool_map

```python
def pool_map(pool: DicePool, map_fn) -> DieRoll
```

Custom rule: for every way the pool can land, run your function on the list of faces and use its integer result.

#### Parameters

* `map_fn`: (required)

  Starlark function `(faces) -> int`.



#### Details

Advanced—use when no built-in pool helper fits (e.g. “sum only dice that matched another die”).
The function receives one argument: the list of rolled values, sorted as the engine stores them.

---

## pool\_map.bucket

```python
def pool_map.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

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

## pool\_map.clamp

```python
def pool_map.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## pool\_map.convert

```python
def pool_map.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## pool\_map.ignore

```python
def pool_map.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## pool\_map.keep

```python
def pool_map.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

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

## pool\_map.remove

```python
def pool_map.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## pool\_map.support\_size

```python
def pool_map.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## success_pool

```python
def success_pool(
    count: int,
    sides: int,
    mode: str = "baseline",
) -> DieRoll
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

Returns a `DieRoll` over how many successes you rolled. `mode` controls 1s and 10s:
`"baseline"` (default), `"ones_cancel"`, `"ones_remove"`, or `"implode"`.

---

## success\_pool.bucket

```python
def success_pool.bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

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

## success\_pool.clamp

```python
def success_pool.clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## success\_pool.convert

```python
def success_pool.convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## success\_pool.ignore

```python
def success_pool.ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## success\_pool.keep

```python
def success_pool.keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

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

## success\_pool.remove

```python
def success_pool.remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## success\_pool.support\_size

```python
def success_pool.support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

### Named outcomes

Turn numeric totals or special roll rules into labeled results.

## scale

```python
def scale() -> Scale
```

Start an ordered outcome scale; chain `.step(label)` or `.step(label, band)` on the result.

(`with` is reserved in Starlark.) Example: `scale().step("MISS", ..6).step("PARTIAL", 7..9)`.

---

---

## bucket

```python
def bucket(dist: DieRoll, scale: Scale, *spec) -> Outcomes
```

Split a numeric total into named bands.

#### Parameters

* `dist`: (required)

  Numeric roll (e.g. `2d6 + stat`).

* `scale`: (required)

  Built with `scale()` and `.step`.



#### Details

With bands on `scale` (from `scale().step(..., band)`), call `bucket(roll, scale)` or
`roll.bucket(scale)`. Otherwise pass **N−1** cut ints or **N** explicit bands (override).

---

---

## classify

```python
def classify(dist: DieRoll, scale: Scale, classify) -> Outcomes
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
    d1: DieRoll,
    d2: DieRoll,
    scale: Scale,
    classify,
) -> Outcomes
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
a full distribution (`DieRoll`), named outcomes (`Outcomes`), a probability (`float`),
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

# DieRoll methods

Ask questions about a numeric `DieRoll` after you build it (often inside `output(..., roll.p_ge(15))`).

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

## clamp

```python
def clamp(min: int, max: int) -> DieRoll
```

Cap every outcome at `min` and `max` (inclusive), merging probability at the bounds.

#### Parameters

* `min`: (required)

  Lower bound (inclusive).

* `max`: (required)

  Upper bound (inclusive).



#### Details

Example: `(3d6 + 5).clamp(3, 18)` for a boosted roll that cannot exceed 18.

---

## support_size

## support\_size

```python
def support_size() -> int
```

How many different totals can occur with non-zero chance (size of the result table).

---

## keep

```python
def keep(spec) -> DieRoll
```

Keep only faces matching `spec`; drop others and renormalize. Not `.p_ge()` on totals.

---

## remove

```python
def remove(spec) -> DieRoll
```

Drop faces matching `spec`; renormalize the rest.

---

## convert

```python
def convert(spec, to: int) -> DieRoll
```

Remap matching faces to `to`; other faces unchanged.

---

## ignore

```python
def ignore(spec) -> DieRoll
```

Remap matching faces to 0 (`convert(spec, 0)`).

---

## bucket

```python
def bucket(scale: Scale, *bands) -> Outcomes
```

Label numeric totals using bands on `scale`, or pass cut list / bands to override.

---

# DicePool methods

Face filters (`keep` / `remove` / `convert` / `ignore`), match counts (`count`), pool match probabilities (`p_any` / `p_none` / `p_at_least`), or total the pool. See [API conventions](../references/api-conventions.md).

## sum

```python
def sum() -> DieRoll
```

Add every die in the pool into one total—turns `dice_pool(4, 6)` into the same idea as `4d6`.

---

## keep

```python
def keep(spec) -> DicePool
```

Keep only matching faces on every die; drop others and renormalize each die.

---

## remove

```python
def remove(spec) -> DicePool
```

Drop matching faces on every die; renormalize each die.

---

## convert

```python
def convert(spec, to: int) -> DicePool
```

Remap matching faces to `to` on every die.

---

## ignore

```python
def ignore(spec) -> DicePool
```

Remap matching faces to 0 on every die.

---

## count

```python
def count(spec) -> DieRoll
```

Distribution of how many dice match `spec`.

---

## order_stat

## order\_stat

```python
def order_stat(k: int) -> DieRoll
```

---

## middle_of

## middle\_of

```python
def middle_of(keep: int) -> DieRoll
```

---

## p_any

## p\_any

```python
def p_any(*spec) -> float
```

P(at least one die matches the optional face spec).

---

## p_none

## p\_none

```python
def p_none(*spec) -> float
```

P(no die matches the optional face spec).

---

## p_at_least

## p\_at\_least

```python
def p_at_least(k: int, *spec) -> float
```

P(at least `k` dice match the optional face spec).

---

## bucket

```python
def bucket(scale: Scale, *spec) -> Outcomes
```

Label the pool total using bands on `scale` (sums first).

---

# Outcomes methods

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

# Scale methods

Build ordered outcome labels and optional numeric bands after `scale()`.

## step

```python
def step(label: str, *band) -> Scale
```

Append one outcome label (low → high). With no band, the step is for `classify` only.

With a band (`IntBand` or desugared `..6`, `7..9`, `10..`), the step buckets numeric totals.

---

