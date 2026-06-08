---
title: "Filtering faces (keep / ignore)"
author: Steve Cooper
---

# Filtering faces (`keep`, `remove`, `convert`, `ignore`)

## At the table

Rules often treat faces differently: “only 5s and 6s can appear,” or “1–4 don’t count toward the total.” Those are **face filters** on each die—not the same as “total ≥ 15” from [lesson 4](04-success.md).

## Try it

Copy the script into the playground editor and click **Run**.

## The script

```text
output("high_die", d(6).keep(5..))
output("conditional_pool_sum", dice_pool(3, 6).keep(5..).sum())
output("ignored_pool_sum", dice_pool(3, 6).ignore(1..4).sum())
```

- `keep(5..)` drops non-matching faces (here 1–4), then **renormalizes** so only 5 and 6 remain.
- `ignore(1..4)` is shorthand for `convert(1..4, 0)`: low faces still roll, but count as **0** when you sum.
- Face specs use the same shapes as in [API conventions](../references/api-conventions.md): a single face, `[1, 2]`, ranges like `5..`, or `at_least(5)`.
- `remove(1..4)` on a d6 is the same die as `keep(5..)`.

## `keep` vs `ignore` vs `p_ge`

| Question | Example |
|----------|---------|
| Each die can only be 5 or 6, then sum | `dice_pool(3, 6).keep(5..).sum()` (totals 15–18 only) |
| Full d6, but 1–4 add 0 to the sum | `dice_pool(3, 6).ignore(1..4).sum()` (totals include 0) |
| Unfiltered 3d6 total at least 15 | `3d6.p_ge(15)` |

## Try this

- Compare means of `d(6).keep(5..)` (5.5) and `dice_pool(3, 6).ignore(1..4).sum()` (5.5)—same mean, different distributions.
- Try `keep(..2)` for “only 1 or 2 can appear.”

## Next

[Lesson 8: Pool success counts](08-pool-success-counts.md)
