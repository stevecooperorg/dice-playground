---
title: "Dice pools (faces still separate)"
author: Steve Cooper
---

# Dice pools (faces still separate)

## At the table

Some rules care about **each die**—the highest die in the pool, how many dice show a 1, whether any die crits—not only the **sum**. You need several dice rolled **together** but not added until you choose to.

## Try it

Copy the script into the playground editor and click **Run**.

## The script

```text
output("pool_sum", dice_pool(3, 6).sum())
output("notation_sum", 3d6)
output("highest_die", dice_pool(4, 6).order_stat(1))
```

- `dice_pool(3, 6)` is three fair d6 **kept separate** until you call `.sum()`.
- `3d6` notation sums automatically—the distribution of `pool_sum` and `notation_sum` matches.
- `order_stat(1)` is the **highest** die in the pool (`k = 2` would be second-highest). Games like Blades in the Dark start from this idea.

## Reading the result

`pool_sum` and `notation_sum` have the same mean (10.5). `highest_die` is a single-die-style table: how often the best of four d6 shows each face.

## Try this

- Compare `dice_pool(2, 6).sum()` to `2d6` from [lesson 2](02-two-dice.md).
- Try `order_stat(2)` on `dice_pool(3, 6)` for “second highest of three.”

More pool recipes: [cookbook index](../cookbook/README.md). Conventions: [API conventions](../references/api-conventions.md).

## Next

[Lesson 7: Filtering faces (`keep` / `ignore`)](07-restrict-faces.md)
