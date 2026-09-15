//! Numeric answer checks for the advanced course's transfer exercises.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

#[test]
fn transfer_probabilities_have_independent_small_case_answers() {
    for (expression, expected) in [
        (
            "(die_faces([-1,0,1]) + die_faces([-1,0,1])).pmf(0)",
            1.0 / 3.0,
        ),
        ("(d(4) + d(4)).pmf(5)", 1.0 / 4.0),
        ("(d(4) * 2).pmf(5)", 0.0),
        (
            "((dice_pool(1,6) + dice_pool(1,8)).order_stat(1) - 2).clamp(0,8).pmf(0)",
            1.0 / 12.0,
        ),
        ("d(6).ignore([1,2]).pmf(0)", 1.0 / 3.0),
        ("d(6).remove([1,2]).pmf(3)", 1.0 / 4.0),
        ("d(6).keep([5,6]).pmf(5)", 0.5),
        ("(keep_highest(2,6,1) - 2).clamp(0,6).p_ge(4)", 11.0 / 36.0),
        ("dice_pool(2,6).count([5,6]).pmf(2)", 1.0 / 9.0),
    ] {
        let result = eval_program(
            "exercise.dice",
            &format!("output(\"answer\", {expression})"),
            EvalProgramOptions::default(),
        )
        .unwrap();
        let OutputEntry::Prob { value, .. } = &result.outputs[0] else {
            panic!("expected probability")
        };
        assert!((*value - expected).abs() < 1e-10, "{expression}");
    }
}

#[test]
fn transfer_means_and_cap_bounds() {
    for (expression, expected_mean, min, max) in [
        ("keep_highest(4,6,2)", 9.344135802469136, 2, 12),
        ("d(6) // 2", 1.5, 0, 3),
        ("explode(d(6),max_depth=0)", 3.5, 1, 6),
        ("open_ended_d100(max_chain=0)", 50.5, -99, 200),
        ("open_ended_d100(max_chain=1)", 50.5, -199, 300),
    ] {
        let result = eval_program(
            "exercise.dice",
            &format!("output(\"answer\", {expression})"),
            EvalProgramOptions::default(),
        )
        .unwrap();
        let OutputEntry::DieRoll { mean, entries, .. } = &result.outputs[0] else {
            panic!("expected distribution")
        };
        assert!((*mean - expected_mean).abs() < 1e-9, "{expression}");
        assert_eq!(entries.first().unwrap().0, min);
        assert_eq!(entries.last().unwrap().0, max);
    }
}

#[test]
fn clue_comparison_range_tracks_complexity() {
    let source = include_str!("../docs/tutorial/31-report-labels.dice")
        .replace("complexity = 6", "complexity = 7");
    let result = eval_program("clues.dice", &source, EvalProgramOptions::default()).unwrap();
    let OutputEntry::Table { entries, .. } = &result.outputs[0] else {
        panic!("expected table")
    };
    assert_eq!(entries.len(), 8);
    for (index, ((label, probability), numerator)) in entries
        .iter()
        .zip([0.0, 1.0, 3.0, 6.0, 10.0, 15.0, 21.0, 26.0])
        .enumerate()
    {
        assert_eq!(label, &format!("{} clues: clean or better", index + 4));
        assert!((*probability - numerator / 36.0).abs() < 1e-10);
    }
}
