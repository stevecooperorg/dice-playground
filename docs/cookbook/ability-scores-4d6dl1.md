---
title: "4d6 drop lowest (ability scores)"
author: Steve Cooper
---

# 4d6 drop lowest (ability scores)

## At the table

Classic **D&amp;D** character creation: roll **four** d6, drop the **lowest**, sum the rest. The average is about **12.24**, noticeably higher than straight 3d6.

## Try it

In the playground, copy the script below into the **editor** and click **Run**.

## The script

```text
output("ability", 4d6dl1)
```

Sugar expands `4d6dl1` to `drop_lowest(4, 6, 1)`, which returns the full distribution of ability scores.

Lesson [5 — Dice notation](../tutorial/05-dice-notation.md) walks through `dl1`, `kh`, and related notation step by step.

## Cookbook

[All recipes](README.md)
