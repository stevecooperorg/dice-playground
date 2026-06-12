//! Smoke tests: every cookbook script evaluates without error.

use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

const COOKBOOK_DIR: &str = "docs/cookbook";

const SAMPLE_PATHS: &[&str] = &[
    "docs/cookbook/the-pool.dice",
    "docs/cookbook/count-high-faces.dice",
    "docs/cookbook/exploding-dice.dice",
    "docs/cookbook/ability-scores-4d6dl1.dice",
    "docs/cookbook/fireball-half-damage.dice",
    "docs/cookbook/blades-in-the-dark.dice",
    "docs/cookbook/brindlewood-bay-theorize.dice",
    "docs/cookbook/cairn-blood-elk.dice",
    "docs/cookbook/rolemaster-open-ended.dice",
    "docs/cookbook/fudge-4df.dice",
];

fn read_sample(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read cookbook sample {}: {e}", path))
}

#[test]
fn cookbook_manifest_covers_all_files() {
    let dir = std::path::Path::new(COOKBOOK_DIR);
    let mut on_disk: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {COOKBOOK_DIR}: {e}"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "dice"))
        .map(|e| {
            e.path()
                .strip_prefix(COOKBOOK_DIR)
                .unwrap()
                .to_string_lossy()
                .trim_start_matches('/')
                .to_string()
        })
        .collect();
    on_disk.sort();
    let mut listed: Vec<String> = SAMPLE_PATHS
        .iter()
        .map(|p| p.strip_prefix("docs/cookbook/").unwrap().to_string())
        .collect();
    listed.sort();
    assert_eq!(
        on_disk, listed,
        "every docs/cookbook/* script must be listed in SAMPLE_PATHS"
    );
}

fn eval_sample_or_panic(rel: &str) -> dice_playground::engine::EvalResult {
    let content = read_sample(rel);
    let r = eval_program(rel, &content, EvalProgramOptions::default())
        .unwrap_or_else(|e| panic!("eval_sample {rel}: {e:#}"));
    dice_playground::engine::EvalResult {
        return_value: r.return_value,
        outputs: r.outputs,
    }
}

#[test]
fn cookbook_the_pool() {
    let res = eval_sample_or_panic(SAMPLE_PATHS[0]);
    assert_eq!(res.outputs.len(), 10);
}

#[test]
fn cookbook_ability_4d6dl1_mean() {
    let res = eval_sample_or_panic(SAMPLE_PATHS[3]);
    match &res.outputs[0] {
        OutputEntry::DieRoll { mean, .. } => {
            assert!((*mean - 12.244598765432098).abs() < 1e-9);
        }
        other => panic!("expected dist, got {other:?}"),
    }
}

#[test]
fn cookbook_fireball_half_damage() {
    let res = eval_sample_or_panic(SAMPLE_PATHS[4]);
    assert_eq!(res.outputs.len(), 2);
    match (&res.outputs[0], &res.outputs[1]) {
        (
            OutputEntry::DieRoll {
                mean: full,
                entries: full_entries,
                ..
            },
            OutputEntry::DieRoll {
                mean: half,
                entries: half_entries,
                ..
            },
        ) => {
            assert!((*full - 28.0).abs() < 1e-9);
            assert_eq!(full_entries.last().map(|(k, _)| *k), Some(48));
            assert_eq!(half_entries.last().map(|(k, _)| *k), Some(24));
            assert!(*half < *full);
        }
        other => panic!("expected two dist outputs, got {other:?}"),
    }
}
