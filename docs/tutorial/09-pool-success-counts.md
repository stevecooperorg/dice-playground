---
title: "Pool success counts"
author: Steve Cooper
---

# Pool success counts

## At the table

Pool games often ask: **how many** dice succeeded, or did **any** die show a particular face? Indie designs like [The Pool](../cookbook/the-pool.md) use “any natural 1”; other rules want “at least two 8+ on 5d10.”

## Try it

Copy the script into the playground editor and click **Run**.

## The script

```text
output("how_many_high", 3d6.count(5..))
output("any_one", dice_pool(2, 6).p_any(1))
```

- `3d6` is a pool (notation leaves dice separate before `.count`).
- `.count(5..)` returns a **distribution** over 0, 1, 2, or 3—the number of dice that showed 5 or 6.
- `.p_any(1)` is a single **probability**: at least one die showed a 1 (11/36 on 2d6).

Related helpers on pools:

- `p_none(spec)` — no die matched.
- `p_at_least(k, spec)` — at least *k* dice matched (e.g. two or more 8+).

Use the same **face spec** as `keep` / `count`: int, `[faces]`, or ranges like `5..`.

## Reading the result

`how_many_high` is a full `DieRoll` table. `any_one` prints one probability (like lesson 4’s `p_ge`).

## Try this

- Match [The Pool](../cookbook/the-pool.md) with `dice_pool(n, 6).p_any(1)` in a loop.
- Output `5d10.count(8..)` and read how often you get 0, 1, … successes.

## Next

[Lesson 10: Many checks at once](10-tables.md)
