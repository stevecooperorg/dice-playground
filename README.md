# Dice Playground

**Live app:** [https://dice-playground.stevecooperorg.workers.dev/](https://dice-playground.stevecooperorg.workers.dev/)

## What is this?

**Dice Playground** is a browser app for `.dice` scripts that compute **exact** probabilities—meet DC 15 on `1d20+7`, miss / partial / hit on a `2d6+mod` move, every total on `4d6dl1`, and similar—not approximations from rolling thousands of times.

For one number—success on a target—you can keep the script small. A classic “roll plus bonus, need at least *N*” check might look like:

```text
roll = 2d10 + 3
output("p_at_least_15", roll.p_ge(15))
```

Run it in the **editor** (**Run** or **Shift+Enter**). **Output** gives a single probability (here, meeting or beating 15 on `2d10+3`).

For a full distribution, D&D-style ability scores are the familiar case: roll four d6, drop the lowest, sum the rest:

```text
output("ability", 4d6dl1)
```

Same steps in the playground. The **text** tab lists every total and its exact probability—not a simulation. For this roll the average is about **12.24**; you can read off how often you roll an 18, a 3, or anything between.

Longer scripts cover the rest of the table: advantage on the d20 (`2d20kh1`), natural 1 and 20 before modifiers, keep-highest pools (`3d6kh2`), exploding dice, save-for-half on `8d6`, or a grid of success rates across modifiers. Familiar faces use dice notation (`2d6`, `4d6dl1`, …); conditions, loops, and several named outputs use **Starlark**. Results appear as **text**, **JSON**, or a **graph**. Step-by-step notation is [lesson 5](docs/tutorial/05-dice-notation.md); worked recipes are in the cookbook (linked from the **[user guide](docs/README.md)**).

Implementation: exact probability (no sampling), Starlark with `.dice` sugar, Leptos WASM front end in one crate; optional **`dice` CLI** and LSP (see below).

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
