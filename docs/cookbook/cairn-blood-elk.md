---
title: "Cairn 2e — Blood Elk horns vs armored PC"
author: Steve Cooper
---

# Cairn 2e — Blood Elk horns vs armored PC

## At the table

[*Cairn*](https://cairnrpg.com/) (2e) combat is swingy and armor matters. Attacks **automatically hit**; the attacker rolls weapon damage, the defender subtracts **Armor**, and whatever is left comes off **HP**.

This recipe models one hit from a **Blood Elk** (horns **d8**) against a typical front-line PC:

| Stat | Value |
|------|-------|
| HP | 4 |
| STR | 11 (for critical damage saves) |
| Armor | 2 (1 worn + 1 shield) |

**Damage steps:**

1. Roll **d8**, subtract **2** Armor (results below 0 count as **0**).
2. Subtract damage from **HP**.
3. If HP lands on **exactly 0**, the PC takes a **Scar** keyed to **how much HP that hit removed** (after armor). Here that only happens on **4** damage—Scar entry **#4 (Broken Limb)**.
4. If damage would push HP **below 0**, the leftover comes off **STR** instead. The PC immediately rolls a **STR save** (d20 ≤ STR after the loss; 1 always succeeds, 20 always fails). Failing means **Critical Damage**—out of the fight and dying without aid. A Blood Elk that inflicts Critical Damage is especially nasty: it **gores** the victim (fiction + Warden fallout).

So on a single elk charge you might see: the horns **glance off armor** (no HP loss), **hit protection** chipped away (HP loss that reads more like guard fatigue than lasting injury), a **Scar** at 0 HP, **STR loss but still fighting** after a passed save, or **Critical Damage** after a failed save.

## Try it

In the playground, copy the script below into the **editor** and click **Run**.

## The script

```text
HP = 4
ARMOR = 2
STR = 11

Scale = scale(["NO_EFFECT", "HIT_PROTECTION_LOSS", "SCAR", "STR_DOWN_OK", "CRITICAL_DAMAGE"])

def damage(horn):
    d = horn - ARMOR
    if d < 0:
        return 0
    return d

def blood_elk_hit(horn, save):
    dmg = damage(horn)
    if dmg == 0:
        return "NO_EFFECT"
    if dmg < HP:
        return "HIT_PROTECTION_LOSS"
    if dmg == HP:
        return "SCAR"
    overflow = dmg - HP
    new_str = STR - overflow
    if save == 1:
        return "STR_DOWN_OK"
    if save == 20:
        return "CRITICAL_DAMAGE"
    if save <= new_str:
        return "STR_DOWN_OK"
    return "CRITICAL_DAMAGE"

out = joint_classify(d(8), d(20), Scale, blood_elk_hit)
output("blood_elk_horns", out)
output("p_protection_loss_or_worse", out.p_at_least("HIT_PROTECTION_LOSS"))
output("p_scar_or_worse", out.p_at_least("SCAR"))
output("p_critical_damage", out.p_at_least("CRITICAL_DAMAGE"))
```

- **`joint_classify`** pairs each horns die result with a **d20 STR save** only when the rules require a save (high damage). Low-damage horns outcomes ignore the save roll in the label function, but the math still convolves correctly.
- **`NO_EFFECT`**: horn **1–2** (0 after armor).
- **`HIT_PROTECTION_LOSS`**: **1–3** HP lost (PC at 3, 2, or 1 HP)—in Cairn, HP is short-term staying power, not deep wounds.
- **`SCAR`**: **4** HP lost → 0 HP exactly → **Broken Limb** on the [Scars table](https://cairnrpg.com/second-edition/players-guide/core-rules/#scars).
- **`STR_DOWN_OK`**: **5–6** damage (**7–8** on the d8)—overflow hits STR, save passed (STR **10** or **9**).
- **`CRITICAL_DAMAGE`**: same big hits, save failed—elk **Critical Damage** (goring) applies.

### Horn die cheat sheet (after 2 Armor)

| d8 | HP damage | Typical result |
|----|-----------|----------------|
| 1–2 | 0 | No effect |
| 3 | 1 | Hit protection loss (3 HP left) |
| 4 | 2 | Hit protection loss (2 HP left) |
| 5 | 3 | Hit protection loss (1 HP left) |
| 6 | 4 | Scar — Broken Limb |
| 7 | 5 | STR 10, then save |
| 8 | 6 | STR 9, then save |

## Cookbook

[All recipes](README.md)
