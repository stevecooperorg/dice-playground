# AGENTS.md

Repo-wide guidance for AI agents. Project conventions, architecture, and
verification commands live in [`docs/AGENT.md`](docs/AGENT.md) and the
`dice-playground-standards` skill (`.agents/skills/dice-playground-standards/SKILL.md`).
Read those first for how the code is organized and how to make changes.

## Cursor Cloud specific instructions

Dice Playground is a single Rust crate that produces two things:

- **`dice` CLI** (native binary, default `cli`+`lsp` features) — `eval`, `render`,
  `docs`, `table-2d10`, `lsp`. Run with `cargo run --bin dice -- <subcommand>`.
- **Web playground** (Leptos CSR WASM) — served by [Trunk](https://trunkrs.dev)
  on `http://127.0.0.1:8081/`. There is no runtime server; output is a static
  `dist/`.

Standard commands are already documented in the `Makefile`, `README.md`, and
`docs/AGENT.md` (`make test`, `make check`, `make check-wasm`, `make serve`,
`make static`, `make release-static`). Prefer those. Notes below are the
non-obvious bits.

### Toolchain

- **Requires Rust stable ≥ 1.85** (edition 2024 is pulled in transitively via
  `allocative_derive`). The startup update script installs/activates `stable` and
  the `wasm32-unknown-unknown` target plus `clippy`/`rustfmt`; you should not need
  to manage the toolchain yourself.
- **`trunk`** (WASM bundler) is required to build/serve the web app and is
  installed by the update script. `trunk serve`/`trunk build` auto-download the
  matching `wasm-bindgen` CLI on first run (needs network); expect a short delay
  the first time.

### Running the web playground

- `make serve` runs the Trunk dev server on `127.0.0.1:8081` with hot reload. Its
  `pre_build` hook (`bin/build-tutorial-site.sh`) shells out to the `dice` binary
  to render the tutorial/cookbook/reference HTML into `static-site/`, so the
  native crate must compile for the server to come up. First start is slow (WASM
  compile + wasm-bindgen download); subsequent rebuilds are fast.
- Verify it's up with `curl -sf http://127.0.0.1:8081/` (serves the WASM app) and
  `http://127.0.0.1:8081/tutorial/index.html` (generated docs).

### Testing / linting caveats

- `make check` runs `cargo test` + `cargo clippy --all-targets -- -Dwarnings` +
  `cargo fmt --check`. Clippy denies `unwrap`/`expect` in non-test code.
- `cargo fmt --check` currently reports diffs on already-committed files under
  `src/engine/` because the pinned-repo formatting predates the current stable
  `rustfmt`. This is pre-existing and unrelated to any single change; do not
  mass-reformat existing files to "fix" it unless that is the explicit task.
- Touching WASM/UI paths: also run `make check-wasm`
  (`cargo check --target wasm32-unknown-unknown --no-default-features`).
