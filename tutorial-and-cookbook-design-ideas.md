# Tutorial and cookbook design ideas

## Status and recommendation

**Planning proposal, not an approved syllabus or a rewrite of the existing content.** Reviewed against repository commit `0140b2e`, including the recently revised API reference.

The recommendation is a **complete editorial rewrite**, retaining useful calculations and test cases rather than preserving the current lesson structure. Build two complementary resources:

- A **guided tutorial** of approximately **32 short lessons**, grouped into eight parts. Each lesson teaches one central modelling idea, introduces only the scripting needed for it, and uses a concrete tabletop situation.
- A **cookbook** growing from 10 entries towards **roughly 40 independently usable recipes**: named game mechanics, reusable building blocks, and a small optional set of non-TTRPG examples. This is a prioritised destination, not a requirement to publish 40 recipes at once.

The tutorial answers **“How do I learn to build a script?”** The cookbook answers **“How do I model this rule or reuse this technique?”** Neither should be an API catalogue. The generated reference now serves that purpose.

The rewrite needs three workstreams, not just new prose:

1. **Learning design:** a deliberate progression through probability, modelling, and scripting.
2. **Rules verification:** exact game/edition/scope, independently checked calculations, and honest limits.
3. **Publication and validation:** one source for code and explanation, reliable whole-document opening, correct output placement, navigation, and tests of what readers actually see.

## 1. Goals and boundaries

### Who we are teaching

Assume the reader understands a tabletop situation but is new to programming, Starlark, and formal probability. They may know D&D and nothing about Blades, or Fate and nothing about percentile systems. A game name is not an explanation: every example must explain the small part of its rules needed for that page.

Do not assume familiarity with variables, method calls, lists, indexing, loops, functions, indentation, Boolean conditions, or callbacks. Do not teach those as an unrelated programming course either: introduce each when it solves a visible tabletop problem.

### What a reader should be able to do afterwards

A successful reader can:

1. Turn a rules paragraph into a clear question: “chance to hit”, “damage after a save”, “chance of two successes”, or “risk if I spend this resource”.
2. Identify which dice are independent and which decisions must inspect the same rolled values.
3. Choose whether to preserve individual faces, add totals, count matches, or produce named outcomes.
4. Write and change their own parameters, then build a small script rather than merely modifying a supplied die size.
5. Express conditional rules with functions and comparisons, including a later roll whose interpretation depends on an earlier result.
6. Distinguish changing a roll from asking a question about it: conditioning, rerolling, ignoring, and querying are not interchangeable.
7. Produce a readable report with useful labels, appropriate comparisons, and an explanation of the result.
8. Check a model using simple cases and independent reasoning rather than trusting a plausible-looking graph.
9. State the model’s assumptions, edition, scope, caps, and exclusions.
10. Recognise when a problem needs a different representation or engine support, rather than silently treating dependent events as independent.

### Non-goals

- A complete rules implementation for every named game.
- A tour of every builtin in API order.
- A stochastic combat simulator, campaign simulator, or universally optimal strategy solver.
- A promise that unlimited explosions are calculated as infinite distributions.
- Teaching every Starlark feature before the reader gets useful results.
- Replacing a purchased rulebook with reproduced rules text.

TTRPGs should remain the clear centre of gravity. Yahtzee and cards can be useful **optional transfer exercises**, not a diversion in the beginner path.

## 2. The two main axes, and a necessary third

The two concerns in the brief must be tracked separately. A long list of games does not create a learning progression, and an elegant progression of abstract operators does not establish usefulness for real games.

| Axis | Question | Planning artefact |
|---|---|---|
| Conceptual learning | What new idea can the reader understand and apply after this page? | Ordered lessons, prerequisites, checkpoints, transfer exercises. |
| Game coverage and fidelity | Which published mechanic is represented, from which edition, under what assumptions? | Rule dossiers, source citations, boundary cases, recipe catalogue. |
| Scripting competence | What can the reader now express without copying a complete answer? | A small syntax progression attached to the lessons. |

Use recurring games to reduce cognitive load. For example, revisit a D&D check when teaching parameters, advantage, natural-face exceptions, and a complete report. Revisit Cairn for armour and a damage-dependent save. Revisit Fate for custom dice, opposition, and a resource decision.

**One main idea per lesson does not mean one API per lesson.** A lesson may combine familiar tools, but should not introduce a new rules system, a new probability abstraction, and several new language constructs at the same time.

A useful lesson card should record:

- Central question and one learning objective.
- Prior lesson concepts required.
- Game, edition, precise mechanic, and whether this is a complete mechanic or a scoped component.
- Any new syntax, with the first place it is explained.
- A prediction to make before running.
- A small executable example and a numerical check.
- One guided change and one transfer task.
- The cookbook recipe that expands the example.

## 3. What exists now

### Scope of this review

Read all **13 tutorial** and **10 cookbook** `.dice` documents, the four corpus/mechanic test files, the learning and probability design documents, and the relevant rendering, handoff, and enumeration code.

Ran:

```text
cargo test --test tutorial_samples --test cookbook_samples \
  --test docs_tutorial_literate --test docs_cookbook_literate
```

All **19 tests** in those files passed. This establishes the tested execution baseline, **not game-rule accuracy or teaching quality**.

Additional small CLI probes confirmed that:

- Starlark list `.append(...)` works in this environment.
- A function named `d20test` now parses and runs.
- `4d6kh2` and `4d6dl2` have the same PMF; likewise `4d6kh3` and `4d6dl1`.
- The displayed introductory snippet in lesson 03 fails because it sends an unsummed pool to `output`; the executable version uses `sum(roll)` and succeeds.

Primary-source spot checks covered D&D 2014 ability/attack rules, Blades core/action rolls, Cairn 2e core rules and the Blood Elk, and Fate Core dice/opposition. **This was not a complete rules audit of every game.** In particular, Brindlewood and Rolemaster need edition-specific rulebook/reference-sheet verification before their descriptions are certified. Source links and their status appear at the end.

### What is worth keeping

- The general progression from one die to combinations and named outcomes.
- The literate `.dice` format: explanation and executable model in one file.
- “At the table”, “Reading the result”, and “Try this” as useful teaching intentions.
- Small, inspectable calculations: 2d6, a flat bonus, 4d6 drop lowest, a match count, and 4dF.
- The distinction between a pool and its total, and between filtering and zeroing faces.
- The selection of mechanics beyond D&D: The Pool, Blades, Brindlewood, Cairn, and Rolemaster.
- The Cairn example’s attempt to connect several rule steps to an actual fictional event.
- Whole-corpus evaluation and existing numerical regression tests.

These are **ingredients**, not a reason to preserve the current prose or numbering.

### Main weaknesses

