# Dice standard library

This reference helps you turn a tabletop rule into a `.dice` script. You do not need to know Rust or Starlark to use the examples. For a first walk-through, start with the [tutorial](../tutorial/index.html).

The playground calculates possible results and their chances; it does not pick one random roll. Each `dice` example below is a complete script you can copy into the playground and run on its own.

## Reading the examples

```dice
roll = 2d6 + 3
output("My roll", roll)
output("Chance of ten or more", roll.p_ge(10))
```

- In `2d6 + 3`, the two six-sided dice are added together and get a +3 bonus.
- `roll = ...` gives a calculation a name. Later lines can use that name.
- `output(...)` shows a result. The text in quotes is a title you choose.
- Values inside parentheses are **arguments**: information an operation needs. Commas separate them.
- `roll.p_ge(10)` calls a **method**, an operation on the value before the dot. Here it asks whether the total reaches 10.
- Lines starting with `#` are comments for you to read; they do not change the calculation.

The `def ... -> ...` lines in this reference describe how to call an operation; they are **signatures**, not scripts to paste into the playground. A signature such as `d(sides: int) -> DieRoll` says that `d` takes a whole number and returns a numeric roll. You write `d(6)`, not the type names.

| Signature notation | What it tells you |
|--------------------|-------------------|
| `int` | A whole number, such as `6`. |
| `float` | A number that can have a decimal part. Probabilities use 0–1: `0.5` means 50%. |
| `str` | Text in quotes, such as `"Hit"`. |
| `list[int]` | Whole numbers in square brackets, such as `[1, 2, 2, 3]`. |
| `name=2` | You may leave this argument out; its default is 2. |
| `*args`, `*spec`, or `*bands` | Extra arguments are accepted; the entry explains which ones. Do not type the star in an ordinary call. |
| `/` or a bare `*` | Arguments before `/` are positional; arguments after a bare `*` must be named, such as `early=True`. These markers are not values to pass. |
| `-> Type` | The kind of value the operation gives back. |

## Core ideas

| Value | Meaning at the table | What to do with it |
|-------|----------------------|--------------------|
| `DieRoll` | A numeric roll or total, with a chance for every possible result. | Show it with `output`, add a modifier, or ask for the chance of reaching a target. |
| `DicePool` | Several dice whose individual faces still matter. | Count successes, choose a die, or call `.sum()` when you need the total. |
| `Scale` | A ladder of labels, from worst to best, optionally with number ranges. | Build it with `scale().step(...)`, then use it to label a roll. |
| `Outcomes` | Chances for named results such as miss, partial, and hit. | Show the results or ask for “partial or better”. |
| `IntBand` | A range describing which whole numbers match. | Use it in face matching or a scale; it is not a roll or a probability itself. |
| `ProbTable` | Several labeled probability questions in one table. | Show it with `output`. |

Operations such as `.keep(...)` and `.step(...)` return new values rather than changing the old ones. Keep the new value by giving it a name or passing it to another operation.

## Combining numeric rolls

| You write | Meaning at the table |
|-----------|----------------------|
| `a + b` | Add two independent numeric rolls. Even `roll + roll` means two independent rolls, not doubling the same face. |
| `roll + 5` | Add a flat bonus to every result. |
| `roll - 3` | Subtract a penalty from every result. |
| `a - b` | Subtract one independent numeric roll from another. |
| `roll * 10` | Multiply each result by 10. The multiplier must be a positive whole number. |
| `roll // 2` | Divide each result by 2 and round down. The divisor must be a positive whole number. |

A bare assignment such as `pool = 2d6` keeps the dice as a `DicePool`. To store a finished total, write `roll = dice_pool(2, 6).sum()`. Dice notation is shortened according to context; explicit `.sum()` makes your intent clear.

Pools have their own joining rules: `pool + pool` puts dice into a larger pool rather than adding their faces. See [API conventions](api-conventions.md) for face matching, pool counts, and the difference between `keep` and `ignore`.

Dice notation such as `4d6dl1` is a shorter way to call some of the functions below. See the [dice notation lesson](../tutorial/05-dice-notation.md).

## Building dice and totals

Start here for ordinary dice, custom faces, and totals that keep or drop dice.

### d

```python
def d(sides: int) -> DieRoll
```

Make one fair die: every face has the same chance.

#### Parameters

* `sides`: (required)

  Number of faces, from 1 upwards. Faces are numbered 1 through this number.



#### Details

`d(6)` means a six-sided die, just like `1d6` in a `.dice` script.
It returns a `DieRoll`: all possible results and their chances, not one random roll.
Give that result to `output` to see its table.

```dice
output("One six-sided die", d(6))
```

### die\_faces

```python
def die_faces(faces: list[int]) -> DieRoll
```

Make a die with numbers of your choosing on its faces.

#### Parameters

* `faces`: (required)

  A non-empty list of whole numbers. Zero, negative numbers, and repeats are allowed.



#### Details

Write the numbers inside square brackets, separated by commas. This is a **list**.
Each entry is one equally likely face, so repeating a number makes it more likely.
The example has four faces: 2 is twice as likely as either 1 or 3.
The result is a `DieRoll` you can add to other rolls or show with `output`.

