# Dice Playground

**Live app:** [https://dice-playground.stevecooperorg.workers.dev/](https://dice-playground.stevecooperorg.workers.dev/)

## What is this?

If you design tabletop RPGs, you often need to know how a dice system behaves: how likely a check is to succeed, what a pool’s average looks like, or how advantage changes the odds. You can answer those questions by writing small scripts that describe the rolls and letting a tool compute the exact probabilities (not Monte Carlo guesses).

**Dice Playground** is a browser app for that. You write scripts in a Starlark-based language with `.dice` sugar, run them in the editor, and read precise distributions and probabilities under **Output**. The site bundles a **[user guide](docs/README.md)** (tutorial, cookbook, and function reference).

Under the hood: exact probability for every outcome (no sampling), **Starlark + `.dice` sugar**, and a **Leptos WASM** UI in one crate (plus an optional `dice` CLI for local development—see below).

## Example: D&D ability scores (4d6, drop lowest)

Classic character creation: roll four six-sided dice, drop the lowest, sum the other three. In a `.dice` script:

```text
output("ability", 4d6dl1)
```

In the playground, paste that into the **editor**, and click **Run**. The **text** tab under **Output** shows the **exact** distribution—every total and its probability—not a simulation. For this roll the average is about **12.24**; you can also see how often you get an 18, a 3, or anything in between. More notation (`2d6`, keep-highest pools, modifiers) is in [lesson 5](docs/tutorial/05-dice-notation.md) of the tutorial.

## Learn the language

Work through the tutorial from the playground **Menu → Tutorial**, or start at the [user guide](docs/README.md).

## Developing locally

```bash
make serve                    # http://127.0.0.1:8081 — playground + /docs/
make test
cargo run --bin dice -- eval examples/tutorial/01-one-die.dice
cargo run --bin dice -- lsp   # stdio language server (editor integration)
```

## Deploy your own copy

The repo includes [Cloudflare Workers](https://developers.cloudflare.com/workers/) deployment via Wrangler. You get the playground, tutorial, and stdlib reference as one static site.

1. Install [Node.js](https://nodejs.org/) and log in: `npx wrangler login`
2. Set a unique Worker name in [`wrangler.toml`](wrangler.toml) (`name = "…"`)
3. Install Wrangler and deploy:

```bash
make cf-install
make cf-deploy
```

Wrangler prints your `*.workers.dev` URL. Full steps, custom domains, and CI tokens: [docs/deploy-cloudflare.md](docs/deploy-cloudflare.md).

To build `dist/` without uploading (S3, GitHub Pages, Azure, etc.): `make release-static` — see [docs/deploy-cdn.md](docs/deploy-cdn.md). Bundle size: [docs/wasm-bundle-size.md](docs/wasm-bundle-size.md).

## Layout


| Path                 | Role                                                 |
| -------------------- | ---------------------------------------------------- |
| `src/engine/`        | Dice probability engine, Starlark guest, playground check/eval, LSP |
| `src/ui/`            | Leptos CSR playground                                |
| `src/bin/dice.rs`    | CLI (`eval`, `docs`, `table-2d10`, `lsp`)            |
| `docs/`              | User guide + references                              |
| `examples/tutorial/` | Sample `.dice` scripts (CI smoke tests)              |


## Docs

- [Tutorial / user guide](docs/README.md) (source for the hosted `/docs/`)
- [Agent / architecture notes](docs/AGENT.md)

## License

[MIT](LICENSE) — fork, modify, and deploy as you like; keep the copyright notice in distributions.
