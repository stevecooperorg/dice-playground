---
title: "Mixed dice pools (+)"
author: Steve Cooper
---

# Mixed dice pools (`+`)

## At the table

Some rules roll **different dice in one throw** and then compare individual results—classic “roll 1d12 and 2d6, take the highest.” That is one pool of three dice, not the sum of a d12 total plus a 2d6 total.

## Try it

Copy the script into the playground editor and click **Run**.

## The script

```text
pool = dice_pool(1, 12) + dice_pool(2, 6)
output("highest", pool.order_stat(1))
output("pool_sum", pool.sum())
output("same_as_three_d6_highest", dice_pool(3, 6).order_stat(1))
```

- `+` on two **`DicePool`** values **joins** them: every die from the left pool, then every die from the right, still rolled independently.
- `d(12) + dice_pool(2, 6)` does the same if you prefer a single die on the left.
- `pool.sum()` is still “add every die”—here 1d12 + 2d6 summed, not the same as `order_stat(1)`.
- `dice_pool(3, 6)` is three **matching** d6; the comparison line shows how that differs from 1d12 + 2d6.

## Reading the result

`highest` tops out at **12** (when the d12 shows 12). The best of three d6 tops out at **6**, so the two `order_stat(1)` tables look very different even though both pools have three dice.

## Try this

- Build `d(20) + dice_pool(2, 6)` and compare `order_stat(1)` to `dice_pool(3, 6).order_stat(1)`.
- Add a flat bonus after you collapse the pool: `(dice_pool(1, 12) + dice_pool(2, 6)).sum() + 2`.

More pool recipes: [cookbook index](../cookbook/README.md). Conventions: [API conventions](../references/api-conventions.md).

## Next

[Lesson 8: Filtering faces (`keep` / `ignore`)](08-restrict-faces.md)
