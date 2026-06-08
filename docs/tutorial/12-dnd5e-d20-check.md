---
title: "D&D 5e d20 checks with bucket"
author: Steve Cooper
---

# D&D 5e d20 checks with bucket

## At the table

A D&D attack or save is not just “did the total beat the DC?” **Natural 1** and **natural 20** are special on the d20 you keep. **Advantage** and **disadvantage** change which natural faces are likely before you add your modifier.

[Lesson 11](11-ordered-outcomes.md) buckets a **raw d20 face** into four fixed ranges. That teaches ordered labels, but it is **not** the same as a DC check with a modifier. Here we stay on the **natural** die (`1d20`, `2d20kh1`, …) and put the modifier into the target number instead of into the roll.

## Adjusted target (stay on 1..20)

Table rule: success when `natural + MOD >= DC`. For integer faces that is the same as:

```text
T = DC - MOD
```

…and **hit** when `natural >= T`, still with nat **1** → critical fail and nat **20** → critical success.

So you build a **four-band scale** on faces `1..20`, then `natural.bucket(Scale)`—the same pattern as PbtA in [lesson 13](13-pbta-2d6-move.md), but the bands move when `DC` or `MOD` change.

## Overlapping bands (`early=True`)

Miss and hit use **open** ranges (same idea as PbtA `..6` and `10..`):

- **Fail:** `at_most(T - 1)` — every face strictly below the target.
- **Success:** `at_least(T)` — target or higher.

Nat **1** and nat **20** sit in those ranges too. Use **`early=True`** on the narrow crit steps so they win when bucketing, without moving them ahead of fail/success in the ladder (so `p_at_least("SUCCESS")` still counts ordinary and critical successes).

Declaration order (ladder rank): crit fail → fail → success → crit success. Match order: both **`early`** crit steps first (in that order), then fail and success.

Wrap the scale in a Starlark **`def`** so you can reuse the same bands for any DC and modifier—`check_scale(DC, MOD)` returns a `Scale` you pass to `bucket` inline. Names that start with `d20` right after `def` break parsing (`d20test` becomes `d(20)` + `test`); use something like `check_scale` or `make_d20test` instead.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
def check_scale(DC, MOD):
    T = DC - MOD
    return (
        scale()
        .step("CRITICAL_FAIL", 1..1, early=True)
        .step("FAIL", at_most(T - 1))
        .step("SUCCESS", at_least(T))
        .step("CRITICAL_SUCCESS", 20..20, early=True)
    )

DC = 15
MOD = 5
natural = 2d20kh1
out = natural.bucket(check_scale(DC, MOD))
output("advantage_check", out)
output("p_hit_or_better", out.p_at_least("SUCCESS"))
```

- **`check_scale`** builds the four-band scale from `T = DC - MOD`; call it wherever you need a check (`1d20.bucket(check_scale(12, 4))`, loops over DCs, and so on).
- **`at_most` / `at_least`** accept expressions; use them when `T` comes from `DC` and `MOD` (range sugar `..(T - 1)` only works with numeric literals in source).
- **`2d20kh1`** is **advantage**. Use **`1d20`** for a normal roll or **`2d20kl1`** for disadvantage.
- Without **`early=True`** on crit bands, overlapping open ranges would mis-label nat 1 or nat 20—see [lesson 11](11-ordered-outcomes.md).

## Reading the result

You get a four-row probability breakdown (one row per outcome label) plus `p_at_least("SUCCESS")`, which includes both ordinary successes and critical successes.

## Try this

- Change `DC` and `MOD` (or add a `for` loop) and call `check_scale(DC, MOD)` for each combination.
- Swap `2d20kh1` for `1d20` and compare probabilities.
- Compare with lesson 11’s fixed bands on `1d20`—notice how crit + DC logic differs.

Builtin details: [standard library reference](../references/stdlib.md) (`scale`, `Scale.step`, `bucket`, `at_most`, `at_least`, `keep_highest`, `keep_lowest`). Use `classify` when house rules do not fit four face bands.

## What’s next

[Powered by the Apocalypse (2d6 + stat)](13-pbta-2d6-move.md)—miss / partial / full on the **total** with `bucket`, the usual PbtA pattern.

See the [user guide index](../README.md).
