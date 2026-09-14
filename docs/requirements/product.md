# What Dice Playground is for

**Intended product behaviour — approved.** [Reading guide](README.md)

## A tool for trying out tabletop rules

Dice Playground is for people who make or adapt tabletop role-playing game rules and want to understand what those rules do. You might be designing a small game for friends, adjusting a monster, writing a house rule, or comparing two ways of resolving a risky action.

You should not need to be a mathematician to ask:

| A design question | What the application should help you see |
|---|---|
| “Will this target number be too hard for a beginner?” | The chance of success at different bonuses or difficulty levels. |
| “Is advantage stronger than a flat bonus?” | How the whole spread of results changes, not just the average. |
| “How often does this move produce a partial success?” | The chances of the named outcomes that matter in the game. |
| “Does adding another die make this pool too reliable?” | The number of successes you can expect and the chance of reaching a threshold. |
| “Will this damage rule be too swingy?” | The range, average, common results, and rare extremes. |
| “How do I explain this house rule to the rest of the group?” | A readable analysis with the assumptions and results beside one another. |

The application informs judgement; it does not decide whether a mechanic is fun, fair, or suitable for a particular group. Correct arithmetic also cannot rescue a model that leaves out an important rule.

## Who it is designed for

The main reader of these requirements is a **casual TTRPG designer or homebrewing GM**: someone who knows what happens at the table but may not know probability terminology or how to write a program from scratch.

The design also serves players comparing options, more experienced designers checking balance, and educators teaching probability through games. Programmers can use the command-line tool, editor integration, and open-source engine, but the browser experience should not require someone to think like a software engineer.

A small amount of scripting is part of the design. This is not a promise of a no-code rule builder. Familiar dice notation, worked examples, and optional LLM assistance are the bridge between a rule described in words and an executable model.

## Calculate the possibilities, not a pile of trial rolls

The core approach is to calculate the possible results of a mechanic and their chances, rather than roll it randomly thousands of times and estimate the answer. Re-running the same model should not change the answer because of sampling luck.

This is the intention behind the word **exact** in the original plans. It needs a practical qualification: the implementation uses finite-precision numbers, and a rule that could continue forever, such as an endlessly exploding die, needs an explicit finite model or approximation. The [mathematical design](../design/probability-engine.md) explains that distinction. The product should not imply that a bounded approximation is the unlimited rule.

A full spread of results is often more useful than one average. Two attacks can have the same average damage but very different risks of doing almost nothing or doing a great deal. The application therefore retains distributions and named outcomes as well as single success probabilities.

## Keep the explanation with the calculation

The main design choice is a **single `.dice` document** that contains both:

- ordinary writing explaining the rule, assumptions, and conclusions; and
- executable examples that calculate the results.

Running the document produces a **report**: the explanation stays in reading order, and the results appear alongside the relevant calculation. A reader should be able to follow the analysis from top to bottom without guessing which detached result belongs to which paragraph.

This matters because a probability dump rarely explains a design decision. “The chance is 65%” is less useful than an explanation of which character, bonus, target, and interpretation of success produced that number.

One source document also avoids keeping an explanation in one file and a copied script in another. The same principle applies to the application’s own tutorials and recipes.

## One Run means the whole document

The intended rhythm is:

**Describe a rule → change the example → Run → read the report → compare or refine.**

Run evaluates the whole document in one go. Earlier calculations and definitions are available to later ones in that run. There is no “run this cell”, “run up to the cursor”, or reactive notebook execution model.

The original rationale is that normal examples are quick enough to run as a whole, so per-cell execution would add complexity without enough benefit. This is a speed goal, not a measured promise that every possible script finishes within a fixed number of milliseconds.

The presentation borrows from a notebook, but execution is closer to rebuilding a report. The aim is a coherent analysis, not a screen full of independently run fragments.

## Teach through examples people recognise

The learning path should begin with a fair die and gradually introduce sums, bonuses, target numbers, pools, success counts, and named outcomes. More involved recipes should use familiar tabletop situations: ability scores, advantage, partial successes, exploding dice, and save-for-half damage.

Lessons should explain **what a rule means at the table before naming the programming operation**. Someone adapting a recipe should be able to identify the values to change and the assumptions to check.

Tutorials and cookbook recipes are themselves runnable `.dice` documents. They are not a separate prose product with code snippets that can drift away from the engine. The [learning-system design](../design/learning-and-llm.md) explains how the same source is read on the website, run in the playground, and checked automatically.

## Let people use and keep their work

The browser is the main entry point. Calculations run locally in the browser; using the engine does not require a runtime application server. The project remains open source under MIT and can be modified and deployed as a static site.

The command-line tool is a second way to evaluate the same documents, check examples automatically, and render HTML reports. Editor integration remains a useful path for people who prefer working in their own editor.

Readable reports are meant to help people communicate their designs. Static HTML rendering is a recorded part of that design. A polished sharing service, PDF export, and image-upload storage are **not** settled requirements just because sharing is valuable.

## What the application is not trying to be

The recorded scope excludes:

- a random dice roller for resolving live play;
- a general-purpose statistics or scientific-notebook environment;
- a generic programming IDE as the product’s destination;
- separate prose-and-code sidecar files or a JSON notebook format;
- partial or reactive cell execution;
- a hosted account, real-time collaboration, or multi-user editing service;
- a native mobile application.

An editor and a file drawer are useful supporting tools, not the main product promise. The single-document model does not itself require removal of the multi-file workspace.

The old material did not establish a business model, paid tiers, growth targets, or a validated competitive claim against other odds calculators. These should not appear as commitments in the extracted requirements.

## What success looks like

A designer can open a worked example, recognise the rule it models, change a bonus or die count, and run it once. The resulting report lets them explain both the odds and the assumptions to someone else. The same document works through the browser and command-line engine, and the published lesson is a rendering of that source.

The original brief suggested measuring time to first useful insight, repeat use, and community-shared recipes. Those are ideas for evaluating the product, not established targets or requirements to add analytics.

---

**Basis:** [source map](../design/source-map.md), S01–S07. Audience emphasis is from the current request. Numerical qualifications are from the existing engine, not from a new product decision.
