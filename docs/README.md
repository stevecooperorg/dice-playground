---
title: "Dice language user guide"
author: Steve Cooper
---

# User guide

Documentation for **people who play tabletop games** and want exact odds—not for Rust contributors (see the [repository README](../README.md) for that).

Scripts use the **`.dice`** extension: familiar notation like `2d6` and `4d6dl1`, plus Starlark for modifiers, loops, and outputs.

Use the site header (or the sections below) for **Tutorial**, **Cookbook**, and **Function reference**.

## Playground

In the playground, each lesson and recipe includes a script to copy into the **editor** (or click **↗ Load in playground** on a script block in the tutorial or cookbook—build-time links open `/` with the script in the URL). Click **Run** (or press **Shift+Enter**) and read results under **Output**—**text**, **json**, or **graph**. Use the **Files** control to manage multiple scripts; **Diagnostics** lists parse and type errors with line numbers.

## Tutorial

A **step-by-step** introduction to the language. Work through the lessons in order.

| Lesson | Topic |
|--------|--------|
| [1. Your first die](tutorial/01-one-die.html) | One fair die |
| [2. Adding two dice](tutorial/02-two-dice.html) | 2d6 |
| [3. Flat bonuses (2d10 + 5)](tutorial/03-modifiers.html) | +5 to every total |
| [4. Meet or beat a target](tutorial/04-success.html) | Success chance on a total |
| [5. Dice notation](tutorial/05-dice-notation.html) | `1d4` … `4d6dl1` |
| [6. Dice pools](tutorial/06-dice-pools.html) | Faces still separate; `order_stat` |
| [7. Mixed dice pools](tutorial/07-mixed-dice-pools.html) | Join pools with `+`; 1d12 + 2d6 |
| [8. Filtering faces](tutorial/08-restrict-faces.html) | `keep` / `remove` / `convert` / `ignore` |
| [9. Pool success counts](tutorial/09-pool-success-counts.html) | `count`, `p_any`, `p_at_least` |
| [10. Many checks at once](tutorial/10-tables.html) | Modifier grid |
| [11. Ordered outcome labels](tutorial/11-ordered-outcomes.html) | Named bands on a roll |
| [12. D&amp;D 5e d20 checks](tutorial/12-dnd5e-d20-check.html) | Nat 1/20, adv/dis, `scale().step` + `bucket` |
| [13. PbtA 2d6 move](tutorial/13-pbta-2d6-move.html) | Miss / partial / full on total |

Start with [lesson 1](tutorial/01-one-die.html): open the literate `.dice` in the playground and click **Run** to read the woven report.

## Cookbook

**Short recipes** for mechanics you see at the table—named after a technique or a well-known game, with pointers to where similar rules appear elsewhere.

See the [cookbook index](../cookbook/index.html).

## Function reference

Generated Markdown for builtins and core types: [references/stdlib.md](references/stdlib.md). Use it while writing scripts in the playground.

Face matching and pool methods: [API conventions](references/api-conventions.md).

## Writing your system with LLM assistance

Tools like [ChatGPT](https://chatgpt.com/), [Claude](https://claude.ai/), or [Cursor](https://cursor.com/) can draft `.dice` scripts from your rules. The site publishes a model-oriented reference at [/llms.txt](/llms.txt)—copy all of it into a **new chat** as the first message so the model knows the syntax and that scripts compute **exact** odds, not simulated rolls.

In your **next message**, describe the mechanic in plain language: die sizes, sum vs count successes, DCs, crits, and what to show under **Output**. Use variables for pool size or modifiers when helpful.

Example second message (d6 pool, successes = number of sixes):

```text
Write a .dice script for this check:

- Roll a pool of d6. Everyone starts with 1 die.
- Add 1 die for each applicable skill and 1 die for each advantage (use variables skill_dice and advantage_dice so I can change them).
- The roll result is not the sum: it is how many dice show a 6.
- Output the full distribution of that success count for skill_dice=0, advantage_dice=0 (one die only) and for skill_dice=2, advantage_dice=1 (four dice).
- Also output P(at least 2 sixes) for the four-dice case.

Use exact probabilities, dice_pool, and count(6) (or equivalent)—no random simulation.
```

Copy the script the model returns into the playground editor, fix any **Diagnostics**, and **Run** to verify the numbers match your intent.

## Contributing

See [AGENT.md](AGENT.md) for how agents and contributors work in this repo.
