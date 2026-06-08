---
title: "Dice language user guide"
author: Steve Cooper
---

# User guide

Documentation for **people who play tabletop games** and want exact odds—not for Rust contributors (see the [repository README](../README.md) for that).

Scripts use the **`.dice`** extension: familiar notation like `2d6` and `4d6dl1`, plus Starlark for modifiers, loops, and outputs.

Use the site header (or the sections below) for **Tutorial**, **Cookbook**, and **Function reference**.

## Playground

In the playground, each lesson and recipe includes a script to copy into the **editor**. Click **Run** (or press **Shift+Enter**) and read results under **Output**—**text**, **json**, or **graph**. Use the **Files** control to manage multiple scripts; **Diagnostics** lists parse and type errors with line numbers.

## Tutorial

A **step-by-step** introduction to the language. Work through the lessons in order.

| Lesson | Topic |
|--------|--------|
| [1. Your first die](tutorial/01-one-die.md) | One fair die |
| [2. Adding two dice](tutorial/02-two-dice.md) | 2d6 |
| [3. Flat bonuses (2d10 + 5)](tutorial/03-modifiers.md) | +5 to every total |
| [4. Meet or beat a target](tutorial/04-success.md) | Success chance |
| [5. Dice notation](tutorial/05-dice-notation.md) | `1d4` … `4d6dl1` |
| [6. Many checks at once](tutorial/06-tables.md) | Modifier grid |
| [7. Ordered outcome labels](tutorial/07-ordered-outcomes.md) | Named bands on a roll |
| [8. D&amp;D 5e d20 checks](tutorial/08-dnd5e-d20-check.md) | Nat 1/20, adv/dis, `classify` |
| [9. PbtA 2d6 move](tutorial/09-pbta-2d6-move.md) | Miss / partial / full on total |

Start with [lesson 1](tutorial/01-one-die.md): copy the script into the playground and read the distribution under **Output**.

## Cookbook

**Short recipes** for mechanics you see at the table—named after a technique or a well-known game, with pointers to where similar rules appear elsewhere.

See the [cookbook index](cookbook/README.md).

## Function reference

Generated Markdown for builtins and core types: [references/stdlib.md](references/stdlib.md). Use it while writing scripts in the playground.

Contributors regenerate after changing engine docs:

```bash
make references
```

Details: [references/README.md](references/README.md).

## Use an LLM to help build your system

If you use ChatGPT, Claude, or similar tools, you can give them a **starter prompt** so they know how Dice Playground scripts work before you describe your homebrew rules.

1. Open the [starter prompt](../llm-starter-prompt.txt). (In the repository source tree, the same text lives in [`llms.txt`](../llms.txt).)
2. Copy the **entire** file into a **new** chat with your LLM and send it once.
3. Describe your mechanic in plain language and ask for a `.dice` script. For example:

   *Roll **3d6**. If **two or more** dice show **1**, that is a **critical failure**. If **two or more** show **6** (boxcars), **critical success**. Otherwise compare the **sum** to **target 10**—success if the total is 10 or higher, otherwise fail. Critical rules take precedence over the target.*

   Paste the script the model gives you into the playground editor and run it.
