---
title: "The Pool — any natural 1 succeeds"
author: Steve Cooper
---

# The Pool — any natural 1 succeeds

## At the table

In James V. West’s [*The Pool*](https://www.drivethrurpg.com/en/product/210088/the-pool), you roll a pool of d6. If **any** die shows a **1**, the action succeeds; otherwise it fails. Pool size depends on how much you stake on the attempt.

## Try it

In the playground, copy the script below into the **editor** and click **Run**. **Output** shows success probabilities for pool sizes 1d through 10d.

## The script

```text
for dice in range(1, 11):
    p = count_in(dice_pool(dice, 6), [1]).p_ge(1)
    output("{}d".format(dice), p)
```

- `dice_pool(n, 6)` is **n** fair d6 not yet summed.
- `count_in(..., [1])` yields a distribution of how many 1s appeared; `.p_ge(1)` is the chance of at least one 1.

Similar “any success die” rules appear in other indie designs; the same pattern works with different target faces or `count_ge` for “8+ on d10” pools.

## Cookbook

[All recipes](README.md)
