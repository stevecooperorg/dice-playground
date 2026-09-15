//! Execute the published Part B models unchanged, then check guided edits and boundaries.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

const L05: &str = include_str!("../docs/tutorial/05-roll-under.dice");
const L06: &str = include_str!("../docs/tutorial/06-averages-and-chances.dice");
const L07: &str = include_str!("../docs/tutorial/07-probability-tables.dice");
const L08: &str = include_str!("../docs/tutorial/08-comparison-loops.dice");
const DND: &str = include_str!("../docs/cookbook/dnd2014-ability-check.dice");
const COC: &str = include_str!("../docs/cookbook/coc7-regular-check.dice");

fn outputs(source: &str) -> Vec<OutputEntry> {
    eval_program("part-b.dice", source, EvalProgramOptions::default())
        .unwrap_or_else(|error| panic!("Part B evaluation: {error:#}"))
        .outputs
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn probability(entry: &OutputEntry) -> f64 {
    match entry {
        OutputEntry::Prob { value, .. } => *value,
        other => panic!("expected probability, got {other:?}"),
    }
}

fn table(entry: &OutputEntry, expected: &[(&str, f64)]) {
    let OutputEntry::Table { entries, .. } = entry else {
        panic!("expected table, got {entry:?}");
    };
    assert_eq!(entries.len(), expected.len());
    for ((label, chance), (expected_label, expected_chance)) in entries.iter().zip(expected) {
        assert_eq!(label, expected_label);
        close(*chance, *expected_chance);
    }
}

#[test]
fn roll_under_is_inclusive_and_guided_changes_keep_the_exact_face_chance() {
    for skill in [40, 50, 65] {
        let out = outputs(&L05.replace("skill = 50", &format!("skill = {skill}")));
        assert_eq!(out.len(), 2);
        close(probability(&out[0]), f64::from(skill) / 100.0);
        close(probability(&out[1]), 0.01);
    }
    close(
        probability(&outputs("output(\"strictly below 40\", d(100).cdf(39))")[0]),
        0.39,
    );
}

#[test]
fn averages_do_not_change_when_the_target_changes() {
    for dc in [6, 15, 20, 25, 26] {
        let out = outputs(&L06.replace("dc = 15", &format!("dc = {dc}")));
        assert_eq!(out.len(), 4);
        for (index, bonus) in [(0, 2), (1, 5)] {
            let OutputEntry::DieRoll { mean, entries, .. } = &out[index] else {
                panic!("expected totals");
            };
            close(*mean, 10.5 + f64::from(bonus));
            assert_eq!(entries.first().unwrap().0, 1 + i64::from(bonus));
            assert_eq!(entries.last().unwrap().0, 20 + i64::from(bonus));
            // Independent face count, including natural 1/20 without attack exceptions.
            let successes = (1..=20).filter(|face| face + bonus >= dc).count();
            close(probability(&out[index + 2]), successes as f64 / 20.0);
        }
    }
}

#[test]
fn table_rows_are_labelled_overlapping_queries_not_an_outcome_partition() {
    let out = outputs(L07);
    assert_eq!(out.len(), 1);
    table(
        &out[0],
        &[
            ("Meet DC 10", 0.8),
            ("Meet DC 15", 0.55),
            ("Meet DC 20", 0.3),
        ],
    );
    let edited = L07.replace(
        "    (\"Meet DC 20\", check.p_ge(20)),",
        "    (\"Meet DC 20\", check.p_ge(20)),\n    (\"Meet DC 25\", check.p_ge(25)),",
    );
    table(
        &outputs(&edited)[0],
        &[
            ("Meet DC 10", 0.8),
            ("Meet DC 15", 0.55),
            ("Meet DC 20", 0.3),
            ("Meet DC 25", 0.05),
        ],
    );
    let transfer = "rows = [(\"Exactly six\", d(6).pmf(6)), (\"Four or more\", d(6).p_ge(4))]\noutput(\"check\", prob_table(rows))";
    table(
        &outputs(transfer)[0],
        &[("Exactly six", 1.0 / 6.0), ("Four or more", 0.5)],
    );
}

#[test]
fn loop_has_five_rows_and_one_output_including_after_guided_edit() {
    let out = outputs(L08);
    assert_eq!(out.len(), 1);
    table(
        &out[0],
        &[
            ("Bonus +2", 0.4),
            ("Bonus +3", 0.45),
            ("Bonus +4", 0.5),
            ("Bonus +5", 0.55),
            ("Bonus +6", 0.6),
        ],
    );
    let changed = outputs(&L08.replace("range(2, 7)", "range(0, 5)"));
    assert_eq!(changed.len(), 1);
    table(
        &changed[0],
        &[
            ("Bonus +0", 0.3),
            ("Bonus +1", 0.35),
            ("Bonus +2", 0.4),
            ("Bonus +3", 0.45),
            ("Bonus +4", 0.5),
        ],
    );
    let transfer = "rows = []\nfor skill in range(40, 45):\n    rows.append((\"Skill \" + str(skill), d(100).cdf(skill)))\noutput(\"Regular difficulty\", prob_table(rows))";
    table(
        &outputs(transfer)[0],
        &[
            ("Skill 40", 0.4),
            ("Skill 41", 0.41),
            ("Skill 42", 0.42),
            ("Skill 43", 0.43),
            ("Skill 44", 0.44),
        ],
    );
}

#[test]
fn ordinary_dnd_recipe_matches_closed_form_across_extreme_inputs() {
    for bonus in [-5, 0, 5, 30] {
        for dc in [0, 6, 15, 25, 26, 40] {
            let source = DND
                .replace("bonus = 5", &format!("bonus = {bonus}"))
                .replace("dc = 15", &format!("dc = {dc}"));
            let out = outputs(&source);
            assert_eq!(out.len(), 2);
            let OutputEntry::Table { entries, .. } = &out[1] else {
                panic!("expected table");
            };
            for ((label, value), target) in entries.iter().zip([dc - 5, dc, dc + 5]) {
                assert_eq!(label, &format!("Meet DC {target}"));
                close(*value, f64::from((21 + bonus - target).clamp(0, 20)) / 20.0);
            }
        }
    }
    for (old, new) in [("bonus = 5", "bonus = 5.5"), ("dc = 15", "dc = 15.5")] {
        assert!(eval_program(
            "invalid.dice",
            &DND.replace(old, new),
            EvalProgramOptions::default()
        )
        .is_err());
    }
}

#[test]
fn coc_recipe_bounds_apply_to_both_comparison_rows() {
    for skill in [1, 50, 89] {
        let source = COC.replace("skill = 50", &format!("skill = {skill}"));
        let out = outputs(&source);
        assert_eq!(out.len(), 1);
        table(
            &out[0],
            &[
                (
                    &format!("Meet regular difficulty at skill {skill}"),
                    f64::from(skill) / 100.0,
                ),
                (
                    &format!("Meet regular difficulty at skill {}", skill + 10),
                    f64::from(skill + 10) / 100.0,
                ),
            ],
        );
    }
    for invalid in ["0", "90", "100", "50.5", "\"50\""] {
        assert!(eval_program(
            "invalid.dice",
            &COC.replace("skill = 50", &format!("skill = {invalid}")),
            EvalProgramOptions::default()
        )
        .is_err());
    }
}

#[test]
fn part_b_has_new_urls_without_reassigning_legacy_identities() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../docs/learning-content.json")).unwrap();
    let pages = manifest["pages"].as_array().unwrap();
    for (id, path) in [
        ("L05", "tutorial/05-roll-under.dice"),
        ("legacy-T05", "tutorial/05-dice-notation.dice"),
        ("L08", "tutorial/08-comparison-loops.dice"),
        ("legacy-T08", "tutorial/08-restrict-faces.dice"),
    ] {
        assert_eq!(
            pages
                .iter()
                .chain(manifest["aliases"].as_array().unwrap().iter())
                .find(|page| page["id"] == id)
                .unwrap()["path"],
            path
        );
    }
    let ids: Vec<_> = pages
        .iter()
        .take(8)
        .map(|page| page["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        ["L01", "L02", "L03", "L04", "L05", "L06", "L07", "L08"]
    );
}