1. **The first half is mostly anonymous mechanics.** Recognisable game situations arrive late; a reader is asked to learn notation before seeing why the tool matters to their game.
2. **Programming knowledge is assumed rather than built.** Variables appear without much explanation; lesson 10 jumps to tuples, string formatting, ranges, a comprehension, and nested iteration. Functions and conditionals do not get a gradual introduction before complex recipes use them.
3. **The progression follows API categories too closely.** Dice notation gets an eight-stage lesson, while constructing and checking a custom rule gets no comparable scaffold.
4. **Examples often stop at “change a number”.** That is a useful first exercise, but not enough to teach construction, debugging, adaptation, or a report that answers a decision question.
5. **Reading the result is shallow or outdated.** References to text/JSON/graph tabs do not consistently describe the current woven-report experience. A mean is often quoted without explaining what it cannot tell you.
6. **There are substantive factual errors.** Some are mathematical, some concern Starlark, and some overgeneralise game rules.
7. **“This script runs” is too often the effective correctness standard.** Named-game recipes generally lack rule citations and independent numerical oracles.
8. **Difficulty varies wildly across cookbook entries.** A one-line die transformation and an unexplained multi-function classifier are offered with little indication of prerequisites.
9. **Report design is inconsistent.** A tutorial argues for one comparison table, while The Pool emits ten separate probability blocks. Some labels name a different event from the event being calculated.
10. **Code duplication is structural, not merely editorial carelessness.** The current renderer hides executable code, encouraging a display-only copy elsewhere. Removing the copies without changing rendering would make the lesson’s code disappear from the website.

## 4. Tutorial audit: retain the ground, replace the sequence

“Retain” below means preserve a calculation or teaching purpose after verification, not keep the current page intact. Proposed lesson IDs refer to section 7.

| Existing page | Reusable material | Rewrite action |
|---|---|---|
| [01 — One die](docs/tutorial/01-one-die.dice) | Fair d6, visible report, one simple edit. | Keep the opening scale. Explain `output`, quotes, parentheses, and what a distribution represents. Interpret one row before introducing the mean. Use a scoped damage-die example. → L01. |
| [02 — Two dice](docs/tutorial/02-two-dice.dice) | 2d6 distribution and mean 7. | Show why 7 has six combinations and 2 only one; compare before running. Call the 2d6 shape triangular, not a normal/bell distribution. → L02. |
| [03 — Modifiers](docs/tutorial/03-modifiers.dice) | Shift by a constant; before/after comparison. | Introduce names and parameters deliberately; use one tested code source. Current displayed code is invalid while the executable copy works. → L03, L06. |
| [04 — Target](docs/tutorial/04-success.dice) | Inclusive threshold and one probability. | Attach it to a specific task; explain 0–1 versus percent and the difference between “exactly” and “at least”. Add a hand-counted check. → L04. |
| [05 — Notation](docs/tutorial/05-dice-notation.dice) | Dice sizes, totals, keep/drop examples and tested means. | Dismantle the eight-stage survey. Teach notation just in time; move the cheat sheet to reference/support material. Correct the false keep/drop claims. → L01–03, L12–13. |
| [06 — Pools](docs/tutorial/06-dice-pools.dice) | Keeping faces available versus summing; highest die. | Give the reader a reason to preserve faces before introducing the type name. Avoid saying notation always sums. → L11–12. |
| [07 — Mixed pools](docs/tutorial/07-mixed-dice-pools.dice) | Joining unlike dice and comparing highest versus sum. | Replace an anonymous d12-plus-d6 example with a verified Cairn dual-weapon component, then offer an invented variant. Explain that `+` on pools joins them. → L16. |
| [08 — Face filters](docs/tutorial/08-restrict-faces.dice) | Valuable keep/remove/ignore distinction; same mean, different distributions. | Split transformations from conditioning. Teach nonstandard face matching only after basic probabilities. Use explicit questions instead of four new verbs at once. → L19, L27. |
| [09 — Counts](docs/tutorial/09-pool-success-counts.dice) | Any natural 1 in The Pool; a success-count PMF. | Split any/none from how-many and enough-successes. Introduce a real hits-counting component with explicit exclusions. → L14–15. |
| [10 — Tables](docs/tutorial/10-tables.dice) | Comparing parameter values and avoiding hundreds of output blocks. | Replace 143 mostly unilluminating rows with a small useful comparison. Teach a list of rows before a loop; a comprehension is optional later shorthand. Correct the immutable-list claim. → L07–08, L31. |
| [11 — Outcomes](docs/tutorial/11-ordered-outcomes.dice) | Scales, labels, and querying ranks. | Use an actual move rather than arbitrary d20 critical bands. Separate making the categories from querying them. Repair stale links, including the absent `games-systems/` example. → L09–10. |
| [12 — D&D checks](docs/tutorial/12-dnd5e-d20-check.dice) | Preserving the natural die; advantage; parameterised boundaries. | Split ordinary ability checks, advantage, and attack-specific exceptions. Use “automatic miss”, not an implied fumble subsystem. Introduce functions earlier. Remove the obsolete function-name warning. → L04/L06, L12, L21–22. |
| [13 — PbtA move](docs/tutorial/13-pbta-2d6-move.dice) | Three bands and useful checks at stat 0 and +2. | Move much earlier. Choose a named game and move; “PbtA” is not one universal ruleset or one interpretation of 7–9. Correct lesson references. → L09–10. |

## 5. Cookbook audit

| Existing recipe | What can survive | Accuracy, teaching, and scope work |
|---|---|---|
| [Ability scores](docs/cookbook/ability-scores-4d6dl1.dice) | `4d6dl1`, support 3–18, mean about 12.2446. | Name the D&D edition and random-generation option. Distinguish one ability score from six scores, array quality, reroll policies, and point buy. Compare against 3d6. |
| [Blades action roll](docs/cookbook/blades-in-the-dark.dice) | Highest die, two-sixes critical, zero-dice lower-of-two branch; callback as a possible implementation. | Zero dice is **not desperate position**. Action criticals have increased effect; position changes consequences. Check commentary about the most common band rather than generalising. Explain the callback or use a clearer probability decomposition, checked against it. |
| [Brindlewood Theorize](docs/cookbook/brindlewood-bay-theorize.dice) | Clues-minus-complexity parameterisation, fourth outcome band, comparison table. | Verify edition, counted clues, eligibility, and 12+ effect from an authorised reference. `p_clean_by_clues` currently calculates 7+, including complications, not clean 10+. The fixed sweep starts at 3 even if Complexity changes. |
| [Cairn Blood Elk](docs/cookbook/cairn-blood-elk.dice) | A strong capstone candidate: armour, HP, STR overflow, and a conditional save in one small encounter. | Default case aligns with the checked core rules and monster entry. Explain the independent potential save versus whether a save is actually needed. The function is not a general combat model: STR reaching 0 must mean death before any saving roll; it currently permits a natural-1 save to avoid that when parameters change. State the supported slice and omitted scar/recovery consequences. |
| [Count high faces](docs/cookbook/count-high-faces.dice) | A clear building block and small count distribution. | Keep as a component recipe, not a disguised full game. Compare count, any, and at-least-two with a hand-counted check. Link to a real hits-based example. |
| [Exploding dice](docs/cookbook/exploding-dice.dice) | One capped exploding d4. | Label it a bounded model and quantify the cap’s significance. It is not a complete Savage Worlds roll: Wild Die, modifiers, critical failure, raises, and resource rules are separate concerns. Do not casually suggest large generic enumeration for multiple exploding dice. |
| [Fireball half damage](docs/cookbook/fireball-half-damage.dice) | 8d6 and flooring each result after a successful save. | This is damage **conditional on the save result**, not overall expected damage. Specify base spell level, edition, one target, and no resistance/Evasion. Add a linked save-plus-damage recipe rather than silently broadening this one. |
| [Fudge/4dF](docs/cookbook/fudge-4df.dice) | Custom faces, duplicate weights, adding four independent dice, extremes. | Use a primary/SRD source rather than Wikipedia. Pick Fate Core when extending to skill, opposition, and outcome rules; avoid treating all Fudge/Fate variants as identical. The possible range is exactly −4 to +4. |
| [Rolemaster](docs/cookbook/rolemaster-open-ended.dice) | Distinction between high and low opening; the +199 and −96 worked paths; capped full open-ended distribution. | Pin the RMSS printing and rulebook section. Do not claim an alleged rulebook typo without a primary citation or erratum. Distinguish the full open-ended procedure from a particular attack or manoeuvre table. Explain the cap precisely. |
| [The Pool](docs/cookbook/the-pool.dice) | Any natural 1 and comparing pool sizes. | Verify against the author’s rules; state that the supplied pool is already assembled. Gambling, awards, narration choices, and future resources are not modelled. Present a single comparison table and diminishing returns. |

