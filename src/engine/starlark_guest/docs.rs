//! Script-writer reference assembled from Starlark documentation metadata.
//!
//! Prose and runnable `.dice` examples belong beside the exposed functions. This
//! module owns only the introduction, topic order, and presentation. Coverage tests
//! require each registered builtin and exposed method to appear exactly once.

use starlark::collections::SmallMap;
use starlark::docs::markdown::render_doc_item_no_link;
use starlark::docs::{DocItem, DocMember, DocModule, DocType};
use starlark::environment::GlobalsBuilder;

use super::dice_pool_value::StarlarkDicePool;
use super::die_roll_value::StarlarkDieRoll;
use super::eval::dice_globals;
use super::outcomes_value::StarlarkOutcomes;
use super::scale_value::StarlarkScale;

/// Documentation for the full eval environment (Starlark standard library + dice builtins).
pub fn full_environment_docs() -> DocModule {
    dice_globals().documentation()
}

/// Documentation for dice-only globals (human-facing function reference).
pub fn dice_stdlib_docs() -> DocModule {
    GlobalsBuilder::new()
        .with(super::eval::dice_module)
        .build()
        .documentation()
}

/// Documentation for `DieRoll` type methods (`pmf`, `cdf`, `p_ge`, etc.).
pub fn die_roll_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkDieRoll>()
}

/// Documentation for `Outcomes` type methods (`pmf`, `p_at_least`, `p_at_most`).
pub fn outcomes_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkOutcomes>()
}

/// Documentation for `DicePool` operations and match probabilities.
pub fn dice_pool_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkDicePool>()
}

/// Documentation for building a `Scale` with `.step(...)`.
pub fn scale_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkScale>()
}

const REFERENCE_INTRO: &str = r#"This reference helps you turn a tabletop rule into a `.dice` script. You do not need to know Rust or Starlark to use the examples. For a first walk-through, start with the [tutorial](../tutorial/index.html).

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

"#;

type BuiltinSection = (&'static str, &'static str, &'static [&'static str]);

const BUILTIN_SECTIONS: &[BuiltinSection] = &[
    (
        "Building dice and totals",
        "Start here for ordinary dice, custom faces, and totals that keep or drop dice.",
        &[
            "d", "die_faces", "dice_pool", "sum", "drop_lowest", "drop_highest",
            "keep_highest", "keep_lowest", "explode", "open_ended_d100", "shift",
        ],
    ),
    (
        "Inclusive ranges",
        "Describe which numbers match. Both endpoints count; these ranges are not rolls by themselves.",
        &["through", "at_most", "at_least"],
    ),
    (
        "Pool rules (faces still matter)",
        "Use these when a rule examines individual dice rather than just the total.",
        &["count", "order_stat", "middle_of", "pool_map", "success_pool"],
    ),
    (
        "Named outcomes",
        "Turn numbers or custom rules into labels such as miss, partial, and hit.",
        &["scale", "bucket", "classify", "joint_classify"],
    ),
    (
        "Showing results",
        "Use output to put a result in the report, and prob_table to compare several chances.",
        &["output", "prob_table"],
    ),
];

struct MethodSection {
    name: &'static str,
    blurb: &'static str,
    order: &'static [&'static str],
    docs: DocType,
}

