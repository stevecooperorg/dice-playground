# Tutorial/cookbook rewrite — core first-release implementation

The implementation now contains **32 ordered lessons and 22 independently
runnable cookbook entries**. This is the first-release scope recommended by
[`tutorial-and-cookbook-design-ideas.md`](../../tutorial-and-cookbook-design-ideas.md),
not a claim to have certified every proposed game or delivered the optional
40-entry destination catalogue.

## Implemented

- Parts A–H: beginner syntax is introduced with the modelling need; the course
  progresses through queries, tables, loops, pools, transformations, functions,
  shared-face classification, conditional saves, rerolls, finite caps, and
  after-roll policies. L32 is a construction-and-validation graduation task.
- All original recipes were rewritten, with scope, inputs, complete source,
  numerical anchors, adaptations, and composition limits. The expanded set adds
  ordinary D&D checks versus attacks, one-target Fireball risk, Fate opposition
  and invocation, Cairn armour and dual weapons, and reusable generic components.
- Every canonical learning document has one executable fence. Its code is
  displayed directly; no separate display-only code copy remains. This is a
  validated interim placement contract, **not** general dynamic output provenance.
- `docs/learning-content.json` owns order, stable IDs, prerequisites, tags,
  review status, and nine old-URL mappings. Previous/next links and both cookbook
  indexes are generated from it. Old URLs explicitly say the lesson moved and
  link to its replacement; they also retain a complete-source download.
- Published sources open as whole documents, preserving filename and prose;
  long sources use the existing bounded storage handoff. Storage failures show
  a download fallback. Dependent fragments are not offered as standalone scripts.
- Static link checks cover tutorial, cookbook, guide, and reference HTML,
  including local anchors, scripts, CSS, and downloads.
- Corpus tests require meaningful outputs, one parsed executable fence, visible
  code, and exactly one rendered output block per evaluated output. Numerical
  fixtures in `docs/learning-checks.json` are supplemented by independent integer
  counts, closed forms, non-default inputs, and checked exercise solutions.

## Important corrections

- Keep-three/drop-one and keep-two/drop-two are identical complete distributions.
  Dropping a positive die cannot increase that throw's total.
- Lists support `.append`; overlapping probability-table questions need not sum
  to one and must not be called independent merely because they occupy rows.
- D&D 2014 ordinary ability checks do not have attack-style natural-1/20 rules.
- Cairn death at zero STR is checked before the potential saving roll.
- Blades zero dice is separate from desperate position. The interactive callback
  is limited to five dice; the cookbook uses an equivalent count decomposition
  supporting 0–20 dice. A seven-die debug-WASM callback took 12.8 seconds, so it
  was removed from the supported interactive range rather than excused by the
  engine's one-million-tuple guard.
- Brindlewood Theorize now follows the user-supplied text: gathered Clues
  establish eligibility at ceil(Complexity/2); accounted-for Clues determine
  the modifier. Clean queries use 10+, not 7+. The supplied text allows a
  Crown tier increase only when every Maven participates; the model explicitly
  stops before Crown use. Unverified RMSS printing/typo claims remain removed.
- Conditional Fireball damage is distinguished from overall target risk;
  halving each total has mean 13.75, not half the full mean (14).
- Reusing a distribution in addition means independent rolls; a policy may not
  inspect a hypothetical reroll before committing to it.

## Verification performed

2026-09-15, local debug builds:

- `make check`: manifest validation and migration tests, Node handoff tests,
  all Rust tests/doctests, clippy with warnings denied, formatting check.
- `make check-wasm`: successful no-default-features WASM compilation.
- `trunk build --dist /tmp/dice-full-site`: real WASM/static build.
- Static site build and link checks across all four documentation sections.
- `node bin/browser-learning-smoke.mjs /tmp/dice-full-site`: real headless
  Chromium, not DOM mocks. Covered loop lesson, Blades callback, Cairn dependent
  save, Fate policy, Brindlewood lesson and supplied-text recipe, Fireball
  capstone, source/filename identity, edited rerun,
  an 84,066-byte document beyond both the URL payload limit and the legacy
  64 KiB source bound (within the literate 256 KiB limit), restricted-storage fallback,
  and an old-URL migration page.
- Native benchmarks: three runs per canonical default plus expensive supported
  parameter bounds. Timings include CLI process startup.

Evidence is stored in [browser validation](learning-browser-validation.json)
and [native benchmarks](learning-native-benchmarks.json). Browser click-to-report
measurements include automation overhead: representative defaults were roughly
0.1–0.3 seconds, five-die callback about 0.5–0.7 seconds, and the twenty-die
count-based recipe about 0.1 seconds. These are measurements on one machine,
not a claim about all hardware. The repeatable smoke budget is five seconds
per selected case; the general browser timeout is thirty seconds.

Re-run via `make check-browser LEARNING_SITE=/path/to/trunk-output` (override
`CHROME` outside the default macOS Chrome path), and `make benchmark-learning`.

## Rules evidence and unresolved gates

Source dossiers: [Part B](learning-rules-part-b.md) and
[Parts C–H](learning-rules-core.md). Probability correctness is not rules
certification; source review is not novice usability review.

**Brindlewood source gate resolved against the supplied text (2026-09-15):**
`target/brindlewood-bay.md` provides the trigger, bands, Crown restriction, and
Keeper guidance. L31/R20 are now sourced pre-Crown standard-mystery models,
with exhaustive small face-count tests for Complexity 6–8 and separate gathered
and accounted Clues. The file does not identify an edition/printing; no broader
version claim is made. Its fingerprint is recorded in the core rules dossier,
and the book text is not copied into published or tracked documentation.

The following gates **cannot honestly be marked complete** from the available
materials:

1. **The Pool author rules:** the author's blog links `jwarts.com/thepoolrpg.pdf`,
   but that address returned an HTML redirect page rather than the PDF during
   this work. Its recipe and L14 are clearly scoped any-one mathematical
   components with source review pending. Supply a legitimate author-rules copy
   to certify the game attribution and precise assembly/resource exclusions.
2. **Rolemaster RMSS:** a legitimate printing/section or erratum is needed.
   L29 and the recipe document the existing helper's bounded mathematical
   procedure without asserting that it matches an unreviewed printing.
3. **Human review:** a scripting novice must attempt the checkpoints and an
   independent rules-knowledgeable reader must review the named scopes.
   Automated browser checks do not substitute for either person.

## Deliberately outside this first release

The plan's optional/second-release catalogue remains a roadmap, not a hidden
completion claim: full CoC success levels and shared-units bonus dice, Traveller
edition-specific tasks, Shadowrun limits/glitches, SWADE full trait resolution,
Pathfinder degree adjustments, death-save/stateful sequences, Yahtzee and cards.
The relevant beginner concepts have explicit generic components where a named
edition was not source-verified. General per-fence runtime output provenance is
also deferred in favour of the tested one-fence authoring restriction.

No optional model is linked as implemented, and no human/source gate is marked
passed merely because a script runs. The executable first-release content and
publication work are complete; these remaining evidence gates need external
inputs rather than more invented rules or unverifiable review claims.