### Corrections with especially strong evidence

These should become acceptance tests or editorial checks for the rewrite:

- **Lesson 05:** 12.2446 is **lower** than the 4d6 mean of 14, but higher than the 3d6 mean of 10.5. Dropping a die cannot increase the total of those same positive dice.
- **Lesson 05:** `4d6kh2 == 4d6dl2` and `4d6kh3 == 4d6dl1` as complete distributions. They do not keep different effective sets of faces.
- **Lesson 10:** list mutation is supported; “lists are immutable” is false here. Two probability-table rows may overlap and be strongly dependent. Say they are **separate questions**, not “independent probabilities”.
- **Lesson 12:** D&D 2014’s natural-1 automatic miss and natural-20 critical-hit rules are attack rules, not general rules for every ability check and saving throw. Death saves are their own special case. [S1–S2]
- **Blades:** zero-dice resolution and desperate position are distinct rules. On one die, bad outcome is more likely than partial; at four dice, full success or critical combined is over 50%. Narrative claims must agree with the displayed comparisons. [S3–S4]
- **Brindlewood:** the existing table label disagrees with its calculation even before any external rules audit.
- **Cairn:** “STR reduced to 0 means death” is a material boundary for an editable model. Defaults that never reach the boundary do not validate the general function. [S5–S6]

## 6. Editorial rules for the new course

### The lesson contract

Use a consistent structure without turning every page into mechanical boilerplate:

1. **The situation:** a few sentences of play, with the exact rule fragment needed.
2. **The question:** one thing the report will answer.
3. **Predict:** a plausible choice or estimate before running.
4. **Build:** a small script, with each new syntax feature explained.
5. **Run and read:** one or two specific results, and why they make sense.
6. **Change one thing:** a guided edit with an expected direction or range.
7. **Try it yourself:** a small transfer task, with an accessible explanation/check of the answer.
8. **What you now know:** the reusable idea, not a list of function names.
9. **Where next:** previous/next, prerequisites, and a deeper recipe.
10. **Rules and model notes:** edition, source, exclusions, and caps where relevant.

For most lessons, aim initially for **5–10 minutes** and one to three meaningful outputs. This is a design target to test with readers, not a fixed word quota. A calculation-heavy advanced lesson may need longer.

### Teach scripting in service of the question

- Introduce quotes, calls, and `output` immediately; variables with the first reusable modifier.
- Use `d(6)` and explicit `dice_pool(...).sum()` when they make a value’s role clear. Explain shorthand in context, not through hidden expansion rules.
- Explain whole numbers, decimals, and 0–1 probability before arithmetic on probabilities.
- Introduce lists and labelled rows before loops. Explain `range`’s excluded endpoint. Teach one loop before two.
- Use ordinary loops before presenting comprehensions as a compact alternative.
- Introduce `def`, parameters, and `return` in a simple reusable calculation **before** custom classification.
- Explain `if`, comparison operators, indentation, list indexing from zero, and passing a function by name before pool callbacks depend on them.
- Use readable parameter names and output titles. Do not require newcomers to decode `p_clean_by_clues` or `STR_DOWN_OK` to understand the report.
- Functions return values; `output` presents results. Keep those roles distinct.

### Teach validation from the beginning

Examples of questions a learner should get used to asking:

- Are the minimum and maximum plausible?
- Can the mean be a number that is never rolled?
- Did I count the target itself?
- Have I accidentally made a conditional chance look unconditional?
- Do these categories cover every result once?
- Is a zero-probability label genuinely impossible, or did I omit a branch?
- Does reusing a variable mean another independent roll or the same face?
- What happens at zero, at the maximum, and just across a rule boundary?

The reader should progress from copying, to modifying, to explaining, to constructing, to checking.

## 7. Proposed tutorial learning ladder

This is a concrete starting syllabus for review, not final titles or file slugs. The default prerequisite is the preceding lesson; selected stronger dependencies are noted. Each part ends in a small checkpoint using already introduced tools.

**Game references below are proposed teaching anchors, not claims that all those new models have already been verified.** Named editions and sources must be pinned in the rule dossiers. Generic or scoped components should be visibly labelled as such.

### Part A — From a die to an answer

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L01 | A roll value represents all its possible results. | One unmodified d6 damage die from a D&D weapon, explicitly before attack/critical rules. | `d(6)`, `output`, strings, calls, comments. | Explain one row and predict what changing to a d8 does. |
| L02 | Adding dice gives unequal total probabilities. | The 2d6 subtotal of a Traveller task roll. | Addition and contextual `2d6` shorthand; explicit total when stored. | Explain 6/36 for a seven and 1/36 for a two. |
| L03 | A flat modifier moves results, not probabilities. | Traveller task with a fixed skill/characteristic modifier. | Variable assignment and a named modifier. | Predict the new range and mean after +2. |
| L04 | Ask an inclusive success question instead of displaying everything. | A specified ordinary Traveller task threshold, verified against the chosen edition. | `.p_ge`, `.pmf`, 0–1 probabilities. | Count successful totals and distinguish exactly eight from eight or more. |

**Checkpoint:** build a small task report from a stated die rule, modifier, and target without copying a finished script.

### Part B — Comparing choices and building useful reports

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L05 | Some games reward low rolls. | Call of Cthulhu 7e regular-success chance at a modest skill value; not a full fumble/degree model. | `.cdf`; inclusive roll-under. | Explain what “skill 50” means on d100 under the stated scope. |
| L06 | Averages and success chances answer different questions. | D&D 2014 ability-check bonuses against a fixed DC; no attack criticals. | Several named calculations and outputs. | Compare two choices using the requested criterion, not only the mean. |
| L07 | One table can answer several related questions. | A few DCs for the same D&D ability check. | Lists, labelled pairs, `prob_table`. | Explain why table rows need not add to 100%. |
| L08 | A loop repeats a calculation, not simulated dice rolls. | How a fixed check changes across a short range of bonuses. | `for`, `range`, indentation, appending rows, simple labels. | Produce five useful rows and explain the range endpoint. |

**Checkpoint:** write a compact comparison and a prose recommendation limited to what the numbers actually show.

