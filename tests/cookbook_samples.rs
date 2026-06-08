//! Smoke tests: every cookbook script evaluates without error.

use dice_playground::engine::{desugar_if_needed, eval_source, OutputEntry};

const COOKBOOK_DIR: &str = "examples/cookbook";

const SAMPLE_PATHS: &[&str] = &[
    "examples/cookbook/the-pool.dice",
    "examples/cookbook/count-high-faces.dice",
    "examples/cookbook/exploding-dice.dice",
    "examples/cookbook/ability-scores-4d6dl1.dice",
    "examples/cookbook/fireball-half-damage.dice",
    "examples/cookbook/blades-in-the-dark.dice",
    "examples/cookbook/brindlewood-bay-theorize.dice",
    "examples/cookbook/cairn-blood-elk.dice",
    "examples/cookbook/rolemaster-open-ended.dice",
    "examples/cookbook/fudge-4df.dice",
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
        .map(|p| p.strip_prefix("examples/cookbook/").unwrap().to_string())
        .collect();
    listed.sort();
    assert_eq!(
        on_disk, listed,
        "every examples/cookbook/* script must be listed in SAMPLE_PATHS"
    );
}

fn eval_sample(rel: &str) -> anyhow::Result<dice_playground::engine::EvalResult> {
    let content = read_sample(rel);
    let expanded = desugar_if_needed(rel, &content)?;
    eval_source(rel, &expanded)
}

fn eval_sample_or_panic(rel: &str) -> dice_playground::engine::EvalResult {
    match eval_sample(rel) {
        Ok(res) => res,
        Err(e) => panic!("eval_sample {rel}: {e:#}"),
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
        dice_playground::engine::OutputEntry::DieRoll { mean, .. } => {
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