fn method_sections() -> [MethodSection; 4] {
    [
        MethodSection {
            name: "DieRoll",
            blurb: "Use these on a numeric roll or total, such as `roll = dice_pool(2, 6).sum()`. Write `roll.p_ge(7)`, not `DieRoll.p_ge(7)`.",
            order: &["mean", "pmf", "p_ge", "cdf", "clamp", "support_size", "keep", "remove", "convert", "ignore", "bucket"],
            docs: die_roll_type_docs(),
        },
        MethodSection {
            name: "DicePool",
            blurb: "Use these on a pool, such as `pool = dice_pool(3, 6)`. Face operations work on each die separately; `.sum()` turns the pool into a total.",
            order: &["sum", "keep", "remove", "convert", "ignore", "count", "order_stat", "middle_of", "p_any", "p_none", "p_at_least", "bucket"],
            docs: dice_pool_type_docs(),
        },
        MethodSection {
            name: "Outcomes",
            blurb: "Use these on named results returned by bucket or classify. Higher and lower mean later and earlier on the scale you built.",
            order: &["pmf", "p_at_least", "p_at_most"],
            docs: outcomes_type_docs(),
        },
        MethodSection {
            name: "Scale",
            blurb: "Use `.step(...)` on a scale to build your ladder of named results.",
            order: &["step"],
            docs: scale_type_docs(),
        },
    ]
}

/// Extract constructors as functions before rendering, so `as_type` metadata
/// cannot accidentally repeat all the returned type's methods under a builtin.
fn builtin_member(item: &DocItem) -> Option<DocMember> {
    match item {
        DocItem::Member(member) => Some(member.clone()),
        DocItem::Type(ty) => ty.constructor.clone().map(DocMember::Function),
        DocItem::Module(_) => None,
    }
}

fn append_member(out: &mut String, name: &str, member: &DocMember) {
    let raw = render_doc_item_no_link(name, &DocItem::Member(member.clone()));
    // Starlark emits a level-two entry heading and level-four detail headings.
    // Nest the entry under our level-two topic without inspecting escaped names,
    // signatures, code fences, or prose inside the body.
    let raw = raw.trim();
    if let Some(body) = raw.strip_prefix("## ") {
        out.push_str("### ");
        out.push_str(body);
    } else {
        out.push_str(raw);
    }
    out.push_str("\n\n");
}

fn append_builtins(out: &mut String, members: &SmallMap<String, DocItem>) {
    for (title, blurb, order) in BUILTIN_SECTIONS {
        out.push_str(&format!("## {title}\n\n{blurb}\n\n"));
        for name in *order {
            if let Some(member) = members.get(*name).and_then(builtin_member) {
                append_member(out, name, &member);
            }
        }
    }
}