### Part C — What information does the rule need?

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L09 | Group totals into named outcomes. | One specified Dungeon World move, such as Defy Danger, with its actual 7–9 meaning. | `scale`, `step`, numeric bands, `bucket`. | Show the three outcome probabilities and explain their boundaries. |
| L10 | Ask “this result” versus “this result or better”. | The same move; no new game to learn. | Label queries and explicit ladder ordering. | Distinguish exactly 7–9 from 7+ and verify the categories sum to one. |
| L11 | A total loses the individual faces. | Compare adding several dice with selecting one die; use a clearly marked Blades highest-die component, not the complete action roll. | `DicePool` versus `DieRoll`, explicit `.sum()`, basic `.order_stat(1)`. | Explain why a total cannot tell you whether there were two sixes. |
| L12 | Advantage changes a distribution rather than adding a fixed bonus. | D&D 2014 ability check with advantage/disadvantage, before attack exceptions. | `keep_highest` / `keep_lowest`, two-die comparison. | Compare normal, advantage, and disadvantage at several targets. |

**Checkpoint:** select the right representation for three short rules: sum, named bands, or faces kept separately.

### Part D — Different ways to read a handful of dice

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L13 | Keeping and dropping describe equivalent selections. | One D&D randomly generated ability score. | Keep/drop notation, comparison with 3d6. | Explain why 4d6 drop one equals 4d6 keep three, but neither equals 4d6 summed. |
| L14 | “Any” and “none” are complementary questions. | The Pool: at least one natural 1, for a supplied pool size. | `.p_any`, `.p_none`, a small existing-style loop. | Derive 11/36 for any 1 on two d6 and discuss diminishing returns. |
| L15 | A count is itself a distribution. | Shadowrun 5e hits on 5–6 as an explicitly isolated component; no limits, Edge, or glitch resolution yet. | `.count`, matching a face list/range, `.p_at_least`. | Distinguish mean hits, any hit, and at least two hits. |
| L16 | Different dice can belong to one pool. | Cairn 2e dual-weapon damage: keep the higher damage die before armour. | Joining mixed pools; sum versus maximum. | Compare one weapon and two weapons without accidentally adding their damage. |

**Checkpoint:** translate three verbal pool rules into three different queries or reductions, and explain the differences.

### Part E — Custom dice and transformations

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L17 | A custom face list describes a real physical die. | Fate Core’s four Fate dice, before adding skill/opposition. | `die_faces`, negative numbers, repeated face weights. | Explain why repeated signs matter and why ±4 each have probability 1/81. |
| L18 | Reusing a distribution is not reusing the same rolled face. | D&D 2014 critical damage dice as a contrast with an explicitly invented “double the rolled damage” rule. | Contrast independent addition with multiplication; state modifier treatment. | Explain why `r + r` and `r * 2` can have the same mean but different possible results. |
| L19 | Armour changes each result and cannot create negative damage. | Cairn 2e damage after armour, before HP/STR consequences. | Subtraction and a zero floor using `clamp`; explicit bounds. | Explain the probability of zero damage and how several raw rolls become the same result. |
| L20 | Round each result, not the average. | D&D 2014 base Fireball damage, conditional on a successful save. | `//`, a conditional-report label. | Explain why the mean of half-rounded-down 8d6 is 13.75, not 14. |

**Checkpoint:** design and check a damage transformation, keeping “conditional on a save” visible in the report.

### Part F — Writing rules of your own

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L21 | A function packages a reusable calculation. | The already familiar D&D ability-check comparison across DCs and bonuses. | `def`, parameters, scope, `return`; return a result rather than output inside a helper. | Write a small helper and call it with two sets of parameters. |
| L22 | Exceptions can depend on the natural die. | D&D 2014 attack roll: ordinary miss/hit/critical; no special critical range or fumble house rules. | `if`, `==`, comparisons, branch order, `classify`; later show a band-based alternative. | Preserve natural 1/20 and test both very easy and impossible-without-a-20 targets. |
| L23 | One custom rule can inspect the same pool faces in several ways. | Complete base Blades action-result categories, including two sixes and the zero-dice exception. | Function-as-argument, `pool_map`, indexing/list operations, integer result codes where necessary. | Distinguish a highest-six from a two-sixes critical. Keep zero dice separate from position. |
| L24 | Compare two independent rolls and define what a tie means. | A named Fate Core action against active opposition, with that action’s tie/outcome interpretation. | `joint_classify`, two callback arguments, difference/margin. | Explain the tie convention and validate equal-skill symmetry where applicable. |

**Checkpoint:** write a small classifier from a new rules paragraph, including its boundary cases, without starting from a complete template.

### Part G — Decisions, rerolls, and open-ended rules

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L25 | A later roll can be interpreted using an earlier result. | Cairn Blood Elk: after armour and HP loss, use reduced STR for a critical-damage save. | A two-stage decision tree inside a joint classifier; build on L19/L21–24. | Explain why low-damage branches ignore a pre-enumerated potential save; handle death before a save when STR reaches zero. |
| L26 | Rerolling once is a policy, not filtering away bad faces. | A generic “reroll a 1 once, keep the second roll” component; link to an edition-specific published reroll recipe. | Choose between first and second independent faces with a function. | Show that a 1 remains possible, and say what information the policy may use. |
| L27 | Conditioning changes the question being asked. | The same reroll example compared with “keep rolling until not a 1”, plus the existing “low dice score zero” component. | `keep`, `remove`, `ignore`/`convert`, without introducing them all as unexplained synonyms. | Distinguish once-only reroll, repeated reroll, conditional distribution, and zeroing. |
| L28 | Infinite rules require explicit finite modelling choices. | A single Savage Worlds-style acing damage die, visibly only a component and with a finite cap. | `explode`, named cap, comparison across cap sizes. | Explain when the cap changes a result and why a capped distribution is not the unlimited game rule. |

**Checkpoint:** choose the correct operation for several similar-sounding reroll/filter rules and state the limits of the resulting model.

### Part H — From mechanics to a report someone can use

| ID | Main concept / question | Familiar situation and scope | Scripting step | Exit check |
|---|---|---|---|---|
| L29 | Similar-looking open-ended systems can have different continuation rules. | Rolemaster RMSS full open-ended d100: low subtraction, high addition, explicit chain cap. | Read and validate a specialised helper; use the known low/high worked paths. | Explain why “explode only the maximum” is not this rule. |
| L30 | Compare a decision made after seeing a result, without using future information. | Fate Core: a legal single invocation against fixed passive opposition, comparing +2 with rerolling; fixed objective and resource budget. | A policy inside a joint calculation; build on L17/L24/L26. | State the policy before evaluating it and prevent it from seeing the hypothetical reroll. |
| L31 | A useful report connects assumptions, comparisons, and conclusions. | Brindlewood Theorize clue-count comparison, after edition-specific verification. | Compose an existing loop/table with outcome queries; optional comprehension only now. | Distinguish any correct theory, clean correct theory, and the 12+ result; derive legal parameter ranges. |
| L32 | Build and defend your own model from a rules brief. | Choose a supplied scoped brief: one-target Fireball including its save, an adapted Cairn strike, or a small homebrew pool mechanic. | No required new syntax; specification → model → checks → report. | Deliver a complete `.dice` document, a correct result, one independent check, and clear exclusions. |

**Graduation task:** start with a rule not previously written out in full. Ask for assumptions if the brief is incomplete. A sample solution is available after the attempt, with reasoning rather than just code.

### Optional advanced workshops, not prerequisites for graduation