```dice
output("Custom die", die_faces([1, 2, 2, 3]))
```

### dice\_pool

```python
def dice_pool(count: int, sides: int) -> DicePool
```

Make several dice that you can examine separately before adding them.

#### Parameters

* `count`: (required)

  Number of dice, at least 1.

* `sides`: (required)

  Number of faces on each die, at least 1. Every die has equally likely faces numbered 1 through this number.



#### Details

The result is a `DicePool`. Use a pool when your rule counts successes or
chooses the highest die. The dice have not been added together yet.
Write `.sum()` after the pool to turn it into a `DieRoll` that `output` can show.

```dice
pool = dice_pool(3, 6)
output("Total of three dice", pool.sum())
output("Number of sixes", pool.count(6))
```

### sum

```python
def sum(value) -> DieRoll
```

Add the dice in a pool to get the chances for each possible total.

#### Parameters

* `value`: (required)

  A `DicePool` to total, or a `DieRoll` to return unchanged.



#### Details

The result is a `DieRoll`; for four d6, the totals run from 4 to 24.
You can also write `pool.sum()`: the dot means “use this operation on this pool”.
This dice helper does not add an ordinary list of numbers.

```dice
pool = dice_pool(4, 6)
output("Four dice added together", sum(pool))
```

### drop\_lowest

```python
def drop_lowest(count: int, sides: int, drop: int) -> DieRoll
```

Leave out the lowest dice, then add the rest.

#### Parameters

* `count`: (required)

  Number of dice to roll, at least 1.

* `sides`: (required)

  Faces per die, at least 1.

* `drop`: (required)

  Number of lowest dice to leave out, 0 or more. A non-empty pool always keeps at least one die, even if you ask to drop them all.



#### Details

For the familiar ability-score rule, roll four d6 and leave out the lowest one.
The result is a `DieRoll` of totals from 3 to 18, just like `4d6dl1`.
Tied low dice are still separate dice; dropping one removes only one of them.

```dice
output("Ability score", drop_lowest(4, 6, 1))
```

### drop\_highest

```python
def drop_highest(count: int, sides: int, drop: int) -> DieRoll
```

Leave out the highest dice, then add the rest.

#### Parameters

* `count`: (required)

  Number of dice to roll, at least 1.

* `sides`: (required)

  Faces per die, at least 1.

* `drop`: (required)

  Number of highest dice to leave out, 0 or more. A non-empty pool always keeps at least one die, even if you ask to drop them all.



#### Details

For example, roll four d6, remove the highest one, and add the other three.
The result is a `DieRoll` of totals, also written `4d6dh1`.
If several dice tie for highest, each still counts as a separate die.

```dice
output("Four d6, leave out the highest", drop_highest(4, 6, 1))
```

### keep\_highest

```python
def keep_highest(count: int, sides: int, keep: int) -> DieRoll
```

Choose the highest dice and add them, leaving the others out.

#### Parameters

* `count`: (required)

  Number of dice to roll; use at least 1.

* `sides`: (required)

  Faces per die, at least 1.

* `keep`: (required)

  Number of highest dice to add, 0 or more. Keeping none gives 0; asking for more than `count` keeps all the dice.



#### Details

Keeping three of four d6 is the same as dropping the lowest one.
The result is a `DieRoll` of totals, also written `4d6kh3` in dice notation.
Keeping just one of two d20 models advantage.

```dice
output("Advantage", keep_highest(2, 20, 1))
```

### keep\_lowest

```python
def keep_lowest(count: int, sides: int, keep: int) -> DieRoll
```

Choose the lowest dice and add them, leaving the others out.

#### Parameters

* `count`: (required)

  Number of dice to roll; use at least 1.

* `sides`: (required)

  Faces per die, at least 1.

* `keep`: (required)

  Number of lowest dice to add, 0 or more. Keeping none gives 0; asking for more than `count` keeps all the dice.



#### Details

Keeping one of two d20 models disadvantage. The result is a `DieRoll`
of the kept total. In dice notation, this example is `2d20kl1`.

```dice
output("Disadvantage", keep_lowest(2, 20, 1))
```

### explode

```python
def explode(dist: DieRoll, max_depth: int = 2) -> DieRoll
```

Roll again on the highest result and add the extra roll to the total.

#### Parameters

* `dist`: (required)

  The roll to repeat. Usually one die, such as `d(6)`. For a total, only its highest possible total triggers another roll of that whole total.

* `max_depth`: (defaults to: `2`)

  Maximum extra rolls, 0 or more. Defaults to 2 when omitted; 0 leaves the roll unchanged.



#### Details

This is an **exploding die**. On a d6, a 6 earns another d6; another 6 can
earn another roll. The result is a `DieRoll` of totals, including those above 6.
To keep the calculation finite, extra rolls stop at `max_depth` even if the last die is a 6.
This models a capped rule, not an unlimited chain.

```dice
output("Exploding d6, at most two extra rolls", explode(d(6), max_depth=2))
```

### open\_ended\_d100

```python
def open_ended_d100(max_chain: int = 8) -> DieRoll
```

Make a Rolemaster-style d100 roll that can finish below 1 or above 100.

