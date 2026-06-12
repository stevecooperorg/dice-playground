//! Weave + render integration tests.

use dice_playground::engine::{
    eval_program, is_literate, render_literate_document, sanitize_woven_html, EvalProgramOptions,
    ProbFormat, WeaveOptions,
};

const MINIMAL_LITERATE: &str = r#"# One die

Fair d6 distribution:

```dice
output("one_d6", 1d6)
```
"#;

#[test]
fn render_literate_document_produces_html_shell() {
    let html = render_literate_document(
        "lesson.dice",
        MINIMAL_LITERATE,
        WeaveOptions {
            prob_format: ProbFormat::Decimal,
            ..Default::default()
        },
    )
    .expect("render");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("tutorial.css"));
    assert!(html.contains("<h1>"));
    assert!(html.contains("one_d6"));
}

#[test]
fn sanitize_removes_script_tags() {
    let clean = sanitize_woven_html("<div>safe</div><script>evil()</script>");
    assert!(!clean.contains("<script>"));
}

#[test]
fn eval_program_still_works_on_literate_source() {
    assert!(is_literate(MINIMAL_LITERATE));
    let r = eval_program(
        "lesson.dice",
        MINIMAL_LITERATE,
        EvalProgramOptions::default(),
    )
    .expect("eval");
    assert_eq!(r.outputs.len(), 1);
}
