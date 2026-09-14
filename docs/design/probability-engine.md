# The probability engine

**Mathematical approach and current implementation — approved.** [Design index](README.md)

The planning material asks for exact tabletop probabilities but largely assumes the core engine already exists. This document therefore explains the mathematics using the existing engine and tests as additional sources. Approving this explanation does not change the engine’s numerical model.

## 1. A roll value represents all its possibilities

At the table, rolling a d6 gives one number. In Dice Playground, `d(6)` represents **all six possible numbers and their chances**. It does not choose one face.

This is a *probability mass function*, or PMF:

```text
p_X(x) = P(X = x)
p_X(x) ≥ 0
Σ_x p_X(x) = 1
```

Here `X` means a random result and `p_X(x)` means its chance of being the particular value `x`. A valid distribution assigns nonnegative probabilities whose total is one.

For a fair die with `s` sides, each face from 1 through `s` has probability `1/s`. A custom face list can contain repeated values: `[1, 2, 2, 3]` gives 2 a probability of one half, not one third. A fixed modifier can be represented by a distribution with probability one at a single value.

The implementation’s numeric distribution, **`DieRoll`**, stores an ordered map from integer outcomes to probabilities: `BTreeMap<i64, f64>`. Different ways to obtain the same number accumulate into that number’s entry. Missing outcomes have probability zero; negative outcomes are possible.

### The main mathematical objects

| Object | What it preserves | Typical use |
|---|---|---|
| `DieRoll` | A PMF over integer results. | Totals, damage, success counts, threshold checks. |
| `DicePool` | A list of independent die distributions before reduction. | Keep/drop, highest die, count matching faces, custom face-sensitive rules. |
| `Scale` | An ordered list of unique labels, optionally with numeric bands. | Define what “partial success or better” means. |
| `Outcomes` | A PMF over a `Scale`’s labels. | Failure/partial/full or critical outcome categories. |
| Scalar probability | One number between zero and one. | Chance of meeting a target. |
| `prob_table` | Labelled probabilities for separate questions. | Compare bonuses and targets. Rows are not one PMF and need not sum to one. |

Keeping a pool distinct from its sum matters: after summing, the result no longer says which die showed a 1 or whether two dice showed 6.

## 2. What “exact” means here

There are three separate issues:

1. **Enumeration rather than simulation.** Finite possibilities are combined deterministically, by enumeration, convolution, or an equivalent formula. Production evaluation does not estimate odds from random trials.
2. **Finite numerical precision.** Probabilities are `f64` floating-point values, not arbitrary-precision rational numbers. Arithmetic and accumulation can introduce rounding error.
3. **Finite models of unbounded rules.** Explosions and open-ended rerolls are capped. Their results describe the capped procedure, not a complete infinite distribution.

The honest product claim is therefore **deterministic calculation of finite distributions, rather than Monte Carlo estimates**, with numerical precision and model limits made explicit.

Normalization is used in the core, with tolerances that vary by path; this is not a proof of symbolic exactness. The raw `from_mass` constructor also does not validate arbitrary input maps. The PMF equations above are mathematical invariants for valid inputs and tests, not a guarantee that every low-level constructor enforces them.

Fractions and “out of N” columns are display reconstructions from floating-point probabilities. The current formatter searches convenient denominators up to 10,000 using a tolerance; it does not retain an exact rational history or the original count of equally likely trials through every operation.

## 3. Independent rolls combine by convolution

Two independent results have joint probability equal to the product of their separate probabilities:

```text
P(X = x, Y = y) = p_X(x) p_Y(y)
```

To add the rolls, combine every pair and add together the probability of pairs with the same total:

```text
p_(X+Y)(z) = Σ_(x+y=z) p_X(x) p_Y(y)
```

This operation is called **convolution**. Two fair d6s have 36 equally likely ordered pairs but only 11 possible totals. Six pairs make 7, so `P(2d6 = 7) = 6/36`; the totals are not uniformly distributed.

`DieRoll::convolve` implements this multiplication and accumulation. Independent subtraction uses the same idea with `x - y`. Repeated convolution can calculate a sum without carrying every complete face tuple through subsequent operations.

### Reusing a distribution is not reusing one rolled value

For `r = d(6)`:

| Expression | Mathematical meaning |
|---|---|
| `r + r` | Add two independent draws from the same distribution. This is ordinary 2d6. |
| `r - r` | Subtract independent draws. Zero has probability `1/6`; it is not guaranteed. |
| `r * 2` | Double each possible result of one die. Only 2, 4, 6, 8, 10, and 12 occur. |

There is no dependency graph that remembers a shared sampled roll merely because two expressions use the same variable.

