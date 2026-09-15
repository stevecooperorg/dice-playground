use std::path::PathBuf;

use dice_playground::engine::{
    render_markdown_static_file, render_stdlib_reference_markdown, MarkdownStaticLayout,
};
use dice_playground::ui::static_site::inject_playground_load_links;

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
        html.contains("href=\"../tutorial/index.html\""),
        "user guide links to the manifest-generated tutorial index"
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
    assert!(html.contains("href=\"api-conventions.html\""));
    assert!(html.contains("href=\"../tutorial/index.html\""));
    assert!(html.contains("<h3>DieRoll.pmf</h3>"));
    assert!(!html.contains("die_faces.bucket"));
    assert!(html.contains("0–1"), "UTF-8 prose must survive rendering");
}

#[test]
fn reference_handoff_links_only_runnable_examples() {
    let (_, html) = render_markdown_static_file(
        &render_stdlib_reference_markdown(),
        MarkdownStaticLayout::Reference,
    );
    let enhanced = inject_playground_load_links(&html).expect("enhance reference");
    let mut examples = 0;
    let mut signatures = 0;
    for rest in enhanced.split("<pre").skip(1) {
        let (block, _) = rest.split_once("</pre>").expect("closed code block");
        if block.contains("language-dice") {
            examples += 1;
            assert_eq!(block.matches("class=\"load-in-playground\"").count(), 1);
        } else if block.contains("language-python") {
            signatures += 1;
            assert!(
                !block.contains("load-in-playground"),
                "signature is not a script"
            );
        }
    }
    assert!(signatures >= 52);
    assert!(
        examples >= signatures,
        "every entry needs a runnable example"
    );
}

#[test]
fn api_conventions_links_back_to_published_reference() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/references/api-conventions.md");
    let md = std::fs::read_to_string(path).expect("conventions page");
    let (_, html) = render_markdown_static_file(&md, MarkdownStaticLayout::Reference);
    assert!(html.contains("href=\"stdlib.html\""));
    assert!(html.contains("href=\"../tutorial/07-mixed-dice-pools.html\""));
    assert!(!html.contains("href=\"stdlib.md\""));
}
