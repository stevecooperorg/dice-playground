---
title: "Dice notation step by step"
author: Steve Cooper
---

# Dice notation step by step

## At the table

Tabletop games write rolls as **XdY** (X dice, Y sides), sometimes with a **bonus** or **drop lowest**. This lesson walks from the smallest form to **4d6dl1** ability scores.

## Try it

In the playground, copy the script below into the **editor**, then click **Run** (or press **Shift+Enter**). **Output** lists five distributions in order—one per stage below.

## Stage 1: one die (`1d4`)

```text
output("one_d4", 1d4)
```

- **`1d4`** — roll one four-sided die (faces 1–4). Same pattern as `1d6`, `1d20`, etc.
- Omitted count means one die: `d4` and `1d4` are the same idea (the tool expands both).

**Mean** ≈ **2.5**.

## Stage 2: add dice (`2d6`)

```text
output("two_d6", 2d6)
```

- **`2d6`** — roll two six-sided dice and **add** them.
- Sum of two independent d6 (same idea as lesson 2); the engine expands this to `dice_pool(2, 6)`.

**Mean** ≈ **7**.

## Stage 3: flat bonus (`2d6 + 3`)

```text
output("two_d6_plus_3", 2d6 + 3)
```

- **`+ 3`** adds 3 to **every** total on the roll—like a fixed stat bonus on the sum ([lesson 3](03-modifiers.md)).
- Equivalent to `shift(2d6, 3)` when you already have a `DieRoll` in a variable.

**Mean** ≈ **10** (7 + 3).

## Stage 4: many dice (`4d6`)

```text
output("four_d6", 4d6)
```

- **`4d6`** — roll four d6 and sum **all** of them (no drops).

**Mean** ≈ **14** (four times the mean of one d6).

## Stage 5: drop lowest (`4d6dl1`)

```text
output("four_d6dl1", 4d6dl1)
```

- **`dl1`** — **d**rop **l**owest **1** die before summing (classic ability-score roll).
- The tool expands this to `drop_lowest(4, 6, 1)`.
- **`4d6dl2`** would drop the two lowest; **`3d6dl1`** is three dice drop one—same `dlN` pattern.

**Mean** ≈ **12.24** (higher than straight `4d6` because the worst die never counts).

## Stage 6: drop highest (`4d6dh1`)

```text
output("four_d6dh1", 4d6dh1)
```

- **`dh1`** — **d**rop **h**ighest **1** die, then sum the rest (the opposite of `dl1`).
- Expands to `drop_highest(4, 6, 1)`.

**Mean** ≈ **8.76**.

## Stage 7: keep highest N (`4d6kh2`)

```text
output("four_d6kh2", 4d6kh2)
```

- **`kh2`** — roll four d6, **k**eep the **h**ighest **2**, sum only those (advantage-style “best two”).
- Expands to `keep_highest(4, 6, 2)`. Not the same as `dl2` (which drops two lows and sums the other two dice).

**Mean** ≈ **9.34**.

## Stage 8: keep lowest N (`3d12kl1`)

```text
output("three_d12kl1", 3d12kl1)
```

- **`kl1`** — **k**eep the **l**owest **1** die and sum it (here, just that single die).
- Expands to `keep_lowest(3, 12, 1)`.

**Mean** ≈ **3.52** (average of the minimum of three d12).

## Suffix cheat sheet

| Suffix | Meaning | Function |
|--------|---------|----------|
| `dlN` | Drop lowest N, sum the rest | `drop_lowest` |
| `dhN` | Drop highest N, sum the rest | `drop_highest` |
| `khN` | Keep highest N, sum those | `keep_highest` |
| `klN` | Keep lowest N, sum those | `keep_lowest` |

## Reading the result

Eight named outputs in the sample script, each with mean and a probability table (or truncated table for large supports).

## Try this

- Change `1d4` to a die your game uses (`1d8`, `1d12`).
- Compare `2d6 + 3` and `shift(2d6, 3)`—means should match.
- Try `4d6kh3` vs `4d6dl1`—both sum three dice, but different sets of faces count.

## Next

[Lesson 6: Many checks at once](06-tables.md)
