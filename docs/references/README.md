---
title: "Function reference"
author: Steve Cooper
---

# Function reference

This section is the **stdlib and type reference** for `.dice` scripts: builtins (`d`, `roll_pool`, `sum`, `classify`, …) and methods on `Dist`, `RollPool`, and `LabelDist`.

| File | Contents |
|------|----------|
| [stdlib.md](stdlib.md) | Generated from Rust doc comments on the dice engine |

Do **not** edit `stdlib.md` by hand. Regenerate from the repo root:

```bash
make references
```

Change doc strings in `src/engine/starlark_guest/eval.rs`, `dist_value.rs`, and `pool_value.rs`, then run `make references` and commit the updated Markdown.

For learning the language in order, use the [tutorial](../README.md#tutorial). For ready-made mechanics, see the [cookbook](../cookbook/README.md).
