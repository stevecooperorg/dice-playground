# Literate `.dice` file format (v1)

Normative rules for authors, parsers, tangle, weave, and tests. Implementations MUST match this document unless a companion SPEC constraint explicitly overrides.

## 1. File identity

- **Extension:** `.dice`
- **Encoding:** UTF-8. Invalid UTF-8 is a parse error.
- **Modes:** **Literate** or **Legacy** (mutually exclusive per file, detected automatically).

## 2. Literate vs legacy detection

A file is **Literate** when it contains **at least one** **executable fence** (see §3.1).

Rules:

- Info string is the text immediately after the opening ` ``` ` on the same line, trimmed of leading/trailing spaces only.
- **Default language:** an **empty** info string (opening line is only backticks and optional trailing spaces) counts as **`dice`** for detection, tangle, and weave.
- **Explicit tag:** info string exactly `dice` (case-sensitive) is also an executable fence.
- **Not executable:** any other info string (`text`, `rust`, `Dice`, `dice,hidden`, etc.) does not trigger literate mode and is not tangled.
- A legacy file is **any** `.dice` that fails the literate test. Legacy content is the **entire file** interpreted as Starlark (after existing desugar), with no fence stripping.

**Assumption:** Authors may use bare ` ``` ` on the opener instead of ` ```dice `; both forms are equivalent for v1.

## 3. Document structure (literate)

A literate file is:

1. **Prose regions** — markdown interpreted by the weave pass (CommonMark via pulldown-cmark).
2. **Executable fences** — Starlark regions extracted by tangle (bare ` ``` ` or ` ```dice `).

There is **no** required document wrapper. Optional YAML front matter is **not** defined in v1; lines starting `---` are prose/markdown unless a future version says otherwise.

### 3.1 Executable fenced code blocks (`dice`)

**Opening line** (regex-oriented spec; either form):

```text
^```[ \t]*$
^```[ \t]*dice[ \t]*$
```

- Backtick run length ≥ three.
- Empty info string **or** info string exactly `dice` after trim (no `{=dice}`, no `dice,hidden`, no other language aliases in v1).

**Closing line:**

```text
^```[ \t]*$
```

- Same or longer backtick run as opener (CommonMark rule).
- Closing fence on its own line; content between open and close is **tangle body** (includes trailing newline on inner lines as authored).

**Multiple fences:** Allowed. Order in file = order in tangle.

### 3.2 Non-executable fenced blocks

Fences with a **non-empty** info string other than exactly `dice` (e.g. ` ```text `, ` ```rust `) are **prose** for weave: rendered as code blocks in HTML, **not** executed.

### 3.3 Inline code

Single-backtick `` ` `` spans in prose are markdown inline code, not Starlark.

## 4. Tangle

**Input:** literate `.dice` source string.  
**Output:** `(tangled_source: String, line_map: LineMap, fences: Vec<FenceMeta>)`.

Algorithm (normative intent):

1. Scan lines for fenced blocks per §3.
2. For each **executable** fence in document order, append body to `tangled_source`.
3. Between consecutive fence bodies, append exactly **one** `\n` if the prior body did not end with `\n`.
4. Record for each fence: start/end lines in source, byte range of body, index in tangle order.
5. Build **LineMap**: for each line in `tangled_source`, map to `(source_line, source_column_offset)` for diagnostic remapping (implementation may use sparse map).

**Starlark module semantics:** One eval of `tangled_source` per Run. All top-level statements and `output()` calls share one module scope.

**Desugar:** Apply `desugar_if_needed(path, tangled_source)` before Starlark parse (unchanged engine behavior).

## 5. Eval

- **One shot** per Run: full tangled module.
- **`output(name, value)`** records outputs in evaluation order; names must be unique enough for weave binding (duplicate names: last wins for binding; implementations SHOULD warn in check mode).
- Legacy files skip tangle; eval full file as today.

## 6. Weave (v1)

**Input:** literate source, `EvalResult` / output list, weave options.  
**Output:** HTML **fragment** (no `<html>` required for playground; CLI render wraps fragment).

### 6.1 Prose

- Split or walk document so **non-fence** regions pass through `markdown_to_html`.
- **Executable fences:** In v1, do not execute markdown inside fence bodies; show optional static `<pre><code>` of source or omit source in report (implementation choice; static docs may hide source, playground may show collapsed source—**not** specified in v1).

### 6.2 Output binding (v1)

After each **executable fence** in document order, append HTML representing outputs registered **during eval of that fence’s contribution** to the tangled module.

Normative approach for v1:

- Track output entries with **tangle fence index** at eval time (engine assigns fence index to each `output()` based on current source position in tangled line map), **or**
- Bind outputs in **global eval order** to fences sequentially (first outputs → first fence, etc.) when each fence contains at least one `output()` call.

Implementations MUST document which strategy they use in code comments; tests MUST use fixtures that disambiguate.

**Minimum:** Every `output()` in the file appears exactly once in the woven report.

### 6.3 Output binding (v1.1 — not v1)

Prose placeholder syntax reserved, **not implemented in v1**:

```text
{{output "name"}}
```

Authors MUST NOT rely on placeholders until v1.1 is specified.

### 6.4 Markdown subset (authoring)

Authors SHOULD limit prose to constructs supported by `pulldown-cmark` with `markdown_options()` in the engine (CommonMark + **GFM tables**):

- ATX headings `#` … `######`
- Paragraphs, emphasis, strong, links, images
- Bullet and ordered lists
- Fenced code blocks (non-executable info strings only)
- GFM pipe tables

Authors SHOULD NOT rely on raw HTML in markdown until sanitization policy explicitly allows tags.

### 6.5 Sanitization

All woven HTML MUST be sanitized in **`src/engine/`** before returning to UI or writing CLI files. Strip scripts, event handlers, and disallowed URLs per sanitizer policy.

## 7. Size and limits

| Limit | Value |
|-------|--------|
| Literate file size | **256 KiB** UTF-8 bytes |
| Legacy file size | Existing `MAX_SOURCE_BYTES` (64 KiB) until unified |
| `output()` count | Existing `MAX_OUTPUT_COUNT` |

## 8. Examples

### 8.1 Minimal literate (file content)

```text
# One die

Fair d6 distribution:

```dice
output("d6", d(6))
```
```

### 8.2 Legacy (unchanged)

```text
output("d6", d(6))
```

No executable fence (bare ` ``` ` or ` ```dice `) → legacy mode.

### 8.3 Two fences, shared scope

```text
Intro.

```dice
bonus = 3
```

```dice
output("check", 1d20 + bonus)
```
```

Tangle yields one module with both statements; weave shows output below the second fence.

### 8.4 Bare opener (equivalent to `dice`)

```text
# Roll

```
output("d6", d(6))
```
```

## 9. Consumer matrix

| Consumer | Parse | Tangle | Eval | Weave |
|----------|-------|--------|------|-------|
| Playground WASM | yes | yes | yes | yes |
| `dice eval` | yes | yes | yes | optional text/json only |
| `dice render` | yes | yes | yes | yes |
| CI tests | yes | yes | yes | optional |

## 10. Versioning

This document is **format v1**. Increment when breaking fence rules, detection, or binding. Capability IDs in `SPEC.md` track product scope; this file tracks **author-facing syntax**.
