#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

export MISTER_SMITH_REPO_ROOT="${MISTER_SMITH_REPO_ROOT:-${REPO_ROOT}}"

cd "${REPO_ROOT}"
exec cargo run -q -p mister-smith-mcp --bin smith-mcp -- --repo-root "${REPO_ROOT}" "$@"
