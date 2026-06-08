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

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
def natural_check_scale(labels, dc, mod):
    t = dc - mod
    s = scale().step(labels[0], 1..1)
    if t >= 20:
        s = s.step(labels[1], 2..19).step(labels[2])
    elif t >= 2:
        s = s.step(labels[1], through(2, t - 1))
        if t <= 19:
            s = s.step(labels[2], through(t, 19))
        else:
            s = s.step(labels[2])
    else:
        s = s.step(labels[1]).step(labels[2], 2..19)
    return s.step(labels[3], 20..20)

LABELS = ["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"]
DC = 15
MOD = 5

Scale = natural_check_scale(LABELS, DC, MOD)
natural = 2d20kh1
out = natural.bucket(Scale)
output("advantage_check", out)
output("p_hit_or_better", out.p_at_least("SUCCESS"))
```

- **`natural_check_scale`** builds crit pins on faces 1 and 20, miss/hit split at `T = DC - MOD` (here `T = 10`), including edge cases when `T` is outside `2..19`.
- When `T` is always in range, you can inline: `scale().step("CRITICAL_FAIL", 1..1).step("FAIL", 2..(T - 1)).step("SUCCESS", T..19).step("CRITICAL_SUCCESS", 20..20)`.
- **`2d20kh1`** is **advantage**. Use **`1d20`** for a normal roll or **`2d20kl1`** for disadvantage.
- Nat 20 still counts as `CRITICAL_SUCCESS` even if `MOD` is negative; nat 1 still counts as `CRITICAL_FAIL` even with a high `MOD`.

## Reading the result

You get a four-row probability breakdown (one row per outcome label) plus `p_at_least("SUCCESS")`, which includes both ordinary successes and critical successes.

## Try this

- Set `MOD` and `DC` to match a character you care about.
- Swap `2d20kh1` for `1d20` and compare probabilities.
- Compare with lesson 11’s fixed bands on `1d20`—notice how crit + DC logic differs.

Builtin details: [standard library reference](../references/stdlib.md) (`scale`, `Scale.step`, `bucket`, `keep_highest`, `keep_lowest`). Use `classify` when house rules do not fit four face bands.

## What’s next

[Powered by the Apocalypse (2d6 + stat)](13-pbta-2d6-move.md)—miss / partial / full on the **total** with `bucket`, the usual PbtA pattern.

See the [user guide index](../README.md).
