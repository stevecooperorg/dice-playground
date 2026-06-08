---
title: API conventions for .dice scripts
---

# API conventions for `.dice` scripts

This page defines the **stable naming pattern** for face matching, pool counts, and probabilities. The full symbol list is in [stdlib.md](stdlib.md).

## FaceSpec (one argument shape)

Many methods take a single **face spec**:

| You write | Meaning |
|-----------|---------|
| `5` | Exactly face 5 |
| `[1, 2]` | Face is 1 or 2 |
| `5..6`, `5..`, `..6` | Inclusive range (desugared to `through` / `at_least` / `at_most`) |
| `at_least(8)`, `through(2, 5)` | Same bands without range sugar |

## Face operations (`keep` / `remove` / `convert` / `ignore`)

On **`DieRoll`** and **`DicePool`**, these four methods use a **FaceSpec**. On a pool, each die is transformed independently (same as the old `restrict` behavior, but with clearer names).

| Method | Effect on one die | Renormalize? | Example on fair `d(6)` |
|--------|-------------------|--------------|-------------------------|
| `keep(spec)` | Only matching faces remain | Yes (mass → 1) | `keep(5..)` → 50% on 5, 50% on 6 |
| `remove(spec)` | Matching faces dropped | Yes | `remove(1..4)` → same PMF as `keep(5..)` |
| `convert(spec, to)` | Matching faces become `to` | No extra step; masses merge if outcomes collide | `convert(1..4, 0)` → 4/6 on 0, 1/6 on 5, 1/6 on 6 |
| `ignore(spec)` | `convert(spec, 0)` | Same as convert | `ignore(1..4)` |

**Two ways “only high faces matter” for a sum:**

- `dice_pool(3, 6).keep(5..).sum()` — each die can only show 5 or 6 (conditional die). Totals are 15–18 only (mean 16.5).
- `dice_pool(3, 6).ignore(1..4).sum()` — full d6 rolls; faces 1–4 count as 0 toward the total. The sum distribution includes **0** (mean 5.5).

`count(spec)` and pool `p_*` methods still use **unfiltered** dice: they count how many dice matched, not a kept or ignored sum.

### Joining mixed pools (`+`)

| Expression | Result |
|------------|--------|
| `dice_pool(a, s) + dice_pool(b, t)` | One `DicePool` with `a + b` dice (left pool, then right) |
| `d(12) + dice_pool(2, 6)` | Same as `dice_pool(1, 12) + dice_pool(2, 6)` |
| `pool + 3` | `pool.sum()` with **+3** on every total (flat modifier) |

See [lesson 7](../tutorial/07-mixed-dice-pools.md).

## Operations by type

### `DieRoll` and `DicePool`

| Method | Returns | Meaning |
|--------|---------|---------|
| `keep(spec)` | Same type | Drop non-matching faces; renormalize |
| `remove(spec)` | Same type | Drop matching faces; renormalize |
| `convert(spec, to)` | Same type | Remap matching faces to `to` |
| `ignore(spec)` | Same type | `convert(spec, 0)` |
| `count(spec)` | `DieRoll` | **DicePool only:** distribution of how many dice matched |

### `DicePool` match probabilities

All return `float` for use in `output("label", p)`.

| Method | Meaning |
|--------|---------|
| `p_any(spec?)` | P(≥1 die matches `spec`); omit `spec` for “pool non-empty” |
| `p_none(spec?)` | P(0 dice match); omit `spec` for “pool empty” |
| `p_at_least(k, spec?)` | P(≥k dice match); omit `spec` for “pool has ≥k dice” |

On **`Outcomes`**, `p_at_least(label)` means **this label or better on the scale**—not a die count. The receiver type disambiguates.

### `DieRoll` total probabilities (unchanged)

| Method | Meaning |
|--------|---------|
| `pmf(n)` | P(total == n) |
| `p_ge(n)` | P(total ≥ n) |
| `cdf(n)` | P(total ≤ n) |

Face filters change per-die outcomes before you sum or query totals; `p_ge` asks about the **numeric total** after those transforms.

## Builtin helpers

```python
count(pool, spec)   # same as pool.count(spec)
```

## Migration (breaking)

| Old | New |
|-----|-----|
| `restrict(spec)` | `keep(spec)` |
| `roll.ge(5)` / `pool.ge(5)` | `keep(5..)` or `keep(at_least(5))` |
| `roll.only([1, 2])` | `keep([1, 2])` |
| `roll.in_band(band)` | `keep(band)` |
| `pool.count_ge(5)` | `pool.count(5..)` |
| `pool.count_in([1])` | `pool.count([1])` |
| `pool.count_in_band(band)` | `pool.count(band)` |
| `count_ge(pool, 5)` | `count(pool, 5..)` |
| `pool.any(1)` | `pool.p_any(1)` |
| `pool.none(1)` | `pool.p_none(1)` |
| `pool.has_at_least(2, 5..)` | `pool.p_at_least(2, 5..)` |
