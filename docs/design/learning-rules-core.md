# Core-course rule dossiers: Parts C–H

Online source review: **2026-09-15**. These are section-specific checks of the
scoped procedures below, not certification of entire games. Explanations are
paraphrased; consult the linked rules for the game. No publisher endorsement is
implied. Independent human rules/novice review remains pending.

## D&D Basic Rules 2014

Sources:
- [Using Ability Scores](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/using-ability-scores): ordinary checks/saves, advantage/disadvantage.
- [Step-by-Step Characters](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/step-by-step-characters): Determine Ability Scores, four d6 keeping three.
- [Combat](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/combat): Attack Rolls, Rolling 1 or 20, Critical Hits.
- [Spells](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/spells): Fireball, base third-level 8d6, Dexterity save for half.
- [Introduction](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/introduction): Round Down.

| Models | Procedure / scope | Parameters and exclusions | Independent oracle |
|---|---|---|---|
| L12 | Select higher/lower of two independent d20 | One ordinary ability check; no attack exceptions, rerolls, stacked advantage | 1−(1−p)² or p² |
| L13 / R03 | Four d6, sum highest three | One score; no array ranking, assignment, or reroll policy | Enumerate 6⁴ tuples; mean 12.2445987654; identical keep/drop PMFs |
| L18 / C11 | Conditional critical damage: add independent damage dice | A single d6 weapon, modifier once; not overall damage | Two-d6 seven has 6/36; doubled d6 cannot make seven |
| L21 / R01 | d20 + supplied bonus ≥ DC | Ordinary ability check, no automatic 1/20 exception | Count satisfying faces 1–20 |
| L22 / R02 | Natural 1 misses; natural 20 critical; otherwise compare modified face with AC | Integer bonus/AC, no expanded critical range, house fumble, or damage model | +5/AC15: miss 9/20, hit 10/20, critical 1/20; test extreme AC and advantage |
| L20 / R04 | Sum 8d6, floor each total divided by two | Conditional on save success/failure; no resistance, vulnerability, Evasion | Means 28 and 13.75; odd-total mass 1/2 |
| L32 / R05 | Pair raw damage with independent save; halve on success; query damage ≥ HP | One target, supplied HP/DC/bonus; no multi-target correlation or death interpretation | At HP28, save +3/DC15: (11/20) × independent integer count of 8d6 ≥28 |

The model never uses attack natural-1/20 exceptions for ordinary saves. Models
choose all stated inputs before rolling; none models a reaction after seeing
an attack or damage unless specifically described as a policy.

## Dungeon World SRD

