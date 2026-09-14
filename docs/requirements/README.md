# Dice Playground: product requirements

**Draft for review. Nothing in this extraction authorises removal of the old planning system or its output.**

Dice Playground helps a tabletop role-playing game designer answer two questions: **“What are the odds?”** and **“How can I explain why I chose this rule?”** The intended result is a readable document containing the rule, the calculation, and the results together.

These documents collect the application’s purpose and design intentions in plain English. They are written for **casual TTRPG designers**, including GMs making house rules, rather than assuming a professional game-design or probability background.

## Start here

| Document | What it explains |
|---|---|
| [What the application is for](product.md) | Who it serves, the problems it addresses, its guiding choices, and what it is not trying to do. |
| [The intended experience](experience.md) | How someone models a rule, runs a document, understands the results, learns, and shares their reasoning. |
| [Review notes and open decisions](review-notes.md) | What remains uncertain, where old documents disagree, and what needs review before the old material can be removed. |

A separate [technical design set](../design/README.md) explains the probability mathematics, document-processing pipeline, application architecture, tutorials, reference documentation, and LLM-assisted authoring. You do not need those implementation details to review the product’s purpose.

## How to read the status of a statement

- **Intended behaviour** records a requirement or a decision already present in the source material. It does **not** mean that the feature has been fully implemented or tested.
- **Planned extension** preserves a design direction that the sources described as later work.
- **Open decision** means the sources did not settle the issue, or contradict one another. This extraction does not quietly choose an answer.
- **Implementation observation** describes code inspected while preparing the technical notes. It is evidence about the current implementation, not a new product requirement.

The original material uses “v1” for the initial **literate-document format**, not for the first release of Dice Playground as a whole. That distinction is retained in the technical notes.

## What has been preserved

The extraction covers all 13 documents in the repository’s `_bmad-output/`, including its two hidden decision logs. Repeated statements have been combined; superseded ideas and unfinished choices have been identified rather than promoted into requirements. The [source map](../design/source-map.md) records where each subject came from.

The mathematical core and some integration details were only assumed by those plans. Their explanations are supplemented from the existing source code, tests, and user reference. Those additional sources are identified separately. The emphasis on casual TTRPG designers comes from the current review request; the older brief also included players, GMs, educators, and developers.

These are standalone documents: understanding the design does not require a BMad skill, workflow, or original planning file. Historical filenames in the source map are provenance, not dependencies. The old material remains unchanged while this draft is reviewed.
