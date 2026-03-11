#!/usr/bin/env python3
"""Install or update the Mister Smith control-plane MCP config and compatibility shims."""

from __future__ import annotations

import argparse
import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CODEX_HOME = Path.home() / ".codex"
DEFAULT_CONTROL_PLANE_REPO = Path.home() / "Repos" / "mister-smith-constitutional-control-plane"

MANAGED_BLOCK_BEGIN = "# BEGIN MISTER SMITH CONTROL PLANE"
MANAGED_BLOCK_END = "# END MISTER SMITH CONTROL PLANE"

LEGACY_SKILLS: Dict[str, str] = {
    "symphony-linear-mister-smith": "Use when work spans Symphony runtime, Linear control-plane state, GitHub PR flow, and the Mister Smith repository.",
    "stage-mister-smith-phase": "Use when a Mister Smith phase/spec pack needs to be sliced into runnable Linear work through the control plane.",
    "symphony-mister-smith-review-dispatch": "Use when Human Review landing and watched-queue refill need to run through the deterministic review-dispatch loop.",
    "mister-smith-frontier-mandate": "Use when frontier legitimacy, autonomy leverage, or security-versus-autonomy tradeoffs need a Mister Smith judgment.",
}


@dataclass(frozen=True)
class BootstrapPaths:
    codex_home: Path
    config_path: Path
    control_plane_repo: Path


def canonical_skill_path(skill_name: str) -> Path:
    return REPO_ROOT / ".codex" / "skills" / skill_name / "SKILL.md"


def render_config_block(paths: BootstrapPaths) -> str:
    tsx_path = paths.control_plane_repo / "node_modules" / ".bin" / "tsx"
    server_path = paths.control_plane_repo / "src" / "server.ts"
    return "\n".join(
        [
            MANAGED_BLOCK_BEGIN,
            '[mcp_servers.mistersmith_control_plane]',
            f'command = "{tsx_path}"',
            f'args = ["{server_path}"]',
            f'cwd = "{paths.control_plane_repo}"',
            "enabled = true",
            (
                'env = { CONTROL_PLANE_TRANSPORT = "stdio", '
                f'MISTER_SMITH_REPO_ROOT = "{REPO_ROOT}", '
                f'MISTER_SMITH_CODEX_CONFIG_PATH = "{paths.config_path}" }}'
            ),
            MANAGED_BLOCK_END,
        ]
    )


def upsert_managed_block(existing: str, managed_block: str) -> str:
    existing = existing.rstrip()
    if MANAGED_BLOCK_BEGIN in existing and MANAGED_BLOCK_END in existing:
        start = existing.index(MANAGED_BLOCK_BEGIN)
        end = existing.index(MANAGED_BLOCK_END) + len(MANAGED_BLOCK_END)
        updated = existing[:start].rstrip()
        tail = existing[end:].lstrip()
        pieces = [piece for piece in [updated, managed_block, tail] if piece]
        return "\n\n".join(pieces).rstrip() + "\n"

    if not existing:
        return managed_block.rstrip() + "\n"

    return existing + "\n\n" + managed_block.rstrip() + "\n"


def render_skill_shim(skill_name: str, description: str) -> str:
    canonical_path = canonical_skill_path(skill_name)
    return "\n".join(
        [
            "---",
            f"name: {skill_name}",
            f"description: {description}",
            "---",
            "",
            f"# {skill_name}",
            "",
            "This is a migration shim.",
            "",
            f"Canonical repo-local skill: [{canonical_path}]({canonical_path})",
            "",
            "Use the repo-local skill inside `/Users/matthewmaggio/Mister-Smith/.codex/skills/` as the authoritative version.",
            "If the repo-local skill pack or MCP registration is missing, run the repo-local bootstrap skill or `python3 scripts/bootstrap_control_plane.py` from the Mister Smith repo.",
        ]
    ) + "\n"


def ensure_parent(path: Path, dry_run: bool) -> None:
    if dry_run:
        return
    path.parent.mkdir(parents=True, exist_ok=True)


def write_if_changed(path: Path, content: str, dry_run: bool) -> bool:
    current = path.read_text(encoding="utf-8") if path.exists() else None
    if current == content:
        return False

    ensure_parent(path, dry_run)
    if not dry_run:
        path.write_text(content, encoding="utf-8")
    return True


def install_config(paths: BootstrapPaths, dry_run: bool) -> bool:
    existing = paths.config_path.read_text(encoding="utf-8") if paths.config_path.exists() else ""
    updated = upsert_managed_block(existing, render_config_block(paths))
    return write_if_changed(paths.config_path, updated, dry_run=dry_run)


def install_shims(codex_home: Path, dry_run: bool) -> List[str]:
    changed = []
    for skill_name, description in LEGACY_SKILLS.items():
        shim_path = codex_home / "skills" / skill_name / "SKILL.md"
        if write_if_changed(shim_path, render_skill_shim(skill_name, description), dry_run=dry_run):
            changed.append(str(shim_path))
    return changed


def validate_paths(paths: BootstrapPaths) -> List[str]:
    problems = []
    if not paths.control_plane_repo.exists():
        problems.append(f"Control-plane repo does not exist: {paths.control_plane_repo}")
    if not canonical_skill_path("mister-smith-control-plane-router").exists():
        problems.append("Canonical repo-local skill pack is not installed in Mister-Smith/.codex/skills yet.")
    return problems


def run_bootstrap(paths: BootstrapPaths, dry_run: bool) -> dict:
    problems = validate_paths(paths)
    config_changed = install_config(paths, dry_run=dry_run)
    shim_changes = install_shims(paths.codex_home, dry_run=dry_run)

    return {
        "dry_run": dry_run,
        "config_path": str(paths.config_path),
        "control_plane_repo": str(paths.control_plane_repo),
        "config_changed": config_changed,
        "shim_changes": shim_changes,
        "problems": problems,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Install the managed Mister Smith control-plane MCP block and compatibility skill shims.",
    )
    parser.add_argument("--dry-run", action="store_true", help="Render changes without writing files.")
    parser.add_argument(
        "--codex-home",
        default=str(DEFAULT_CODEX_HOME),
        help="Codex home directory containing config.toml and skills/.",
    )
    parser.add_argument(
        "--control-plane-repo",
        default=os.environ.get("MISTER_SMITH_CONTROL_PLANE_REPO", str(DEFAULT_CONTROL_PLANE_REPO)),
        help="Path to the mister-smith-constitutional-control-plane repository.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    codex_home = Path(args.codex_home).expanduser()
    paths = BootstrapPaths(
        codex_home=codex_home,
        config_path=codex_home / "config.toml",
        control_plane_repo=Path(args.control_plane_repo).expanduser(),
    )
    result = run_bootstrap(paths, dry_run=args.dry_run)
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
