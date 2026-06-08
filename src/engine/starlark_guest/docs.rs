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
    GlobalsBuilder::new().with(super::eval::dice_module).build().documentation()
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

/// Render dice stdlib + `Dist` method reference as Markdown.
pub fn render_stdlib_reference_markdown() -> String {
    let mut out = String::from("# Dice standard library\n\n");
    out.push_str(&render_doc_item_no_link(
        "dice_stdlib",
        &DocItem::Module(dice_stdlib_docs()),
    ));
    out.push_str("\n\n# Dist type\n\n");
    out.push_str(&render_doc_item_no_link(
        "Dist",
        &DocItem::Type(dist_type_docs()),
    ));
    out.push_str("\n\n# RollPool type\n\n");
    out.push_str(&render_doc_item_no_link(
        "RollPool",
        &DocItem::Type(roll_pool_type_docs()),
    ));
    out.push_str("\n\n# LabelDist type\n\n");
    out.push_str(&render_doc_item_no_link(
        "LabelDist",
        &DocItem::Type(label_dist_type_docs()),
    ));
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
        let item = docs
            .members
            .get("d")
            .expect("d should be documented");
        let md = render_doc_item_no_link("d", item);
        assert!(md.contains("fair die"));
    }

    #[test]
    fn dist_pmf_doc_renders() {
        let ty = dist_type_docs();
        let member = ty
            .members
            .get("pmf")
            .expect("pmf method");
        let DocMember::Function(_) = member else {
            panic!("pmf should be a function");
        };
        let md = render_doc_item_no_link("pmf", &DocItem::Member(member.clone()));
        assert!(md.contains("P(X = value)"));
    }
}
