# Literate documents: one source, one execution

**Document contract and rationale — approved.** [Design index](README.md)

## Why a document layer exists

Starlark understands code, not an ordinary paragraph explaining a house rule. Dice Playground therefore needs a document layer that separates explanation from executable code before calling the existing evaluator.

The chosen source is **markdown-first `.dice`**, rather than prose hidden inside string literals, a new notebook JSON format, or a prose file paired with a script. Markdown makes the raw file readable and fits the tutorial-writing workflow. The document layer then produces both a program to evaluate and a report to read.

The format described here is the initial literate **v1** contract. The underlying dice language and legacy scripts continue to exist.

## File identity and mode detection

A `.dice` file is UTF-8. Invalid UTF-8 is an input/parse error. There are two mutually exclusive modes:

| Mode | How it is selected | What executes |
|---|---|---|
| **Literate** | At least one executable fenced code block is present. | The bodies of all executable blocks, combined in source order. |
| **Legacy** | No executable fence is present. | The entire file, through the existing dice-desugaring and Starlark path. |

The extension alone does not mean that a file is markdown. A document containing only prose or only `text`-labelled examples does **not** become literate under this rule; without an executable fence it is treated as a legacy script and may fail to parse. A literate document can still produce a prose-only report if its executable code emits no outputs.

The switch is intentionally automatic to keep existing scripts working. It does introduce a compatibility edge case: a formerly legacy file containing markdown-style bare fences could now be detected as literate.

**Retained detection limitation:** fence detection precedes native Starlark lexing. A flush-left executable fence inside what would otherwise be a triple-quoted Starlark string can therefore select literate mode and execute its body. The shared lexer preserves native strings when tokenizing a script, but it does not override this document-mode precedence. Highlighting follows the same precedence. Changing it needs a separate format decision: treating markdown prose as Starlark could conversely let prose quotes hide real fences. Avoid fence-shaped executable blocks inside legacy multiline strings.

## Which code blocks execute?

A *fence* is a line of backticks marking the beginning or end of a code block. The initial contract specifies:

- an opening run of **at least three backticks**;
- either an empty information string or the information string **exactly `dice`**, case-sensitive;
- a closing line containing a backtick run at least as long as the opener, with no code on that line; and
- executable content between the two fence lines.

