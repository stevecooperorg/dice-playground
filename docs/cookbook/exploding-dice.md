---
title: "Exploding dice"
author: Steve Cooper
---

# Exploding dice

## At the table

**Exploding** (or “open-ended”) dice add another roll when you hit the die’s **maximum** face, and keep adding until you stop exploding. *Savage Worlds* uses this on raising and damage; many house rules explode d4s or d6s on max.

## Try it

In the playground, copy the script below into the **editor** and click **Run**. **Output** shows the distribution of one **d4** exploding up to **two** extra rolls on max.

## The script

```text
output("exploded", explode(d(4), 2))
```

- `explode(die, max_depth)` sums the initial roll plus rerolls while the face equals the die’s maximum.
- Increase `max_depth` for longer tails; very high depths widen the support quickly.

For **multiple** exploding dice, build a pool of exploded dice (e.g. sum independent `explode(d(6), 2)` via convolution) or roll pools with per-die rules via `pool_map`.

## Cookbook

[All recipes](README.md)
