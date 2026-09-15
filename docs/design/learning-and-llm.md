# Tutorials, documentation, and LLM assistance

**Learning-system design and implementation approach — approved.** [Design index](README.md)

## Documentation is part of the product

The intended reader knows a tabletop rule before they know a probability API. Documentation should therefore begin with what happens at the table, explain the modelling choice, and only then introduce the operation that expresses it.

A tutorial is a gradual learning path. A cookbook recipe is a reusable example of a complete mechanic. A function reference is a precise lookup tool. An LLM reference helps a drafting tool use the same language correctly. These serve different readers but should not disagree about the language or maintain conflicting copies of a calculation.

## One executable source for each lesson and recipe

The key requirement is that tutorial and cookbook entries are **literate `.dice` files**. Each source contains its explanation and executable model together and can be evaluated unchanged by the CLI and playground.

```text
docs/tutorial/*.dice ──┐
                      ├── same engine ── evaluate in tests
 docs/cookbook/*.dice ─┤                ├─ evaluate in playground
                      │                └─ weave with dice render
                      │                          ▼
                      └────────────────── static HTML pages
```

The website is a rendering of those sources, not a separate collection of hand-maintained lesson prose with copied scripts elsewhere. An index or generated API reference can still be markdown; the single-source rule applies to the explanation/calculation pair in a lesson.

Every complete lesson should run without a user first executing fragments from another page. A multi-fence lesson may share definitions within its own one-shot run.

### Preserve the learning coverage

The recorded tutorial progression is retained:

| Stage | Topics |
|---|---|
| Basic totals | One fair die; adding two dice; flat bonuses; meeting a target. |
| Notation and pools | Dice notation; separate dice in a pool; mixed-size pools. |
| Face-sensitive rules | Keeping/removing/converting/ignoring faces; counting successes. |
| Comparing and naming results | Tables of checks; ordered outcome labels. |
| Familiar complete rules | D&D-style natural d20 results and advantage/disadvantage; PbtA-style 2d6 outcome bands. |

The inspected corpus contains 13 tutorial lessons. Cookbook coverage includes 4d6-drop-lowest ability scores, exploding dice, save-for-half fireball damage, counting high faces, Fudge/4dF, The Pool, Blades in the Dark, Brindlewood Bay, Cairn, and Rolemaster-style open-ended rolls. There are currently 10 recipe sources.

Those counts are a preservation baseline, not a permanent limit on content. Naming a game is not enough to define a rule: recipes still need to state the exact variant, assumptions, and finite caps being modelled.

## Publication pipeline

The inspected `bin/build-tutorial-site.sh` already builds the literate corpus without Pandoc:

| Source or step | Build behaviour |
|---|---|
| Tutorial `.dice` files | Call `dice render` and write pages under `dist/tutorial/`. |
| Cookbook `.dice` files | Call `dice render --layout cookbook` and write pages under `dist/cookbook/`. |
| Shared report CSS | Copy `tutorial-static/tutorial.css` into the site. |
| Lesson and recipe navigation | Generate indexes from the corpus and document headings. |
| `docs/README.md` | Use `dice render-md` for the user-guide page. |
| `docs/references/stdlib.md` | Use `dice render-md` for the reference page and index. |
| `llms.txt` | Copy it to the site root as `/llms.txt`. |
| Legacy snippet links | Run `dice enhance-static-site` after rendering. |

The renderer shares engine parsing, desugaring, evaluation, and weaving with the playground. The current native render function calls those lower-level functions directly rather than literally passing through `eval_program`; uniform limit enforcement should be verified rather than assumed.

Readers receive HTML already rendered at build time. Browsing a lesson does not require a client-side markdown renderer or running the probability engine. Running its source in the playground re-evaluates and re-weaves it in WASM.

The historical dual-build/Pandoc steps were a safe route to this architecture, not a continuing requirement. The original plan deliberately began with one pilot lesson before converting the remaining tutorial and cookbook.

### Whole-document handoff is the intended endpoint

A reader should be able to open the **whole `.dice` lesson** in the playground: explanation, setup, all executable blocks, and source identity. Otherwise a later block can lose the definitions it depends on, and a literate report turns back into a detached snippet.

**Current implementation observation:** the static enhancer still finds individual HTML code blocks and creates links that carry their text to the playground. It does not publish a complete-source, page-level lesson handoff. Its URL helper has a 7,000-character encoded-length limit; the static producer does not provide a working whole-document fallback for longer sources. Browser support for another storage path is not proof that this static link producer uses it.

The migration plan explicitly called for replacing migrated-page snippet injection with “open `.dice` in playground”. That remaining integration gap should be preserved, not erased by saying the content migration is finished.

## The API reference is generated, with editorial framing

The reference is built from Starlark documentation metadata for the registered builtins and exposed value types. The renderer adds curated introductions, topic ordering, and terminology so that it is more useful than an alphabetical dump of signatures.

