//! Checked numerical anchors for every new lesson and rewritten recipe.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};
use serde_json::Value;
use std::path::Path;

#[test]
fn published_examples_match_independent_anchors_and_normalize() {
    let fixtures: Value =
        serde_json::from_str(include_str!("../docs/learning-checks.json")).unwrap();
    for (path, checks) in fixtures.as_object().unwrap() {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("docs")
                .join(path),
        )
        .unwrap();
        let result = eval_program(path, &source, EvalProgramOptions::default())
            .unwrap_or_else(|e| panic!("{path}: {e:#}"));
        let json = serde_json::to_value(&result.outputs).unwrap();
        let entries = json.as_array().unwrap();
        let find = |name: &str| {
            entries
                .iter()
                .find(|e| e["name"] == name)
                .unwrap_or_else(|| panic!("{path}: missing {name}"))
        };
        for (kind, expected) in checks.as_object().unwrap() {
            for (name, value) in expected.as_object().unwrap() {
                let entry = find(name);
                match kind.as_str() {
                    "means" | "probabilities" => {
                        let field = if kind == "means" { "mean" } else { "value" };
                        let actual = entry[field].as_f64().unwrap();
                        assert!(
                            (actual - value.as_f64().unwrap()).abs() < 1e-9,
                            "{path}: {name}: {actual} != {value}"
                        );
                    }
                    "pmfs" | "outcomes" | "tables" => {
                        let rows = entry["entries"].as_array().unwrap();
                        for (label, probability) in value.as_object().unwrap() {
                            let row = rows.iter().find(|r| {
                                r[0].as_str().is_some_and(|s| s == label)
                                    || r[0].as_i64().is_some_and(|n| n.to_string() == *label)
                            });
                            let actual = row.map_or(0.0, |r| r[1].as_f64().unwrap());
                            assert!(
                                (actual - probability.as_f64().unwrap()).abs() < 1e-9,
                                "{path}: {name}/{label}: {actual} != {probability}"
                            );
                        }
                        if kind == "outcomes" {
                            let scale = entry["scale"].as_array().unwrap();
                            for label in value.as_object().unwrap().keys() {
                                assert!(
                                    scale.iter().any(|item| item.as_str() == Some(label)),
                                    "{path}: missing outcome label {label}"
                                );
                            }
                        }
                        if kind == "tables" {
                            assert_eq!(
                                rows.len(),
                                value.as_object().unwrap().len(),
                                "{path}: table row count"
                            );
                        }
                    }
                    _ => panic!("unknown fixture kind {kind}"),
                }
            }
        }
        for output in result.outputs {
            let probabilities: Vec<f64> = match output {
                OutputEntry::DieRoll { entries, .. } => entries.iter().map(|(_, p)| *p).collect(),
                OutputEntry::Outcomes { entries, .. } => entries.iter().map(|(_, p)| *p).collect(),
                OutputEntry::Prob { value, .. } => {
                    assert!(value.is_finite() && (0.0..=1.0).contains(&value));
                    continue;
                }
                OutputEntry::Table { entries, .. } => {
                    assert!(entries
                        .iter()
                        .all(|(_, p)| p.is_finite() && (0.0..=1.0).contains(p)));
                    continue;
                }
            };
            assert!(probabilities.iter().all(|p| p.is_finite() && *p >= 0.0));
            assert!(
                (probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-9,
                "{path}: normalization"
            );
        }
    }
}

