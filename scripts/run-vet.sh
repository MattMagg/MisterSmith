#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FIND_SESSION_SCRIPT="$REPO_ROOT/scripts/find_codex_session.py"
EXPORT_SCRIPT="$REPO_ROOT/.codex/skills/vet/scripts/export_codex_session.py"

usage() {
  cat <<'EOF'
Usage: scripts/run-vet.sh [--session-file PATH] [--no-history] [vet args...]

Run vet from this repository using the repo's Codex profile and, by default,
Codex conversation history from the current workspace session.

Examples:
  scripts/run-vet.sh "Fix the vet workflow"
  scripts/run-vet.sh --base-commit main "Review this refactor"
  scripts/run-vet.sh --session-file ~/.codex/sessions/.../session.jsonl "Review this diff"
EOF
}

if [[ $# -gt 0 && ( "$1" == "--help" || "$1" == "-h" ) ]]; then
  usage
  exit 0
fi

if ! command -v vet >/dev/null 2>&1; then
  echo "vet is not installed. Install verify-everything first." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to discover Codex sessions and export history." >&2
  exit 1
fi

if [[ ! -f "$FIND_SESSION_SCRIPT" ]]; then
  echo "Codex session discovery helper not found at $FIND_SESSION_SCRIPT" >&2
  exit 1
fi

if [[ ! -f "$EXPORT_SCRIPT" ]]; then
  echo "Codex history export script not found at $EXPORT_SCRIPT" >&2
  exit 1
fi

SESSION_FILE=""
USE_HISTORY=1
VET_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --session-file)
      if [[ $# -lt 2 ]]; then
        echo "--session-file requires a path argument" >&2
        exit 1
      fi
      SESSION_FILE="$2"
      shift 2
      ;;
    --no-history)
      USE_HISTORY=0
      shift
      ;;
    *)
      VET_ARGS+=("$1")
      shift
      ;;
  esac
done

HAS_CONFIG=0
HAS_HISTORY_LOADER=0
for ((i = 0; i < ${#VET_ARGS[@]}; i++)); do
  case "${VET_ARGS[$i]}" in
    --config|-c)
      HAS_CONFIG=1
      ;;
    --history-loader)
      HAS_HISTORY_LOADER=1
      ;;
  esac
done

COMMAND=(vet)
if [[ $HAS_CONFIG -eq 0 ]]; then
  COMMAND+=(--config codex)
fi

if [[ $USE_HISTORY -eq 1 && $HAS_HISTORY_LOADER -eq 0 ]]; then
  if [[ -z "$SESSION_FILE" ]]; then
    if ! SESSION_FILE="$(python3 "$FIND_SESSION_SCRIPT" --cwd "$REPO_ROOT" 2>&1)"; then
      echo "$SESSION_FILE" >&2
      echo "Unable to auto-discover the current Codex session file." >&2
      echo "Retry with --session-file PATH or skip history with --no-history." >&2
      exit 1
    fi
  fi

  printf -v HISTORY_LOADER 'python3 %q --session-file %q' "$EXPORT_SCRIPT" "$SESSION_FILE"
  COMMAND+=(--history-loader "$HISTORY_LOADER")
fi

COMMAND+=("${VET_ARGS[@]}")
exec "${COMMAND[@]}"
