---
title: "Dice language user guide"
author: Steve Cooper
---

# User guide

Documentation for **people who play tabletop games** and want exact odds—not for Rust contributors (see the [repository README](../README.md) for that).

Scripts use the **`.dice`** extension: familiar notation like `2d6` and `4d6dl1`, plus Starlark for modifiers, loops, and outputs.

Use the site header (or the sections below) for **Tutorial**, **Cookbook**, and **Function reference**.

## Playground

On a lesson or recipe, choose **Open this document in the playground** to load its complete source, including explanation and setup. The static page downloads the same-origin `.dice` source and hands it to the editor through browser storage, avoiding URL-length limits. If storage is unavailable, use **Download source** instead. Select **Run** (Shift+Enter) to read the woven report, with executable code and results together. **Files** manages scripts; **Diagnostics** lists errors.

## Tutorial

Start with [lesson 1](tutorial/01-one-die.html). The first four lessons form a
continuous pilot: read a die distribution, add dice, name a modifier, then build
and independently check a success report. Each includes predictions, guided
edits, and a transfer exercise with an answer check.

The [tutorial index](../tutorial/index.html) is generated from the content
manifest and shows the current order, objectives, and review status. Later
lessons retain the older sequence while the planned eight-part course is
rewritten. Existing URLs retain their identities; future lessons must not reuse
an old URL for a different topic.

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
