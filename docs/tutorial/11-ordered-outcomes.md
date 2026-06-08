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
Scale = scale(["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"])
roll = 1d20
out = roll.bucket(Scale, ..5, 6..10, 11..15, 16..)
output("check", out)
output("p_success_plus", out.p_at_least("SUCCESS"))
```

- `scale` defines an **ordered** list of labels (low rank → high rank).
- `roll.bucket(Scale, …)` maps the numeric total into those labels using **one inclusive range per label** (`..5`, `6..10`, `11..15`, `16..`). Same odds as `bucket(roll, Scale, [5, 10, 15])` with cut-style bands.
- `output` on a `Outcomes` shows a probability table (each label and its exact chance) in scale order in **text** and **json**.
- `p_at_least("SUCCESS")` sums every label at that rank or higher.

## Reading the result

The **text** tab lists each label and its probability. The **json** tab uses `"kind": "ordinal"` with `scale` and `entries`.

## Try this

- Change the ranges to match your game’s DC bands.
- Add `out.p_at_most("FAIL")` for “failure or worse.”

Builtin details: [standard library reference](../references/stdlib.md).

## What’s next

[D&D 5e d20 checks](12-dnd5e-d20-check.md)—nat 1, nat 20, advantage, and modifiers with `classify` (not `bucket` on `1d20 + MOD` alone).

For **two dice** with custom rules (e.g. white/black PbtA-style), see `games-systems/white-and-black-story.dice` and `joint_classify` in the reference.
