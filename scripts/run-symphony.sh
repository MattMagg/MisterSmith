#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SYMPHONY_ROOT="${SYMPHONY_ROOT:-$HOME/Repos/symphony/elixir}"
WORKFLOW_PATH="${WORKFLOW_PATH:-$REPO_ROOT/WORKFLOW.md}"
DOTENV_PATH="${DOTENV_PATH:-$REPO_ROOT/.env}"
SYMPHONY_PORT="${SYMPHONY_PORT:-4000}"

if [[ ! -d "$SYMPHONY_ROOT" ]]; then
  echo "Symphony checkout not found at $SYMPHONY_ROOT" >&2
  exit 1
fi

if [[ ! -f "$WORKFLOW_PATH" ]]; then
  echo "Workflow file not found at $WORKFLOW_PATH" >&2
  exit 1
fi

if [[ ! -f "$DOTENV_PATH" ]]; then
  echo "Env file not found at $DOTENV_PATH" >&2
  exit 1
fi

set -a
# Symphony does not auto-load repo .env files, so export this shell's copy first.
source "$DOTENV_PATH"
set +a

if [[ -z "${LINEAR_API_KEY:-}" ]]; then
  echo "LINEAR_API_KEY is missing after loading $DOTENV_PATH" >&2
  exit 1
fi

cd "$SYMPHONY_ROOT"

if command -v mise >/dev/null 2>&1; then
  exec mise exec -- ./bin/symphony "$WORKFLOW_PATH" --port "$SYMPHONY_PORT" --i-understand-that-this-will-be-running-without-the-usual-guardrails "$@"
fi

exec ./bin/symphony "$WORKFLOW_PATH" --port "$SYMPHONY_PORT" --i-understand-that-this-will-be-running-without-the-usual-guardrails "$@"
