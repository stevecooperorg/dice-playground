---
name: dice-playground-standards
description: Contributes to Dice Playground with modular engine/UI separation, explanatory docs, and strict Rust quality and testing. Use when changing src/engine, src/ui, tests, or docs in dice-playground; when adding features; or when the user asks about project conventions, architecture, or contribution standards.
---

# Dice Playground standards

## Project intent

This repository is **open source** and meant for **sharing and modification**. Every change should keep the codebase **modular**, **clear**, **well documented**, and **well tested**.

## Architecture

| Layer | Path | Responsibility |
|-------|------|----------------|
| **Core (engine)** | `src/engine/` | Exact probability (PMF), Starlark guest, CLI/LSP hooks, playground eval API |
| **UI** | `src/ui/` | Leptos CSR WASM; calls engine via `src/engine/playground.rs` and related APIs |
| **Bins** | `src/bin/` | `dice` CLI, `dice-playground` binary |

**Rules:**

- Engine code must not depend on Leptos, `web-sys`, or UI-only types.
- UI stays thin: presentation, wiring, and WASM concerns—not probability logic.
- Shared public surface lives on `dice_playground::engine` (see `src/lib.rs`).

Layout, features, and verification commands: [docs/AGENT.md](../../../docs/AGENT.md).

## Documentation style

Favor **clear, explanatory** prose so a programmer **without a probability background** can follow the idea or know what to look up.

**Rust (`//!` and `///`):**

- State *what* the type/function does in plain language, then *how* it relates to tabletop dice when relevant.
- Name the math concept once (PMF, convolution, enumeration) and give intuition before notation.
- Public items: **doc comments with runnable ` ```rust` examples** (see Code Quality).
- Link to user-facing docs when behavior is user-visible (`docs/tutorial/`, `docs/cookbook/`).

**User guide (`docs/`):**

- Table-first explanations for mechanics (“at the table” meaning before API names).
- Short “Core ideas” sections before function lists (see `src/engine/starlark_guest/docs.rs` reference intro pattern).

More examples: [documentation-style.md](documentation-style.md).

## Code Quality

- No `unwrap()` or `expect()` in production code (enforced by workspace lints)
- Use `anyhow` for all errors, including context. Do not use thiserror.
- All public APIs must have doc comments with examples
- Run `cargo clippy --all-targets -- -Dwarnings` and `cargo fmt` before commits
- No unsafe code (forbidden at workspace level)
- After any change, run `cargo format`
- After any major change, run `make check` which will run unit tests, clippy, and cargo format.

## Modularity

- Prefer small, focused modules over large, monolithic ones
- Break down code into smaller, more manageable pieces when it becomes too complex
- Look for abstract and general solutions to problems, then use them in specific cases
- Avoid traits where possible; prefer enums for multiple related cases

## Testing

- Every module must have comprehensive unit tests (aim for 100% coverage)
- Unit tests in same file as code using `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `tempfile` for test fixtures
- Test both success and error paths

## Verification workflow

After edits:

1. **Any change:** `cargo fmt` (or `make fmt`)
2. **Major change:** `make check` (tests + clippy with warnings denied + format check)
3. **UI / WASM:** `make check-wasm` when touching WASM paths or `no-default-features` builds

Run `cargo test` frequently while developing engine logic.
