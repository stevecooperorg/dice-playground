---
title: WASM markdown-to-HTML library spike
type: chore
created: 2026-06-12
status: done
baseline_commit: 087097fee2f90a852cbca0000af51eac1cf14284
context:
  - ../../.agents/skills/dice-playground-standards/SKILL.md
  - ../planning-artifacts/research/technical-prose-encoding-in-dice-files-research-2026-06-12.md
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Literate `.dice` needs prose woven to HTML in the engine for both WASM playground and native CLI; we need a proven Rust library that compiles on `wasm32-unknown-unknown` with minimal markdown features.

**Approach:** Spike `pulldown-cmark` (primary candidate) in `src/engine/` with a tiny `markdown_to_html` API, unit tests on native, and `make check-wasm` plus optional wasm size note. Record why alternatives were rejected in Design Notes.

## Boundaries & Constraints

**Always:** Engine-only (no Leptos/web-sys); no `unwrap`/`expect`; public API has doc example; run `cargo test` and `make check-wasm`.

**Ask First:** Adding second markdown crate; expanding markdown feature set beyond tutorial subset.

**Never:** Pandoc/JS markdown in UI; full literate tangle/weave; playground UI changes in this spike.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| HAPPY_PATH | `# Hi\n\n[link](https://x.test)` | HTML contains `<h1>`, `<p>`, `<a href=` | N/A |
| EMPTY | `""` | Empty or whitespace-only HTML string | N/A |
| CODE_FENCE | Markdown with ` ```dice ` fence | Fence rendered as `<pre><code>` (prose pass-through) | N/A |

</frozen-after-approval>

## Code Map

- `Cargo.toml` — `pulldown-cmark` 0.13, `default-features = false`, `features = ["html"]`
- `src/engine/markdown_html.rs` — `markdown_to_html`
- `src/engine/mod.rs` — re-export
- `tests/wasm_eval_smoke.rs` — `wasm_markdown_to_html_smoke`

## Tasks & Acceptance

**Execution:**
- [x] `Cargo.toml` — add `pulldown-cmark` with minimal features — pure-Rust CommonMark for wasm
- [x] `src/engine/markdown_html.rs` — `markdown_to_html(&str) -> String` using `Parser` + `html::push_html` — shared weave primitive
- [x] `src/engine/mod.rs` — `mod markdown_html;` and `pub use markdown_html::markdown_to_html`
- [x] `src/engine/markdown_html.rs` — `#[cfg(test)]` covering happy path, empty, heading+link, fenced code

**Acceptance Criteria:**
- Given a markdown string with a heading and paragraph, when `markdown_to_html` runs, then output contains expected HTML tags. **Met.**
- Given the workspace, when `make check-wasm` runs, then it exits 0 with the new engine code linked. **Met.**
- Given the spike completes, when documented in Design Notes, then library choice and rejected alternatives are recorded with wasm rationale. **Met.**

## Spec Change Log

(none)

## Design Notes

**Chosen: `pulldown-cmark` 0.13** (`default-features = false`, feature `html` for `html::push_html`).

| Candidate | Verdict |
|-----------|---------|
| **pulldown-cmark** | **Use.** Pure Rust, no C deps, already used widely on wasm32; CommonMark + fenced code matches tutorial subset; small API surface. |
| **comrak** | Defer. GFM-focused, larger dependency tree and feature set beyond spike needs; better if we later require tables/extensions Pandoc-style—revisit with `cargo bloat` then. |
| **Custom subset parser** | Defer. Only if bundle measurement forces it; maintenance cost high. |
| **Pandoc / JS (marked)** | Rejected. Not in-engine; breaks CLI/WASM parity. |

**WASM proof:** `make check-wasm` links `pulldown-cmark` into `dice-playground` for `wasm32-unknown-unknown --no-default-features`. Release size delta not measured ( `cargo-bloat` not installed); follow `docs/wasm-bundle-size.md` after next `make release-static`.

**Follow-ups for architecture:** HTML sanitization before DOM insertion (`ammonia` or UI policy); literate tangle separate from markdown weave; do not enable pulldown `simd` on wasm without testing.

## Verification

**Commands:**
- `cargo test --lib markdown_html` — pass (4 tests)
- `cargo test wasm_markdown_to_html_smoke` — pass
- `make check-wasm` — pass

**Manual checks (if no CLI):**
- (none)
