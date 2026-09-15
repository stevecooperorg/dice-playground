---
title: "Function reference"
author: Steve Cooper
---

# Function reference

This is the **function and type reference** for people writing `.dice` scripts. It assumes you know your tabletop rule, but may be new to scripting and Starlark. Each entry explains what the operation does, what to pass in, what comes back, and gives a complete example to run in the playground.

| File | Contents |
|------|----------|
| [stdlib.md](stdlib.md) | Builtins and methods on `DieRoll`, `DicePool`, `Outcomes`, and `Scale`, generated from script-facing Rust doc comments |
| [api-conventions.md](api-conventions.md) | Shared patterns for face matching, counts, and probabilities |

Do **not** edit `stdlib.md` by hand. Regenerate from the repo root:

```bash
make references
```

For contributors: edit the doc comments in `src/engine/starlark_guest/eval.rs` (builtins), or `die_roll_value.rs`, `dice_pool_value.rs`, `outcomes_value.rs`, and `scale_value.rs` (methods). These comments address **script writers**, not Rust developers.

- Start with what happens at the table; explain unfamiliar scripting terms when needed.
- Explain the returned value and important defaults, limits, or errors.
- Document each argument under `# Arguments`, using its actual parameter name.
- Include a fenced `dice` example that runs on its own and calls `output`. Do not rely on setup from another entry.
- If adding an API, update its topic or method list in `src/engine/starlark_guest/docs.rs`. Tests require every registered API to appear exactly once.

Run `make references`, then `cargo test --test docs_reference` and commit the regenerated Markdown. Tests check the snapshot, run every reference example, and verify key behavioral explanations. Unit tests also enforce API coverage, argument help, and heading structure.

The website build generates a fresh reference into its output directory, without modifying the checked-in Markdown. It publishes the reference as HTML (also at `/references/`) and downloadable Markdown, plus the API conventions page. Signatures describe calls; only runnable examples receive playground links.

For learning the language in order, use the [tutorial](../README.md#tutorial). For ready-made mechanics, see the [cookbook](../cookbook/README.md).
