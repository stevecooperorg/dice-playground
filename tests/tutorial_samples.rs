use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use dice_playground::engine::{desugar_if_needed, eval_source, OutputEntry};

const TUTORIAL_DIR: &str = "examples/tutorial";

const SAMPLE_PATHS: &[&str] = &[
    "examples/tutorial/01-one-die.dice",
    "examples/tutorial/02-two-d6.dice",
    "examples/tutorial/03-modifier-shift.dice",
    "examples/tutorial/04-success-chance.dice",
    "examples/tutorial/05-dice-notation.dice",
    "examples/tutorial/06-table-2d10.dice",
    "examples/tutorial/07-ordered-outcomes.dice",
    "examples/tutorial/08-dnd5e-d20-check.dice",
    "examples/tutorial/09-pbta-2d6-move.dice",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sample_path(rel: &str) -> PathBuf {
    manifest_dir().join(rel)
}

fn eval_sample(rel: &str) -> dice_playground::engine::EvalResult {
    let path = sample_path(rel);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read tutorial sample {}: {e}", path.display()));
    let path_str = path.to_string_lossy();
    let expanded = desugar_if_needed(&path_str, &content)
        .unwrap_or_else(|e| panic!("desugar {}: {e}", path.display()));
    eval_source(&path_str, &expanded).unwrap_or_else(|e| panic!("eval {}: {e:#}", path.display()))
}

#[test]
fn tutorial_manifest_covers_all_files() {
    let dir = manifest_dir().join(TUTORIAL_DIR);
    let mut on_disk = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display())) {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension() == Some(OsStr::new("dice")) {
            let rel = path
                .strip_prefix(manifest_dir())
                .expect("under manifest")
                .to_string_lossy()
                .into_owned();
            on_disk.push(rel);
        }
    }
    on_disk.sort();
    let mut expected: Vec<String> = SAMPLE_PATHS.iter().map(|s| (*s).to_owned()).collect();
    expected.sort();
    assert_eq!(
        on_disk, expected,
        "every examples/tutorial/* script must be listed in SAMPLE_PATHS"
    );
}

#[test]
fn tutorial_01_one_die() {
    let res = eval_sample(SAMPLE_PATHS[0]);
    assert_eq!(res.outputs.len(), 1);
    match &res.outputs[0] {
        OutputEntry::DieRoll {
            name,
            entries,
            mean,
        } => {
            assert_eq!(name, "one_d6");
            assert_eq!(entries.len(), 6);
            assert!((*mean - 3.5).abs() < 1e-9);
        }
        other => panic!("expected dist output, got {other:?}"),
    }
}

#[test]
fn tutorial_02_two_d6() {
    let res = eval_sample(SAMPLE_PATHS[1]);
    assert_eq!(res.outputs.len(), 1);
    match &res.outputs[0] {
        OutputEntry::DieRoll { name, mean, .. } => {
            assert_eq!(name, "two_d6");
            assert!((*mean - 7.0).abs() < 1e-9);
        }
        other => panic!("expected dist output, got {other:?}"),
    }
}

#[test]
fn tutorial_03_modifier_shift() {
    let res = eval_sample(SAMPLE_PATHS[2]);
    assert_eq!(res.outputs.len(), 2);
    let (base_mean, shifted_mean) = match (&res.outputs[0], &res.outputs[1]) {
        (
            OutputEntry::DieRoll {
                name: n0, mean: m0, ..
            },
            OutputEntry::DieRoll {
                name: n1, mean: m1, ..
            },
        ) => {
            assert_eq!(n0, "roll_base");
            assert_eq!(n1, "roll_plus_5");
            (*m0, *m1)
        }
        other => panic!("expected two dist outputs, got {other:?}"),
    };
    assert!((base_mean - 11.0).abs() < 1e-9);
    assert!((shifted_mean - base_mean - 5.0).abs() < 1e-9);
}

#[test]
fn tutorial_04_success_chance() {
    let res = eval_sample(SAMPLE_PATHS[3]);
    assert_eq!(res.outputs.len(), 1);
    match &res.outputs[0] {
        OutputEntry::Prob { name, value } => {
            assert_eq!(name, "p_at_least_15");
            assert!(*value > 0.0 && *value < 1.0);
        }
        other => panic!("expected prob output, got {other:?}"),
    }
}

