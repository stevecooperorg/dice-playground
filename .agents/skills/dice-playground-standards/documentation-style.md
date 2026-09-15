# Documentation style (reference)

## Audience

For the Rust engine API, assume the reader can program in Rust but may not know probability concepts such as probability mass functions (PMF), convolution, or independence. Bridge that gap before using precise terms.

For the **script-facing Rust doc comments** in `src/engine/starlark_guest/`, assume the reader knows their tabletop rule but is **new to scripting and Starlark**. These comments become the public script reference, not Rust implementation documentation. Match the tutorials' voice: describe what happens at the table, explain the operation, and show how to read its result.

## Rust doc example pattern

```rust
/// Combines two independent numeric roll distributions by adding outcomes.
///
/// Mathematically this is the **convolution** of the two PMFs: each pair of
/// faces from `left` and `right` is summed, and probabilities multiply.
///
/// # Example
///
/// ```
/// use dice_playground::engine::DieRoll;
/// let a = DieRoll::die(6).unwrap();
/// let b = DieRoll::die(6).unwrap();
/// let sum = a.convolve(&b).unwrap(); // same distribution as `2d6`
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn example() {}
```

## User-facing reference pattern

1. **Core ideas** — `DieRoll` vs `DicePool` vs `Outcomes` in plain language, with `Scale` for the ladder of labels.
2. **At the table** — what the rule does before introducing technical terms.
3. **How to use it** — arguments, returned value, important defaults and limits, and common confusions.
4. **Try it** — a complete fenced `dice` example that runs independently and calls `output`. Explain unfamiliar syntax such as lists, method calls, or callback functions. Rust examples belong in the core Rust API, not in script-facing entries.

Use the real parameter names under `# Arguments` so Starlark can attach their explanations to the generated signatures. Distinguish a probability (0–1) from a count or average, and do not display the latter as a probability.

Regenerate the function reference with `make references` → `docs/references/stdlib.md`. Run `cargo test --test docs_reference` to check drift and execute the examples. Add new APIs to the curated lists in `src/engine/starlark_guest/docs.rs`; unit tests enforce exact coverage, argument help, and one runnable example per entry.

## What to avoid

- Dumping formulas without saying what they mean for a die roll.
- Hiding behavior in UI that belongs in `src/engine/`.
- Public APIs without examples or without `anyhow::Result` error context on fallible operations.
