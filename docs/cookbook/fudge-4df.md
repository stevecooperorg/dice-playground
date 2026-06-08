---
title: "Fudge — 4dF (fudge dice)"
author: Steve Cooper
---

# Fudge — 4dF (fudge dice)

## At the table

[*Fudge*](https://en.wikipedia.org/wiki/Fudge_(role-playing_game_system)) (Freeform Universal Do-it-yourself Gaming Engine) and many *Fate* games use **fudge dice**—often notated **4dF**. Each die is a d6 with **two minus (−), two blank (0), and two plus (+)** faces. You roll **four** fudge dice and **add** the results, so the modifier is usually between **−4 and +4**, centered on **0**.

That roll shifts your **trait level** on Fudge’s adjective ladder (Terrible → … → Superb). The dice rarely dominate a skilled character’s baseline, but **+4** or **−4** can still swing a check into critical territory—exact thresholds depend on the ladder and difficulty your table uses.

Physically, players read `+`, `−`, and blank; in the playground we model each die as **`−1`, `0`, and `+1`**.

## Try it

Copy the script into the playground **editor** and click **Run**. **Output** shows the full **4dF** distribution plus the chance of hitting **+4 or better** and **−4 or worse** (common “extreme shift” bands).

## The script

```text
FudgeDie = die_faces([-1, -1, 0, 0, 1, 1])
four_df = FudgeDie + FudgeDie + FudgeDie + FudgeDie
output("4dF", four_df)
output("p_plus4_or_better", four_df.p_ge(4))
output("p_minus4_or_worse", four_df.cdf(-4))
```

- `die_faces([...])` builds one **custom** die; duplicate entries in the list weight each face (here, two of each symbol on a d6).
- Adding four independent `FudgeDie` distributions is the same as rolling **4dF** and summing—no need for uniform `dice_pool(n, 6)`.

## Cookbook

[All recipes](README.md)