| Workshop | Transfer concept | Boundaries |
|---|---|---|
| A01 — Several turns or repeated tests | State, stopping conditions, and a fixed policy. | Start with a small bounded scenario. Full death saves or combat need a feasibility spike; do not pretend repeated marginal distributions preserve state. |
| A02 — Yahtzee, one throw | Matching faces and mutually exclusive pattern categories. | One five-dice throw, not optimal play over three throws and a score sheet. Use the real category definitions and state how overlaps are treated. |
| A03 — Two cards without replacement | Dependence and a changing sample space. | Standard deck, explicit identity/suit representation, small exact count. Do not model it as two independent `d(52)` draws. A scalar/table calculation is acceptable; no native deck type is implied. |
| A04 — More intricate resolution systems | Degrees of success and information that must not be collapsed early. | Candidate examples: Pathfinder 2e degree adjustments or CoC 7e bonus/penalty dice with the shared units digit. Verify rules and implementation before scheduling. |

### Why this order is different

- Real game contexts appear from the start, not only at the end.
- A reader produces a useful comparison report by L08 and named results by L10.
- Tables, loops, functions, and callbacks are separate steps instead of one syntax cliff.
- Familiar calculations recur with one added idea, rather than repeatedly starting from an unfamiliar game.
- Explicit independence arrives before general custom/joint models.
- Rerolls, conditioning, and explosions are compared as distinct mechanics.
- Advanced lessons ask the reader to write, check, and explain a policy or report, not merely run larger expressions.

The exact placement of L11 and the first use of `DicePool` needs a beginner pilot: if explicit totals prove confusing earlier, introduce a minimal pool-versus-total explanation sooner without moving all pool mechanics forward.

## 8. Expanded cookbook design

### Organisation

Offer two indexes over the **same recipes**, not two sets of duplicated pages:

1. **By game/edition** for someone arriving with a rulebook question.
2. **By mechanic**: totals, target checks, graded outcomes, pools, custom faces, damage, rerolls/explosions, opposed rolls, decisions, and state/dependence.

Each entry should show difficulty, prerequisites, parameters, model scope, and whether it is a component or a complete scoped procedure. A recipe can be advanced while remaining self-contained; it should link to explanations rather than reteach ten lessons inline.

### Recipe contract

A recipe should contain:

- **Question and use case.** A report somebody might genuinely want.
- **Game / edition / source / rule section.** Or a visible “generic building block” label.
- **Scope.** What is included and, equally importantly, what is not.
- **Inputs.** Meaning, defaults, permitted ranges, and relationships between parameters.
- **Model sketch.** The dice, transformations, choices, and order of resolution.
- **Complete runnable source.** No hidden definitions or dependency on having run another page.
- **Reading the report.** A few anchor numbers and the conclusion they support.
- **Adaptations.** Safe substitutions and changes that require a different model.
- **Checks and limits.** Boundary cases, an independent oracle, numerical tolerances, and any finite cap.
- **Related lessons and component recipes.** With explicit composition constraints.

A building block is not automatically a composable random variable. For example, separate hit and damage reports do not by themselves preserve a joint hit-and-damage event. Say how a component must be integrated, not merely “combine these scripts”.

### Candidate catalogue

**Model status:** **F** = finite procedure compatible in principle with existing operations; **B** = bounded model of a potentially unbounded rule; **S** = needs an implementation/rules feasibility spike. None of these letters means the new recipe is already verified. All named-game entries require the rule-source gate.

| ID | Candidate recipe | Kind / distinctive value | Model |
|---|---|---|---|
| R01 | D&D 2014 ordinary ability checks | Basic threshold, bonuses, advantage/disadvantage; explicitly not attack criticals. | F |
| R02 | D&D 2014 attacks: miss, hit, critical | Natural-face exceptions and extreme AC/modifier cases. | F |
| R03 | D&D 2014 one ability score, 3d6 versus 4d6 drop lowest | Preserve and improve the existing recipe; not a full six-score generation policy. | F |
| R04 | D&D 2014 Fireball damage given save failure/success | Preserve the conditional transformation recipe. | F |
| R05 | D&D 2014 Fireball: one target’s overall damage/risk | Add save probability; state spell level, HP question, and excluded features. | F |
| R06 | D&D 2014 weapon damage including misses and criticals | Joint attack/damage modelling, not multiplying unrelated displayed results. | F |
| R07 | Traveller task checks and Effect | Total versus margin; edition-specific difficulty and modifiers. | F |
| R08 | Traveller boon/bane | Keep/drop as a real task mechanic, with edition verification. | F |
| R09 | Call of Cthulhu 7e success levels | Regular/hard/extreme/critical/fumble boundaries and rounding. | F |
| R10 | Call of Cthulhu 7e bonus/penalty dice | Preserve the shared units digit; it is not simply highest/lowest of two independent d100 totals. | S |
| R11 | GURPS 4e skill check | 3d6 roll-under with the chosen edition’s automatic/critical exceptions. | F |
| R12 | Fate Core 4dF plus skill | Preserve the custom-die recipe and extend it with meaningful task questions. | F |
| R13 | Fate Core opposition and outcome levels | Active versus passive opposition, tie, margin, and a specified action. | F |
| R14 | Fate Core one invocation: +2 or reroll? | A resource decision against fixed opposition, with a stated objective and legal timing. | S |
| R15 | Shadowrun 5e raw hits | A clearly scoped 5–6 success-count component; no Edge or limits silently implied. | F |
| R16 | Shadowrun 5e hits and glitches | Two properties of the same pool; verify glitch/critical-glitch rules and display their relationship honestly. | S |
| R17 | The Pool: success by supplied pool size | Preserve any-natural-1 and add a useful diminishing-returns table. | F |
| R18 | Blades: base action-result categories | Preserve highest die, criticals, and zero dice; separate position/effect from numerical result bands. | F |
| R19 | Blades: resistance stress | A different reading of a pool; explicitly include the critical result and chosen source version. | F |
| R20 | Brindlewood Bay: Theorize | Preserve parameter comparison; validate clue eligibility/counting and correct the result labels. | F |
| R21 | Cairn 2e: damage versus armour | A short damage-transform component. | F |
| R22 | Cairn 2e: dual weapons and the higher die | A recognisable mixed-pool example; not a sum. | F |
| R23 | Cairn 2e: Blood Elk hit, scars, and critical damage | Preserve the rich scenario; either validate editable inputs or visibly constrain them, including the STR-zero boundary. | F |
| R24 | Savage Worlds Adventure Edition: a basic Wild Card trait roll | Trait die, Wild Die, acing, critical failure, modifier/raise rules under a narrowly stated scope. No Bennies or special abilities by default. | B/S |
| R25 | Savage Worlds Adventure Edition: acing damage dice | Contrast summing independently acing dice with exploding the whole total. | B/S |
| R26 | Rolemaster RMSS: full open-ended d100 | Preserve the high/low distinction and publish cap sensitivity. | B |
| R27 | Pathfinder 2e: degrees of success | Compare total-based degree with natural-1/20 degree adjustments; pin legacy/Remaster source and exclusions. | F/S |
| R28 | D&D 2014 death saves under a fixed intervention policy | Bounded/stateful sequence, early stopping, special 1/20 outcomes; no incoming damage unless specified. | S |
| C01 | Count, any, none, enough: the same small pool | Preserve the current count-high-faces building block; compare the questions side by side. | F |
| C02 | Only high dice count: filter or score zero? | Preserve the current filter examples; make the ambiguous wording explicit. | F |
| C03 | Reroll a bad face once | First/second roll decision; link to an edition-verified published reroll rule. | F |
| C04 | Several independent checks: all, any, or a required count | Explain independence, fixed probabilities, and when the shortcut stops applying. | F |
| C05 | A two-stage check with a specified policy | A bounded decision tree, not an implied general combat solver. | F |
| C06 | Opposed rolls with different tie rules | Show how win/tie/lose changes between “ties defend” and another explicit convention. | F |
| C07 | Custom/weighted dice | Duplicate faces, asymmetric dice, and checks that weights are preserved. | F |
| C08 | Damage floors, ceilings, and rounding order | Demonstrate non-commuting transformations using a clearly invented rule. | F |
| C09 | How much does an explosion cap matter? | Preserve the simple exploding die; compare caps and finite versus unlimited expectations where analytically available. | B |
| C10 | Repeated actions with state | A small fixed-policy example that retains HP/resources/stopping conditions; larger generalisation gated. | S |
| X01 | Yahtzee patterns on one throw | Optional real-game transfer example, not three-roll optimisation. | F |
| X02 | Two cards without replacement | Optional dependence lesson with a standard deck and a tiny countable event. | F/S |

