---
title: "Rolemaster — open-ended d100"
author: Steve Cooper
---

# Rolemaster — open-ended d100

## At the table

Most Rolemaster rolls are **1–100 rolls** (also called **d100** rolls): roll **two d10s**, one as tens and one as ones, for a result from **01** to **100** (**00** counts as **100**).

On top of that flat percentile, the books define three related procedures (RMSS core rules):

### Low open-ended roll

Make a **1–100** roll. If the result is **01–05**, roll again and **subtract** that second result from the first. If that second roll is **96–00**, roll a third time and subtract it too, and keep going until a reroll is **not** **96–00**. The **total of all those rolls** (first minus the rerolls) is the low open-ended result.

**Example:** Initial **04**, then **97**, then **03** → **04 − 97 − 03 = −96**. (The rulebook’s “97” line is a typo for **96–00**—97 is in the high-open band, not 01–05.)

### High open-ended roll

Make a **1–100** roll. If the result is **96–00**, roll again and **add** the second result. If that roll is again **96–00**, roll a third time and add it, and so on until a reroll is **not** **96–00**. The **total sum** is the high open-ended result.

**Example:** Initial **99**, then **96**, then **04** → **99 + 96 + 04 = 199**.

### Open-ended roll

An **open-ended roll** is **both** low open-ended and high open-ended: **01–05** on the first die starts the subtracting chain; **96–00** on the first die starts the adding chain; **06–95** uses the first roll only.

Only **96–00** on a **reroll** extends the chain—other faces (including **01–05** on a reroll) are applied once and then stop unless the next reroll is again **96–00**. That is how a maneuver can crater to **−400** or spike past **500** before skill bonuses.

In this recipe, **96–00** means the five outcomes **96, 97, 98, 99, and 100** (100 is the table **00**).

| Procedure | First roll triggers | Reroll chain continues on |
|-----------|---------------------|---------------------------|
| Low open-ended | **01–05** | **96–00** only |
| High open-ended | **96–00** | **96–00** only |
| Open-ended (full) | **01–05** or **96–00** | **96–00** only |

Many tables use only **high open-ended** (attacks) or the full **open-ended** roll (maneuvers). Static rolls sometimes treat **66** or **100** specially and skip opening—this script models the core rule above only.

## Try it

Copy the script into the playground **editor** and click **Run**. **Output** includes the full **open-ended** distribution (with a chain cap), the chance of a flat **50** with no reroll, and rough tail weights (**150+** and **0 or below**).

## The script

```text
oe = open_ended_d100(8)
output("open_ended_d100", oe)
output("p_unmodified_mid", oe.pmf(50))
output("p_at_least_150", oe.p_ge(150))
output("p_zero_or_below", oe.cdf(0))
```

- `open_ended_d100(max_chain)` implements the **open-ended roll** on **1–100**. `max_chain` limits how many consecutive **96–00** rerolls may follow an open trigger (raise it for longer tails, like `explode(..., max_depth)`).
- Compare with [`explode`](exploding-dice.md): Savage Worlds-style explode only fires on the **maximum** face and always **adds**; Rolemaster opens on **both** ends and **subtracts** on low open.

## Cookbook

[All recipes](README.md)
