# CDN / static hosting

Ship the **`dist/`** directory produced by:

```bash
make release-static
```

Expect **~3 MB** total (mostly one release `.wasm`; **~1 MB** over gzip). Debug builds (`make static` / `trunk build` without `--release`) produce **~24 MB** wasm — do not upload those. See [wasm-bundle-size.md](./wasm-bundle-size.md).

Contents:

- Playground WASM app (`index.html`, JS, `.wasm`)
- `/docs/` — user guide hub (links to tutorial, cookbook, reference)
- `/tutorial/` — step-by-step lessons (`dice render` HTML)
- `/cookbook/` — mechanic recipes (`dice render` HTML)
- `/references/` — function reference (generated stdlib)

## Upload

Upload all of `dist/` to any static host (S3 + CloudFront, GitHub Pages, Azure Static Web Apps, etc.).

**Cloudflare Workers:** use Wrangler from this repo — [deploy-cloudflare.md](./deploy-cloudflare.md) (`make cf-deploy`).

## SPA fallback (optional)

The playground is primarily `/` plus static paths under `/docs/`, `/tutorial/`, `/cookbook/`, and `/references/`. If you add client-side routes later, configure a fallback to `/index.html`.

Example **Azure Static Web Apps** (`staticwebapp.config.json` at site root):

```json
{
  "navigationFallback": {
    "rewrite": "/index.html",
    "exclude": ["/tutorial/*", "/cookbook/*", "/docs/*", "/references/*", "/*.{css,js,wasm,png,svg,ico}"]
  }
}
```

## Local preview

```bash
make serve
# or serve dist/ with any static file server after make release-static
```