#### Parameters

* `max_chain`: (defaults to: `8`)

  Maximum additional high-result continuations after the first extra roll. Defaults to 8; must be 0 or more. Even 0 allows the first extra roll. The cap makes this a finite version of the rule.



#### Details

A first roll of 6–95 stands as it is. On 1–5, roll again and subtract;
on 96–100, roll again and add. Further rolls continue only on 96–100,
keeping the same subtracting or adding direction. A printed 00 means 100 here.
The result is a `DieRoll` of final totals.

```dice
output("Open-ended d100", open_ended_d100())
```

### shift

```python
def shift(dist: DieRoll, delta: int) -> DieRoll
```

Add the same bonus or penalty to every possible total.

#### Parameters

* `dist`: (required)

  A numeric roll or total.

* `delta`: (required)

  Whole-number modifier. Use a negative number for a penalty.



#### Details

The returned `DieRoll` keeps the same chances but moves the numbers.
Adding 3 to a d20 changes its results from 1–20 to 4–23.
Usually `roll + 3` is the simplest way to write this; `shift(roll, 3)` does the same thing.

```dice
output("d20 with a +3 bonus", shift(d(20), 3))
```

## Inclusive ranges

Describe which numbers match. Both endpoints count; these ranges are not rolls by themselves.

### through

```python
def through(lo: int, hi: int) -> IntBand
```

Describe a range of whole numbers, including both ends.

#### Parameters

* `lo`: (required)

  Lowest number to include.

* `hi`: (required)

  Highest number to include; must be at least `lo`.



#### Details

`through(5, 6)` means “5 or 6”, also written `5..6` in a `.dice` script.
It returns an `IntBand`, a range for matching faces or defining outcome bands.
It does not roll a die or calculate a chance on its own.

```dice
output("Number of 5s and 6s", dice_pool(3, 6).count(through(5, 6)))
```

### at\_most

```python
def at_most(hi: int) -> IntBand
```

Describe “this number or lower”, including the number itself.

#### Parameters

* `hi`: (required)

  Highest number to include.



#### Details

`at_most(2)` matches 2, 1, 0, and every lower whole number.
It returns an `IntBand` range, also written `..2` in a `.dice` script.
Use it to match faces or define outcome bands, not to ask for a probability directly.

```dice
output("Number of low dice", dice_pool(3, 6).count(at_most(2)))
```

### at\_least

```python
def at_least(lo: int) -> IntBand
```

Describe “this number or higher”, including the number itself.

#### Parameters

* `lo`: (required)

  Lowest number to include.



#### Details

`at_least(5)` matches 5, 6, 7, and every higher whole number.
It returns an `IntBand` range, also written `5..` in a `.dice` script.
This describes which faces match; `count` or a probability method works out their chances.

```dice
output("Number of successes on 5+", dice_pool(3, 6).count(at_least(5)))
```

## Pool rules (faces still matter)

Use these when a rule examines individual dice rather than just the total.

### count

```python
def count(pool: DicePool, spec) -> DieRoll
```

Count how many dice show a matching face, rather than adding their values.

#### Parameters

* `pool`: (required)

  Dice made with `dice_pool`.

* `spec`: (required)

  Faces that count: one number such as `6`, a non-empty list such as `[5, 6]`, or a range such as `at_least(5)` (also written `5..` in `.dice` scripts).



#### Details

If each 5 or 6 is a success, three dice can give 0, 1, 2, or 3 successes.
The returned `DieRoll` gives the chance of each count. `pool.count(spec)`
is another way to write the same operation.

```dice
pool = dice_pool(3, 6)
output("Successes on 5 or 6", count(pool, [5, 6]))
```

### order\_stat

```python
def order_stat(pool: DicePool, k: int) -> DieRoll
```

Choose one die by its position from highest to lowest.

#### Parameters

* `pool`: (required)

  The dice to compare.

* `k`: (required)

  Position counting from 1 at the top; cannot exceed the number of dice.



#### Details

Use 1 for the highest die, 2 for the second-highest, and so on.
Ties still occupy separate positions: in 6, 6, 2 the second-highest is 6.
The result is a `DieRoll` of the chosen value, not a sum. You can also write `pool.order_stat(k)`.

```dice
output("Highest of three d6", order_stat(dice_pool(3, 6), 1))
```

### middle\_of

```python
def middle_of(pool: DicePool, keep: int) -> DieRoll
```

Add the middle dice, leaving out the lowest and highest results.

#### Parameters

* `pool`: (required)

  The dice to sort and choose from.

* `keep`: (required)

  How many middle dice to add, from 1 to the number of dice.



#### Details

With three dice and `keep=1`, this chooses the middle die. With five dice
and `keep=3`, it adds the middle three. The result is a `DieRoll` of totals.
If the number left out is odd, one more die is removed from the high end
than from the low end. You can also write `pool.middle_of(keep)`.

```dice
output("Middle of three d6", middle_of(dice_pool(3, 6), 1))
```

### pool\_map

```python
def pool_map(pool: DicePool, map_fn) -> DieRoll
```

Work out the chances for a custom rule that looks at all the dice together.

#### Parameters

