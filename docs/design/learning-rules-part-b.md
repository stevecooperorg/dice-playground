# Part B: rules dossiers and review evidence

These dossiers separate **source fidelity** from **mathematical checks**.
Reviewed on 2026-09-15. Both models remain pilots: independent human rules
review, novice use, and real-browser review are pending. Neither is a full
game implementation or endorsed by its publisher.

## D&D 2014 ordinary ability checks

| Field | Record |
|---|---|
| Identity | Dungeons & Dragons, Basic Rules 2014, chapter 7: Using Ability Scores, online text |
| Source | https://www.dndbeyond.com/sources/dnd/basic-rules-2014/using-ability-scores |
| Sections checked | Ability Checks; Typical Difficulty Classes; Proficiency Bonus; Skills; Strength Checks |
| Used by | L06, L07, L08, R01 (`dnd2014-ability-check.dice`) |
| Procedure | When a check is called for, roll a d20, add the applicable bonus, succeed if total equals or exceeds DC |
| Scope | One ordinary, unopposed ability check; no advantage/disadvantage, rerolls, spells, special features, passive/group checks, or contests |
| Inputs | Total integer bonus and integer DC. The bonus is supplied, not calculated from character creation; numerical boundary DCs are supported for validation |
| Information/timing | Choose the bonus/DC before the roll; no after-roll decision policy |
| Exceptions | Equality succeeds. Do not import natural-1 automatic failure or natural-20 automatic success from attack rules |
| Oracle | Count natural faces 1–20 with face + bonus ≥ DC. Closed form: clamp(21 + bonus − DC, 0, 20) / 20 |
| Tests | `tests/learning_part_b.rs`: defaults, DC edits, bonuses −5/0/+5/+30, DCs 0/6/15/25/26/40, invalid non-integer inputs |
| Review state | Primary source checked; numerical checks automated; human/browser review pending |

At bonus +5 and DC 15, faces 10–20 give 11/20. DC 6 gives 20/20,
DC 25 gives 1/20, and DC 26 gives 0/20. These are **ability-check** bounds,
not attack-roll bounds. The higher mean of a bonus-shifted d20 is not itself
a success probability. Reusing the same distribution to ask several threshold
questions does not make those questions independent.

The climbing situation illustrates an Athletics check; the source does not
mandate that every cliff has DC 15. The example's DC is an explicit DM input.
Do not infer eligibility or the handling of repeated failure from these odds.

## Call of Cthulhu 7e regular difficulty

| Field | Record |
|---|---|
| Identity | Call of Cthulhu 7th Edition Quick-Start Rules, current PDF linked by Chaosium's quickstart page |
| Publisher route | https://www.chaosium.com/cthulhu-quickstart/ |
| PDF | https://www.chaosium.com/content/FreePDFs/CoC/CHA23131%20Call%20of%20Cthulhu%207th%20Edition%20Quick-Start%20Rules.pdf |
| Sections checked | Reading D100 (Percentile Dice); Skill Rolls and Difficulty Levels, printed p. 10 |
| Used by | L05, L08 checkpoint, R09-regular (`coc7-regular-check.dice`) |
| Procedure | Interpret percentile dice as 1–100; meet regular difficulty when roll ≤ skill |
| Scope | Unopposed, single regular-difficulty check; combines all successes meeting that difficulty, not exactly the Regular success level |
| Inputs | Integer skill 1–99 for the component. Recipe accepts 1–89 so the +10 comparison also stays below 100 |
| Information/timing | Skill fixed before the roll; no pushing or Luck-spending policy |
| Exceptions/exclusions | Double zero encodes 100, not 0. No bonus/penalty dice, opposed rolls, individual Hard/Extreme/critical/fumble labels, or skills 100+ |
| Oracle | At skill s within scope, exactly s of 100 equally likely results meet the threshold |
| Tests | `tests/learning_part_b.rs`: skills 40/50/65, recipe bounds 1/89, rejected 0/90/100 and non-integers, 40–44 checkpoint table |
| Review state | Primary PDF read; numerical checks automated; human/browser review pending |

PDF SHA-256 at review:
`39b6a53915ddf9f48831ae45ac7180b6c7d5fc7bf49e1b7fe764c28ed1885436`.
The URL may change its contents; this fingerprint records which download was
read without distributing the publisher's PDF in the repository. The PDF was
larger than the web extraction limit, so it was downloaded from Chaosium and
read using local text extraction. No unofficial rules summary was substituted.

This component is deliberately narrower than candidate R09 (all success levels)
in the design plan. It does not certify fumble boundaries or bonus/penalty dice.
When those are added, check their edition-specific exceptions separately.

## Publication and exercise checks

- New lesson URLs do not replace the old 05–08 slugs. Stable IDs and manifest
  order distinguish the rewritten path from legacy pages.
- All six new pages use one executable fence and run from a fresh evaluation.
- Output names, row labels, row counts, and expected probabilities are asserted.
- Guided changes evaluate edited copies of the complete published source.
  Transfer answers have independent scripts/face counts rather than a check
  that their probabilities merely lie between zero and one.
- A script can pass these tests without teaching effectively. Human pilot and
  actual browser/WASM acceptance gates remain open.
