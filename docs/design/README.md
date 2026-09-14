# Dice Playground: technical design

**Approved by the project owner on 2026-09-14.** These are the maintained technical design notes behind the [product requirements](../requirements/README.md). Approval preserves the documented open questions and does not authorise an implementation rewrite.

The central design is simple: **one probability engine evaluates one document, and several interfaces present or publish the result**. The browser, command-line tool, and documentation build should not develop different meanings for the same dice rule.

## Reading order

| Document | Main question |
|---|---|
| [The probability engine](probability-engine.md) | What is calculated, what does “exact” mean, and how are dice combined mathematically? |
| [Literate documents](literate-documents.md) | How do prose and code become one executable program and a readable report? |
| [Reports, tables, and charts](reports-and-visualisation.md) | How do structured results become useful views without changing their meaning? |
| [Application architecture](architecture.md) | Where do responsibilities live, how do the browser and CLI share the engine, and what constraints protect the design? |
| [Tutorials, documentation, and LLM assistance](learning-and-llm.md) | How does the learning material stay executable, publishable, and useful to both people and drafting tools? |
| [Source map](source-map.md) | Which original documents support the design, and which implementation sources supplement them? |

[Review notes](../requirements/review-notes.md) collect unresolved choices and source contradictions. The main explanations keep those uncertainties visible rather than silently converting them into implementation decisions.

## Terms used in these notes

| Term | Meaning |
|---|---|
| **Distribution / PMF** | A table of possible results and the probability of each result. PMF means *probability mass function*. |
| **Pool** | Several dice kept separate until a rule says to sum them, count successes, or keep/drop dice. |
| **Literate document** | One source file containing human-readable explanation and executable code. |
| **Tangle** | Extract executable code from a literate document into one program. |
| **Desugar** | Translate convenient dice notation into the underlying scripting language. |
| **Weave** | Turn prose and calculated outputs into a report. |
| **WASM** | WebAssembly: the compiled form that lets the Rust engine run in a browser. |
| **Static publication** | Generate HTML and assets ahead of time and serve them as files; no application server evaluates the document for each reader. |

## Intent versus implementation

The earlier design material concentrates on the move from an editor with detached results to single-file literate reports. It assumes much of the mathematical engine already exists. Consequently:

- document, output, and architecture requirements are primarily extracted from that material;
- the mathematical explanation is grounded in the current engine and its tests;
- current implementation observations are identified as such, not treated as proof that every requirement is complete; and
- obsolete workflow instructions and role assignments are not part of the application design.

The technical notes preserve important format and compatibility details so they do not disappear with the old system. They are not intended to replace the generated API reference or to reproduce every function signature.
