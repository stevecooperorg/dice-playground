use starlark::collections::SmallMap;
use starlark::docs::markdown::render_doc_item_no_link;
use starlark::docs::{DocItem, DocModule, DocType};
use starlark::environment::GlobalsBuilder;

use super::dist_value::StarlarkDist;
use super::eval::dice_globals;
use super::label_value::StarlarkLabelDist;
use super::pool_value::StarlarkRollPool;

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

/// Documentation for `Dist` type methods (`pmf`, `cdf`, `p_ge`, etc.).
pub fn dist_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkDist>()
}

/// Documentation for `LabelDist` type methods (`pmf`, `p_at_least`, `p_at_most`).
pub fn label_dist_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkLabelDist>()
}

pub fn roll_pool_type_docs() -> DocType {
    DocType::from_starlark_value::<StarlarkRollPool>()
}

const REFERENCE_INTRO: &str = r#"This reference lists everything built into `.dice` scripts beyond basic Starlark (variables, `for` loops, lists). New here? Work through the [tutorial](../README.md#tutorial) first—it introduces notation like `2d6` and `4d6dl1`, which the playground expands into the functions below.

## Core ideas

**`Dist`** — a finished numeric roll (or total) with exact chances for each possible result. Example: `2d6` is a `Dist`; so is `4d6dl1`. Use **`output("name", dist)`** to print its table in the playground.

**`RollPool`** — several dice rolled together but **not** added yet. Use this when the rule cares about *individual* faces (highest die, count successes, Blades-style pools). Call **`.sum()`** on the pool when you only need the total.

**`LabelDist`** — chances for **named** outcomes (miss / partial / hit, crit fail / success, and so on) instead of raw numbers.

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

/// `as_type` builtins render as a whole `Dist` type doc; keep only the constructor section.
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
            DocItem::Member(member) => render_doc_item_no_link(name, &DocItem::Member(member.clone())),
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

/// Render dice stdlib + `Dist` method reference as Markdown.
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
            "roll_pool",
            "pool",
            "sum",
            "drop_lowest",
            "drop_highest",
            "keep_highest",
            "keep_lowest",
            "explode",
            "shift",
        ],
        &stdlib.members,
    );

    append_members(
        &mut out,
        "Pool rules (faces still matter)",
        "These need a `RollPool` from `roll_pool` / `pool` before you total the dice.",
        &[
            "count_ge",
            "count_in",
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
        &["result_type", "bucket", "classify", "joint_classify"],
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
        "Dist methods",
        "Ask questions about a numeric `Dist` after you build it (often inside `output(..., roll.p_ge(15))`).",
        &["mean", "pmf", "p_ge", "cdf", "support_size"],
        &dist_type_docs(),
    );

    append_type_methods(
        &mut out,
        "RollPool methods",
        "Turn a pool into a single total when the rule no longer cares about separate dice.",
        &["sum"],
        &roll_pool_type_docs(),
    );

    append_type_methods(
        &mut out,
        "LabelDist methods",
        "Query named outcome bands (PbtA moves, graded success, etc.).",
        &["pmf", "p_at_least", "p_at_most"],
        &label_dist_type_docs(),
    );

    out
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
            "roll_pool",
            "pool",
            "sum",
            "count_ge",
            "count_in",
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
            "result_type",
            "bucket",
            "classify",
            "joint_classify",
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
    fn dist_pmf_doc_renders() {
        let ty = dist_type_docs();
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
