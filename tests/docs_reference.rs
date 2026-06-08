use std::path::PathBuf;

use dice_playground::engine::render_stdlib_reference_markdown;

#[test]
fn references_stdlib_md_matches_renderer() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/references/stdlib.md");
    let on_disk = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    let fresh = render_stdlib_reference_markdown();
    assert_eq!(
        on_disk, fresh,
        "docs/references/stdlib.md is out of date; run: make references"
    );
}