* `pool`: (required)

  The dice your rule examines.

* `map_fn`: (required)

  Your function's name, without calling it with parentheses. It receives a list of faces in pool order (not sorted) and must return a whole number.



#### Details

Try helpers such as `count` and `order_stat` first. When they do not fit,
write a small function: `def` names your rule, and `return` supplies its answer.
The engine calls it for every possible combination and collects the answers
into a `DieRoll`. Large pools can take a long time because every combination is checked.

Here doubles score 2 and everything else scores 0. Square brackets pick a
die from the list: `[0]` is the first, `[1]` the second. `==` asks whether they are equal.
Keep the indentation shown: it marks which lines belong to the rule and the `if`.

```dice
def score_doubles(faces):
    if faces[0] == faces[1]:
        return 2
    return 0

output("Doubles score", pool_map(dice_pool(2, 6), score_doubles))
```

### success\_pool

```python
def success_pool(count: int, sides: int, mode: str = "baseline") -> DieRoll
```

Count successes for a specific rule: even faces and the highest face succeed; highest faces also roll again.

#### Parameters

* `count`: (required)

  Starting number of dice, 0 or more.

* `sides`: (required)

  Number of faces on each die, at least 1.

* `mode`: (defaults to: `"baseline"`)

  One of the quoted names in the table. Defaults to `"baseline"`.



#### Details

On a d6, 2, 4, and 6 each earn one success, and each 6 adds another d6.
On a d5, 2, 4, and 5 succeed, and each 5 adds another d5.
The result is a `DieRoll` of success counts, never below 0.
This is not a general target-number system: for “5 or higher succeeds”, use `count` instead.

| Mode | What happens to rolled 1s? |
|------|---------------------------|
| `"baseline"` | Nothing extra. |
| `"ones_cancel"` | Any 1 stops all extra dice earned in that round; successes still count. |
| `"ones_remove"` | Each 1 subtracts one success from the final count. |
| `"implode"` | Each 1 earns a penalty roll of the same die; an odd result subtracts one success. Penalty rolls do not trigger more rolls. |

Extra dice are processed in rounds, capped at 48 rounds including the first.
This is a finite approximation to unlimited explosions. Start with small pools.

```dice
output("Even faces succeed, sixes explode", success_pool(1, 6))
```

## Named outcomes

Turn numbers or custom rules into labels such as miss, partial, and hit.

### scale

```python
def scale() -> Scale
```

Start a ladder of named results, such as miss, partial success, and full success.

`scale()` takes no arguments and returns an empty `Scale`. Add labels from
worst to best with `.step(...)`. Each step returns the longer ladder, so
you can write one step after another. Quotes mark labels as text.
A scale describes the rule; use `bucket` to find its chances for a roll.

```dice
results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
output("Move result", bucket(2d6, results))
```

You can omit the number ranges when using `classify` to choose labels yourself,
or when supplying the ranges separately to `bucket`.

### bucket

```python
def bucket(dist: DieRoll, scale: Scale, *spec) -> Outcomes
```

Group numeric totals into named results, such as miss, partial success, and hit.

#### Parameters

* `dist`: (required)

  The numeric roll or total to label.

* `scale`: (required)

  Your ladder of labels, made with `scale()` and `.step(...)`.

* `*spec`: (required)

  Optional replacement ranges. Supply one fewer cut point than labels, in increasing order (as a list or separate numbers), or one range per label as separate arguments, such as `at_most(6), through(7, 9), at_least(10)`.



#### Details

First build a scale with a number range for each label. `bucket` then returns
`Outcomes`: the chance of each label, rather than each individual total.
You can also write `roll.bucket(results)`.

```dice
results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
output("Move result", bucket(2d6, results))
```

Every possible total must match a band, or the script reports an error.
If bands overlap, steps marked `early=True` are checked first, then the others;
within each group, the first matching step wins.

You can instead supply cut points: for three labels, `[6, 9]` means 6 or less,
7–9, and 10 or more. These replace any ranges already on the scale.

```dice
results = scale().step("Miss").step("Partial").step("Hit")
output("Move result using cut points", bucket(2d6, results, [6, 9]))
```

### classify

```python
def classify(dist: DieRoll, scale: Scale, classify) -> Outcomes
```

Choose a label for each possible roll using a rule you write yourself.

#### Parameters

* `dist`: (required)

  The numeric roll your rule examines.

* `scale`: (required)

  All allowed labels, in order from worst to best.

* `classify`: (required)

  Your function's name, without parentheses. It receives one whole number and must return a label on the scale, spelled exactly the same way.



#### Details

Use `bucket` for simple number bands. Use `classify` when a special result,
such as a natural 20, needs its own rule. The returned `Outcomes` contains
the chance of each label. Your rule sees the value of the roll you pass in:
pass the unmodified die if natural faces matter, not a total with a bonus already added.

`def` starts your rule, `if` tests a condition, and `return` gives its label.
`==` means “is equal to”. Keep the indentation shown.

```dice
def name_roll(face):
    if face == 20:
        return "Critical"
    if face >= 10:
        return "Hit"
    return "Miss"

results = scale().step("Miss").step("Hit").step("Critical")
output("d20 check", classify(d(20), results, name_roll))
```