The *information string* is the text after the opening backticks. The contract describes trimming surrounding whitespace, but disagrees about spaces versus tabs and about indentation. These are explicit [review questions](../requirements/review-notes.md#document-rules); flush-left fences with no extra whitespace are the unambiguous authoring form.

| Opening label | Meaning |
|---|---|
| `dice` | Execute the block. |
| No label | Execute the block: the default language is dice. |
| `text`, `rust`, or any other non-empty label | Display as prose/code, but do not execute it. |
| `Dice`, `dice,hidden`, or `{=dice}` | Not an executable alias in v1. |

Only backtick fences are specified as executable. Inline backtick spans are prose, not executable statements. A non-executable block must be handled as a complete block; its closing bare fence is not a new executable opener.

There is no required wrapper or notebook metadata. YAML front matter is not defined in v1; a line such as `---` remains ordinary markdown rather than a configuration header. Per-block hidden/echo metadata is also deferred.

### Example: two stages of one explanation

The following is the content of a single file:

````text
# A check with a bonus

We are comparing a d20 check with a fixed bonus against DC 15.

```dice
BONUS = 3
```

Now calculate the chance of meeting the target.

```dice
roll = d(20) + BONUS
output("chance_to_pass", roll.p_ge(15))
```
````

Both blocks share one program scope. The report places the output after the second block, where the probability is calculated. Changing `BONUS` and pressing Run evaluates both blocks again.

For a short legacy script, the file can simply contain:

```text
output("d6", d(6))
```

No prose wrapper or migration is required for that case.

## Tangle: turn the document into one program

The tangle pass extracts executable block bodies in document order. It returns:

- the combined Starlark source;
- a source map from combined-code locations to original-document locations; and
- metadata for each executable fence, including its source start/end lines, body byte range, and order.

All extracted code is evaluated as **one Starlark module**. This is not a loop that evaluates each block separately. Variables, functions, and top-level statements share the same module during that Run.

The detailed format’s joining rule is to preserve body newlines and add a single separator newline **only if the preceding body lacks one**. The shorter historical specification instead said “a single newline between bodies” without that qualification. The shared-preparation change explicitly follows the detailed rule for extracted bodies: append each body once, ignore empty bodies for separators, and add a separator only after nonempty code lacking a newline. This fixes duplicated/phantom tangle lines and their shifted diagnostics; it does not change fence grammar. The existing extractor still omits the final fence-adjacent LF from each body and preserves CR characters. Source-line accounting follows the actual emitted bytes, including trailing blank lines and empty fences.

An external line map is preferred to inserting `# line` comments into the program. The result then passes through the existing range/dice notation expansion and Starlark parser. Introducing prose does not require replacing the probability engine or changing the Starlark dependency.

## Diagnostics must point back to what the author sees

The user edits the `.dice` document, not the tangled program. Check errors, parse errors, and runtime diagnostics should therefore refer to the original source line and column. An unclosed fence should identify a document location directly.

The mapping contract includes column offsets, not only line numbers. Any additional changes introduced by desugaring need to be considered when preserving accurate columns. The intended LSP path also uses this mapping; the old architecture explicitly left detailed LSP integration to later hardening.

Tests include errors in later fences after prose, blank lines and empty fences, plus shorthand earlier on the same line and Unicode before the error. Expansion byte maps compose with the literate line map. See [shared lexical preparation](architecture.md#shared-lexical-preparation-current-implementation) for the current pipeline and bounded LSP support.

## Weave: combine prose with evaluated results

The weave pass turns the original document and evaluated outputs into an HTML **fragment**. The playground inserts that fragment into its report container. The command-line renderer wraps it in a full page with navigation and shared stylesheet references.

Prose is rendered with the shared Rust markdown renderer. Executable blocks are not interpreted as markdown. The initial format allows the report to show their source as static code, hide it, or collapse it; that presentation choice was not settled.

The supported authoring subset is:

- headings, paragraphs, emphasis, and strong emphasis;
- links and ordinary markdown image references;
- bullet and numbered lists;
- non-executing code blocks and inline code; and
- **GFM pipe tables**, explicitly adopted by the table design.

“CommonMark plus tables” does not promise all Pandoc or GFM extensions. Callouts, mathematics, footnotes, and arbitrary raw HTML are not guaranteed authoring features. Safe raw-HTML policy is a separate security decision.

All resulting user-content HTML must pass engine-side sanitization before browser insertion or CLI publication. [Report rendering](reports-and-visualisation.md) covers output tables, labels, and chart attributes.

## Output placement and identity

Executable fence source is now displayed directly before its results, escaped and sanitized as code. Tutorial/cookbook authors must not maintain a second display-only copy.

The initial placement rule is **immediately below the executable fence responsible for an output**, in document order. Within a fence, outputs retain evaluation order. At minimum every recorded `output()` entry appears once in the report.

The source specification allows two implementation strategies:

1. attach output entries to a fence index using execution/source positions; or
2. assign outputs sequentially from global evaluation order in cases where a sequential fence interpretation is valid.

It requires the selected strategy to be documented and tested with fixtures that distinguish the alternatives. This is an important unresolved boundary, not a reason to imply arbitrary outputs can always be assigned correctly by counting source text.

**Implementation observation:** the current weave uses a static count of lines containing `output(` to consume outputs in global evaluation order, then attaches leftovers to the last fence. That is a heuristic, not precise runtime provenance. Loops, conditionals, multiple calls on one line, and functions defined in one block but called in another need explicit treatment before claiming general per-fence placement.

The rewritten learning corpus uses a tested **single executable fence per document** restriction. All setup and outputs live in that fence, so loops and helper calls retain evaluation order without relying on per-fence provenance. The manifest validator and actual literate-parser corpus tests enforce this interim contract; general multi-fence provenance remains deferred.

The original format also says duplicate names use the **last output for named binding**, with a check-mode warning recommended. That sits awkwardly beside “every output appears once”, particularly when a chart is located by name but its adjacent table belongs to an earlier entry. The [review notes](../requirements/review-notes.md#document-rules) preserve the need for a consistent identity policy. Authors can avoid the ambiguity by using distinct names.

### Deferred prose placeholders

The reserved later syntax is:

```text
{{output "name"}}
```

It is **not part of v1**. Earlier research suggested several placeholder spellings and combining placeholders with automatic placement; the later architecture explicitly deferred that feature. Authors should not rely on any spelling until the feature is specified.

## Compatibility and limits

- Legacy scripts bypass tangle and keep their existing full-file behaviour.
- Literate files are limited to **256 KiB of UTF-8 source**; legacy files retain **64 KiB**.
- Existing output-count and computational guards remain relevant. A separate limit on tangled code was discussed but not fixed.
- Breaking changes to detection, fence rules, or binding should be treated as a format-version change rather than an invisible implementation detail.

| Consumer | Parse/tangle literate input | Evaluate | Weave |
|---|---|---|---|
| Playground | Yes | Whole document | Full report fragment. |
| `dice eval` | Yes | Whole document | Not required for text/JSON output. |
| `dice render` | Yes | Whole document | Full page wrapping the fragment. |
| Automated corpus checks | Yes | Whole document | Optional render checks in addition to evaluation. |

The same source bytes should work in all consumers. It is not enough for the website to render a lesson whose copied script only happens to execute elsewhere.

## Checks that preserve the contract

The original capability tests translate to these practical checks:

1. A minimal literate file with prose and one executable fence is recognised and evaluated.
2. Bare and explicit `dice` fences execute equivalently; other labels remain display-only.
3. Two fences share scope and execute in source order.
4. A legacy script without executable fences retains its previous results.
5. Diagnostics in later blocks point to the original document.
6. Reports contain prose and the correctly placed outputs, with no missing or repeated entries.
7. Non-executing examples remain readable and do not accidentally execute.
8. Sanitization and source-size limits are exercised, including failure paths.
9. One tutorial source evaluates in both native and browser paths and renders for static publication.

---

**Basis:** [source map](source-map.md), S01–S09 and S11. The current placement heuristic is an observation from `src/engine/literate/weave.rs`, not an approval of that heuristic for all documents.
