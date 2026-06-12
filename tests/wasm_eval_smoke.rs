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
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tutorial/01-one-die.dice");
    let src = std::fs::read_to_string(&path).expect("read");
    let r = eval_program(
        "01-one-die.dice",
        &src,
        EvalProgramOptions {
            prob_format: ProbFormat::Decimal,
        },
    )
    .expect("eval");
    assert!(!r.report_html.is_empty());
    assert!(r.text.contains("one_d6") || r.outputs.len() == 1);
}

#[test]
fn wasm_eval_smoke_literate_report_html() {
    let src = "# Hi\n\n```dice\noutput(\"d6\", 1d6)\n```\n";
    let r = eval_program("lit.dice", src, EvalProgramOptions::default()).expect("eval");
    assert!(!r.report_html.is_empty());
    assert!(r.report_html.contains("<h1>"));
}

#[test]
fn wasm_markdown_to_html_smoke() {
    use dice_playground::engine::markdown_to_html;
    let html = markdown_to_html("## odds\n\nSee [guide](/docs/).\n");
    assert!(html.contains("<h2>"));
    assert!(html.contains("href=\"/docs/\""));
}
