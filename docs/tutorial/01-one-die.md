---
title: "Your first die"
author: Steve Cooper
---

# Your first die

## At the table

You roll one ordinary six-sided die and want to see how likely each face is.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
output("one_d6", 1d6)
```

- `1d6` is one fair die (faces 1 through 6); the tool expands it before running.
- `output(...)` records the full distribution so it appears in the results.

This tool **counts** outcomes; it does not simulate thousands of rolls.

## Reading the result

Under **Output**, the **text** tab shows a header with **mean** (average roll, 3.5 for one d6) and a small table: each face and its probability (about `0.166667` per face). The **graph** tab charts the same distribution.

## Try this

- Change `1d6` to `1d20` in the editor and run again.

## Next

[Lesson 2: Adding two dice](02-two-dice.md)
