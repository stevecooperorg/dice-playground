# Deploy on Cloudflare Workers

Ship the static site from `dist/` with [Wrangler](https://developers.cloudflare.com/workers/wrangler/) and **Workers Static Assets** (no custom Worker script required).

## Prerequisites

- [Cloudflare account](https://dash.cloudflare.com/sign-up)
- Node.js 18+ (for Wrangler)
- Build tools: Rust, Trunk, Pandoc (see [README](../README.md))
- Log in once: `npx wrangler login`

## One-time setup

1. Edit **`wrangler.toml`** and set `name` to a unique Worker name in your account (e.g. `my-dice-playground`).
2. Install Wrangler locally:

   ```bash
   make cf-install
   ```

## Deploy

```bash
make cf-deploy
```

This runs `make release-static` (release WASM + tutorial/reference HTML into `dist/`), then `wrangler deploy`.

Your app is served at `https://<name>.<your-subdomain>.workers.dev` (Wrangler prints the URL). Attach a custom domain in the Cloudflare dashboard under **Workers & Pages → your worker → Settings → Domains & Routes**.

## Preview locally

After a release build:

```bash
make cf-preview
```

Opens the deployed asset bundle at http://127.0.0.1:8787 (same files as production, without uploading).

## Routing

- **`not_found_handling = "single-page-application"`** — unknown paths fall back to `/index.html` (playground shell).
- **`html_handling = "auto-trailing-slash"`** — `/tutorial/` serves `tutorial/index.html`; lesson links use `.html` paths from the Pandoc build.

Tutorial, cookbook, reference, and user guide (`/docs/`) trees are real static files under `/tutorial/`, `/cookbook/`, `/references/`, and `/docs/`, so they are not swallowed by the SPA fallback when the file exists.

## CI

Set `CLOUDFLARE_API_TOKEN` (Workers edit permission) and run:

```bash
make cf-install
make cf-deploy
```

Use a token from **My Profile → API Tokens → Create Token → Edit Cloudflare Workers** template.

## Other hosts

Any static host can serve the same `dist/` output; see [deploy-cdn.md](./deploy-cdn.md).