fn dist_mean_by_name(res: &dice_playground::engine::EvalResult, name: &str) -> f64 {
    for out in &res.outputs {
        if let OutputEntry::DieRoll { name: n, mean, .. } = out {
            if n == name {
                return *mean;
            }
        }
    }
    panic!("missing dist output {name:?}");
}

#[test]
fn tutorial_05_dice_notation() {
    let res = eval_sample(SAMPLE_PATHS[4]);
    assert_eq!(res.outputs.len(), 8);
    assert!((dist_mean_by_name(&res, "one_d4") - 2.5).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "two_d6") - 7.0).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "two_d6_plus_3") - 10.0).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "four_d6") - 14.0).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "four_d6dl1") - 12.244598765432098).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "four_d6dh1") - 8.755401234567925).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "four_d6kh2") - 9.344135802469168).abs() < 1e-9);
    assert!((dist_mean_by_name(&res, "three_d12kl1") - 3.5208333333333326).abs() < 1e-9);
}

#[test]
fn tutorial_07_ordered_outcomes() {
    let res = eval_sample(SAMPLE_PATHS[6]);
    assert_eq!(res.outputs.len(), 2);
    match &res.outputs[0] {
        OutputEntry::Outcomes {
            name,
            scale,
            entries,
        } => {
            assert_eq!(name, "check");
            assert_eq!(scale.len(), 4);
            assert_eq!(entries.len(), 4);
            let sum: f64 = entries.iter().map(|(_, p)| p).sum();
            assert!((sum - 1.0).abs() < 1e-9);
        }
        other => panic!("expected ordinal output, got {other:?}"),
    }
    match &res.outputs[1] {
        OutputEntry::Prob { name, value } => {
            assert_eq!(name, "p_success_plus");
            assert!(*value > 0.0 && *value < 1.0);
        }
        other => panic!("expected prob output, got {other:?}"),
    }
}

#[test]
fn tutorial_08_dnd5e_d20_check() {
    let res = eval_sample(SAMPLE_PATHS[7]);
    assert_eq!(res.outputs.len(), 2);
    match &res.outputs[0] {
        OutputEntry::Outcomes {
            name,
            scale,
            entries,
        } => {
            assert_eq!(name, "advantage_check");
            assert_eq!(scale.len(), 4);
            assert_eq!(entries.len(), 4);
            let sum: f64 = entries.iter().map(|(_, p)| p).sum();
            assert!((sum - 1.0).abs() < 1e-9);
        }
        other => panic!("expected ordinal output, got {other:?}"),
    }
    match &res.outputs[1] {
        OutputEntry::Prob { name, value } => {
            assert_eq!(name, "p_hit_or_better");
            assert!(*value > 0.0 && *value < 1.0);
        }
        other => panic!("expected prob output, got {other:?}"),
    }
}

#[test]
fn tutorial_09_pbta_2d6_move() {
    let res = eval_sample(SAMPLE_PATHS[8]);
    assert_eq!(res.outputs.len(), 3);
    match &res.outputs[0] {
        OutputEntry::Outcomes {
            name,
            scale,
            entries,
        } => {
            assert_eq!(name, "move");
            assert_eq!(scale, &["MISS", "PARTIAL", "FULL_SUCCESS"]);
            assert_eq!(entries.len(), 3);
            let sum: f64 = entries.iter().map(|(_, p)| p).sum();
            assert!((sum - 1.0).abs() < 1e-9);
        }
        other => panic!("expected ordinal output, got {other:?}"),
    }
    // STAT=2: full success needs shifted total >= 10 → 2d6 >= 8 → 15/36
    match &res.outputs[1] {
        OutputEntry::Prob { name, value } => {
            assert_eq!(name, "p_full_success");
            assert!((*value - 15.0 / 36.0).abs() < 1e-9);
        }
        other => panic!("expected prob output, got {other:?}"),
    }
}

#[test]
fn tutorial_06_table_2d10() {
    let res = eval_sample(SAMPLE_PATHS[5]);
    assert_eq!(res.outputs.len(), 1);
    const EPS: f64 = 1e-9;
    match &res.outputs[0] {
        OutputEntry::Table { name, entries } => {
            assert_eq!(name, "success_grid");
            assert_eq!(entries.len(), 13 * 11);
            for (label, value) in entries {
                assert!(label.starts_with("modifier "));
                assert!(
                    *value >= -EPS && *value <= 1.0 + EPS,
                    "prob out of range for {label}: {value}"
                );
            }
        }
        other => panic!("expected single table output, got {other:?}"),
    }
}
