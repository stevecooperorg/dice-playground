---
title: "Blades in the Dark — action roll"
author: Steve Cooper
---

# Blades in the Dark — action roll

## At the table

[*Blades in the Dark*](https://www.evilhat.com/en/home/home.html#blades) resolves risky actions with **d6 pools**. Roll all your dice at once and use only the **highest** face—everything else is ignored unless you need a critical (see below).

| Highest die | What happens |
|-------------|----------------|
| **1–3** | **Bad outcome.** Things go wrong: you likely miss your goal and the GM brings extra trouble. |
| **4–5** | **Partial success.** You get what you were after, but with a cost—harm, heat, reduced effect, a hard choice, or similar. |
| **6** | **Full success.** It works; things go as you hoped. |
| **Two or more 6s** | **Critical success.** As a full success, plus an extra edge (position, effect, or another boon depending on the move). |

**Building the pool:** Take a number of dice equal to a **rating**—usually a player’s **action** (Prowl, Skirmish, Attune, and the rest) or sometimes crew **Tier**, a situation bonus, or a push. Ratings are often **one to four dice** in play. Even **one die** is respectable: you have a **50%** chance of partial success or better (4+ on that single die).

**Zero or negative dice:** If you would roll no dice (or fewer than zero after modifiers), roll **2d6** and use the **lower** die only—the desperate position. You still use the same outcome bands, but you **cannot** roll a critical from a desperate roll (at most one 6 counts).

Most specialized rolls in the book (resistance, group actions, fortune rolls, etc.) are variations on this core chart. When you are learning, you can always fall back to “roll the pool, read the highest die” and look up the exact twist later.

**Why partial success dominates:** On typical pools, **4/5 is the most common band**—characters often succeed, but rarely cleanly. That matches the game’s pitch: scrappy criminals in too deep. Complications are where play gets interesting; the dice keep nudging you there on purpose.

## Try it

In the playground, copy the script below into the **editor** and click **Run**. **Output** entries **`0d`** through **`7d`** give exact odds for each outcome band. Pool sizes **1d–4d** are what you see most often at the table; **`0d`** is the desperate case. **`5d`–`7d`** are included so you can see how crit-heavy larger pools behave.

## The script

```text
Scale = scale(["BAD", "MESSY", "CLEAN", "CRITICAL"])

def blades_pool(faces):
    high = max(faces)
    if high == 6:
        if len([f for f in faces if f == 6]) >= 2:
            return 3
        return 2
    if high >= 4:
        return 1
    return 0

def code_label(c):
    return ["BAD", "MESSY", "CLEAN", "CRITICAL"][c]

def desperate_label(n):
    if n == 6:
        return "CLEAN"
    if n >= 4:
        return "MESSY"
    return "BAD"

output("0d", classify(keep_lowest(2, 6, 1), Scale, desperate_label))
for dice in range(1, 8):
    codes = pool_map(dice_pool(dice, 6), blades_pool)
    output("{}d".format(dice), classify(codes, Scale, code_label))
```

Script labels map to the table above: `BAD` (1–3), `MESSY` (partial / 4–5), `CLEAN` (full / single 6), `CRITICAL` (two or more 6s).

- `pool_map` walks every joint outcome of the pool as a **list of face values**—needed because crits depend on **how many 6s** showed up, not the highest face alone.
- `keep_lowest(2, 6, 1)` is the desperate roll (lowest of 2d6); `desperate_label` never returns `CRITICAL`.
- On the **`1d`** row, `MESSY` + `CLEAN` + `CRITICAL` sums to **50%**—the “one die is pretty good” case from the rulebook.

## Cookbook

[All recipes](README.md)
