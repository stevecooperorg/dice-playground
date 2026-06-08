---
title: "Many checks at once"
author: Steve Cooper
---

# Many checks at once

## At the table

You want a **grid**: for many modifiers and target numbers, what is the chance 2d10 + modifier meets each target?

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
roll = 2d10
rows = [
    ("modifier {}, target {}".format(mod, target), (roll + mod).p_ge(target))
    for mod in range(-2, 11)
    for target in range(0, 11)
]
output("success_grid", prob_table(rows))
```

Starlark supports **list comprehensions** like Python: the two `for` clauses walk every modifier and target and build one `(label, probability)` tuple per pair.

Equivalent with nested loops (lists are immutable, so you concatenate instead of `.append`):

```text
rows = []
for mod in range(-2, 11):
    for target in range(0, 11):
        rows = rows + [
            ("modifier {}, target {}".format(mod, target), (roll + mod).p_ge(target))
        ]
```

## Strategy: one table, not many `output` calls

Calling `output(name, probability)` inside a loop is fine for a handful of values, but each call becomes its **own** one-row “Prob” block in the UI—repeated headers and hundreds of tiny tables.

For grids and parameter sweeps:

1. Build a list of `(label, probability)` rows—often with a **comprehension** over your parameter ranges.
2. Call **`output` once** with `prob_table(rows)`.

`prob_table` takes a list of `(string label, probability)` tuples. Probabilities are **independent**—they do not need to sum to 1 (unlike `LabelDist` / `bucket` outcomes, which describe a single roll).

Use a plain `output(..., float)` when you want exactly one probability. Use `prob_table` when you want one multi-row table with the same `% / frac / X` columns as distribution output.

## Reading the result

One block `output success_grid: Table` with a row per modifier/target pair (labels like `modifier -2, target 0`, `modifier 5, target 3`, …). For a structured copy, open the **json** tab—table rows use `"kind": "table"`.

## Try this

- Narrow `range` in the editor to match your game (fewer lines).
- Change `2d10` to another roll you care about.
- Tweak the label string (e.g. `"target {} @ modifier {}".format(target, mod)`) if that sorts better in a spreadsheet.

## What’s next

[Ordered outcome labels](07-ordered-outcomes.md)—named success bands instead of only numeric totals.

See the [user guide index](../README.md) and [standard library reference](../references/stdlib.md) for builtins.
