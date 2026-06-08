---
title: "Adding two dice"
author: Steve Cooper
---

# Adding two dice

## At the table

You roll two six-sided dice and add them—classic 2d6.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**).

## The script

```text
output("two_d6", 2d6)
```

`2d6` means roll two six-sided dice and add them. The engine builds the combined distribution (2 is unlikely, 7 is most common).

## Reading the result

Under **Output**, **mean** should be about **7**. The **text** tab’s table shows more weight on middle totals (6–8) and less on 2 or 12; **graph** makes the bell shape obvious.

## Try this

- In the editor, try `2d10` or `1d10 + 1d10` for the same roll.

## Next

[Lesson 3: Flat bonuses (2d10 + 5)](03-modifiers.md)
