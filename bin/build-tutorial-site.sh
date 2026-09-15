#!/usr/bin/env bash
# Compile docs with the CLI, then publish the validated learning inventory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST="${1:-${ROOT}/dist}"
OUT="${DIST}/tutorial"
REF_OUT="${DIST}/references"
GUIDE_OUT="${DIST}/docs"

# Validate before removing any prior generated output.
python3 "${SCRIPT_DIR}/learning-site.py"
rm -rf "${OUT}" "${DIST}/cookbook" "${GUIDE_OUT}" "${REF_OUT}"
mkdir -p "${OUT}" "${DIST}/cookbook" "${GUIDE_OUT}" "${REF_OUT}"
cp "${ROOT}/tutorial-static/tutorial.css" "${ROOT}/tutorial-static/open-document.js" "${OUT}/"

for section in tutorial cookbook; do
  for dice in "${ROOT}/docs/${section}/"*.dice; do
    slug="$(basename "${dice}" .dice)"
    cp "${dice}" "${DIST}/${section}/${slug}.dice"
    cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- render \
      "${dice}" -o "${DIST}/${section}/${slug}.html" --layout "${section}"
  done
done

REF_MD="${REF_OUT}/stdlib.md"
cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- docs --out "${REF_MD}"
cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- render-md "${REF_MD}" -o "${REF_OUT}/stdlib.html" --layout reference
cp "${REF_OUT}/stdlib.html" "${REF_OUT}/index.html"
cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- render-md "${ROOT}/docs/references/api-conventions.md" -o "${REF_OUT}/api-conventions.html" --layout reference
cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- render-md "${ROOT}/docs/README.md" -o "${GUIDE_OUT}/index.html" --layout guide
cp "${ROOT}/llms.txt" "${DIST}/llms.txt"

python3 "${SCRIPT_DIR}/learning-site.py" "${DIST}"
cargo run --quiet --manifest-path "${ROOT}/Cargo.toml" --bin dice -- enhance-static-site "${DIST}"
python3 "${SCRIPT_DIR}/check-learning-links.py" "${DIST}"
echo "Built and link-checked learning site: ${DIST}"