Sources: [Playing the Game](https://www.dungeonworldsrd.com/playing-the-game/),
Making Moves (roll+modifier means sum two d6), and
[Moves](https://www.dungeonworldsrd.com/moves/), Defy Danger.
The online SRD is the reviewed source; no uninspected printing/page is asserted.

L09, L10, and DW01 model the numeric bands of Defy Danger: 6− gives the GM an
outcome, 7–9 prompts a worse outcome, hard bargain, or ugly choice, and 10+
avoids the threat. The relevant stat modifier is supplied. The fiction, move
trigger, specific GM response, and player choice are outside the model.
The middle band is not described as universal unqualified success.

Oracle at modifier zero: 15/36, 15/36, 6/36. At +2: 6/36, 15/36, 15/36.
The bands cover each total once; seven-or-more is a different query from exactly
the middle band. No after-roll choices are inferred from the result ladder.

## Blades in the Dark

Sources: [The Core System](https://bladesinthedark.com/core-system) and
[Action Roll](https://bladesinthedark.com/action-roll), online SRD.

L11 is only a highest-face component. L23 and R18 read full base action-result
categories: highest 1–3/4–5/6, with at least two sixes critical. Zero dice reads
the lower of two and cannot critical. Negative assembled pools use that same
zero-dice procedure, but the scripts require callers to supply zero rather
than silently accepting negative input. Position and effect are separate;
critical action results increase effect.

The callback supports integers 0–5 for responsiveness; the cookbook's equivalent
count decomposition supports 0–20. No stress, resistance, pool assembly, or
fictional consequences are modelled. Oracle: disjoint enumeration through five
dice, then a separate closed form tested through twenty. Two dice produce
9/36,16/36,10/36,1/36; zero produces 27/36,8/36,1/36,0.

## Cairn second edition

Sources: [Player's Guide Core Rules](https://cairnrpg.com/second-edition/players-guide/core-rules/),
Armor, Attack Modifiers, Critical Damage, Attribute Loss, Scars; and
[Warden's Guide Blood Elk](https://cairnrpg.com/second-edition/wardens-guide/bestiary/#blood-elk).

| Models | Scope and timing | Oracle |
|---|---|---|
| L16 / R22 | Two weapons roll together, keep higher before armour; d6+d8 example | Maximum ≥6 is 1−(5/6)(5/8)=23/48 |
| L19 / R21 | Subtract integer armour 0–3 from d8, floor at zero | Armour2: zero 1/4, mean 21/8 |
| L25 / R23 | One normal horns d8 hit; armour, HP, STR overflow, then reduced-STR save if alive | Default categories 1/4,3/8,1/8,19/160,21/160,0 |

Blood Elk models flag a scar but do not resolve its random table, recovery,
goring aftermath, or subsequent attacks. Positive integer HP and STR are
required at the start. Reduced STR ≤0 means death before the save; a natural 1
cannot rescue it. Otherwise save 1 always succeeds and 20 fails. Potential saves
on lower-damage branches are ignored, not used to manufacture another decision.

## Fate Core

Sources: [Taking Action, Dice, and the Ladder](https://fate-srd.com/fate-core/taking-action-dice-ladder),
[Four Outcomes](https://fate-srd.com/fate-core/four-outcomes),
[Four Actions: Overcome](https://fate-srd.com/fate-core/four-actions), and
[Invoking Aspects](https://fate-srd.com/fate-core/invoking-compelling-aspects).
The online Fate Core SRD, not an unspecified Fudge/Fate variant, is the scope.

- **L17 / R12:** four independent equiprobable −1/0/+1 signs; no skill or action.
  Range exactly −4…+4, extremes 1/81 each, zero 19/81, mean zero.
- **L24 / R13:** two independent equal-skill rolls, active opposition to an
  overcome action. Tie reaches the goal at minor cost; positive margin succeeds,
  margin ≥3 succeeds with style. Below opposition does not decide whether to
  accept serious cost. Tie oracle 1107/6561; strict win/loss each 2727/6561.
- **L30 / R14:** one paid invocation of an agreed relevant aspect, one available
  fate point, fixed passive opposition. Observe the initial roll; preserve an
  existing strict success, take +2 if it guarantees the objective, otherwise
  reroll all dice and accept the replacement. Do not examine the replacement
  before deciding. No free invokes, further invokes, or defender resources.
  At equal skill/opposition +2, objective is strict success without cost:
  66/81 + (15/81)(31/81) = 5811/6561. Point-spend chance is 50/81.

This is a stated objective and fixed legal policy, not a general optimal strategy
across resources, future scenes, or alternative costs.

## Brindlewood Bay — supplied-text review

Identity: Jason Cordova, *Brindlewood Bay: A Dark & Cozy Mystery Game*, supplied
by the user as `target/brindlewood-bay.md`, reviewed 2026-09-15. The front matter
names the title and author but no edition/printing. These models are tied to
this text, not an inferred edition or all versions of the game.

Source SHA-256:
`74897619af2c0009b3ff5ccfc149cf7534a9ab5764f723eb9d21a818c73980c6`.
The full text stays in ignored `target/`; no build or test needs to publish it.
The [publisher’s game page](https://www.gauntlet-rpg.com/brindlewood-bay.html)
establishes where to find the game, not proof of the specific rules reviewed.

Sections checked in the supplied file:
- **Rolling Dice**, **The Crowns**, and **Theorize** (lines 92–118 and 202–216).
- **Complexity**, **Clues**, and **Void Clues** (lines 646–684).
- Keeper guidance for **Theorize**, including accounted Clues and repeats
  after a Keeper reaction (lines 914–932).

| Field | Checked procedure / scope |
|---|---|
| Models | L31 and R20; initial standard-mystery roll before Crown use |
| Eligibility | Gather at least ceil(Complexity/2) ordinary Clues and reach consensus |
| Roll | Two d6 + Clues incorporated or explained away − Complexity |
| Gathered versus accounted | The trigger uses gathered Clues; not every gathered Clue automatically contributes to the roll |
| Bands | 6− incorrect; 7–9 correct with complication or dangerous/complicated opportunity; 10+ correct with an opportunity to resolve; 12+ additionally reveals a conspiracy figure |
| Inputs | Standard Complexity 6–8; gathered at least the minimum; accounted integer 0–gathered. Consensus and clue qualification are assumed, not automated |
| Exceptions | No advantage, disadvantage, or effects of other moves. In this text Crowns can raise the tier only if every Maven puts one on |
| Timing / exclusions | Initial roll only; no Crown decision, resource availability, repeat-attempt policy, special clue conversion, or Void Mystery variant |
| Oracle | Enumerate the 36 independent d6 pairs at each modifier. At equal accounted Clues and Complexity, exclusive band counts are 15/15/5/1 and nested 7+/10+/12+ counts are 21/6/1 |
| Boundaries | C6 needs 3 gathered; C7 needs 4. C7/gathered5/accounted4 has modifier −3, not −2. Increasing gathered without increasing accounted leaves odds unchanged once eligible |
| Review | Source checked against supplied text; mathematical tests passed; edition/printing unidentified; independent human rules/novice reviews remain pending |

The source differs materially from unverified older descriptions claiming
that Theorize cannot be affected by Crowns. We follow the supplied text while
explicitly excluding Crown use, rather than modelling it as a flat die bonus.
Tests in `tests/brindlewood_theorize.rs` do not depend on the local book file.

## Explicit generic / source-pending components

| Pages | What is actually specified | Gate / exclusions |
|---|---|---|
| L14 / R17 | Any natural 1 in supplied independent d6 pool; 1−(5/6)^n | The Pool author PDF unavailable at the author's linked URL; source review pending. No gambling, awards, or narration model |
| L15 / C01 | Count 5–6 on three d6; PMF 8/27,12/27,6/27,1/27 | Generic, not certified Shadowrun; no Edge, limits, or glitches |
| L26 / C03 | Replace first 1 once; keep second even if 1 | Generic house rule; P(1)=1/36, mean47/12; no inferred named reroll ability |
| L27 / C02 | Conditioning, repeated-reroll final distribution, and scoring zero are distinct | Generic; no stopping-time or resource-cost model |
| L28 / C09 | d6 maximum explosion, explicit maximum extra rolls | Generic bounded component, not full SWADE. Event disagreement ≤(1/6)^(cap+1); not a mean-error bound |
| L29 / R26 | Existing helper: initial 1–5 subtract, 96–100 add, later 96–100 continues same direction | RMSS printing source pending; no alleged typo; max_chain counts continuations after first extra roll |

The author of The Pool links its PDF from
[The Pool — Videos and Thoughts](https://doomslakers.blogspot.com/2017/04/the-pool-videos-and-thoughts.html).
The linked `jwarts.com/thepoolrpg.pdf` did not return a PDF during review.
Unofficial uploaded books and community summaries were not used to certify
RMSS. The Pool and RMSS gates still require legitimate source excerpts;
Brindlewood now has the supplied-text review recorded above.
