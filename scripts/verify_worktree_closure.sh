#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/verify_worktree_closure.sh [--fetch] [--require-upstream] [--require-sync]

Checks that the current Git worktree is clean. Optional flags also require a
configured upstream and a fully synced branch state.

Options:
  --fetch             Fetch the upstream remote before sync checks.
  --require-upstream  Fail if the current branch has no upstream.
  --require-sync      Fail unless the branch is exactly in sync with upstream.
  -h, --help          Show this help text.
EOF
}

require_upstream=false
require_sync=false
fetch_upstream=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fetch)
      fetch_upstream=true
      ;;
    --require-upstream)
      require_upstream=true
      ;;
    --require-sync)
      require_upstream=true
      require_sync=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if ! git rev-parse --show-toplevel >/dev/null 2>&1; then
  echo "Not inside a git repository." >&2
  exit 1
fi

branch="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
if [[ -z "$branch" ]]; then
  echo "Detached HEAD is not allowed for clean-closure verification." >&2
  exit 1
fi

porcelain="$(git status --porcelain=v1)"
if [[ -n "$porcelain" ]]; then
  echo "Worktree is not clean for branch '$branch':" >&2
  printf '%s\n' "$porcelain" >&2
  exit 1
fi

upstream=""
if git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' >/dev/null 2>&1; then
  upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}')"
fi

if [[ "$require_upstream" == true && -z "$upstream" ]]; then
  echo "Branch '$branch' has no configured upstream." >&2
  exit 1
fi

if [[ "$fetch_upstream" == true && -n "$upstream" ]]; then
  remote="${upstream%%/*}"
  git fetch "$remote" --prune >/dev/null 2>&1
fi

if [[ "$require_sync" == true ]]; then
  counts="$(git rev-list --left-right --count "${upstream}...HEAD")"
  behind="${counts%% *}"
  ahead="${counts##* }"
  if (( behind > 0 || ahead > 0 )); then
    echo "Branch '$branch' is not synced with '$upstream' (behind=$behind ahead=$ahead)." >&2
    exit 1
  fi
fi

echo "Worktree closure OK for '$branch'."