### Publication priorities

**First release:** rewrite every existing recipe, split misleadingly broad ones where needed, and add the most useful missing foundations: ordinary D&D checks versus attacks, Traveller tasks, CoC regular checks/levels, Fate opposition, Cairn mixed damage, and reroll-once. Aim for **20–24 well-checked entries**, not a rushed catalogue of names.

**Second release:** extend towards the catalogue above, including more decision examples and the source/feasibility-gated systems. Keep state-heavy models and the two non-TTRPG exercises out of the critical path.

Use three criteria to choose additions: **recognition**, **a genuinely different mechanic**, and **verifiability**. Do not add five near-identical target-number systems simply to increase the game count.

## 9. Accuracy policy: game rules and probability are separate reviews

### A rule dossier for every named-game model

Before drafting a polished lesson or recipe, record:

| Field | Required content |
|---|---|
| Identity | Game, edition, printing/version or SRD revision where available. |
| Source | Publisher/author rules or authorised SRD/reference, section/page, link, and access/review date. A product page establishes identity, not a rule. |
| Procedure | A concise paraphrase of the exact resolution order. |
| Scope | Full scoped procedure, isolated component, house rule, or finite approximation. |
| Parameters | Legal values, dependencies, defaults, and deliberate exclusions. |
| Timing/information | What the player knows when each choice is made; whether a reroll replaces or competes with the first roll. |
| Exceptions | Natural results, ties, criticals, rounding, zero dice, death/absorption, special-case thresholds. |
| Oracle | Independent hand count, closed-form probability, rulebook worked example, or separately implemented tiny enumeration. |
| Review status | Proposed, source checked, maths checked, browser checked, published. |

Prefer primary sources. Use community discussions to discover ambiguities, not as the final authority when official text is available. When access is unavailable, mark the claim pending and request a legitimate source/excerpt; do not silently certify it from memory.

Quote sparingly, paraphrase the procedure, link to the source, and meet applicable attribution/licence requirements. Avoid implying publisher endorsement.

### Scope labels that readers should actually see

- **Complete scoped mechanic:** “D&D 2014 ordinary weapon attack against a fixed AC, with the listed exclusions.”
- **Component:** “8d6 damage, conditional on a successful save.”
- **House rule:** “Reroll any result below three once.”
- **Bounded approximation:** “Acing die with at most six extra rolls.”

Do not bury these distinctions in a contributor document.

### Useful independent numerical anchors

These are potential regression checks for the specified mathematical models, not proof that every surrounding game rule has been represented:

| Model | Anchor |
|---|---|
| Fair d6 | Each face 1/6; mean 3.5. |
| 2d6 | Exactly 7 is 6/36; total mean 7. |
| D&D-style normal attack, +5 against AC 15, ordinary 1/20 exceptions | Hit including critical is 11/20. With advantage, 319/400; critical is 39/400. Also test extreme AC separately so incorrect generic check logic is exposed. |
| 4d6 drop lowest | Mean 12.2445987654…; same PMF as keeping the highest three. |
| Three d6, each 5–6 counts | Counts 0/1/2/3 have probabilities 8/27, 12/27, 6/27, 1/27. |
| Any natural 1 on two d6 | 11/36. |
| Three-band 2d6 total at modifier 0 | 6−: 15/36; 7–9: 15/36; 10+: 6/36. The game/move supplies the interpretation. |
| 4dF | Mean 0; ±4 each 1/81; zero 19/81. |
| Base Fireball half-rounded-down damage | Full mean 28; successful-save conditional mean 13.75. |
| Blades two dice | Bad 9/36, partial 16/36, clean single-six 10/36, critical 1/36. |
| Blades zero-dice rule | Bad 27/36, partial 8/36, clean 1/36, critical 0. |
| Current Cairn default slice: HP 4, armour 2, STR 11, horns d8 | No effect 1/4; HP loss 3/8; scar 1/8; STR loss/save passed 19/160; critical damage 21/160. Further parameter changes need their own rule-boundary checks. |
| Exploding fair d6, one extra roll allowed | 1–5 each 1/6; 7–12 each 1/36; no 6. Explain why the terminal 12 is a cap artefact relative to unlimited acing. |

For an ordinary exploding die with maximum-face probability `q < 1` and a cap of `D` extra rolls, the probability that the cap interrupts an otherwise continuing chain is `q^(D+1)`. This is a bound on disagreement for an event under the same roll sequence, **not automatically a bound on mean damage error**. Teach those as different quantities. This diagnostic is not currently returned by the API.

## 10. Implementation constraints that shape the content

Do not write a polished advanced lesson first and discover later that it relies on imaginary APIs or unusable performance.

| Capability/constraint | Consequence for this plan |
|---|---|
| Numeric distributions use floating-point probabilities. | Say deterministic finite calculation rather than simulation; do not promise symbolic rational exactness from fraction formatting. |
| Ordinary totals use convolution; custom face-sensitive operations may enumerate complete tuples. | Large ordinary damage sums can be fine when a similarly sized `pool_map` is not. Pick the least expensive representation that preserves the needed information. |
| `MAX_JOINT_CELLS` is 1,000,000 for guarded pool enumeration. | Five d6 are 7,776 cases; seven d6 are 279,936; eight d6 are 1,679,616. Do not casually extend the current Blades callback to arbitrary pool sizes. A limit on one path is not a universal performance guarantee for all APIs. |
| Bare dice notation can preserve a pool in an assignment. | Teach explicit totals where needed; avoid examples whose meaning seems to change mysteriously when moved into a variable. |
| Reusing a `DieRoll` in arithmetic combines independent rolls. | Preserve shared information inside a joint rule; do not combine marginal results and claim the original dependence remains. |
| `pool_map` returns integer outcomes; `joint_classify` returns named outcomes. | Some composite numeric models need integer-coded rules or manual probability tables. Prototype the public script, not just the Rust engine idea. |
| No public general-purpose weighted-PMF builder or native deck/state-process API is established by the current reference. | Do not invent `mix`, `reroll`, `deck`, or Markov-chain helpers in lesson drafts. Use supported finite constructions or separate an engine proposal. |
| A pure fixed-length callback can model some sequential rules using pre-enumerated potential rolls. | Explain why unused future rolls are ignored correctly, and why this is not permission for a policy to inspect them early. |
| Five d20 faces already give 3,200,000 tuples. | A naive bounded death-save sequence can exceed the generic pool limit. Explore aggregated state transitions rather than assuming a five-roll `pool_map` will work. |
| Explosions and Rolemaster chains have finite caps. | All such lessons and recipes need explicit bounds, cap sensitivity, and performance checks. Large exploded mixed pools need a spike. |
| `load` is disabled in public playground evaluation. | A cookbook entry must be self-contained. Shared helpers require an authoring/build solution, not instructions to import another recipe at runtime. |
| Public source/output limits exist. | Longer prose does not imply unlimited executable work. Keep reports focused; benchmark in the browser, not only native tests. |

