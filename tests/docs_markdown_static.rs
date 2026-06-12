use std::path::PathBuf;

use dice_playground::engine::{render_markdown_static_file, MarkdownStaticLayout};

#[test]
fn user_guide_renders_from_readme() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/README.md");
    let md =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let (title, html) = render_markdown_static_file(&md, MarkdownStaticLayout::Guide);
    assert!(title.contains("guide") || title.contains("Guide"));
    assert!(html.contains("<strong>User guide</strong>"));
    assert!(html.contains("<h1>"));
    assert!(
        html.contains("<table>"),
        "user guide tutorial index should render as a table"
    );
}

#[test]
fn stdlib_reference_renders_from_markdown() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/references/stdlib.md");
    let md = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {} (run: make references): {e}", path.display()));
    let (_title, html) = render_markdown_static_file(&md, MarkdownStaticLayout::Reference);
    assert!(html.contains("stdlib") || html.contains("Standard") || html.contains("<h1>"));
    assert!(html.contains("../tutorial/tutorial.css"));
}
