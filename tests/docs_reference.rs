use std::path::PathBuf;

use dice_playground::engine::{eval_source, render_stdlib_reference_markdown};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

#[test]
fn references_stdlib_md_matches_renderer() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/references/stdlib.md");
    let on_disk = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    let fresh = render_stdlib_reference_markdown();
    assert_eq!(
        on_disk, fresh,
        "docs/references/stdlib.md is out of date; run: make references"
    );
}

#[test]
fn every_reference_dice_example_runs_on_its_own() {
    let md = render_stdlib_reference_markdown();
    let mut code = None;
    let mut count = 0;
    for event in Parser::new(&md) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
                if language.as_ref() == "dice" =>
            {
                code = Some(String::new());
            }
            Event::Text(text) => {
                if let Some(source) = &mut code {
                    source.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(source) = code.take() {
                    count += 1;
                    let path = format!("reference-example-{count}.dice");
                    let result = eval_source(&path, &source)
                        .unwrap_or_else(|e| panic!("{path} failed:\n{source}\n{e:#}"));
                    assert!(
                        !result.outputs.is_empty(),
                        "{path} has no visible result:\n{source}"
                    );
                }
            }
            _ => {}
        }
    }
    // Per-entry coverage is also enforced against live metadata in docs.rs.
    assert!(
        count >= 52,
        "expected an introduction and an example for every API"
    );
}

#[test]
fn documented_invalid_inputs_report_errors() {
    for source in [
        "dice_pool(0, 6)",
        "d(6).keep(9)",
        "d(6).remove(through(1, 6))",
        "through(5, 2)",
        "dice_pool(2, 6).order_stat(0)",
        "dice_pool(2, 6).middle_of(3)",
        "bucket(d(6), scale().step(\"Only six\", through(6, 6)))",
        "bucket(d(6), scale().step(\"All\", through(1, 6))).pmf(\"Unknown\")",
    ] {
        assert!(
            eval_source("reference-error.dice", source).is_err(),
            "expected a documented error for {source}"
        );
    }
}

#[test]
fn reference_explanations_match_engine_behavior() {
    // Check the distinctions taught by the reference, not only snippet syntax.
    let source = r#"
def close(actual, expected):
    if abs(actual - expected) > 0.000000001:
        fail("Unexpected probability or average")

close(d(6).mean(), 3.5)
if (2d6).support_size() != 11:
    fail("Two d6 have eleven possible totals")
close(d(6).keep([5, 6]).pmf(5), 0.5)
close(d(6).remove(1).pmf(2), 0.2)
close(d(6).ignore(through(1, 4)).pmf(0), 4.0 / 6.0)
close(d(6).convert(6, 12).pmf(12), 1.0 / 6.0)
close(dice_pool(3, 6).keep([5, 6]).sum().mean(), 16.5)
close(dice_pool(3, 6).ignore(through(1, 4)).sum().mean(), 5.5)
close(dice_pool(3, 6).p_at_least(2, at_least(5)), 7.0 / 27.0)
close(dice_pool(3, 6).p_any(), 1.0)
close(dice_pool(3, 6).p_none(), 0.0)
close(dice_pool(3, 6).p_at_least(4), 0.0)
close(dice_pool(3, 6).p_at_least(0, 6), 1.0)
close(drop_lowest(2, 6, 2).mean(), keep_highest(2, 6, 1).mean())
close(drop_highest(2, 6, 2).mean(), keep_lowest(2, 6, 1).mean())
close(keep_highest(2, 6, 3).mean(), 7.0)
close(keep_lowest(2, 6, 0).pmf(0), 1.0)
close(explode(d(6), max_depth=0).mean(), 3.5)
close(explode(d(6), max_depth=2).pmf(18), 1.0 / 216.0)
close(open_ended_d100(max_chain=0).pmf(50), 0.01)

results = scale().step("Miss").step("Partial").step("Hit")
move = bucket(2d6, results, [6, 9])
close(move.pmf("Partial"), 15.0 / 36.0)
close(move.p_at_least("Partial"), 21.0 / 36.0)
close(move.p_at_most("Partial"), 30.0 / 36.0)

# A later, early=True step takes priority without moving its ladder rank.
results = scale().step("Normal", through(1, 20)).step("Critical", through(20, 20), early=True)
close(bucket(d(20), results).pmf("Critical"), 0.05)
"#;
    eval_source("reference-semantics.dice", source).expect("reference behavior");
}
