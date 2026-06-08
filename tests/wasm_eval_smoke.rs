use dice_playground::engine::{eval_program, EvalProgramOptions, OutputEntry, ProbFormat};

#[test]
fn wasm_eval_smoke_two_d6() {
    let src = r#"output("two_d6", d(6) + d(6))"#;
    let r = eval_program("spike.dice", src, EvalProgramOptions::default()).expect("eval");
    match &r.outputs[0] {
        OutputEntry::DieRoll { mean, .. } => assert!((*mean - 7.0).abs() < 1e-9),
        other => panic!("expected dist, got {other:?}"),
    }
}

#[test]
fn wasm_eval_smoke_sugar_2d6() {
    let src = r#"output("two_d6", 2d6)"#;
    let r = eval_program("spike.dice", src, EvalProgramOptions::default()).expect("eval");
    assert!(r.text.contains("two_d6"));
}

#[test]
fn wasm_eval_smoke_tutorial_one_die() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/tutorial/01-one-die.dice");
    let src = std::fs::read_to_string(&path).expect("read");
    let r = eval_program(
        "01-one-die.dice",
        &src,
        EvalProgramOptions {
            prob_format: ProbFormat::Decimal,
        },
    )
    .expect("eval");
    assert!(r.text.contains("d6"));
}
