# The intended experience

**Product requirements — approved.** [Reading guide](README.md)

## Start from a question or a worked example

A designer may begin with a small question, such as “What is the chance of rolling at least 15?”, or open a tutorial or cookbook recipe that resembles their mechanic.

The playground provides an editor for the active `.dice` document, a way to manage workspace files, a Run action, diagnostics, and results. **Shift+Enter** is the keyboard shortcut for Run. New-document templates are intended to encourage the explanation-and-code style, without breaking existing short scripts.

Keeping several documents in the workspace is compatible with analysing one complete document at a time. The old plans leave the long-term role of the workspace open; they do not authorise removing it.

## Describe the rule and make the assumptions visible

A document can contain headings, paragraphs, lists, links, simple tables, and images referenced from its prose. Calculations live in clearly marked code blocks. In a block, familiar notation such as `2d6` or `4d6dl1` sits alongside variables and repeated calculations written in a small scripting language called Starlark.

For example, a document about a risky action might explain:

1. which dice are rolled;
2. which bonus is added;
3. what counts as failure, partial success, and full success;
4. which variants are being compared; and
5. how to interpret the resulting probabilities.

The document is not required to use that outline. The important requirement is that prose and executable calculations can be kept together and read in a sensible order.

Ordinary code examples that are meant only to be read must be distinguishable from code to be executed. The [document format](../design/literate-documents.md) explains the marking rules, including the important fact that an unlabelled fenced block is executable by default.

## Run once and get an understandable answer

Run evaluates all executable parts of the document as one program. A value defined near the top can be used later. The user does not need to run blocks in a particular sequence.

If the document has an error, diagnostics should point to the **original document’s** line and column, not to a hidden intermediate program. An unclosed code block should also identify its location. A failed Run shows diagnostics rather than presenting a fresh successful Output panel.

Existing `.dice` files containing only Starlark and dice notation remain usable. Authors do not have to turn every tiny calculation into a report. The document detector, rather than the filename extension alone, chooses between these two styles.

## Read a report, with other views available

There should be **one Output area** after the editor and any diagnostics. It supports these views:

| View | What it is for |
|---|---|
| **Report** | Read the explanation with formatted results in place. For a short script without prose, this can show formatted results alone. |
| **text** | Read or copy the plain-text form also used by the command-line tool. |
| **json** | Inspect or reuse the structured numerical data. Most casual users need not use this view. |
| **graph** | Browse the chartable outputs together. |

Report comes first in the tab order, followed by text, json, and graph. A successful literate Run with a report should select Report by default. The user’s choice remains until the next successful Run. If no report is available, text is the normal fallback, then json. Whether a legacy script’s outputs-only HTML should also become the default is recorded as a [review question](review-notes.md#result-presentation).

Text and JSON remain available for literate documents; choosing a report-oriented product does not mean discarding those useful views. The UI should avoid duplicate Report and Graph sections outside the shared panel.

Tabs without meaningful content can be omitted or disabled. A successful literate document with no `output()` calls can still show its prose. The details of empty graph tabs are not fully consistent in the source material and need a small UI decision, rather than an invented requirement.

The controls must at least be keyboard-focusable and expose proper tab and panel roles to assistive technology. Arrow-key tab navigation was proposed for later refinement, not established as an implemented feature.

## Keep each result close to its explanation

The initial report format places an output immediately after the executable block responsible for it. Every output should appear once, in a predictable order. This is what connects a calculation to the explanation around it.

An output should have a useful name, such as “ordinary attack” or “with advantage”. Repeated names and outputs created inside loops or functions expose unresolved placement details; these are preserved in the [review notes](review-notes.md#document-rules), not hidden by the phrase “inline output”.

Writing a special placeholder in prose to move a named result somewhere else is a **later extension**, not part of the initial format. Whether the report shows, hides, or collapses the executable source itself is also left open. Source visibility does not change what is executed.

## Tables and charts should answer different questions

A table is useful when the designer wants a particular value. A chart is useful when they want to see the shape of the results.

- Reports use real, readable HTML tables rather than a fixed-width block of padded text.
- Lesson tables normally show several representations of probability, such as a percentage, a fraction, and a count out of a sample space. A less cluttered selected-format view is part of the table design, but its control is not fully specified.
- Named outcome bands retain their intended order, such as failure → partial success → full success.
- Table shortening must not silently change the underlying distribution. A chart uses the full available data even if the printed table folds small tails into labelled rows.

The **planned inline-chart design** puts a chart above the table for each suitable result, so a reader does not have to switch views to connect the numbers to their shape. Numeric distributions use line charts; named outcomes and individual probabilities use bar charts. Large comparison tables should remain tables rather than produce unreadable charts; the recorded default cut-off is 32 rows.

Static HTML reports may show only tables: the initial inline-chart plan relies on the playground to draw charts. It does not promise an interactive chart bundle or generated chart images in an exported HTML page.

## Learn using the same kind of document

Tutorials introduce one idea at a time; cookbook recipes show useful complete mechanics. Both should:

- explain the tabletop situation before the syntax;
- state relevant assumptions and limits;
- be executable as complete documents, without copying missing setup from elsewhere;
- use the same engine as user-created documents; and
- be checked automatically so that examples remain valid when the language changes.

The documentation website is a rendered view of these documents. Navigation and the function reference can use different source formats; “single source” means that a lesson’s explanation and runnable model do not have separate competing copies.

Opening a lesson in the playground should preserve the **whole literate source**, including its explanations and earlier definitions. Older “load this code block” links are a transitional implementation, not the intended final handoff.

## Optional help from an LLM

A designer can give an external language model the published `llms.txt` reference, then describe a rule in ordinary words and ask for a `.dice` document. The user brings the result back into the playground, reads it, fixes diagnostics, and checks that its assumptions match the intended rule.

The model is a **drafting assistant**, not the probability engine. A script that runs can still model the wrong rule. A useful prompt includes the die sizes, whether dice are summed or counted, modifiers, targets, special results, and the outputs to compare.

The documented integration is a text reference and an authoring workflow. It is not a built-in chat service, an automatic model call, a hosted API-key system, or a promise to send a user’s document to an LLM.

## Preserve and share the reasoning

The essential artefact is the `.dice` source. The command-line renderer can turn a literate document into a full HTML page for static publication. The playground’s Report view and the static page should use comparable typography and table styling.

The broader ambition is for designers to share an understandable analysis. The sources leave PDF-like export, one-click publishing, image uploads, generated images, and asset storage unresolved. Ordinary image links in prose do not settle any of those storage or export questions.

## Review scenarios

These scenarios make the intent concrete without claiming the implementation has passed them:

1. **Compare a house rule:** open a recipe, change a modifier, Run once, and read both the assumptions and the revised result in one report.
2. **Build an explanation in stages:** define a value in an early block, use it in a later one, and see the later result in the right place.
3. **Find an error:** deliberately break a line in a later block and receive a diagnostic pointing back to that line in the document.
4. **Keep a small script:** run an old plain-Starlark `.dice` file and retain its numerical behaviour and text/JSON views.
5. **Reuse a lesson:** evaluate the same source in the browser and CLI, and publish its rendered HTML without maintaining a copied script.
6. **Treat generated code critically:** draft a mechanic with an LLM, inspect its assumptions, and verify its outputs with the actual engine.

---

**Basis:** [source map](../design/source-map.md), S01–S04 and S07–S12; additional learning and LLM detail from the current user guide and `llms.txt`.
