#!/usr/bin/env bash
# Compile docs to static HTML via `dice render` / `dice render-md` (no Pandoc).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
if command -v git >/dev/null 2>&1 && git -C "${ROOT}" rev-parse --show-toplevel >/dev/null 2>&1; then
  ROOT="$(git -C "${ROOT}" rev-parse --show-toplevel)"
fi

DOCS="${ROOT}/docs/tutorial"
SITE="${ROOT}/tutorial-static"
DIST="${1:-${ROOT}/dist}"
OUT="${DIST}/tutorial"
COOKBOOK_OUT="${DIST}/cookbook"
GUIDE_OUT="${DIST}/docs"
REF_OUT="${DIST}/references"

rm -rf "${OUT}" "${COOKBOOK_OUT}" "${GUIDE_OUT}" "${REF_OUT}"
mkdir -p "${OUT}" "${COOKBOOK_OUT}" "${GUIDE_OUT}" "${REF_OUT}"

cp "${SITE}/tutorial.css" "${OUT}/"

literate_title_from_dice() {
  local dice="$1"
  sed -n 's/^#[[:space:]]*//p' "${dice}" | head -1
}

INDEX_ITEMS=""
LESSON_COUNT=0

shopt -s nullglob
for dice in $(printf '%s\n' "${DOCS}"/*.dice | sort); do
  slug="$(basename "${dice}" .dice)"
  cargo run --quiet --bin dice -- render "${dice}" -o "${OUT}/${slug}.html"
  title="$(literate_title_from_dice "${dice}")"
  if [[ -z "${title}" ]]; then
    title="${slug}"
  fi
  LESSON_COUNT=$((LESSON_COUNT + 1))
  INDEX_ITEMS="${INDEX_ITEMS}
        <li>
          <a href=\"${slug}.html\">
            <strong>${LESSON_COUNT}. ${title}</strong>
          </a>
        </li>"
done
shopt -u nullglob

cat >"${OUT}/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Dice tutorial — Playground</title>
    <link rel="stylesheet" href="tutorial.css" />
  </head>
  <body>
    <header>
      <strong>Dice language tutorial</strong>
      <a href="../docs/index.html">User guide</a>
      <a href="../cookbook/index.html">Cookbook</a>
      <a href="../references/">Function reference</a>
      <a href="/">Open playground</a>
    </header>
    <main>
      <h1>Tutorial</h1>
      <p class="muted">
        Step-by-step lessons for tabletop players who want exact odds. Each lesson includes a
        script to copy into the <a href="/">playground</a> editor—click <strong>Run</strong> (or
        <strong>Shift+Enter</strong>) and read the woven <strong>report</strong>.
      </p>
      <ol class="lesson-list">${INDEX_ITEMS}
      </ol>
      <p class="muted">
        <a href="../references/">Function reference</a> for builtins and types.
        <a href="../cookbook/index.html">Cookbook</a> — short mechanic recipes.
      </p>
    </main>
  </body>
</html>
EOF

COOKBOOK_DOCS="${ROOT}/docs/cookbook"
COOKBOOK_INDEX_ITEMS=""
COOKBOOK_COUNT=0

shopt -s nullglob
for dice in $(printf '%s\n' "${COOKBOOK_DOCS}"/*.dice | sort); do
  slug="$(basename "${dice}" .dice)"
  cargo run --quiet --bin dice -- render "${dice}" -o "${COOKBOOK_OUT}/${slug}.html" --layout cookbook
  title="$(literate_title_from_dice "${dice}")"
  if [[ -z "${title}" ]]; then
    title="${slug}"
  fi
  COOKBOOK_COUNT=$((COOKBOOK_COUNT + 1))
  COOKBOOK_INDEX_ITEMS="${COOKBOOK_INDEX_ITEMS}
        <li>
          <a href=\"${slug}.html\">
            <strong>${title}</strong>
          </a>
        </li>"
done
shopt -u nullglob

if [[ ${COOKBOOK_COUNT} -gt 0 ]]; then
  cat >"${COOKBOOK_OUT}/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Dice cookbook — Playground</title>
    <link rel="stylesheet" href="../tutorial/tutorial.css" />
  </head>
  <body>
    <header>
      <strong>Cookbook</strong>
      <a href="../docs/index.html">User guide</a>
      <a href="../tutorial/index.html">Tutorial</a>
      <a href="/">Open playground</a>
    </header>
    <main>
      <h1>Cookbook</h1>
      <p class="muted">
        Short recipes for common tabletop mechanics. Open a recipe in the
        <a href="/">playground</a> and click <strong>Run</strong> for a woven report.
      </p>
      <ol class="lesson-list">${COOKBOOK_INDEX_ITEMS}
      </ol>
    </main>
  </body>
</html>
EOF
fi

REF_MD="${ROOT}/docs/references/stdlib.md"
if [[ -f "${REF_MD}" ]]; then
  cargo run --quiet --bin dice -- render-md "${REF_MD}" -o "${REF_OUT}/stdlib.html" --layout reference
  cp "${REF_OUT}/stdlib.html" "${REF_OUT}/index.html"
fi

GUIDE_MD="${ROOT}/docs/README.md"
if [[ -f "${GUIDE_MD}" ]]; then
  cargo run --quiet --bin dice -- render-md "${GUIDE_MD}" -o "${GUIDE_OUT}/index.html" --layout guide
fi

LLMS_TXT="${ROOT}/llms.txt"
if [[ -f "${LLMS_TXT}" ]]; then
  cp "${LLMS_TXT}" "${DIST}/llms.txt"
  echo "Copied llms.txt: ${DIST}/llms.txt"
fi

cargo run --quiet --bin dice -- enhance-static-site "${DIST}"
echo "Injected playground load links (dice enhance-static-site)"

echo "Built tutorial site: ${OUT} (${LESSON_COUNT} lessons)"
[[ ${COOKBOOK_COUNT} -gt 0 ]] && echo "Built cookbook: ${COOKBOOK_OUT} (${COOKBOOK_COUNT} recipes)"
[[ -f "${GUIDE_OUT}/index.html" ]] && echo "Built user guide: ${GUIDE_OUT}/index.html"
[[ -f "${REF_OUT}/index.html" ]] && echo "Built reference: ${REF_OUT}/index.html"
