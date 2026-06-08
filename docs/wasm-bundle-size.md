# WASM and `dist/` bundle size

Most of what people call a “23 MB dist” is an **unoptimized debug WASM** build. A **CDN release** (`make release-static`) is dominated by a single **~3 MB** `.wasm` file (often **~1 MB gzip** on the wire).

## Quick reference

| Build | Command | Typical `.wasm` | Notes |
|--------|---------|-----------------|--------|
| Dev / debug | `make static`, `trunk build`, `make serve` | **~24 MB** | Fast compile, no LTO, debug symbols |
| Release (CDN) | `make release-static` | **~3 MB** | `[profile.release]` + `--no-default-features` for wasm |
| Release + gzip | CDN `Content-Encoding: gzip` | **~1 MB** | Host compresses `.wasm` |
| Release + brotli | CDN brotli | **~850 KB–1 MB** | Usually smaller than gzip |

Tutorial and reference HTML under `dist/tutorial/` and `dist/references/` add **~70 KB** total.

After changing release profile settings, run `make release-static` and check sizes (the Makefile prints a short report).

---

## Why debug is so large

1. **Trunk dev builds** use Cargo’s **dev** profile unless you pass `--release`.
2. **`data-wasm-opt="0"`** in `index.html` skips Binaryen **wasm-opt** so local builds stay fast (see below).
3. The app links **Starlark** (interpreter + types) and **Leptos CSR** — a large but expected dependency tree for an in-browser evaluator.

---

## Compiler and Cargo settings (already used for release)

Root `Cargo.toml` `[profile.release]`:

- **`opt-level = "z"`** — optimize for size (slightly smaller wasm than `"s"`).
- **`lto = true`**, **`codegen-units = 1`** — cross-crate inlining and dead-code elimination.
- **`strip = true`** — strip symbols from linked artifacts.
- **`panic = "abort"`** — no unwinding tables in wasm (smaller binary; panics trap).

WASM builds exclude CLI/LSP via Trunk/Cargo:

```bash
cargo check --target wasm32-unknown-unknown --no-default-features
```

That drops `clap`, `csv`, and `starlark_lsp` from the browser bundle.

Optional **target-specific** flags in `.cargo/config.toml` (if you need more):

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+bulk-memory"]
```

Only add flags you have measured; defaults are usually enough.

---

## Trunk and `wasm-opt` (Binaryen)

In `index.html`:

```html
<link data-trunk rel="rust" data-bin="dice-playground" data-wasm-opt="0" />
```

| `data-wasm-opt` | Effect |
|-----------------|--------|
| `0` | Skip wasm-opt (default here) — reliable, fast |
| `s` | `-Os` size optimization |
| `4` | `-O4` aggressive (slow, may fail on some toolchains) |

Trunk downloads its own `wasm-opt`. If post-link optimization **fails** (exit 1), keep `0` and rely on the Rust release profile; you still get ~3 MB wasm.

To try manually after a release build:

```bash
wasm-opt -Os -o dist/opt.wasm dist/*_bg.wasm
```

Install [Binaryen](https://github.com/WebAssembly/binaryen) if you want a system `wasm-opt`.

---

## Post-build and CDN

1. **Always ship release:** `make release-static`, not `make static`.
2. **Compress at the edge:** Enable **gzip** and/or **brotli** for `.wasm` and `.js`. Browsers accept precompressed assets when `Content-Encoding` matches.
3. **`Cache-Control`:** Long cache for hashed filenames (`dice-playground-*_bg.wasm`); short or no cache for `index.html`.
4. **Precompress in CI (optional):** `gzip -9 -k dist/*_bg.wasm` and upload `.wasm.gz` if your host supports serving them with correct headers.

Tailwind is loaded from **cdn.tailwindcss.com** in `index.html` (not part of `dist/` size). For offline or stricter CSP, replace with a built CSS file (trade-off: smaller third-party dependency vs larger static CSS).

---

## Profiling what is inside the wasm

On a release `.wasm` (or the cdylib artifact under `target/wasm32-unknown-unknown/release/`):

```bash
# Install once
cargo install twiggy

twiggy top -n 30 dist/*_bg.wasm
twiggy dominators dist/*_bg.wasm
```

Use this to see whether **starlark**, **serde**, **leptos**, or app code dominates before refactoring.

Rust native bloat (for comparison on host triple, not wasm):

```bash
cargo bloat --release --target wasm32-unknown-unknown --bin dice-playground -n 30 --no-default-features
```

---

## Architecture changes (larger effort, bigger wins)

These are ordered roughly by impact vs effort.

### 1. Lazy-loaded eval worker

Load a **small main thread** (UI only) and **`import()` a worker** that owns Starlark eval. Users get faster first paint; eval code downloads when they first run. Two wasm modules or one wasm + worker entrypoint — design depends on `wasm-bindgen` worker support and shared state.

### 2. Split crates: `dice-engine-wasm` vs UI

- **`dice-engine`**: Starlark, PMF, playground API — `cdylib` for wasm.
- **`dice-playground-ui`**: Leptos, depends on engine via **JS/wasm boundary** or **thin FFI**.

Lets you ship engine-only wasm for embeds and keeps UI rebuilds from relinking Starlark when unchanged (with careful caching).

### 3. Trim Starlark / engine surface

- Audit which Starlark modules and stdlib hooks are required for the playground vs full CLI.
- Replace rarely used Rust helpers with smaller implementations.
- Avoid pulling **allocative** or debug-only paths into release wasm if any remain feature-gated on native only.

### 4. Leptos and UI

- Ensure **only CSR** features are enabled (already `leptos` with `csr`).
- Reduce component tree or defer heavy panels until opened.
- Prefer lightweight DOM updates over large in-memory structures in the main thread.

### 5. Alternative eval backends (research)

A smaller expression language or precompiled dice IR would shrink wasm dramatically but is a **product** change, not a build tweak.

---

## Dev workflow vs production

| Goal | Use |
|------|-----|
| Fast iteration | `make serve` (debug wasm, ~24 MB — fine locally) |
| Size check before deploy | `make release-static` |
| CI gate (optional) | Fail if `*_bg.wasm` exceeds a threshold (e.g. 4 MB) |

---

## Related

- [deploy-cdn.md](./deploy-cdn.md) — upload `dist/` after `make release-static`
- [AGENT.md](./AGENT.md) — repo layout and wasm32 constraints
