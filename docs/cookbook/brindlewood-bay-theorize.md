---
title: "Brindlewood Bay — Theorize"
author: Steve Cooper
---

# Brindlewood Bay — Theorize

## At the table

In [*Brindlewood Bay*](https://evilhat.itch.io/brindlewood-bay), the **Theorize** move is how the Mavens nail down whodunit. Once they have talked through a theory (and hold enough Clues—at least half the mystery’s **Complexity**, rounded up), they roll:

**2d6 + Clues in the theory − Complexity**

Read the total:

| Total | Result |
|-------|--------|
| **6−** | The theory is **wrong**; the Keeper responds. |
| **7–9** | The theory is **right**, but the Keeper adds a snag to the answer or makes acting on it risky. |
| **10–11** | The theory is **right**; the Keeper offers a clear chance to stop the culprit or avert disaster. |
| **12+** | As 10+, and someone tied to Brindlewood’s conspiracy **outs themselves** to the Mavens. |

“Clues in the theory” means clues you folded into the explanation or accounted for—not every Clue the table might hold.

## Try it

In the playground, copy the script below into the **editor** and click **Run**. Defaults use Complexity **6** and **5** clues in the roll; tweak `COMPLEXITY` and `CLUES` in the editor to match your mystery.

## The script

```text
Scale = scale(["INCORRECT", "CORRECT_COMPLICATION", "CORRECT", "CONSPIRACY_REVEAL"], ..6, 7..9, 10..11, 12..)

COMPLEXITY = 6
CLUES = 5

roll = 2d6 + CLUES - COMPLEXITY
out = roll.bucket(Scale)
output("theorize", out)
output("p_not_wrong", out.p_at_least("CORRECT_COMPLICATION"))
output("p_clean_correct", out.p_at_least("CORRECT"))
output("p_conspiracy_reveal", out.p_at_least("CONSPIRACY_REVEAL"))

rows = [
    ("{} clues in theory".format(c), (2d6 + c - COMPLEXITY).bucket(Scale).p_at_least("CORRECT_COMPLICATION"))
    for c in range(3, 9)
]
output("p_clean_by_clues", prob_table(rows))
```

- Four labels get **four ranges**: `..6`, `7..9`, `10..11`, `12..` (same idea as the [PbtA 2d6 lesson](../tutorial/13-pbta-2d6-move.md), with an extra top band). Equivalent to cuts `[6, 9, 11]`.
- `p_at_least("CORRECT")` is **10+** (clean correct plus conspiracy reveal).
- `prob_table` lists how **7+** odds change (any correct theory, including complications) if you put **3–8** clues into the theory at fixed Complexity (3 is the minimum when Complexity is 6—half Complexity, rounded up).

## Cookbook

[All recipes](README.md)
