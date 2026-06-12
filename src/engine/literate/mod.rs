//! Literate `.dice` documents: detection, fence parse, and tangle.
//!
//! A literate file mixes markdown prose with executable ` ``` ` / ` ```dice ` fences.
//! Tangle concatenates fence bodies into one Starlark module for a single eval per Run.
//! See `_bmad-output/specs/spec-literate-dice/literate-dice-format.md`.

mod fence;
mod parse;
mod tangle;
mod weave;
pub use parse::{parse as parse_literate, LiterateDocument};
pub use tangle::{tangle as tangle_literate, LineMap, TangleResult};
pub use weave::{
    render_literate_document, sanitize_woven_html, weave_literate, LiterateStaticLayout,
    WeaveOptions,
};

/// Maximum literate document size (prose + code), UTF-8 bytes.
pub const MAX_LITERATE_BYTES: usize = 256 * 1024;

/// True when the source contains at least one executable fence per format v1.
///
/// # Example
///
/// ```
/// use dice_playground::engine::is_literate;
/// assert!(!is_literate("output(d(6))"));
/// assert!(is_literate("# Title\n\n```dice\noutput(d(6))\n```\n"));
/// ```
pub fn is_literate(source: &str) -> bool {
    parse::contains_executable_fence(source)
}

/// Map a 1-based line in tangled Starlark back to a 1-based line in the original `.dice` file.
pub fn source_line_for_tangled(line_map: &LineMap, tangled_line_1based: u32) -> u32 {
    tangle::source_line_for_tangled(line_map, tangled_line_1based)
}