The course must distinguish **decisions before rolling**, **decisions after observing a result**, and **sequences where the state changes**. These are different modelling jobs. “Best” is also incomplete without an objective: highest success probability, lowest chance of harm, or greatest expected damage may recommend different policies.

## 11. Publication work needed before the full rewrite

### 11.1 Show executable code without maintaining two copies

In the current [`weave.rs`](src/engine/literate/weave.rs), executable fences contribute their outputs but not their source code to the rendered page. Non-executing `text` fences supply the visible examples. That explains why so many files duplicate code.

**Recommendation:** add a tested way to display the executable source alongside its result. Default to visible source for tutorial/cookbook material; decide separately whether other user-authored reports need a hide/collapse option. Do not assume an unsupported fence annotation already exists.

Until this is available, do not delete the display copies and declare the source problem solved. A temporary generated display copy is preferable to two human-maintained versions, but it should be transitional.

### 11.2 Open the whole lesson or recipe

The current static enhancer attaches snippet links. It can select a display-only fragment with missing context; lesson 03 provides a concrete failure. Large inline payloads also encounter the current 7,000-character encoded-length handoff limit.

**Recommendation:** publish the `.dice` source and provide a page-level **Open this lesson in the playground** link that loads the full document, with source identity. A same-origin published-source route or an equivalent bounded handoff needs design and tests; it is not available merely because the source exists in Git.

Snippet opening can remain for explicitly standalone examples. Do not offer arbitrary dependent fragments as though they were complete scripts. Cross-page independence and shared definitions within one document are compatible goals.

### 11.3 Make multi-block report placement reliable

Output-to-fence binding currently counts source lines containing `output(` and consumes global outputs in order, putting leftovers at the last fence. Loops, conditional outputs, and helper functions can break the intended placement.

Resolve this before relying on a long, staged lesson with loops in several blocks. Either implement dependable output provenance, or define and test a constrained authoring pattern for the first release. A safe interim pattern is one final output-producing block, but the resulting reading experience must still be assessed.

### 11.4 Navigation and discovery

The build currently discovers flat `.dice` directories and sorts filenames. The cookbook is effectively an alphabetical list; tutorial previous/next links are manually inconsistent.

Plan for:

- A tutorial index with parts, lesson objectives, prerequisites, and progress/checkpoints.
- Generated previous/next navigation from one ordered manifest.
- Cookbook game and mechanic indexes with component/full-model and difficulty labels.
- Stable content IDs independent of display numbering.
- Redirects/aliases from old public lesson URLs when the sequence changes.
- Link checks on **built HTML**, including anchors and downloads, not only source Markdown paths.
- Updates to `docs/README.md`, cross-links, and relevant `llms.txt` guidance.

Do not simply create nested cookbook folders: the current non-recursive build and tests would omit them. Either keep flat source paths with metadata initially, or change discovery deliberately.

### 11.5 Decide where metadata lives

Prefer a small, validated content manifest keyed by stable IDs for order, prerequisites, tags, aliases, and rule-review status. Keep explanations and calculations in the literate source. YAML front matter is not currently a defined literate `.dice` metadata facility; adding it is a format change, not an authoring convention we can assume.

No particular manifest syntax is essential to this plan. Avoid maintaining separate lesson lists in the website, user guide, and multiple tests.

## 12. Validation: four kinds of correctness

| Layer | What it establishes | Required checks |
|---|---|---|
| Execution | The document runs. | Evaluate every complete `.dice` source unchanged through the public API; require meaningful outputs. |
| Mathematical/model correctness | The calculation implements the stated procedure. | Independent oracle, exact small cases within tolerance, boundary inputs, normalization/category coverage, transformation/equivalence checks. |
| Rules fidelity | The procedure matches the named edition and scope. | Source dossier review, exception checklist, explicit exclusions, no unsourced “usual rule” substitutions. |
| Learning/publication | A newcomer sees and can use the intended lesson. | Visible code matches executed code, whole-document handoff works, outputs appear in the right place, links resolve, exercises have checked answers, and a beginner can complete the task. |

### Improvements to the existing tests

- Preserve existing useful assertions, but replace tests that merely check “between 0 and 1” with specific expected results where feasible.
- The cookbook currently has detailed tests for only a few examples; whole-corpus execution is not a substitute for recipe-specific oracles.
- Test non-default inputs, especially natural 1/20, target equality, zero/invalid pools, criticals, HP exactly zero, STR zero, and the cap boundary.
- Verify tutorial exercises and any text-only code offered for copying. The current duplicated source can pass corpus evaluation while the visible snippet fails.
- Assert the meaning of labels: a “clean success” table must not query “success including complications”.
- Build representative multi-fence documents with loops and helpers to test output placement.
- Run a real browser/WASM smoke workflow for representative lessons. `make check-wasm` is a compilation check, not browser execution or a usability test.
- Benchmark the slowest recipes at documented defaults and supported parameter bounds. Choose a browser responsiveness budget after measuring; avoid promising one from native timings.
- Ask at least one scripting novice to use the first part without verbal rescue. Record where they cannot predict, edit, explain, or construct the requested model.

## 13. Proposed work plan and exit criteria

### Phase 0 — Preserve the evidence and define the scope

**Work:** retain an inventory of all 23 current documents, their calculations, expected outputs, known defects, and old URLs. Turn the highest-risk game descriptions into source dossiers. Agree the first-release game/edition set and the meaning of “complete scoped mechanic”.

**Exit:** every current example has a preserve/split/replace destination; no named system is treated as verified merely because its script runs.

### Phase 1 — Pilot the course, not just the page template

**Work:** draft L01–04 as a continuous mini-course. Prototype a late lesson such as L23 or L25 to expose callback, source-display, and report-placement constraints. Prepare the corresponding small cookbook examples. Test with a novice and a rules-knowledgeable reviewer.

**Exit:** the beginner can construct a slightly changed task report; the advanced prototype uses supported APIs and has an independent check. Revise lesson size/order based on observed difficulty.

### Phase 2 — Fix the authoring/publication foundation

**Work:** visible executable source, whole-document opening, output placement or a tested interim restriction, metadata/manifest, generated navigation, and built-link validation. Establish a recipe test harness and rules-review checklist.

**Exit:** one complete pilot lesson survives edit → test → static render → open in playground → rerun without duplicate source or lost setup. One loop-containing staged document places outputs correctly under the chosen contract.

This phase should not become a complete website redesign. Fix the features the new course actually depends on.

### Phase 3 — Write and review Parts A–D

**Work:** basic rolls, queries, comparisons, loops, named outcomes, and pool reading. Reuse verified small calculations, but rewrite the prose and exercises from the new lesson cards. Publish a first useful group of cookbook entries alongside these lessons.