For a rule that needs the **same faces** in several tests, use one joint mapping or classification operation. For example, a pool callback can inspect the same tuple for both “two sixes” and “total at least ten”. Once only a marginal distribution has been returned, its relationship to a separately calculated output has not been retained.

## 4. Queries and transformations are different operations

A query asks a question without changing the distribution:

```text
pmf(t)  = P(X = t)
cdf(t)  = P(X ≤ t)
p_ge(t) = P(X ≥ t)
E[X]    = Σ_x x p_X(x)
```

`E[X]` is the mean: the long-run average, not necessarily a possible result. Threshold queries are inclusive. A d20 plus 5 meets DC 20 when the natural die is at least 15: `6/20 = 30%`.

A deterministic transformation changes results while preserving their associated probability:

```text
P(f(X) = y) = Σ_(x : f(x)=y) p_X(x)
```

Adding a bonus shifts every result. Scaling, clamping, converting selected faces, and floor-dividing damage all move or merge probability mass. For half damage rounded down:

```text
P(floor(X/2) = k) = P(X = 2k) + P(X = 2k+1)
```

The implementation uses floor division, including for negative values; `-7 // 2` is `-4`.

### Conditioning is not ignoring a failed die

`keep(A)` means “consider only results in set A, and rescale their probabilities to total one”:

```text
P(X = x | X in A) = p_X(x) / P(X in A), for x in A
```

`remove(A)` conditions on the complement. An empty retained distribution is an error. `ignore(A)` instead converts matching outcomes to zero; it does not throw away those trials.

For a fair d6:

| Operation | Result |
|---|---|
| Keep only 5 and 6 | 5 and 6 each have probability `1/2`. |
| Change 1–4 to zero | Zero has probability `4/6`; 5 and 6 each retain `1/6`. |
| Ask for the chance of 5 or more | Returns `1/3`; the die itself is unchanged. |

On a pool these face transformations apply to each constituent die. Conditioning each die before summing is generally different from conditioning the final total. This distinction deserves prominent teaching because natural-language phrases such as “only high dice count” can mean different mechanics.

## 5. Face-sensitive pool rules enumerate joint outcomes

For independent pool dice `X_1` through `X_n`, a complete tuple has probability:

```text
P(X_1=x_1, …, X_n=x_n) = ∏_i p_i(x_i)
```

A rule `g` converts each tuple to a result:

```text
P(g(X_1, …, X_n) = z) = Σ_(tuples mapped to z) ∏_i p_i(x_i)
```

This supports mixed die sizes and weighted faces as well as identical fair dice. Custom mappings receive the tuple in stored die order. Operations such as keep-highest explicitly sort a copy; a callback must not assume its input is already sorted.

For **4d6 drop lowest**, there are `6^4 = 1296` tuples. Drop the lowest face in each, sum the other three, and accumulate equal totals. Results range from 3 to 18, with mean approximately 12.2446.

For **advantage on a d20**, use the highest of two dice. The chance of reaching 15 is:

```text
1 - P(both dice are below 15)
= 1 - (14/20)^2
= 51%
```

This changes the shape of the distribution; it is not equivalent to a fixed bonus. Order statistics generalise this idea: `k = 1` selects the highest die, `k = 2` the second-highest, and so on.

### Counting successes can have a faster formula

If `A` is the set of successful faces, the number of successes is:

```text
C = Σ_i 1_(X_i in A)
```

For `n` identical independent fair dice with per-die success probability `q`, this is binomial:

```text
P(C = k) = choose(n, k) q^k (1-q)^(n-k)
```

The engine uses this formula for identical fair dice and joint enumeration for other pool shapes. For three d6s succeeding on 5 or 6, probabilities for 0, 1, 2, and 3 successes are `(8, 12, 6, 1)/27`; the chance of at least two is `7/27`.

Pool `p_any`, `p_none`, and `p_at_least` query matching-die counts. Omitting their face specification instead asks about the deterministic pool length. These are not the same operations as querying a total or an ordered label.

## 6. Named results are a distribution over an ordered scale

A classification function maps numeric results to labels:

```text
P(L = label) = Σ_(x classified as label) p_X(x)
```

For a PbtA-style 2d6 move, with miss at 6 or less, partial success at 7–9, and full success at 10 or more:

| Outcome | Probability |
|---|---|
| Miss | `15/36` |
| Partial | `15/36` |
| Full | `6/36` |

“Partial or better” is `21/36`. The scale’s declaration order defines “better”; it is not alphabetical ordering. Labels with zero mass can still appear in ordered output.

Numeric bucketing uses inclusive bands. The implementation checks bounded `early` steps first, then other bounded steps in declaration order; the first match wins. Positive-probability outcomes left uncovered cause an error. Fully unbounded label-only steps do not serve as numeric catch-all bands.

