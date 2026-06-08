---
title: "D&D 5e d20 checks with classify"
author: Steve Cooper
---

# D&D 5e d20 checks with classify

## At the table

A D&D attack or save is not just “did the total beat the DC?” **Natural 1** and **natural 20** are special on the d20 you keep. **Advantage** and **disadvantage** change which natural faces are likely before you add your modifier.

Lesson 7’s `bucket` splits a single number line into bands. That works when only totals matter. It does **not** treat nat 1 and nat 20 as exceptions on the die face—which 5e needs.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
Scale = scale(["CRITICAL_FAIL", "FAIL", "SUCCESS", "CRITICAL_SUCCESS"])
DC = 15
MOD = 5

def label(n):
    if n == 1:
        return "CRITICAL_FAIL"
    if n == 20:
        return "CRITICAL_SUCCESS"
    if n + MOD >= DC:
        return "SUCCESS"
    return "FAIL"

natural = 2d20kh1
out = classify(natural, Scale, label)
output("advantage_check", out)
output("p_hit_or_better", out.p_at_least("SUCCESS"))
```

- **`2d20kh1`** is **advantage** (roll two d20, keep the highest one). Same idea as `keep_highest(2, 20, 1)` from [lesson 5](05-dice-notation.md).
- Use **`1d20`** for a normal roll or **`2d20kl1`** for disadvantage.
- `classify(dist, scale, fn)` applies your function to each outcome in `dist` and builds an `Outcomes`. Here `n` is the **natural** kept die, not `n + MOD`.
- Nat 20 still counts as `CRITICAL_SUCCESS` even if `MOD` is negative; nat 1 still counts as `CRITICAL_FAIL` even with a high `MOD`.

## Reading the result

You get a four-row probability breakdown (one row per outcome label) plus `p_at_least("SUCCESS")`, which includes both ordinary successes and critical successes.

## Try this

- Set `MOD` and `DC` to match a character you care about.
- Swap `2d20kh1` for `1d20` and compare probabilities.
- Compare with lesson 7’s `bucket(1d20 + MOD, …)` on the same DC—notice how crit bands differ.

Builtin details: [standard library reference](../references/stdlib.md) (`classify`, `keep_highest`, `keep_lowest`).

## What’s next

[Powered by the Apocalypse (2d6 + stat)](09-pbta-2d6-move.md)—miss / partial / full on the **total** with `bucket`, the usual PbtA pattern.

See the [user guide index](../README.md). A dedicated `dnd5e_d20_check` helper may be added later; `classify` is the general tool underneath.