### joint\_classify

```python
def joint_classify(
    d1: DieRoll,
    d2: DieRoll,
    scale: Scale,
    classify,
) -> Outcomes
```

Choose a named result by looking at two independent rolls together.

#### Parameters

* `d1`: (required)

  First roll, passed to the first argument of your function.

* `d2`: (required)

  Second, independent roll, passed to the second argument.

* `scale`: (required)

  All allowed labels in order from worst to best.

* `classify`: (required)

  Your function's name, without parentheses. It must accept two whole numbers and return one of the scale's labels.



#### Details

For an opposed check, compare one player's roll with another's. Your function
receives both numbers and returns a label; the engine checks every possible pair.
The result is `Outcomes`, with a chance for each label.
Even if you pass the same roll variable twice, the two rolls are independent,
not two references to the same rolled face.

```dice
def compare(left, right):
    if left > right:
        return "Win"
    if left == right:
        return "Tie"
    return "Lose"

results = scale().step("Lose").step("Tie").step("Win")
output("Opposed d6 rolls", joint_classify(d(6), d(6), results, compare))
```

`def` names the rule, and `return` supplies its answer. The indented lines
belong to the rule; `>` means “greater than” and `==` means “equal to”.

## Showing results

Use output to put a result in the report, and prob_table to compare several chances.

### output

```python
def output(*args) -> None
```

Show a result in the playground report.

#### Parameters

* `*args`: (required)

  Usually two arguments: `output("Title", result)`. With just `output(result)`, a title is supplied automatically.



#### Details

Call `output` at least once so you have a result to read. Put a helpful title
in quotes, followed by the result. You can show a full numeric roll (`DieRoll`),
named results (`Outcomes`), one probability, or a table made with `prob_table`.
A `DicePool` must first become a result, for example with `.sum()` or `.count(6)`.

```dice
roll = 2d6 + 3
output("Total with bonus", roll)
output("Chance of 10 or more", roll.p_ge(10))
```

`roll = ...` gives the calculation a name so you can use it on later lines.
A probability is a number from 0 to 1: 0.5 means 50%. Do not multiply it
by 100 before passing it to `output`. The call records the result and returns
`None` (no value); do not use it to build another roll.

### prob\_table

```python
def prob_table(rows: list) -> ProbTable
```

Put several probability questions into one table for comparison.

#### Parameters

* `rows`: (required)

  A list of `(label, probability)` pairs. Labels are quoted text; probabilities are numbers from 0 to 1, such as 0.5 for 50%, not 50.



#### Details

Each row has a text label and a probability. Rows answer separate questions;
they may overlap and do not have to add to 100%. The result is a `ProbTable`;
pass it to `output` to show it.

Square brackets make the list of rows. Parentheses group each label with its chance.
Here `.p_ge(...)` asks for the chance of meeting or beating a target.

```dice
roll = dice_pool(2, 6).sum()
rows = [("Need 7+", roll.p_ge(7)), ("Need 10+", roll.p_ge(10))]
output("Compare targets", prob_table(rows))
```

## DieRoll methods

Use these on a numeric roll or total, such as `roll = dice_pool(2, 6).sum()`. Write `roll.p_ge(7)`, not `DieRoll.p_ge(7)`.

### DieRoll.mean

```python
def DieRoll.mean() -> float
```

Find the average total you would get over many rolls.

A d6 has a mean of 3.5 even though no face shows 3.5. This is an average,
not the most likely result and not a probability. The result is a decimal number.
The report for a full roll already includes its mean; use this method when
you need the number in a calculation.

```dice
roll = d(6)
average = roll.mean()  # 3.5; a number you can use in later calculations.
output("The roll, with its mean in the report", roll)
```

### DieRoll.pmf

```python
def DieRoll.pmf(value: int) -> float
```

Find the chance of rolling exactly one number.

#### Parameters

* `value`: (required)

  The whole-number result you want. An impossible result has chance 0.



#### Details

For two d6, a total of 7 has a chance of 6 out of 36, about 16.7%.
The returned probability is a number from 0 to 1, not a percentage from 0 to 100.
`pmf` is short for “probability mass function”; think “chance of this exact result”.
Use `.p_ge(7)` instead if you mean “7 or higher”.

```dice
roll = dice_pool(2, 6).sum()
output("Exactly seven", roll.pmf(7))
```

### DieRoll.p\_ge

```python
def DieRoll.p_ge(value: int) -> float
```

Find the chance of meeting or beating a target number.

#### Parameters

* `value`: (required)

  Lowest successful total.



#### Details

Use this for “need 15 or more” checks. The letters `ge` mean “greater than
or equal to”, so the target itself counts. The result is a probability from 0 to 1.
This checks the finished total, not each separate die.

```dice
roll = 2d10 + 3
output("Meet a target of 15", roll.p_ge(15))
```

### DieRoll.cdf

```python
def DieRoll.cdf(value: int) -> float
```

Find the chance of this total or anything lower.

#### Parameters

* `value`: (required)

  Highest total to include.



#### Details

