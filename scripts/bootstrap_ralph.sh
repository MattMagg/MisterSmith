#!/usr/bin/env bash
set -euo pipefail

RALPH_REPO_URL="${RALPH_REPO_URL:-https://github.com/mikeyobrien/ralph-orchestrator.git}"
RALPH_STATE_ROOT="${RALPH_STATE_ROOT:-$HOME/.local/share/mister-smith/ralph-orchestrator}"
RALPH_SOURCE_ROOT="${RALPH_SOURCE_ROOT:-$RALPH_STATE_ROOT/source}"
RALPH_INSTALL_ROOT="${RALPH_INSTALL_ROOT:-$RALPH_STATE_ROOT/install}"
RALPH_GIT_REF="${RALPH_GIT_REF:-origin/main}"

mkdir -p "$RALPH_STATE_ROOT"

if [[ ! -d "$RALPH_SOURCE_ROOT/.git" ]]; then
  git clone "$RALPH_REPO_URL" "$RALPH_SOURCE_ROOT"
else
  current_remote="$(git -C "$RALPH_SOURCE_ROOT" remote get-url origin)"
  if [[ "$current_remote" != "$RALPH_REPO_URL" ]]; then
    git -C "$RALPH_SOURCE_ROOT" remote set-url origin "$RALPH_REPO_URL"
  fi
fi

if [[ -n "$(git -C "$RALPH_SOURCE_ROOT" status --porcelain --untracked-files=no)" ]]; then
  echo "Managed Ralph source checkout is dirty at $RALPH_SOURCE_ROOT; refusing to overwrite." >&2
  echo "Move or clean that checkout, then rerun ./scripts/bootstrap_ralph.sh." >&2
  exit 1
fi

git -C "$RALPH_SOURCE_ROOT" fetch origin --tags

target_commit="$(git -C "$RALPH_SOURCE_ROOT" rev-parse "$RALPH_GIT_REF")"
git -C "$RALPH_SOURCE_ROOT" checkout --detach "$target_commit" >/dev/null 2>&1

(
  cd "$RALPH_SOURCE_ROOT"
  cargo install --locked --force --path crates/ralph-cli --root "$RALPH_INSTALL_ROOT"
)

installed_bin="$RALPH_INSTALL_ROOT/bin/ralph"
if [[ ! -x "$installed_bin" ]]; then
  echo "Expected managed Ralph binary at $installed_bin after install." >&2
  exit 1
fi

installed_version="$("$installed_bin" --version)"
metadata_dir="$RALPH_STATE_ROOT/metadata"
metadata_file="$metadata_dir/current.env"
mkdir -p "$metadata_dir"

cat >"$metadata_file" <<EOF
RALPH_REPO_URL=$RALPH_REPO_URL
RALPH_GIT_REF=$RALPH_GIT_REF
RALPH_SOURCE_ROOT=$RALPH_SOURCE_ROOT
RALPH_INSTALL_ROOT=$RALPH_INSTALL_ROOT
RALPH_COMMIT=$target_commit
RALPH_VERSION=$installed_version
EOF

echo "Managed Ralph install updated."
echo "source_root=$RALPH_SOURCE_ROOT"
echo "install_root=$RALPH_INSTALL_ROOT"
echo "commit=$target_commit"
echo "version=$installed_version"
