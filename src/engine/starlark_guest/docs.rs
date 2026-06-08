use starlark::collections::SmallMap;
use starlark::docs::markdown::render_doc_item_no_link;
use starlark::docs::{DocItem, DocModule, DocType};
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

pub fn dice_pool_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkDicePool>()
}

const REFERENCE_INTRO: &str = r#"This reference lists everything built into `.dice` scripts beyond basic Starlark (variables, `for` loops, lists). New here? Work through the [tutorial](../README.md#tutorial) first—it introduces notation like `2d6` and `4d6dl1`, which the playground expands into the functions below.

**Naming:** face matching (`keep` / `remove` / `convert` / `ignore`), `count`, and pool `p_*` methods follow [API conventions](api-conventions.md).

## Core ideas

**`DieRoll`** — a finished numeric roll (or total) with exact chances for each possible result. Example: `2d6` is a `DieRoll`; so is `4d6dl1`. Use **`output("name", roll)`** to print its table in the playground.

**`DicePool`** — several dice rolled together but **not** added yet. Use this when the rule cares about *individual* faces (highest die, count successes, Blades-style pools). Call **`.sum()`** on the pool when you only need the total.

**`Outcomes`** — chances for **named** outcome bands (miss / partial / hit, crit fail / success, and so on) instead of raw numbers.

## Combining rolls (operators)

| You write | Meaning at the table |
|-----------|----------------------|
| `a + b` | Two **independent** rolls added together (e.g. `1d6 + 1d6` or `2d6 + 1d4`). |
| `roll + 5` | Flat **modifier** added to every outcome of `roll` (same as `shift(roll, 5)`). |
| `roll - 3` | Subtract 3 from every outcome. |
| `a - b` | Independent rolls subtracted (less common; niche mechanics). |
| `roll * 10` | Multiply **each** outcome (e.g. tens die reading). |
| `roll // 2` | **Halve** each outcome, round down (typical “half damage on save”). |

Dice notation (`1d20`, `3d6kh2`, …) is sugar for these functions—see the [dice notation lesson](../tutorial/05-dice-notation.md).

## Builtin functions (by topic)

"#;

/// `as_type` builtins render as a whole `DieRoll` type doc; keep only the constructor section.
fn truncate_type_doc_to_constructor(name: &str, md: &str) -> String {
    let needle = format!("\n## {name}.");
    if let Some(idx) = md.find(&needle) {
        return md[..idx].trim().to_string();
    }
    let needle2 = format!("\n## {name}_");
    if let Some(idx) = md.find(&needle2) {
        return md[..idx].trim().to_string();
    }
    md.trim().to_string()
}

/// Starlark's markdown renderer mis-labels some `as_type` builtins; normalize for readers.
fn polish_builtin_markdown(name: &str, md: &str) -> String {
    let mut s = md.trim().to_string();
    let bad_heading = format!("# `{name}` type");
    if let Some(rest) = s.strip_prefix(&bad_heading) {
        s = format!("## {name}{rest}");
    }
    s = s.replace(&format!("def `{name}` type"), &format!("def {name}"));
    if !s.starts_with(&format!("## {name}")) {
        s = format!("## {name}\n\n{s}");
    }
    s.push_str("\n\n---\n\n");
    s
}

fn append_members(
    out: &mut String,
    section_title: &str,
    section_blurb: &str,
    order: &[&str],
    members: &SmallMap<String, DocItem>,
) {
    out.push_str(&format!("### {section_title}\n\n{section_blurb}\n\n"));
    for name in order {
        let Some(item) = members.get(*name) else {
            continue;
        };
        let raw = match item {
            DocItem::Member(member) => {
                render_doc_item_no_link(name, &DocItem::Member(member.clone()))
            }
            DocItem::Type(ty) => {
                let full = render_doc_item_no_link(name, &DocItem::Type(ty.clone()));
                truncate_type_doc_to_constructor(name, &full)
            }
            _ => continue,
        };
        out.push_str(&polish_builtin_markdown(name, &raw));
    }
}

fn append_type_methods(
    out: &mut String,
    type_title: &str,
    type_blurb: &str,
    order: &[&str],
    ty: &DocType,
) {
    out.push_str(&format!("# {type_title}\n\n{type_blurb}\n\n"));
    for name in order {
        let Some(member) = ty.members.get(*name) else {
            continue;
        };
        let raw = render_doc_item_no_link(name, &DocItem::Member(member.clone()));
        out.push_str(&polish_builtin_markdown(name, &raw));
    }
}

