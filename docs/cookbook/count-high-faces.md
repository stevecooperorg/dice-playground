---
title: "Count high faces in a pool"
author: Steve Cooper
---

# Count high faces in a pool

## At the table

Many games care about **how many dice** in a pool beat a threshold—not just the total. Examples include counting successes on d10 pools (Storyteller / World of Darkness family), tallying raises, or custom house rules on “how many dice rolled 5+.”

## Try it

In the playground, copy the script below into the **editor** and click **Run**. It asks: on **3d6**, how many dice rolled **greater than 4**?

## The script

```text
def count_high(faces):
    return len([f for f in faces if f > 4])

output("success_count", pool_map(roll_pool(3, 6), count_high))
```

- `pool_map` walks every joint outcome of the pool and calls your function with a **list of face values**.
- Inside the function, a list comprehension filters faces—handy when the rule is easier to read in Starlark than as a single builtin.

For a fixed threshold on fair dice you can also use `count_ge(roll_pool(n, sides), threshold)` without a custom function.

## Cookbook

[All recipes](README.md)
