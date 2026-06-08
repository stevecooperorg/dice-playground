# Dice standard library

# dice_stdlib

Dice probability builtins for Starlark scripts.

Use `+` on `Dist` values to convolve (sum of independent outcomes).
Use `-` for independent differences (e.g. `2d10 - 3d6`). Use `shift` for flat modifiers.
Use `*` to scale every outcome (`1d4 * 10`). Use `//` for per-outcome floor division (`8d6 // 2`).

## bucket

```python
bucket: LabelDist
```

---

## classify

```python
classify: LabelDist
```

---

## count\_ge

```python
count_ge: Dist
```

---

## count\_in

```python
count_in: Dist
```

---

## d

```python
d: Dist
```

---

## die\_faces

```python
die_faces: Dist
```

---

## drop\_highest

```python
def drop_highest(count: int, sides: int, drop: int) -> Dist
```

Roll `count` dice, drop the `drop` highest, sum the rest.

#### Parameters

* `count`: (required)

  Dice in the pool.

* `sides`: (required)

  Faces per die.

* `drop`: (required)

  How many highest results to remove before summing.



---

## drop\_lowest

```python
def drop_lowest(count: int, sides: int, drop: int) -> Dist
```

Roll `count` dice, drop the `drop` lowest, sum the rest (e.g. 4d6 drop lowest 1).

#### Parameters

* `count`: (required)

  Dice in the pool.

* `sides`: (required)

  Faces per die.

* `drop`: (required)

  How many lowest results to remove before summing.



---

## explode

```python
explode: Dist
```

---

## joint\_classify

```python
joint_classify: LabelDist
```

---

## keep\_highest

```python
def keep_highest(count: int, sides: int, keep: int) -> Dist
```

Roll `count` dice, keep the `keep` highest, sum them.

#### Parameters

* `count`: (required)

  Dice in the pool.

* `sides`: (required)

  Faces per die.

* `keep`: (required)

  How many highest results to sum.



---

## keep\_lowest

```python
def keep_lowest(count: int, sides: int, keep: int) -> Dist
```

Roll `count` dice, keep the `keep` lowest, sum them.

#### Parameters

* `count`: (required)

  Dice in the pool.

* `sides`: (required)

  Faces per die.

* `keep`: (required)

  How many lowest results to sum.



---

## middle\_of

```python
middle_of: Dist
```

---

## order\_stat

```python
order_stat: Dist
```

---

## output

```python
def output(*args) -> None
```

Record a distribution or probability for playground output (text, json, graph).

---

## pool

```python
pool: RollPool
```

---

## pool\_map

```python
pool_map: Dist
```

---

## prob\_table

```python
prob_table: ProbTable
```

---

## result\_type

```python
result_type: ResultScale
```

---

## roll\_pool

```python
roll_pool: RollPool
```

---

## shift

```python
def shift(dist: Dist, delta: int) -> Dist
```

Add a constant to every outcome (modifier), without convolving with another die.

#### Parameters

* `dist`: (required)

  Input distribution.

* `delta`: (required)

  Amount to add to each outcome.



---

## success\_pool

```python
success_pool: Dist
```

---

## sum

```python
sum: Dist
```

# Dist type

# `Dist` type

## Dist.cdf

```python
def Dist.cdf(value: int) -> float
```

Cumulative distribution: P(X <= value).

#### Parameters

* `value`: (required)

  Upper bound (inclusive).



---

## Dist.mean

```python
def Dist.mean() -> float
```

Expected value (mean) of the distribution.

---

## Dist.p\_ge

```python
def Dist.p_ge(value: int) -> float
```

Probability of meeting or beating a target: P(X >= value).

#### Parameters

* `value`: (required)

  Target outcome (inclusive).



---

## Dist.pmf

```python
def Dist.pmf(value: int) -> float
```

Probability mass at an exact outcome: P(X = value).

#### Parameters

* `value`: (required)

  Outcome to query.



---

## Dist.support\_size

```python
def Dist.support_size() -> int
```

Number of outcomes with non-zero probability.

# RollPool type

# `RollPool` type

## RollPool.sum

```python
def RollPool.sum() -> Dist
```

Sum all dice in the pool into one outcome distribution.

# LabelDist type

# `LabelDist` type

## LabelDist.p\_at\_least

```python
def LabelDist.p_at_least(label: str) -> float
```

Probability of this label or any higher-ranked label on the scale.

---

## LabelDist.p\_at\_most

```python
def LabelDist.p_at_most(label: str) -> float
```

Probability of this label or any lower-ranked label on the scale.

---

## LabelDist.pmf

```python
def LabelDist.pmf(label: str) -> float
```

Probability mass at an exact label: P(X = label).