/// Render the script-writer reference from API metadata and curated topic ordering.
///
/// ```rust
/// use dice_playground::engine::render_stdlib_reference_markdown;
/// let markdown = render_stdlib_reference_markdown();
/// assert!(markdown.starts_with("# Dice standard library"));
/// ```
pub fn render_stdlib_reference_markdown() -> String {
    let mut out = String::from("# Dice standard library\n\n");
    out.push_str(REFERENCE_INTRO);
    append_builtins(&mut out, &dice_stdlib_docs().members);
    for section in method_sections() {
        out.push_str(&format!(
            "## {} methods\n\n{}\n\n",
            section.name, section.blurb
        ));
        for name in section.order {
            if let Some(member) = section.docs.members.get(*name) {
                append_member(&mut out, &format!("{}.{name}", section.name), member);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

    use super::*;

    fn assert_exact_coverage<'a>(listed: &[&str], actual: impl Iterator<Item = &'a str>) {
        let unique: BTreeSet<_> = listed.iter().copied().collect();
        assert_eq!(unique.len(), listed.len(), "duplicate reference entries");
        assert_eq!(
            unique,
            actual.collect(),
            "update the reference's curated API list"
        );
    }

    #[test]
    fn every_builtin_is_renderable_and_listed_exactly_once() {
        let docs = dice_stdlib_docs();
        let listed: Vec<_> = BUILTIN_SECTIONS
            .iter()
            .flat_map(|(_, _, names)| *names)
            .copied()
            .collect();
        assert_exact_coverage(&listed, docs.members.keys().map(String::as_str));
        for (name, item) in &docs.members {
            assert!(
                builtin_member(item).is_some(),
                "cannot render builtin {name}"
            );
        }
    }

    #[test]
    fn every_type_method_is_listed_exactly_once() {
        for section in method_sections() {
            assert_exact_coverage(
                section.order,
                section.docs.members.keys().map(String::as_str),
            );
        }
        // Every type exposed through an as_type constructor must either have a
        // method section or have no methods (IntBand and ProbTable currently).
        let covered: BTreeSet<_> = method_sections()
            .iter()
            .map(|s| s.docs.ty.to_string())
            .collect();
        for (_, item) in &dice_stdlib_docs().members {
            if let DocItem::Type(ty) = item {
                assert!(
                    ty.members.is_empty() || covered.contains(&ty.ty.to_string()),
                    "missing method section for {}",
                    ty.ty
                );
            }
        }
    }

    #[test]
    fn constructor_rendering_does_not_include_type_methods() {
        let docs = dice_stdlib_docs();
        for name in ["d", "die_faces", "dice_pool", "open_ended_d100", "pool_map"] {
            let member =
                builtin_member(docs.members.get(name).expect("builtin")).expect("constructor");
            let mut md = String::new();
            append_member(&mut md, name, &member);
            assert!(md.contains(&format!("def {name}(")), "{md}");
            assert_eq!(
                md.lines().filter(|l| l.starts_with("### ")).count(),
                1,
                "{md}"
            );
            assert!(!md.contains(&format!("def {name}.")), "{md}");
            assert!(!md.contains(" type"), "{md}");
        }
    }

    #[test]
    fn reference_headings_are_unique_and_nested() {
        let md = render_stdlib_reference_markdown();
        let mut headings = BTreeSet::new();
        let mut heading = None;
        let mut title_count = 0;
        let mut entry_count = 0;
        let mut previous_level = 0;
        for event in Parser::new(&md) {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let number = level as usize;
                    assert!(number <= previous_level + 1, "skipped heading level");
                    previous_level = number;
                    heading = Some((level, String::new()));
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some((_, title)) = &mut heading {
                        title.push_str(&text);
                    }
                }
                Event::End(TagEnd::Heading(_)) => {
                    let (level, title) = heading.take().expect("heading start");
                    if level == HeadingLevel::H1 {
                        title_count += 1;
                    }
                    if level == HeadingLevel::H3 {
                        entry_count += 1;
                    }
                    if level != HeadingLevel::H4 {
                        assert!(headings.insert(title), "duplicate topic or API heading");
                    }
                }
                _ => {}
            }
        }
        assert_eq!(title_count, 1);
        let expected = dice_stdlib_docs().members.len()
            + method_sections()
                .iter()
                .map(|s| s.docs.members.len())
                .sum::<usize>();
        assert_eq!(entry_count, expected);
    }

    #[test]
    fn api_entries_have_prose_examples_and_parameter_help() {
        let globals = dice_stdlib_docs();
        let mut entries: Vec<_> = globals
            .members
            .iter()
            .map(|(name, item)| (name.clone(), builtin_member(item).expect("builtin")))
            .collect();
        for section in method_sections() {
            entries.extend(
                section
                    .docs
                    .members
                    .into_iter()
                    .map(|(name, member)| (format!("{}.{name}", section.name), member)),
            );
        }
        for (name, member) in entries {
            let DocMember::Function(function) = member else {
                panic!("expected function: {name}");
            };
            let docs = function.docs.as_ref().expect("description");
            assert!(!docs.summary.is_empty(), "missing summary: {name}");
            let md = render_doc_item_no_link(
                &name,
                &DocItem::Member(DocMember::Function(function.clone())),
            );
            assert!(md.contains("```dice\n"), "missing runnable example: {name}");
            for param in function
                .params
                .regular_params()
                .chain(function.params.args.iter())
                .chain(function.params.kwargs.iter())
            {
                assert!(
                    param.docs.as_ref().is_some_and(|d| !d.summary.is_empty()),
                    "missing argument help: {name}.{}",
                    param.name
                );
            }
        }
    }
}
