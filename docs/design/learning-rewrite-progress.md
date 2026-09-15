# Tutorial and cookbook rewrite: first staged release

Implements the pilot and initial publication foundation from
[`tutorial-and-cookbook-design-ideas.md`](../../tutorial-and-cookbook-design-ideas.md).
This is **not completion of the 32-lesson course or 20–24-recipe first release**.

## Delivered

- L01–L04: a continuous beginner pilot with predictions, explained syntax,
  checked numbers, guided changes, and construction exercises with solutions.
- R23: a source-checked Cairn 2e advanced prototype. The model checks death at
  zero STR before the potential save. Default categories and low-STR boundaries
  have independent numerical tests; invalid input ranges fail explicitly.
- Executable fences are visible in woven reports. Pilot pages maintain exactly
  one executable block and no display-only code copy.
- Whole-document static handoff publishes the original `.dice` alongside HTML.
  The page fetches a same-origin source (redirects rejected), enforces the public
  64 KiB source limit, stores content plus filename using the existing pending
  load contract, and opens the playground. Failures display a download fallback.
- A versioned manifest inventories all 23 existing URLs and stable content IDs,
  with destinations for retained legacy lessons. Generated previous/next links
  and cookbook game/mechanic indexes use that inventory. No URLs were reassigned,
  so redirects are not needed for this release.
- The site build checks local tutorial/cookbook links, downloads, assets, and
  anchors after HTML generation. Stale links found by that check were repaired.
- `make check` validates the manifest and runs Node handoff tests as well as Rust
  tests, clippy, and formatting. Rust tests cover the full literate corpus,
  pilot arithmetic, Cairn boundaries, and loop/helper output placement.

## Authoring contract during migration

Pilot documents have **one executable fence**. All definitions and outputs live
there; prose may precede/follow it. This restriction is checked by the manifest
validator. It avoids relying on the current renderer's static `output(` count
for loops and helper calls across multiple fences. General output provenance is
still pending; the renderer has not acquired dynamic source-location tracking.

Legacy documents may still contain duplicate display snippets and older
multi-fence structures. Their status is displayed, not silently upgraded to
reviewed. Replace them page by page rather than mechanically deleting examples
that may explain a distinct intermediate calculation.

## Source/review record

Reviewed online on 2026-09-15:

- Cairn 2e Player's Guide, Core Rules: Saves, Armor, Attacking & Damage,
  Critical Damage, Attribute Loss, Scars.
  https://cairnrpg.com/second-edition/players-guide/core-rules/
- Cairn 2e Warden's Guide, Blood Elk: horns d8 and critical-damage goring effect.
  https://cairnrpg.com/second-edition/wardens-guide/bestiary/#blood-elk

R23 scope excludes scar resolution, recovery, repeated attacks, and impaired or
enhanced attacks. Integer HP > 0, armour 0–3, and STR 1–20 are the supported
inputs. Oracle: count the 160 equiprobable horn/potential-save pairs, with 19
passed and 21 failed saves at defaults; STR 1 makes both overflowing horn faces
fatal. No extra random save is actually required by low-damage branches.

The opening lessons deliberately use **generic components**, not uncertified
Traveller or D&D edition claims. Their mechanics are completely stated on each
page. Replace those anchors with named-game contexts only after acquiring and
checking the relevant edition sources.

Brindlewood and Rolemaster retain **source pending** labels. All other unchanged
recipes retain **legacy** status, not an implied rules certification.

## Remaining work / gates

1. Pilot with a scripting novice and an independent rules-knowledgeable reader;
   record whether they can construct the checkpoint report without rescue.
2. Run a real browser/WASM load → edit → rerun smoke test, including restricted
   browser storage and long documents. Node DOM mocks and WASM compilation are
   not a substitute for this. Measure browser responsiveness before promising
   a performance budget.
3. Rewrite L05–L32 and the remaining recipes, including the plan's known
   mathematical and game-description corrections. Expand to 20–24 reviewed
   recipes before pursuing the optional catalogue.
4. Add full rule dossiers, difficulty and explicit prerequisite metadata as
   each new page is commissioned. Acquire legitimate Brindlewood and RMSS
   edition-specific sources; do not infer certification from successful eval.
5. Eliminate legacy duplicated display code; validate all transfer exercises
   and non-default parameters, not just the pilot's.
6. Preserve URL identity during later sequence migration: add explicit aliases
   before renaming any existing slug. Validate final documentation links across
   the entire site, not only the learning sections.

The design plan remains the destination; this record identifies the implemented
slice without marking human review, browser checks, or future content complete.
