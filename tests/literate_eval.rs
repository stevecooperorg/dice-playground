//! Integration tests for literate `.dice` tangle + eval (format v1).

use dice_playground::engine::{
    check_source, eval_program, is_literate, parse_literate, tangle_literate, EvalProgramOptions,
};

#[test]
fn minimal_literate_eval_runs_both_outputs_in_one_module() {
    let src = r#"# Two rolls

```dice
output("a", 1d6)
```

```dice
output("b", 1d6)
```
"#;
    assert!(is_literate(src));
    let doc = parse_literate(src).expect("parse");
    assert_eq!(doc.fences.len(), 2);
    let tangled = tangle_literate(&doc);
    assert!(tangled.tangled.contains("output(\"a\""));
    assert!(tangled.tangled.contains("output(\"b\""));

    let r = eval_program("two.dice", src, EvalProgramOptions::default()).expect("eval");
    assert_eq!(r.outputs.len(), 2);
}

#[test]
fn legacy_script_still_evals_without_fences() {
    let src = "output(\"two_d6\", 2d6)\n";
    assert!(!is_literate(src));
    let r = eval_program("legacy.dice", src, EvalProgramOptions::default()).expect("eval");
    assert_eq!(r.outputs.len(), 1);
    assert!(r.report_html.is_empty());
}

#[test]
fn docs_tutorial_one_die_literate() {
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tutorial/01-one-die.dice");
    let src = std::fs::read_to_string(&path).expect("read");
    assert!(is_literate(&src));
    let r = eval_program("01-one-die.dice", &src, EvalProgramOptions::default()).expect("eval");
    assert_eq!(r.outputs.len(), 1);
    assert!(!r.report_html.is_empty());
}

#[test]
fn bare_fence_opener_tangles_and_evals() {
    let src = "# Roll\n\n```\noutput(\"d6\", 1d6)\n```\n";
    assert!(is_literate(src));
    let r = eval_program("bare.dice", src, EvalProgramOptions::default()).expect("eval");
    assert_eq!(r.outputs.len(), 1);
    let check = check_source("bare.dice", src).expect("check");
    assert!(!check.has_errors());
}