Use classification callbacks for irregular rules. A rule about natural 1 and natural 20 must examine the natural die, not assume that a modified total retains that metadata. `joint_classify` similarly combines two independent input distributions before applying the classification rule.

## 7. Exploding dice need an explicit stopping rule

An ordinary exploding die rerolls its maximum result and adds the new roll. An unlimited version can continue indefinitely. A finite PMF implementation therefore needs a finite stopping rule.

`explode(dist, max_depth)` permits at most that many **extra** rolls; the Starlark default is 2. At the cap it keeps the result already rolled and stops, even on a maximum. It does not discard the remaining probability mass.

For a d6 with one extra roll allowed:

- 1–5 each have probability `1/6`;
- 7–12 each have probability `1/36`;
- 6 is impossible; and
- the second 6 stops at 12, whereas an unlimited explosion would continue.

For an ordinary positive die whose maximum has probability `q < 1`, a depth-`D` cap interrupts a continuing chain with probability `q^(D+1)`. That gives a bound on the probability of disagreement with the unlimited rule when both use the same roll sequence. This is a mathematical explanation, **not an error estimate currently returned by the API**.

If the base mean is `μ`, the capped and unlimited means are:

```text
E[S_D] = μ (1 + q + … + q^D)
E[S_infinite] = μ / (1-q)
```

For the one-extra-roll d6, these are `49/12` and `21/5` respectively. A cap can matter even when the omitted continuation event is rare.

Exploding a summed distribution is also different from exploding each die: exploding a 2d6 total triggers on total 12 and rerolls another whole 2d6 total.

### Other supported open-ended mechanics

- **Rolemaster-style d100:** initial 1–5 subtracts a reroll chain; initial 96–100 adds one; within either chain, only 96–100 continues. `max_chain` caps subsequent continuation; its default is 8. Zero still permits the first reroll after the initial trigger.
- **Specialised `success_pool`:** an even face **or the maximum face** counts as a success, and maxima generate more dice. Modes retain successes, subtract ones, let a one suppress that wave’s explosions, or roll non-recursive penalty dice for ones. The implementation stops after at most 48 waves and clamps final successes at zero.

The specialised helper is not a generic configurable success-threshold system. Its game-system nickname must not replace the actual rule description. Nor is a 48-wave tail automatically negligible for every accepted input: a one-sided exploding die does not terminate in the unlimited model.

## 8. Cost and safeguards

Two distributions with `m` and `n` stored outcomes require `m × n` pair visits for convolution, plus map accumulation. A general pool with support sizes `s_1 … s_n` can require `∏ s_i` tuple visits. Keep/drop adds sorting work; callbacks add their own evaluation cost.

The implementation uses repeated convolution for identical fair-die sums and a binomial shortcut for their success counts. It does not apply every possible mathematical optimisation to every pool shape.

`MAX_JOINT_CELLS` is currently **1,000,000** for relevant pool-enumeration paths. General enumeration checks support products for overflow. This is **not a universal execution or memory bound**: convolution, direct joint classification, specialised exploding pools, and some fast paths have different behaviour. The uniform enumerator’s exponentiation also warrants overflow review. There is no automatic fallback to simulation in the inspected paths.

The source-size and output-count limits belong to the surrounding [application architecture](architecture.md). They should not be described as a complete protection against expensive calculations.

## 9. Test mathematical meaning, not just successful execution

Useful checks include:

- finite, nonnegative masses and total probability near one for valid distributions;
- known answers for fair dice, 2d6, advantage, and 4d6 drop lowest;
- `P(X ≤ k) + P(X ≥ k+1) = 1`;
- a bonus shifting thresholds consistently;
- agreement between equivalent formulations, such as highest-of-pool and keep-one;
- mass merging under floor division, conversion, and clamping;
- conditioning remaining distinct from zero conversion;
- label ordering, coverage, and special-case precedence;
- agreement between core operations and their Starlark bindings; and
- explicit checks of capped explosion behaviour, not merely its mean.

The existing `total_variation_distance` helper compares two valid PMFs by `½ Σ_x |p(x)-q(x)|`. This detects distributional differences that matching means can hide.

Existing unit tests cover many of these identities. The specialised exploding-pool tests also use seeded simulation as a sanity check; that does not turn production evaluation into Monte Carlo or prove agreement with an unlimited rule.

---

**Implementation sources:** [source map](source-map.md), R01–R03: `die_roll.rs`, `dice_pool.rs`, `enumerate.rs`, `ordinal.rs`, `poly_explode.rs`, `core.rs`, Starlark bindings, output formatting, and API conventions. Small probability examples and the explosion bound above are mathematical derivations from those semantics.
