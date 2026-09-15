//! Numerical and publication acceptance checks for the rewrite pilots.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

fn eval(source: &str) -> Vec<OutputEntry> {
    eval_program("pilot.dice", source, EvalProgramOptions::default())
        .unwrap_or_else(|error| panic!("pilot evaluation: {error:#}"))
        .outputs
}

fn probability(source: &str) -> f64 {
    match eval(source)
        .last()
        .unwrap_or_else(|| panic!("missing output"))
    {
        OutputEntry::Prob { value, .. } => *value,
        other => panic!("expected probability: {other:?}"),
    }
}

#[test]
fn pilot_exercise_answers_and_small_counts() {
    for (expression, expected) in [
        ("(d(6) + d(6)).pmf(7)", 6.0 / 36.0),
        ("(d(6) + d(6)).pmf(2)", 1.0 / 36.0),
        ("(d(4) + d(4)).pmf(5)", 1.0 / 4.0),
        ("(d(6) + d(6) + 2).p_ge(8)", 26.0 / 36.0),
        ("(d(8) + 2).p_ge(7)", 0.5),
        ("(d(8) + 2).pmf(7)", 0.125),
        ("(d(8) + 2).p_ge(3)", 1.0),
        ("(d(8) + 2).p_ge(11)", 0.0),
    ] {
        assert!(
            (probability(&format!("output(\"check\", {expression})")) - expected).abs() < 1e-10
        );
    }
}

#[test]
fn cairn_default_categories_and_death_boundary() {
    let source = include_str!("../docs/cookbook/cairn-blood-elk.dice");
    for (label, expected) in [
        ("NO_EFFECT", 1.0 / 4.0),
        ("HIT_PROTECTION_LOSS", 3.0 / 8.0),
        ("SCAR", 1.0 / 8.0),
        ("STR_DOWN_OK", 19.0 / 160.0),
        ("CRITICAL_DAMAGE", 21.0 / 160.0),
        ("DEATH", 0.0),
    ] {
        let query = format!("{source}\n```dice\noutput(\"check\", out.pmf(\"{label}\"))\n```\n");
        assert!((probability(&query) - expected).abs() < 1e-10, "{label}");
    }
    for (strength, expected) in [(1, 0.25), (2, 0.125), (3, 0.0)] {
        let changed = source.replace("STR = 11", &format!("STR = {strength}"));
        let query = format!("{changed}\n```dice\noutput(\"death\", out.pmf(\"DEATH\"))\n```\n");
        assert!((probability(&query) - expected).abs() < 1e-10);
    }
    for (old, new) in [
        ("HP = 4", "HP = 0"),
        ("ARMOR = 2", "ARMOR = 4"),
        ("STR = 11", "STR = 0"),
    ] {
        assert!(eval_program(
            "invalid.dice",
            &source.replace(old, new),
            EvalProgramOptions::default()
        )
        .is_err());
    }
}

#[test]
fn single_fence_loop_outputs_follow_source_and_precede_explanation() {
    use dice_playground::engine::{render_literate_document, WeaveOptions};
    let source = "# Staged report\n\n```dice\ndef report(sides):\n    output(\"die\", d(sides))\nfor sides in range(4, 7):\n    report(sides)\n```\n\nAfter all results.\n";
    let html = render_literate_document("staged.dice", source, WeaveOptions::default()).unwrap();
    assert_eq!(html.matches("data-dice-output=\"die\"").count(), 3);
    let code = html.find("<code class=\"language-dice\">").unwrap();
    let first_output = html.find("data-dice-output=").unwrap();
    let last_output = html.rfind("data-dice-output=").unwrap();
    assert!(code < first_output);
    assert!(last_output < html.find("After all results.").unwrap());
}
