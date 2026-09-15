//! The supplied Brindlewood procedure: eligibility is not the roll modifier.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

const RECIPE: &str = include_str!("../docs/cookbook/brindlewood-bay-theorize.dice");

fn source(complexity: i32, gathered: i32, accounted: i32) -> String {
    RECIPE
        .replace("complexity = 6", &format!("complexity = {complexity}"))
        .replace(
            "gathered_clues = 6",
            &format!("gathered_clues = {gathered}"),
        )
        .replace(
            "accounted_clues = 6",
            &format!("accounted_clues = {accounted}"),
        )
}

#[test]
fn all_bands_and_nested_questions_match_independent_face_counts() {
    for complexity in 6..=8 {
        let minimum = (complexity + 1) / 2;
        for gathered in [minimum, minimum + 2, complexity + 4] {
            for accounted in 0..=gathered {
                let result = eval_program(
                    "theorize.dice",
                    &source(complexity, gathered, accounted),
                    EvalProgramOptions::default(),
                )
                .unwrap();
                assert_eq!(result.outputs.len(), 2);
                let OutputEntry::Outcomes { entries, .. } = &result.outputs[0] else {
                    panic!("expected categories");
                };
                let mut counts = [0_u32; 4];
                for first in 1..=6 {
                    for second in 1..=6 {
                        let total = first + second + accounted - complexity;
                        let index = if total <= 6 {
                            0
                        } else if total <= 9 {
                            1
                        } else if total <= 11 {
                            2
                        } else {
                            3
                        };
                        counts[index] += 1;
                    }
                }
                for (label, count) in [
                    "Incorrect",
                    "Correct with complications",
                    "Clean, without revelation",
                    "Clean plus conspiracy revelation",
                ]
                .iter()
                .zip(counts)
                {
                    let probability = entries
                        .iter()
                        .find(|(name, _)| name == label)
                        .map_or(0.0, |(_, p)| *p);
                    assert!(
                        (probability - f64::from(count) / 36.0).abs() < 1e-10,
                        "C={complexity}, gathered={gathered}, accounted={accounted}: {label}"
                    );
                }
                let OutputEntry::Table { entries, .. } = &result.outputs[1] else {
                    panic!("expected queries");
                };
                assert_eq!(entries.len(), 3);
                for ((label, actual), (expected_label, count)) in entries.iter().zip([
                    ("Any correct theory (7+)", counts[1] + counts[2] + counts[3]),
                    ("Clean or better (10+)", counts[2] + counts[3]),
                    ("Conspiracy revelation (12+)", counts[3]),
                ]) {
                    assert_eq!(label, expected_label);
                    assert!((actual - f64::from(count) / 36.0).abs() < 1e-10);
                }
            }
        }
    }
}

#[test]
fn eligibility_rounds_up_and_rejects_invalid_relationships() {
    for (complexity, gathered, accounted) in [
        (6, 2, 2),
        (7, 3, 3),
        (8, 3, 3),
        (6, 6, -1),
        (6, 6, 7),
        (5, 6, 6),
        (9, 6, 6),
    ] {
        assert!(eval_program(
            "invalid.dice",
            &source(complexity, gathered, accounted),
            EvalProgramOptions::default()
        )
        .is_err());
    }
    for (old, new) in [
        ("complexity = 6", "complexity = 6.5"),
        ("gathered_clues = 6", "gathered_clues = 6.5"),
        ("accounted_clues = 6", "accounted_clues = 6.5"),
    ] {
        assert!(eval_program(
            "invalid.dice",
            &RECIPE.replace(old, new),
            EvalProgramOptions::default()
        )
        .is_err());
    }
    for (complexity, gathered, accounted) in [(6, 3, 3), (7, 4, 4), (8, 4, 4), (7, 5, 4)] {
        assert!(eval_program(
            "eligible.dice",
            &source(complexity, gathered, accounted),
            EvalProgramOptions::default()
        )
        .is_ok());
    }
}

#[test]
fn unaccounted_clues_do_not_change_odds_once_eligible() {
    let first = eval_program(
        "first.dice",
        &source(7, 4, 4),
        EvalProgramOptions::default(),
    )
    .unwrap();
    let second = eval_program(
        "second.dice",
        &source(7, 5, 4),
        EvalProgramOptions::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(first.outputs).unwrap(),
        serde_json::to_value(second.outputs).unwrap()
    );
}
