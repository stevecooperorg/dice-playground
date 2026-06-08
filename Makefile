.PHONY: help test check check-wasm serve static release-static references cli fmt clean \
	cf-install cf-deploy cf-preview FORCE

TRUNK ?= trunk
NPM ?= npm
WRANGLER ?= npx wrangler

all: help

help:
	@echo "Dice Playground — Makefile"
	@echo ""
	@echo "  make test            cargo test (engine + UI + integration)"
	@echo "  make check           test + clippy (-Dwarnings) + fmt --check"
	@echo "  make check-wasm      wasm32 check (no CLI/LSP features)"
	@echo "  make serve           Trunk dev server (:8081); pandoc required for /tutorial/"
	@echo "  make static          Trunk debug build + tutorial HTML in dist/"
	@echo "  make release-static  Trunk release build + tutorial (CDN artifact)"
	@echo "  make cf-install      npm install (Wrangler for Cloudflare deploy)"
	@echo "  make cf-deploy       release-static + wrangler deploy"
	@echo "  make cf-preview      release-static + wrangler dev (:8787)"
	@echo "  make references      Regenerate docs/references/stdlib.md"
	@echo "  dice enhance-static-site  Add playground links to built HTML under dist/"
	@echo "  make cli             cargo build --release --bin dice"
	@echo "  make fmt             cargo fmt"
	@echo "  make clean           rm -rf dist/"

test: FORCE
	cargo test

check: FORCE
	cargo test
	cargo clippy --all-targets -- -Dwarnings
	cargo fmt --check

check-wasm: FORCE
	cargo check --target wasm32-unknown-unknown --no-default-features

serve: FORCE
	env -u NO_COLOR -u TRUNK_NO_COLOR $(TRUNK) serve

static: FORCE
	@echo "*** Debug WASM (~24MB) — use 'make release-static' for CDN (~3MB wasm)"
	env -u NO_COLOR -u TRUNK_NO_COLOR $(TRUNK) build

release-static: FORCE
	env -u NO_COLOR -u TRUNK_NO_COLOR $(TRUNK) build --release
	@echo ""
	@echo "*** dist/ size report (CDN artifact)"
	@du -sh dist
	@WASM=$$(ls dist/*_bg.wasm 2>/dev/null | head -1); \
	if [ -n "$$WASM" ]; then \
	  echo "    wasm: $$(ls -lh "$$WASM" | awk '{print $$5}')  ($$WASM)"; \
	  echo "    wasm (gzip -9): $$(gzip -9 -c "$$WASM" | wc -c | awk '{printf "%.2f MB\n", $$1/1024/1024}')"; \
	fi

cf-install: FORCE
	$(NPM) install

cf-deploy: release-static FORCE
	$(WRANGLER) deploy

cf-preview: release-static FORCE
	$(WRANGLER) dev --ip 127.0.0.1 --port 8787

references: FORCE
	cargo run --bin dice -- docs --out docs/references/stdlib.md
	@echo "*** Wrote docs/references/stdlib.md"

cli: FORCE
	cargo build --release --bin dice

fmt: FORCE
	cargo fmt

clean: FORCE
	rm -rf dist static-site