Use this for roll-under rules, where low numbers are good.
`.cdf(8)` includes 8 itself. It returns a probability from 0 to 1.
`cdf` means “cumulative distribution function”: add up the chances from the bottom.

```dice
roll = dice_pool(2, 6).sum()
output("Eight or less", roll.cdf(8))
```

### DieRoll.clamp

```python
def DieRoll.clamp(min: int, max: int) -> DieRoll
```

Set a floor and a ceiling for the final result.

#### Parameters

* `min`: (required)

  Lowest result allowed.

* `max`: (required)

  Highest result allowed; must be at least `min`.



#### Details

Anything below the floor becomes the floor; anything above the ceiling
becomes the ceiling. Those chances are kept, not thrown away.
The returned `DieRoll` models a rule such as “damage is at least 1, at most 6”.

```dice
damage = d(6) + 2
output("Damage capped at six", damage.clamp(1, 6))
```

### DieRoll.support\_size

```python
def DieRoll.support_size() -> int
```

Count how many different numeric results are possible.

Two d6 have 11 possible totals, 2 through 12, even though there are 36
ways the dice can land. The returned whole number counts totals, not combinations.
“Support” is the mathematical name for the set of possible results.

```dice
roll = dice_pool(2, 6).sum()
possible_totals = roll.support_size()  # 11, counting totals 2 through 12.
output("Two dice: eleven possible totals", roll)
```

### DieRoll.keep

```python
def DieRoll.keep(spec) -> DieRoll
```

Keep only matching results and recalculate their chances to add up to 100%.

#### Parameters

* `spec`: (required)

  One number, a non-empty list of numbers, or a range such as `at_least(5)`. After `.sum()`, this matches whole totals, not the individual dice.



#### Details

On a d6, keeping 5 and 6 makes each of them 50% likely. This describes
a roll restricted to those results, not the chance of rolling them on an ordinary d6.
Use `.p_ge(5)` for that probability. The returned `DieRoll` leaves the original unchanged.
Keeping no possible results is an error.

```dice
output("Only fives and sixes", d(6).keep([5, 6]))
```

### DieRoll.remove

```python
def DieRoll.remove(spec) -> DieRoll
```

Remove matching results and share all the probability among those left.

#### Parameters

* `spec`: (required)

  Results to remove: one number, a non-empty list, or a range such as `at_most(2)`. For a summed roll, these are totals.



#### Details

Removing 1 from a fair d6 leaves 2–6, each with a 20% chance.
This is different from `.ignore(1)`, which keeps the chance of rolling 1
but makes that result worth 0. The result is a new `DieRoll`;
removing every possible result is an error.

```dice
output("A die without ones", d(6).remove(1))
```

### DieRoll.convert

```python
def DieRoll.convert(spec, to: int) -> DieRoll
```

Change matching results to a new number without changing how often they happen.

#### Parameters

* `spec`: (required)

  Results to change: one number, a non-empty list, or a range such as `at_least(5)`. On a summed roll this matches totals.

* `to`: (required)

  New whole-number value for every match.



#### Details

For a “sixes count double” rule, turn 6 into 12. Other results stay as they are.
If several results become the same number, their chances add together.
The result is a new `DieRoll`; the original is unchanged.

```dice
output("Sixes count as twelve", d(6).convert(6, 12))
```

### DieRoll.ignore

```python
def DieRoll.ignore(spec) -> DieRoll
```

Make matching results worth zero, while keeping their chance of happening.

#### Parameters

* `spec`: (required)

  Results to turn into 0: one number, a non-empty list, or a range. On a summed roll this matches whole totals.



#### Details

On a d6, ignoring 1–4 gives a 4-in-6 chance of 0, plus the usual chances
of 5 and 6. Unlike `.remove(...)`, it does not rule out those rolls.
It returns a new `DieRoll` and is shorthand for `.convert(spec, 0)`.

```dice
output("Only high faces add points", d(6).ignore(through(1, 4)))
```

### DieRoll.bucket

```python
def DieRoll.bucket(scale: Scale, *bands) -> Outcomes
```

Turn this roll's totals into named results using your scale.

#### Parameters

* `scale`: (required)

  Your ordered labels and their number ranges.

* `*bands`: (required)

  Optional replacement cut points or ranges, following the same rules as the `bucket` function. For two labels, `[6]` means 6 or less, then 7 or more.



#### Details

This is the same as `bucket(roll, results)`. It returns `Outcomes`,
which you can show as a table or ask about with `.p_at_least("Hit")`.
Each possible total must be covered by a band.

```dice
results = scale().step("Miss", at_most(6)).step("Hit", at_least(7))
roll = dice_pool(2, 6).sum()
output("Move result", roll.bucket(results))
```

## DicePool methods

Use these on a pool, such as `pool = dice_pool(3, 6)`. Face operations work on each die separately; `.sum()` turns the pool into a total.

### DicePool.sum

```python
def DicePool.sum() -> DieRoll
```

Add all the dice in this pool to get one total.

The returned `DieRoll` gives the chance of each total; for three d6, these run from 3 to 18.
Use it when you no longer need to examine individual faces.
The original pool is unchanged, so you can still ask it other questions.

