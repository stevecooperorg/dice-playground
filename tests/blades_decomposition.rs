//! Independent integer enumeration and closed-form oracle for the responsive recipe.
use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry};

#[test]
fn decomposition_matches_small_face_counts_and_large_closed_forms() {
    let source = include_str!("../docs/cookbook/blades-in-the-dark.dice");
    for dice in 0_u32..=20 {
        let result = eval_program(
            "blades.dice",
            &source.replace("dice = 2", &format!("dice = {dice}")),
            EvalProgramOptions::default(),
        )
        .unwrap();
        let OutputEntry::Table { entries, .. } = &result.outputs[0] else {
            panic!("expected table");
        };
        let expected = if dice == 0 {
            [27.0 / 36.0, 8.0 / 36.0, 1.0 / 36.0, 0.0]
        } else {
            let none = (5.0_f64 / 6.0).powi(dice as i32);
            let bad = 0.5_f64.powi(dice as i32);
            let clean = f64::from(dice) / 6.0 * (5.0_f64 / 6.0).powi(dice as i32 - 1);
            [bad, none - bad, clean, 1.0 - none - clean]
        };
        assert!((entries.iter().map(|(_, p)| p).sum::<f64>() - 1.0).abs() < 1e-10);
        for ((_, actual), expected) in entries.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-10);
        }
        if (1..=5).contains(&dice) {
            let mut counts = [0_u64; 4];
            for mut tuple in 0..6_u64.pow(dice) {
                let mut highest = 0;
                let mut sixes = 0;
                for _ in 0..dice {
                    let face = tuple % 6 + 1;
                    tuple /= 6;
                    highest = highest.max(face);
                    if face == 6 {
                        sixes += 1;
                    }
                }
                let category = if sixes >= 2 {
                    3
                } else if highest == 6 {
                    2
                } else if highest >= 4 {
                    1
                } else {
                    0
                };
                counts[category] += 1;
            }
            for ((_, actual), count) in entries.iter().zip(counts) {
                assert!((actual - count as f64 / 6_u64.pow(dice) as f64).abs() < 1e-10);
            }
        }
    }
}
