#!/usr/bin/env bash
# Compile docs/tutorial and docs/cookbook to static HTML; references to dist/references/.
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

if ! command -v pandoc >/dev/null 2>&1; then
  echo "pandoc is required to build the tutorial site (brew install pandoc / apt install pandoc)" >&2
  exit 1
fi

rm -rf "${OUT}" "${COOKBOOK_OUT}" "${GUIDE_OUT}" "${REF_OUT}"
mkdir -p "${OUT}" "${COOKBOOK_OUT}" "${GUIDE_OUT}" "${REF_OUT}"

cp "${SITE}/tutorial.css" "${OUT}/"

rewrite_lesson_links() {
  sed -E \
    -e 's|\]\(\.\./README\.md\)|](../docs/index.html)|g' \
    -e 's|\]\(\.\./references/stdlib\.md\)|](../references/stdlib.html)|g' \
    -e 's|\]\(\.\./references/stdlib\.md#([^)]*)\)|](../references/stdlib.html#\1)|g' \
    -e 's|\]\(\.\./README\.md#([^)]*)\)|](../docs/index.html#\1)|g' \
    -e 's|\]\(\.\./\.\./README\.md#([^)]*)\)|](../docs/index.html#\1)|g' \
    -e 's|\]\(\.\./\.\./README\.md\)|](../docs/index.html)|g' \
    -e 's|\]\(\./([0-9]{2}-[^)]+)\.md\)|](\1.html)|g' \
    -e 's|\]\(([0-9]{2}-[^)]+)\.md\)|](\1.html)|g'
}

rewrite_cookbook_links() {
  sed -E \
    -e 's|\]\(README\.md\)|](index.html)|g' \
    -e 's|\]\(\.\./README\.md#tutorial\)|](../docs/index.html#tutorial)|g' \
    -e 's|\]\(\.\./README\.md\)|](../docs/index.html)|g' \
    -e 's|\]\(\.\./references/stdlib\.md\)|](../references/stdlib.html)|g' \
    -e 's|\]\(\.\./tutorial/([0-9]{2}-[^)]+)\.md\)|](../tutorial/\1.html)|g' \
    -e 's|\]\(([a-z0-9-]+)\.md\)|](\1.html)|g'
}

rewrite_guide_links() {
  sed -E \
    -e 's|\]\(\.\./README\.md\)|](https://github.com/stevecooperorg/dice-playground)|g' \
    -e 's|\]\(tutorial/([0-9]{2}-[^)]+)\.md\)|](../tutorial/\1.html)|g' \
    -e 's|\]\(cookbook/README\.md\)|](../cookbook/index.html)|g' \
    -e 's|\]\(references/stdlib\.md\)|](../references/stdlib.html)|g' \
    -e 's|\]\(references/stdlib\.md#([^)]*)\)|](../references/stdlib.html#\1)|g' \
    -e 's|\]\(references/README\.md\)|](../references/index.html)|g'
}

frontmatter_title() {
  local md="$1"
  sed -n '/^---$/,/^---$/{
    /^title:/{
      s/^title:[[:space:]]*"\(.*\)"$/\1/
      s/^title:[[:space:]]*\(.*\)$/\1/
      p
      q
    }
  }' "${md}" | head -1
}

INDEX_ITEMS=""
LESSON_COUNT=0

shopt -s nullglob
for md in "${DOCS}"/*.md; do
  slug="$(basename "${md}" .md)"
  tmp="$(mktemp)"
  rewrite_lesson_links <"${md}" >"${tmp}"
  pandoc "${tmp}" \
    --from markdown \
    --to html5 \
    --standalone \
    --template "${SITE}/document.html" \
    --css tutorial.css \
    -o "${OUT}/${slug}.html"
  rm -f "${tmp}"

  title="$(frontmatter_title "${md}")"
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
        <strong>Shift+Enter</strong>) and read results under <strong>Output</strong>.
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
for md in "${COOKBOOK_DOCS}"/*.md; do
  slug="$(basename "${md}" .md)"
  if [[ "${slug}" == "README" ]]; then
    continue
  fi
  tmp="$(mktemp)"
  rewrite_cookbook_links <"${md}" >"${tmp}"
  pandoc "${tmp}" \
    --from markdown \
    --to html5 \
    --standalone \
    --template "${SITE}/cookbook-document.html" \
    --css ../tutorial/tutorial.css \
    -o "${COOKBOOK_OUT}/${slug}.html"
  rm -f "${tmp}"

  title="$(frontmatter_title "${md}")"
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

if [[ -f "${COOKBOOK_DOCS}/README.md" ]]; then
  tmp="$(mktemp)"
  rewrite_cookbook_links <"${COOKBOOK_DOCS}/README.md" >"${tmp}"
  pandoc "${tmp}" \
    --from markdown \
    --to html5 \
    --standalone \
    --template "${SITE}/cookbook-document.html" \
    --css ../tutorial/tutorial.css \
    -o "${COOKBOOK_OUT}/index.html"
  rm -f "${tmp}"
else
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
      <ol class="lesson-list">${COOKBOOK_INDEX_ITEMS}
      </ol>
    </main>
  </body>
</html>
EOF
fi
shopt -u nullglob

REF_MD="${ROOT}/docs/references/stdlib.md"
if [[ -f "${REF_MD}" ]]; then
  tmp="$(mktemp)"
  sed -E 's|\]\(\.\./tutorial/([0-9]{2}-[^)]+)\.md\)|](../tutorial/\1.html)|g' <"${REF_MD}" >"${tmp}"
  pandoc "${tmp}" \
    --from markdown \
    --to html5 \
    --standalone \
    --template "${SITE}/reference-document.html" \
    --css ../tutorial/tutorial.css \
    -o "${REF_OUT}/stdlib.html"
  cp "${REF_OUT}/stdlib.html" "${REF_OUT}/index.html"
  rm -f "${tmp}"
fi

GUIDE_MD="${ROOT}/docs/README.md"
if [[ -f "${GUIDE_MD}" ]]; then
  tmp="$(mktemp)"
  rewrite_guide_links <"${GUIDE_MD}" >"${tmp}"
  pandoc "${tmp}" \
    --from markdown \
    --to html5 \
    --standalone \
    --template "${SITE}/guide-document.html" \
    --css ../tutorial/tutorial.css \
    -o "${GUIDE_OUT}/index.html"
  rm -f "${tmp}"
fi

echo "Built tutorial site: ${OUT} (${LESSON_COUNT} lessons)"
[[ ${COOKBOOK_COUNT} -gt 0 ]] && echo "Built cookbook: ${COOKBOOK_OUT} (${COOKBOOK_COUNT} recipes)"
[[ -f "${GUIDE_OUT}/index.html" ]] && echo "Built user guide: ${GUIDE_OUT}/index.html"
[[ -f "${REF_OUT}/index.html" ]] && echo "Built reference: ${REF_OUT}/index.html"