**Exit:** L01–16 form a coherent path with no unintroduced required syntax; every example and exercise is checked; the reader can build a comparison and choose a suitable representation.

### Phase 4 — Write and review Parts E–H

**Work:** custom dice, independence, damage transformations, functions, classification, joint rules, rerolls, conditioning, explosions, decisions, and capstone reports. Prototype the difficult policies before promising them in the index.

**Exit:** L17–32 have verified rule scope, tested boundaries, acceptable browser performance, and usable transfer tasks. The final task tests construction, not memorisation.

### Phase 5 — Expand the cookbook deliberately

**Work:** complete the first-release 20–24 recipe set, then add the remaining catalogue as source and implementation checks pass. Pair game recipes with relevant components. Keep gated state-heavy and non-TTRPG workshops visibly optional.

**Exit:** the cookbook can be browsed by game or mechanic; every entry is independently runnable, source-reviewed where named, and clear about what can safely be changed.

### Phase 6 — Migrate and publish

**Work:** update indexes, old-URL mappings, user guide, examples referenced by `llms.txt`, and cross-links. Run full native checks, WASM compilation, actual browser smoke tests, static-site/link checks, and the slow-recipe benchmark set.

**Exit:** no old URL silently points to a different lesson because a number was reused; no promised game mechanic lacks a reviewed scope; source and published experience agree. Keep a release note explaining the new path and important corrections.

### Definition of done for one page

- [ ] One clear question/objective and appropriate prerequisites.
- [ ] A named and sourced game/edition, or an explicit generic/house-rule label.
- [ ] No required syntax appears before it is explained or linked as a prerequisite.
- [ ] Exactly one maintained executable model for each demonstrated calculation.
- [ ] Complete file runs from a fresh session.
- [ ] Meaningful expected results and at least one independent check.
- [ ] Non-default/boundary parameters checked or explicitly prohibited.
- [ ] Explanation distinguishes what the model computes from what the wider game does.
- [ ] Report labels, probability units, output placement, and conclusions are correct.
- [ ] Whole-document opening and published links work.
- [ ] Exercises have verified answers or acceptance criteria.
- [ ] Game-rule and beginner-reading reviews are recorded.

## 14. Decisions to settle before large-scale drafting

These do not prevent the plan from being useful now. Suggested defaults are included to avoid stalling on everything at once.

| Decision | Suggested default |
|---|---|
| Course length and pacing | Use the 32-lesson outline as a budget; validate with the first four lessons rather than locking all titles immediately. |
| D&D version | Use **2014** for the first rewrite of the existing examples; label it prominently. Add 2024 differences as separately checked material, not silent substitutions. |
| PbtA anchor | Choose one named game and move, initially Dungeon World Defy Danger if the source is accessible. Keep “common 2d6 bands” as a generic component when no move is intended. |
| Fudge/Fate focus | Preserve the 4dF mathematics; use **Fate Core** for opposition and invocation examples so the rules context stays consistent. |
| Source availability | Prefer accessible authorised text. Keep Brindlewood/Rolemaster edition details pending until a legitimate precise source is in hand. |
| Repeated/sequential models | Teach bounded policies first. Treat general combat/death-save state models as a separate feasibility task. |
| Beginner shorthand | Teach useful dice notation, but use explicit `.sum()` when storing totals. Do not turn compatibility quirks into the lesson’s main subject. |
| Solutions | Provide checked explanations after each exercise or in linked solutions; no unsupported interactive exercise system is required. |
| Reuse between lessons and recipes | Reuse concepts and checked kernels, but keep each published document runnable. Consider build-time inclusion only if it actually reduces drift without hiding code. |
| Publishing scope | Ship coherent parts and verified recipes progressively; do not wait for all 40 catalogue candidates. |

## 15. Sources and evidence

### Repository sources

- [Existing tutorial corpus](docs/tutorial/01-one-die.dice) and all other lessons inventoried in section 4.
- [Existing cookbook corpus](docs/cookbook/ability-scores-4d6dl1.dice) and all other recipes inventoried in section 5.
- [Tutorial numerical tests](tests/tutorial_samples.rs), [cookbook tests](tests/cookbook_samples.rs), [whole tutorial evaluation](tests/docs_tutorial_literate.rs), and [whole cookbook evaluation](tests/docs_cookbook_literate.rs).
- [Learning/documentation design](docs/design/learning-and-llm.md), [probability-engine design](docs/design/probability-engine.md), and [literate document contract](docs/design/literate-documents.md).
- [Updated script API reference](docs/references/stdlib.md) and [reference authoring guidance](docs/references/README.md).
- [Static site builder](bin/build-tutorial-site.sh), [literate renderer/output binding](src/engine/literate/weave.rs), [snippet enhancer](src/ui/static_site.rs), and [handoff size policy](src/ui/playground_handoff.rs).
- [Joint enumeration and cap](src/engine/enumerate.rs) and [public evaluation/guardrails](src/engine/playground.rs).

### External rules sources consulted

These are bounded spot checks, not a certification of the entire proposed catalogue. Record a dated, section-specific dossier when turning a candidate into publishable content.

| ID | Source | What this review checked |
|---|---|---|
| S1 | [D&D Basic Rules 2014: Using Ability Scores](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/using-ability-scores) | Ability checks, ordinary saves, advantage/disadvantage, and comparison to DC. |
| S2 | [D&D Basic Rules 2014: Combat](https://www.dndbeyond.com/sources/dnd/basic-rules-2014/combat) | Natural 1/20 on attacks, critical damage dice, shared area-effect damage roll, and distinct death-save rules. |
| S3 | [Blades in the Dark: The Core System](https://bladesinthedark.com/core-system) | Highest die, multiple-sixes critical, and zero/negative dice taking the lower of two with no critical. |
| S4 | [Blades in the Dark: Action Roll](https://bladesinthedark.com/action-roll) | Position/effect as distinct from dice count, action-roll procedure, and increased effect on a critical. |
| S5 | [Cairn 2e Player’s Guide: Core Rules](https://cairnrpg.com/second-edition/players-guide/core-rules/) | Roll-under saves, armour, dual weapons, HP-zero scars, STR overflow/save, and death at zero STR. |
| S6 | [Cairn 2e Warden’s Guide: Bestiary](https://cairnrpg.com/second-edition/wardens-guide/bestiary/#blood-elk) | Blood Elk horns d8 and critical-damage description. |
| S7 | [Fate Core SRD: Taking Action, Dice, and the Ladder](https://fate-srd.com/fate-core/taking-action-dice-ladder) and [Four Outcomes](https://fate-srd.com/fate-core/four-outcomes) | Four Fate dice, skill plus roll, active/passive opposition, and outcome/margin framing. Invocation policy still needs its own rules check. |
| S8 | [The Gauntlet: Brindlewood Bay](https://www.gauntlet-rpg.com/brindlewood-bay.html) | Publisher/product and availability of reference materials only. Theorize’s exact edition-specific procedure was **not** verified from this product page. |
| S9 | [Chaosium: Call of Cthulhu Quickstart](https://www.chaosium.com/cthulhu-quickstart/) | Official route to rules material for future verification; not a completed audit of percentile exceptions or bonus dice. |

For additional proposed games, source acquisition and rule review remain explicit tasks. Do not mistake an attractive example idea for a verified model.