`make references` invokes the native documentation command and writes `docs/references/stdlib.md`. `tests/docs_reference.rs` compares the committed file against the renderer output, runs each fenced `.dice` example independently, and checks key behavioral explanations. The script-facing Rust comments assume readers are new to scripting and Starlark: they explain arguments, returned values, defaults, and tabletop meaning rather than Rust internals.

The renderer extracts constructor metadata directly instead of trimming type documentation as text. Builtins appear once in their topic, and type methods have qualified headings such as `DieRoll.pmf`. Unit tests require exact coverage of registered APIs, parameter help, runnable examples, and a consistent heading hierarchy.

The site build regenerates the reference into the build output without changing the committed snapshot, then publishes HTML, downloadable Markdown, and the linked API conventions page. Reference signatures are not offered as runnable playground snippets.

The human-maintained [API conventions](../references/api-conventions.md) page explains stable patterns that a signature alone cannot teach:

- face matching by exact value, a list, or an inclusive band;
- conditioning with `keep`/`remove` versus converting to zero with `ignore`;
- joining pools versus summing totals; and
- the difference between a total threshold, a count of matching dice, and a rank on an outcome scale.

The detailed symbol list belongs in the [existing reference](../references/stdlib.md), not in a second hand-maintained API catalogue inside these design notes.

## LLM integration is a reference and review workflow

The recorded integration is intentionally simple:

1. Publish the model-oriented `llms.txt` reference.
2. A user gives it to an **external** chat or coding tool.
3. The user describes a mechanic in plain language.
4. The model drafts a `.dice` script or literate document.
5. The user brings the draft into the playground, reads it, resolves diagnostics, runs it, and checks its modelling choices.

This does not require an in-app LLM provider, API keys, a chat panel, automatic transmission of documents, or autonomous correction. None of those features is established by the old brief.

The reference should help a model draft the right abstraction, not merely plausible syntax. It needs to explain:

| Common source of error | Guidance to preserve |
|---|---|
| Treating a distribution as one random roll | Calculate distributions; use loops for comparisons, not simulated trials. Repeated distribution use normally assumes independence. |
| Confusing a pool with its total | Keep individual dice when faces matter; sum explicitly when only a total matters. Dice sugar is context-sensitive, so do not describe every bare `2d6` expression as the same type. |
| Applying critical rules to a modified total | Examine natural faces when the rule depends on natural 1/20 or matching faces. |
| Treating filtering as a success query | Conditioning changes the distribution; a threshold query returns a probability. |
| Treating labels as numbers | Define their order with a scale, and return declared labels from classifiers. |
| Assuming Starlark is Python | The playground disables `load`; the dice `sum` builtin is not Python list summation; pool-map callbacks return integer results. |
| Creating an invalid literate document | Explain bare/`dice` fences, display-only `text` fences, shared scope, whole-file Run, and legacy fallback. |
| Overstating unlimited explosions | State the actual cap and the distinction between a finite model and the unlimited mechanic. |

A useful user prompt identifies die sizes, whether to sum or count successes, modifiers, targets, critical exceptions, what should be compared, and what outputs to produce. The design intention is to draft complete explainable documents, not to make a model the source of probability truth.

`llms.txt` is currently maintained separately from the generated API reference. That means it can drift. Running a generated example checks syntax and execution; a human still needs to verify that the script represents the intended game rule. A successful Run alone does not establish that a natural-language instruction was interpreted correctly.

## Validation and known integration gaps

The documented programme-level completion criteria are:

- tutorials and recipes have one literate source each;
- every source evaluates unchanged in automated checks;
- static pages are generated from those sources;
- the playground presents the same document’s report;
- native and WASM paths share the language and rendering logic;
- legacy scripts retain their behaviour; and
- authoring guidance and bundle measurements reflect the new design.

Current tests include whole-corpus evaluation, selected mechanic results, generated-reference equality, literate parsing/weaving, and markdown page rendering. `make check-wasm` is a **compilation check**. A test file named `wasm_eval_smoke.rs` is not by itself evidence of execution in an actual browser.

Several observations still need follow-up rather than a false “complete” label:

1. **Whole-document opening:** snippet links do not establish the intended complete-source handoff.
2. **Internal copy drift:** some `.dice` lessons repeat a display-only `text` example in an executable block later. Evaluation tests do not prove those two in-file copies stay equal.
3. **Published links:** some user-guide/reference links target markdown files not emitted by the current static build. Rendering smoke tests do not crawl those destinations.
4. **Reference consistency:** shorthand descriptions of dice notation in `llms.txt` and reference introductions should be checked against actual pool-versus-total semantics.
5. **LSP and source locations:** document-aware editor support is not proved by successful CLI rendering.

These are scoped implementation observations, not an instruction to redesign the product or repair them during this documentation extraction. They are carried into the [review notes](../requirements/review-notes.md#implementation-observations).

---

**Basis:** [source map](source-map.md), S01–S06 and S13; R03–R05 for the current corpus, reference renderer, static build, handoff, tests, and LLM instructions.