```dice
pool = dice_pool(3, 6)
output("Three dice added together", pool.sum())
```

### DicePool.keep

```python
def DicePool.keep(spec) -> DicePool
```

Restrict every die to matching faces, then recalculate each die's chances.

#### Parameters

* `spec`: (required)

  Faces to allow on each die: one number, a non-empty list, or a range such as `at_least(5)`.



#### Details

Keeping 5 and 6 on three d6 makes every die show either 5 or 6 with equal chances.
It does not roll ordinary dice and discard the low ones: the pool still has three dice.
The returned `DicePool` is new; the original pool is unchanged.
It is an error if any die has no possible face left.

```dice
pool = dice_pool(3, 6).keep([5, 6])
output("Only fives and sixes: total 15–18", pool.sum())
```

### DicePool.remove

```python
def DicePool.remove(spec) -> DicePool
```

Remove matching faces from each die, not dice from the pool.

#### Parameters

* `spec`: (required)

  Faces to exclude on each die: one number, a non-empty list, or a range such as `at_most(2)`.



#### Details

Removing 1 leaves faces 2–6 on each d6, with equal chances.
The result is a new `DicePool` with the same number of dice.
Use `.ignore(1)` instead if ones should still happen but contribute 0.
It is an error if any die loses all its possible faces.

```dice
output("Three dice with no ones", dice_pool(3, 6).remove(1).sum())
```

### DicePool.convert

```python
def DicePool.convert(spec, to: int) -> DicePool
```

Give matching faces a different value on every die.

#### Parameters

* `spec`: (required)

  Faces to change: one number, a non-empty list, or a range such as `at_least(5)`.

* `to`: (required)

  New whole-number value for every matching face.



#### Details

For “sixes count double”, change each 6 to 12 before adding the dice.
The faces keep their original chances. This returns a new `DicePool`
and leaves the original unchanged.

```dice
pool = dice_pool(3, 6).convert(6, 12)
output("Total with double sixes", pool.sum())
```

### DicePool.ignore

```python
def DicePool.ignore(spec) -> DicePool
```

Make matching faces contribute zero when you add the dice.

#### Parameters

* `spec`: (required)

  Faces worth zero: one number, a non-empty list, or a range.



#### Details

Low faces still happen; they just score no points. This differs from
`.remove(...)`, which makes those faces impossible. The result is a new
`DicePool` with the same number of dice, equivalent to `.convert(spec, 0)`.

```dice
pool = dice_pool(3, 6).ignore(through(1, 4))
output("Add only fives and sixes", pool.sum())
```

### DicePool.count

```python
def DicePool.count(spec) -> DieRoll
```

Count one success for each die that shows a matching face.

#### Parameters

* `spec`: (required)

  Faces that succeed: one number, a non-empty list such as `[5, 6]`, or a range such as `at_least(5)`.



#### Details

Three dice can give 0, 1, 2, or 3 successes. The returned `DieRoll`
shows how likely each count is. It counts dice, not their face values:
a 5 and a 6 give two successes, not eleven points.

```dice
pool = dice_pool(3, 6)
output("Successes on 5+", pool.count(at_least(5)))
```

### DicePool.order\_stat

```python
def DicePool.order_stat(k: int) -> DieRoll
```

Choose the highest die, second-highest die, or another position from the top.

#### Parameters

* `k`: (required)

  Position counting from 1 at the top, no greater than the number of dice.



#### Details

`1` means highest; `2` means second-highest. Ties count separately:
in 6, 6, 2 the second-highest is 6. The result is a `DieRoll` of that one value.
This is the same as `order_stat(pool, k)`.

```dice
output("Best of three dice", dice_pool(3, 6).order_stat(1))
```

### DicePool.middle\_of

```python
def DicePool.middle_of(keep: int) -> DieRoll
```

Add the middle dice after setting aside low and high results.

#### Parameters

* `keep`: (required)

  Number of middle dice to add, from 1 to the number of dice.



#### Details

With three dice, keeping one chooses the middle die. With five dice,
keeping three adds the middle three. If an odd number of dice must be left out,
one more is removed from the high end. Returns a `DieRoll`,
just like `middle_of(pool, keep)`.

```dice
output("Middle die", dice_pool(3, 6).middle_of(1))
```

### DicePool.p\_any

```python
def DicePool.p_any(*spec) -> float
```

Find the chance that at least one die shows a matching face.

#### Parameters

* `*spec`: (required)

  Optional face match: one number, a non-empty list, or a range such as `at_least(5)`. If omitted, this only checks whether the pool has any dice. Pools created by `dice_pool` are non-empty, so the answer is 1. It does not test for non-zero faces.



#### Details

Use this for “any six is a success”. It returns a probability from 0 to 1;
0.5 means 50%. Two or more matching dice still count as success, not as extra probability.

```dice
output("At least one six", dice_pool(3, 6).p_any(6))
```

### DicePool.p\_none

```python
def DicePool.p_none(*spec) -> float
```

Find the chance that none of the dice show a matching face.

#### Parameters

* `*spec`: (required)

  Optional face match: one number, a non-empty list, or a range. If omitted, this only checks whether the pool is empty. Pools created by `dice_pool` are non-empty, so the answer is 0.