#[test]
fn edited_boundary_inputs_and_transfer_tasks() {
    let fixtures = [
        (
            "tutorial/09-named-outcomes.dice",
            "modifier = 0",
            "modifier = 2",
            "result.pmf(\"Threat avoided\")",
            15.0 / 36.0,
        ),
        (
            "tutorial/12-advantage.dice",
            "dc = 11",
            "dc = 20",
            "advantage.p_ge(dc)",
            39.0 / 400.0,
        ),
        (
            "tutorial/15-counting-successes.dice",
            "[5, 6]",
            "[6]",
            "pool.p_at_least(2, [6])",
            2.0 / 27.0,
        ),
        (
            "tutorial/19-armour-floor.dice",
            "armour = 2",
            "armour = 3",
            "damage.pmf(0)",
            3.0 / 8.0,
        ),
        (
            "tutorial/22-natural-face-exceptions.dice",
            "ac = 15",
            "ac = 100",
            "result.p_at_least(\"Hit\")",
            1.0 / 20.0,
        ),
        (
            "tutorial/22-natural-face-exceptions.dice",
            "ac = 15",
            "ac = 1",
            "result.p_at_least(\"Hit\")",
            19.0 / 20.0,
        ),
        (
            "tutorial/23-shared-pool-rule.dice",
            "dice = 2",
            "dice = 0",
            "result.pmf(\"Critical\")",
            0.0,
        ),
        (
            "tutorial/23-shared-pool-rule.dice",
            "dice = 2",
            "dice = 0",
            "result.pmf(\"Bad\")",
            27.0 / 36.0,
        ),
        (
            "tutorial/26-reroll-once.dice",
            "first == 1",
            "first <= 2",
            "result.pmf(1)",
            2.0 / 36.0,
        ),
        (
            "tutorial/30-after-roll-decisions.dice",
            "skill = 2",
            "skill = 3",
            "result.pmf(\"Strict success\")",
            (76.0 * 81.0 + 5.0 * 50.0) / 6561.0,
        ),
        (
            "tutorial/32-build-and-defend.dice",
            "hp = 28",
            "hp = 49",
            "result.pmf(\"Damage reaches HP\")",
            0.0,
        ),
        (
            "tutorial/32-build-and-defend.dice",
            "hp = 28",
            "hp = 4",
            "result.pmf(\"Damage reaches HP\")",
            1.0,
        ),
    ];
    for (path, old, new, expression, expected) in fixtures {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("docs")
                .join(path),
        )
        .unwrap();
        assert!(source.contains(old));
        let edited = format!(
            "{}\n```dice\noutput(\"boundary\", {expression})\n```\n",
            source.replace(old, new)
        );
        let result = eval_program(path, &edited, EvalProgramOptions::default()).unwrap();
        let OutputEntry::Prob { value, .. } = result.outputs.last().unwrap() else {
            panic!("expected probability")
        };
        assert!(
            (*value - expected).abs() < 1e-9,
            "{path}: {new}: {value} != {expected}"
        );
    }
    for (path, upper_invalid) in [
        ("tutorial/23-shared-pool-rule.dice", 6),
        ("cookbook/blades-in-the-dark.dice", 21),
    ] {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("docs")
                .join(path),
        )
        .unwrap();
        for invalid in [-1, upper_invalid] {
            assert!(eval_program(
                path,
                &source.replace("dice = 2", &format!("dice = {invalid}")),
                EvalProgramOptions::default()
            )
            .is_err());
        }
    }
}

#[test]
fn fireball_risk_has_an_independent_integer_count_oracle() {
    let mut counts = vec![1_u64];
    for _ in 0..8 {
        let mut next = vec![0; counts.len() + 6];
        for (total, count) in counts.iter().enumerate() {
            for face in 1..=6 {
                next[total + face] += count;
            }
        }
        counts = next;
    }
    let expected = counts.iter().skip(28).sum::<u64>() as f64 / 6_u64.pow(8) as f64 * 11.0 / 20.0;
    let source = include_str!("../docs/tutorial/32-build-and-defend.dice");
    let result = eval_program("capstone.dice", source, EvalProgramOptions::default()).unwrap();
    let OutputEntry::Prob { value, .. } = result.outputs.last().unwrap() else {
        panic!("expected risk")
    };
    assert!((*value - expected).abs() < 1e-10);
}
