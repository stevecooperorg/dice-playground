# Documentation style (reference)

## Audience

Assume the reader can program in Rust or Starlark but may not know:

- Probability mass functions (PMF), convolution, independence
- Tabletop notation (`2d6`, `4d6dl1`, keep-highest pools)

Bridge that gap in one or two sentences, then use precise terms.

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

1. **Core ideas** — `Dist` vs `RollPool` vs `LabelDist` in plain language.
2. **At-the-table table** — what the player experiences.
3. **API list** — function names last, grouped by topic.

Regenerate function reference: `make references` → `docs/references/stdlib.md`.

## What to avoid

- Dumping formulas without saying what they mean for a die roll.
- Hiding behavior in UI that belongs in `src/engine/`.
- Public APIs without examples or without `anyhow::Result` error context on fallible operations.
