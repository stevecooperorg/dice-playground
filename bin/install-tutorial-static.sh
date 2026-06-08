#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST="${1:-${CRATE}/dist}"

exec "${SCRIPT_DIR}/build-tutorial-site.sh" "${DIST}"
