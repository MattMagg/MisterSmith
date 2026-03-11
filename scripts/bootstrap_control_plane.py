#!/usr/bin/env python3
"""Install the repo-local Smith skill pack and verify Codex MCP bootstrap."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class SkillTemplate:
    name: str
    description: str
    body: str


SKILL_TEMPLATES = {
    "mister-smith-control-plane-router": SkillTemplate(
        name="mister-smith-control-plane-router",
        description=(
            "Use when any Mister Smith request touches Symphony, Linear, GitHub PR flow, "
            "phase slicing, queue dispatch, runtime reconciliation, workspace hygiene, or "
            "bootstrap readiness and needs the correct control-plane route first."
        ),
        body="""# Mister Smith Control-Plane Router

Use the `smith` MCP tools first for any Mister Smith workflow request.

## Primary route

1. Call `route_workflow_request` with the operator request.
2. Call `get_control_plane_snapshot` when the route needs current repo, PR, Linear, or runtime evidence.
3. Follow the recommended Smith tool chain before reaching for any raw fallback skills.

## Fallback

- Use the repo-local `linear` skill only when the Smith MCP does not expose the needed Linear mutation or query.
- Do not use non-Mister-Smith app workflows when the Smith MCP already covers the operation.
""",
    ),
    "mister-smith-control-plane-bootstrap": SkillTemplate(
        name="mister-smith-control-plane-bootstrap",
        description="Use when the Mister Smith control-plane MCP or repo-local skill shims need to be installed, repaired, re-pointed, or audited in the local Codex environment.",
        body="""# Mister Smith Control-Plane Bootstrap

Use the `smith` MCP tools first when checking bootstrap and readiness.

## Workflow

1. Call `audit_workflow_readiness`.
2. If repo-local canonical skills are missing, run `python3 scripts/bootstrap_control_plane.py` from the Mister Smith repo.
3. Call `get_server_runtime_info` after control-plane source edits to verify the live MCP version.
4. If runtime metadata is stale, call `reload_server`.
5. If readiness still fails, fix the reported checks before continuing.

## Notes

- Require `smith` as the configured MCP server name.
- Treat repo-local canonical skills as the authoritative skill pack for this repository.
""",
    ),
    "symphony-linear-mister-smith": SkillTemplate(
        name="symphony-linear-mister-smith",
        description="Use when a Mister Smith task spans Symphony runtime, Linear control-plane state, GitHub PR flow, local repo truth, queue triage, runtime reconciliation, or workspace hygiene.",
        body="""# Symphony Linear Mister Smith

Use the `smith` MCP tools first for combined Symphony, Linear, GitHub, and repo operations.

## Primary tools

- `get_control_plane_snapshot`
- `get_symphony_checkout_snapshot`
- `plan_workspace_adjustments`
- `sync_linear_with_runtime`
- `refresh_symphony`
- `sync_symphony_main`

## Rules

- Snapshot first, mutate second.
- Prefer Smith workflow tools over raw GraphQL or ad hoc shell commands when the operation is already modeled.
- Only use the repo-local `linear` skill for uncovered Linear gaps.
""",
    ),
    "stage-mister-smith-phase": SkillTemplate(
        name="stage-mister-smith-phase",
        description="Use when a Mister Smith SpecKit phase or tasks pack needs to be turned into deterministic Linear slices, blocker chains, prep-slice opportunities, and runnable watched-queue work.",
        body="""# Stage Mister Smith Phase

Use the `smith` MCP tools first to derive runnable phase slices and stage only honest work.

## Workflow

1. Call `plan_phase_execution`.
2. Review runnable slices, blocked slices, and prep opportunities.
3. Call `apply_phase_execution_plan` to stage only the runnable work.

## Rules

- Do not stage blocked slices just to fill the queue.
- Preserve blocker chains and prep-slice honesty.
""",
    ),
    "symphony-mister-smith-review-dispatch": SkillTemplate(
        name="symphony-mister-smith-review-dispatch",
        description="Use when a Mister Smith Human Review handoff should be reviewed and landed, or when the watched queue has spare capacity and needs a deterministic refill through the control plane.",
        body="""# Symphony Mister Smith Review Dispatch

