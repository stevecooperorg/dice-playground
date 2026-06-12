//! Every literate lesson under docs/tutorial/ must eval in CI.

use std::path::PathBuf;

use dice_playground::engine::{eval_program, is_literate, EvalProgramOptions};

#[test]
fn all_docs_tutorial_dice_are_literate_and_eval() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tutorial");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "dice"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= 13,
        "expected full tutorial corpus, found {}",
        paths.len()
    );
    for path in paths {
        let src = std::fs::read_to_string(&path).expect("read");
        assert!(is_literate(&src), "{} should be literate", path.display());
        let name = path.file_name().unwrap().to_string_lossy();
        eval_program(&name, &src, EvalProgramOptions::default())
            .unwrap_or_else(|e| panic!("eval {}: {e:#}", path.display()));
    }
}
