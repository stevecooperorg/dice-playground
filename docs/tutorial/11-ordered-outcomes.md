---
title: "Ordered outcome labels"
author: Steve Cooper
---

# Ordered outcome labels

## At the table

Some games speak in **named bands** (failure, success, critical) instead of a raw total. You still want exact probabilities—and sometimes “success or better” on that ladder.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
Scale = (
    scale()
    .step("CRITICAL_FAIL", ..5)
    .step("FAIL", 6..10)
    .step("SUCCESS", 11..15)
    .step("CRITICAL_SUCCESS", 16..)
)
roll = 1d20
out = roll.bucket(Scale)
output("check", out)
output("p_success_plus", out.p_at_least("SUCCESS"))
```

- `scale()` starts an empty ladder; each `.step(label, band)` adds one rank (low → high). Omit the band when you only need labels for `classify`.
- Bands on `Scale` map each total into a label (`..5`, `6..10`, `11..15`, `16..`). `roll.bucket(Scale)` uses them (same odds as `bucket(roll, scale, [5, 10, 15])` with a label-only scale).
- `output` on a `Outcomes` shows a probability table (each label and its exact chance) in scale order in **text** and **json**.
- `p_at_least("SUCCESS")` sums every label at that rank or higher.

## Reading the result

The **text** tab lists each label and its probability. The **json** tab uses `"kind": "ordinal"` with `scale` and `entries`.

## Try this

- Change the ranges to match your game’s DC bands.
- Add `out.p_at_most("FAIL")` for “failure or worse.”

Builtin details: [standard library reference](../references/stdlib.md) (`scale`, `Scale.step`, `bucket`).

## What’s next

[D&D 5e d20 checks](12-dnd5e-d20-check.md)—nat 1, nat 20, advantage, and modifiers with `scale().step(...)` on the **natural** die (not fixed bands on `1d20` alone, and not `1d20 + MOD` for true 5e crits).

For **two dice** with custom rules (e.g. white/black PbtA-style), see `games-systems/white-and-black-story.dice` and `joint_classify` in the reference.