/// Render dice stdlib + `DieRoll` method reference as Markdown.
pub fn render_stdlib_reference_markdown() -> String {
    let stdlib = dice_stdlib_docs();
    let mut out = String::from("# Dice standard library\n\n");
    out.push_str(REFERENCE_INTRO);

    append_members(
        &mut out,
        "Building dice and totals",
        "Start here for ordinary dice, custom faces, and summed pools.",
        &[
            "d",
            "die_faces",
            "dice_pool",
            "sum",
            "drop_lowest",
            "drop_highest",
            "keep_highest",
            "keep_lowest",
            "explode",
            "open_ended_d100",
            "shift",
            "through",
            "at_most",
            "at_least",
        ],
        &stdlib.members,
    );

    append_members(
        &mut out,
        "Inclusive ranges",
        "Integer bands for face filters and bucketing. In `.dice` scripts you can also write `6..94`, `..6`, and `10..` (inclusive endpoints).",
        &["through", "at_most", "at_least"],
        &stdlib.members,
    );

    append_members(
        &mut out,
        "Pool rules (faces still matter)",
        "These need a `DicePool` from `dice_pool` before you total the dice.",
        &[
            "count",
            "order_stat",
            "middle_of",
            "pool_map",
            "success_pool",
        ],
        &stdlib.members,
    );

    append_members(
        &mut out,
        "Named outcomes",
        "Turn numeric totals or special roll rules into labeled results.",
        &["scale", "bucket", "classify", "joint_classify"],
        &stdlib.members,
    );

    append_members(
        &mut out,
        "Showing results",
        "Always end scripts with `output` so the playground prints tables and charts.",
        &["output", "prob_table"],
        &stdlib.members,
    );

    append_type_methods(
        &mut out,
        "DieRoll methods",
        "Ask questions about a numeric `DieRoll` after you build it (often inside `output(..., roll.p_ge(15))`).",
        &[
            "mean",
            "pmf",
            "p_ge",
            "cdf",
            "clamp",
            "support_size",
            "keep",
            "remove",
            "convert",
            "ignore",
            "bucket",
        ],
        &die_roll_type_docs(),
    );

    append_type_methods(
        &mut out,
        "DicePool methods",
        "Face filters (`keep` / `remove` / `convert` / `ignore`), match counts (`count`), pool match probabilities (`p_any` / `p_none` / `p_at_least`), or total the pool. See [API conventions](../references/api-conventions.md).",
        &[
            "sum",
            "keep",
            "remove",
            "convert",
            "ignore",
            "count",
            "order_stat",
            "middle_of",
            "p_any",
            "p_none",
            "p_at_least",
            "bucket",
        ],
        &dice_pool_type_docs(),
    );

    append_type_methods(
        &mut out,
        "Outcomes methods",
        "Query named outcome bands (PbtA moves, graded success, etc.).",
        &["pmf", "p_at_least", "p_at_most"],
        &outcomes_type_docs(),
    );

    append_type_methods(
        &mut out,
        "Scale methods",
        "Build ordered outcome labels and optional numeric bands after `scale()`.",
        &["step"],
        &scale_type_docs(),
    );

    out
}

/// Documentation for `Scale` type methods (`with`).
pub fn scale_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkScale>()
}

#[cfg(test)]
mod tests {
    use starlark::docs::markdown::render_doc_item_no_link;
    use starlark::docs::{DocItem, DocMember};

    use super::*;

    #[test]
    fn dice_stdlib_docs_lists_expected_symbols() {
        let docs = dice_stdlib_docs();
        for name in [
            "d",
            "die_faces",
            "explode",
            "open_ended_d100",
            "dice_pool",
            "sum",
            "count",
            "order_stat",
            "middle_of",
            "pool_map",
            "success_pool",
            "drop_lowest",
            "drop_highest",
            "keep_highest",
            "keep_lowest",
            "shift",
            "output",
            "prob_table",
            "scale",
            "bucket",
            "classify",
            "joint_classify",
            "through",
            "at_most",
            "at_least",
        ] {
            assert!(
                docs.members.contains_key(name),
                "missing documented symbol {name}"
            );
        }
    }

    #[test]
    fn d_function_doc_renders_with_summary() {
        let docs = dice_stdlib_docs();
        let item = docs.members.get("d").expect("d should be documented");
        let md = render_doc_item_no_link("d", item);
        assert!(md.contains("fair die"));
    }

    #[test]
    fn die_roll_pmf_doc_renders() {
        let ty = die_roll_type_docs();
        let member = ty.members.get("pmf").expect("pmf method");
        let DocMember::Function(_) = member else {
            panic!("pmf should be a function");
        };
        let md = render_doc_item_no_link("pmf", &DocItem::Member(member.clone()));
        assert!(md.contains("exactly"));
    }

    #[test]
    fn reference_markdown_includes_designer_intro_and_d_docs() {
        let md = render_stdlib_reference_markdown();
        assert!(md.contains("Core ideas"));
        assert!(md.contains("Building dice and totals"));
        assert!(md.contains("fair die"));
        assert!(md.contains("output"));
    }
}