#### Details

Use this for “avoid rolling any ones”. It returns a probability from 0 to 1
and is the opposite of `.p_any(...)`: their answers add up to 1.

```dice
output("No ones", dice_pool(3, 6).p_none(1))
```

### DicePool.p\_at\_least

```python
def DicePool.p_at_least(k: int, *spec) -> float
```

Find the chance of getting enough matching dice.

#### Parameters

* `k`: (required)

  Minimum number of matching dice, 0 or more. Asking for 0 always gives 1; asking for more dice than the pool contains gives 0.

* `*spec`: (required)

  Optional face match: one number, a non-empty list, or a range. If omitted, this only checks whether the pool contains at least `k` dice, regardless of faces.



#### Details

For “at least two successes, with each 5 or 6 succeeding”, ask for two matches
to `at_least(5)`. The result is a probability from 0 to 1, not a success count.
Use `.count(...)` instead to see every possible count and its chance.

```dice
output("Two or more successes", dice_pool(3, 6).p_at_least(2, at_least(5)))
```

### DicePool.bucket

```python
def DicePool.bucket(scale: Scale, *spec) -> Outcomes
```

Add the dice, then group the total into named results.

#### Parameters

* `scale`: (required)

  Your ordered labels and their number ranges. Every possible total must be covered.

* `*spec`: (required)

  Optional replacement cut points or ranges, as for the `bucket` function. For two labels, `[6]` means 6 or less, then 7 or more.



#### Details

This is shorthand for `pool.sum().bucket(results)`. It returns `Outcomes`,
not a count of how many individual dice match a band. Use `.count(...)`
when each die earns a success separately.

```dice
results = scale().step("Miss", at_most(6)).step("Hit", at_least(7))
pool = dice_pool(2, 6)
output("Result of the total", pool.bucket(results))
```

## Outcomes methods

Use these on named results returned by bucket or classify. Higher and lower mean later and earlier on the scale you built.

### Outcomes.pmf

```python
def Outcomes.pmf(label: str) -> float
```

Find the chance of exactly one named result.

#### Parameters

* `label`: (required)

  A label on your scale, in quotes. Spelling and capital letters must match. An unknown label is an error; a known but impossible result has chance 0.



#### Details

`.pmf("Partial")` counts only partial successes, not full hits as well.
The result is a probability from 0 to 1. `pmf` is the mathematical name
for the chance of one exact result.

```dice
results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
move = bucket(2d6, results)
output("Exactly a partial success", move.pmf("Partial"))
```

### Outcomes.p\_at\_least

```python
def Outcomes.p_at_least(label: str) -> float
```

Find the chance of this result or any higher step on your ladder.

#### Parameters

* `label`: (required)

  Lowest step to include, spelled exactly as on your scale. An unknown label is an error.



#### Details

If the scale runs from miss to partial to hit, “at least partial” includes
both partial and hit. Higher means added later with `.step(...)`, not
alphabetical order. Put your labels from worst to best when building the scale.
The result is a probability from 0 to 1.

```dice
results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
move = bucket(2d6, results)
output("Partial success or better", move.p_at_least("Partial"))
```

### Outcomes.p\_at\_most

```python
def Outcomes.p_at_most(label: str) -> float
```

Find the chance of this result or any lower step on your ladder.

#### Parameters

* `label`: (required)

  Highest step to include, spelled exactly as on your scale. An unknown label is an error.



#### Details

On a miss, partial, hit scale, “at most partial” includes miss and partial.
Lower means added earlier with `.step(...)`. The chosen label itself is included.
The result is a probability from 0 to 1.

```dice
results = scale().step("Miss", at_most(6)).step("Partial", through(7, 9)).step("Hit", at_least(10))
move = bucket(2d6, results)
output("Partial success or worse", move.p_at_most("Partial"))
```

## Scale methods

Use `.step(...)` on a scale to build your ladder of named results.

### Scale.step

```python
def Scale.step(label: str, *band, early: bool = False) -> Scale
```

Add the next named result to your ladder, working from worst to best.

#### Parameters

* `label`: (required)

  A unique text label in quotes, such as `"Hit"`.

* `*band`: (required)

  Optional number range, such as `at_most(6)`, `through(7, 9)`, or `at_least(10)`. A step without a range does not act as a catch-all for `bucket`.

* `early`: (defaults to: `False`)

  Optional named argument, `early=True` or `early=False`. Defaults to `False`.



#### Details

Each call returns a new `Scale`; it does not change the old one.
Save the result with `results = results.step(...)`, or put several `.step(...)`
calls one after another. Give each label a number range when using `bucket`.
You can leave ranges out for `classify`, or supply them separately to `bucket`.

```dice
results = scale().step("Miss", at_most(6))
results = results.step("Partial", through(7, 9))
results = results.step("Hit", at_least(10))
output("Move result", bucket(2d6, results))
```

For overlapping ranges, `early=True` means “check this step before ordinary steps”.
`True` is Starlark's word for yes. Early steps are checked in the order added,
then ordinary steps in their order; the first match wins. This does not change
the ladder order used by `.p_at_least(...)` and `.p_at_most(...)`.

