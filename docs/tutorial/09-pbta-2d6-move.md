---
title: "Powered by the Apocalypse (2d6 + stat)"
author: Steve Cooper
---

# Powered by the Apocalypse (2d6 + stat)

## At the table

In **Powered by the Apocalypse** games (Apocalypse World, Dungeon World, Monster of the Week, and many others), you roll **2d6 + stat** and read the **total**:

| Total | Usual result |
|-------|----------------|
| **10+** | Full success |
| **7–9** | Partial success—you get what you want, but with a cost, complication, or harder choice |
| **6−** | Miss—the GM makes a move |

There is no separate “natural 20” on the dice; the whole outcome is the **total** (2d6 + stat). That is exactly what lesson 7’s `bucket` is for.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
Outcome = result_type(["MISS", "PARTIAL", "FULL_SUCCESS"])
STAT = 2

roll = 2d6 + STAT
out = bucket(roll, Outcome, [6, 9])
output("move", out)
output("p_full_success", out.p_at_least("FULL_SUCCESS"))
output("p_partial_or_better", out.p_at_least("PARTIAL"))
```

- `2d6 + STAT` is the usual PbtA notation (lesson 3); same odds as `shift(2d6, STAT)` if you prefer the function form.
- `bucket` with cuts `[6, 9]` and three labels maps totals **≤ 6** → `MISS`, **7–9** → `PARTIAL`, **10+** → `FULL_SUCCESS`.
- `p_at_least("PARTIAL")` is the chance you do **not** miss (partial or full).

## Reading the result

You get a three-row probability breakdown for the move, plus scalar probabilities for full success and for “7+” style outcomes.

With `STAT = 0`, the bands match the core rulebook: 15/36 miss, 15/36 partial, 6/36 full success on the 2d6 alone.

## Try this

- Set `STAT` to a typical score for your playbook (+0, +2, +3).
- Compare `p_full_success` across stats in a small loop (lesson 6 style) if you want a chart.

## What’s next

See the [user guide index](../README.md). Lesson 8 covered when you need `classify` on a **natural** die (D&D 5e); PbtA totals stay on `bucket`.
