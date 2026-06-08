#!/usr/bin/env bash
# Fingerprint host static/ so Docker/ACA publishes pick up UI changes.
set -euo pipefail

STATIC_DIR="${1:?static directory}"
STAMP="${STATIC_DIR}/.ui-build-stamp"

if [[ ! -f "${STATIC_DIR}/index.html" ]]; then
  echo "Missing ${STATIC_DIR}/index.html (run Trunk build + copy-static first)" >&2
  exit 1
fi

hash="$(
  find "${STATIC_DIR}" -type f ! -name '.ui-build-stamp' -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 \
    | shasum -a 256 \
    | awk '{print $1}'
)"
printf '%s\n' "${hash}" > "${STAMP}"
echo "Wrote ${STAMP} (${hash})"
