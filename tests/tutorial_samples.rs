//! Tutorial inventory, opening lessons, and preserved keep/drop regression oracles.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};
use std::path::PathBuf;

#[test]
fn tutorial_manifest_covers_all_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../docs/learning-content.json")).unwrap();
    let mut listed: Vec<_> = manifest["pages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["path"].as_str())
        .filter(|p| p.starts_with("tutorial/"))
        .map(str::to_owned)
        .collect();
    let mut actual: Vec<_> = std::fs::read_dir(root.join("docs/tutorial"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "dice"))
        .map(|p| format!("tutorial/{}", p.file_name().unwrap().to_string_lossy()))
        .collect();
    listed.sort();
    actual.sort();
    assert_eq!(listed, actual);
    assert_eq!(listed.len(), 32, "the complete core course");
}

#[test]
fn opening_lessons_have_checked_means() {
    for (source, expected) in [
        (include_str!("../docs/tutorial/01-one-die.dice"), vec![3.5]),
        (include_str!("../docs/tutorial/02-two-dice.dice"), vec![7.0]),
        (
            include_str!("../docs/tutorial/03-modifiers.dice"),
            vec![7.0, 9.0],
        ),
    ] {
        let result = eval_program("lesson.dice", source, EvalProgramOptions::default()).unwrap();
        let means: Vec<_> = result
            .outputs
            .iter()
            .filter_map(|entry| match entry {
                OutputEntry::DieRoll { mean, .. } => Some(*mean),
                _ => None,
            })
            .collect();
        assert_eq!(means.len(), expected.len());
        for (actual, expected) in means.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-10);
        }
    }
}

#[test]
fn keep_drop_equivalences_compare_whole_distributions() {
    let source = "drop_one = 4d6dl1\nkeep_three = 4d6kh3\ndrop_two = 4d6dl2\nkeep_two = 4d6kh2\noutput(\"drop one\", drop_one)\noutput(\"keep three\", keep_three)\noutput(\"drop two\", drop_two)\noutput(\"keep two\", keep_two)";
    let result = eval_program("equivalence.dice", source, EvalProgramOptions::default()).unwrap();
    for (a, b) in [(0, 1), (2, 3)] {
        match (&result.outputs[a], &result.outputs[b]) {
            (
                OutputEntry::DieRoll { entries: left, .. },
                OutputEntry::DieRoll { entries: right, .. },
            ) => {
                assert_eq!(left.len(), right.len());
                for ((lf, lp), (rf, rp)) in left.iter().zip(right) {
                    assert_eq!(lf, rf);
                    assert!((lp - rp).abs() < 1e-10);
                }
            }
            _ => panic!("expected distributions"),
        }
    }
}