Use the `smith` MCP tools first for Human Review landing and watched-queue refill.

## Workflow

1. Call `review_merge_dispatch_cycle`.
2. If needed, inspect a specific issue with `get_issue_execution_snapshot`.
3. Only fall back to narrower Smith tools when the dispatch loop identifies a concrete follow-up.

## Rules

- Prefer the deterministic review-dispatch loop over manual PR-by-PR handling.
- Do not bypass Smith queue and runtime reconciliation when refilling capacity.
""",
    ),
    "mister-smith-frontier-mandate": SkillTemplate(
        name="mister-smith-frontier-mandate",
        description="Use when a Mister Smith phase, issue, PR, or design decision needs a frontier-legitimacy judgment about supervised autonomy, scope drift, or security-versus-autonomy balance.",
        body="""# Mister Smith Frontier Mandate

Use the `smith` MCP tools first for frontier-legitimacy judgments.

## Primary tools

- `evaluate_issue_legitimacy`
- `classify_follow_up_work`

## Rules

- Use legitimacy judgments before staging or advancing questionable work.
- Distinguish frontier leverage from drift before proposing queue movement.
""",
    ),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parents[1]),
        help="Repository root where the canonical Smith skill pack should live.",
    )
    parser.add_argument(
        "--config-path",
        default=None,
        help="Codex config to inspect. Defaults to MISTER_SMITH_CODEX_CONFIG_PATH or ~/.codex/config.toml.",
    )
    parser.add_argument(
        "--rewrite",
        action="store_true",
        help="Rewrite existing canonical skill files instead of only creating missing ones.",
    )
    return parser.parse_args()


def resolve_config_path(raw_path: str | None) -> Path:
    if raw_path:
        return Path(raw_path).expanduser()

    env_path = Path(
        str(
            (
                __import__("os").environ.get("MISTER_SMITH_CODEX_CONFIG_PATH")
                or (Path.home() / ".codex" / "config.toml")
            )
        )
    )
    return env_path.expanduser()


def detect_server_name(config_path: Path) -> tuple[bool, str | None]:
    if not config_path.exists():
        return True, None

    raw = config_path.read_text(encoding="utf-8")
    match = re.search(r"\[mcp_servers\.(smith)\]", raw)
    return False, match.group(1) if match else None


def render_skill(template: SkillTemplate) -> str:
    return (
        f"---\n"
        f"name: {template.name}\n"
        f"description: {template.description}\n"
        f"---\n\n"
        f"{template.body}"
    )


def ensure_skill_pack(repo_root: Path, rewrite: bool) -> tuple[list[str], list[str]]:
    created: list[str] = []
    existing: list[str] = []
    skills_root = repo_root / ".codex" / "skills"
    skills_root.mkdir(parents=True, exist_ok=True)

    for skill_name, template in SKILL_TEMPLATES.items():
        skill_path = skills_root / skill_name / "SKILL.md"
        if skill_path.exists() and not rewrite:
            existing.append(skill_name)
            continue

        skill_path.parent.mkdir(parents=True, exist_ok=True)
        skill_path.write_text(render_skill(template), encoding="utf-8")
        if skill_name in existing:
            continue
        created.append(skill_name)

    return created, existing


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).expanduser().resolve()
    config_path = resolve_config_path(args.config_path).resolve()

    created, existing = ensure_skill_pack(repo_root, rewrite=args.rewrite)
    config_missing, server_name = detect_server_name(config_path)

    payload = {
        "repo_root": str(repo_root),
        "skills": {
            "created": created,
            "existing": existing,
            "root": str(repo_root / ".codex" / "skills"),
        },
        "config": {
            "path": str(config_path),
            "missing": config_missing or server_name is None,
            "server_name": server_name,
        },
        "next_action": (
            "Run smith.audit_workflow_readiness, then smith.get_server_runtime_info, to verify bootstrap and live runtime readiness."
        ),
    }
    json.dump(payload, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if server_name is not None else 1


if __name__ == "__main__":
    raise SystemExit(main())
