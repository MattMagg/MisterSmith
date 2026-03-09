#!/usr/bin/env python3
"""Scan NATS authorization configs for unsafe wildcard permissions.

The scan intentionally skips documentation trees and Markdown files so repo
specs and design docs do not trigger false positives on example snippets.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys


CANDIDATE_SUFFIXES = {".conf", ".cfg", ".json", ".yaml", ".yml", ".tmpl"}
EXCLUDED_DIRS = {
    ".git",
    ".venv",
    "__pycache__",
    "archive",
    "docs",
    "node_modules",
    "spec",
    "specs",
    "target",
    "venv",
}
AUTH_MARKERS = (
    "authorization",
    "permissions",
    "accounts",
    "users",
    "publish",
    "subscribe",
    "$JS",
)
GENERIC_WILDCARD_PATTERN = re.compile(r"""(?<![\w$])['"]?>['"]?(?![\w.])""")


def iter_candidates(path: Path) -> list[Path]:
    if path.is_file():
        return [path] if is_candidate_file(path) else []

    candidates: list[Path] = []
    for candidate in path.rglob("*"):
        if candidate.is_dir():
            continue
        if any(part in EXCLUDED_DIRS for part in candidate.parts):
            continue
        if is_candidate_file(candidate):
            candidates.append(candidate)
    return candidates


def is_candidate_file(path: Path) -> bool:
    return path.suffix.lower() in CANDIDATE_SUFFIXES and path.suffix.lower() != ".md"


def should_scan(text: str, path: Path) -> bool:
    lowered_path = str(path).lower()
    if any(marker in lowered_path for marker in ("nats", "auth", "permission")):
        return True
    lowered = text.lower()
    return any(marker.lower() in lowered for marker in AUTH_MARKERS)


def strip_comments(line: str) -> str:
    return line.split("#", 1)[0].strip()


def find_violations(path: Path) -> list[str]:
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        text = path.read_text(encoding="utf-8", errors="ignore")

    if not should_scan(text, path):
        return []

    violations: list[str] = []
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = strip_comments(raw_line)
        if not line:
            continue

        if "$JS.>" in line:
            violations.append(
                f"{path}:{line_number}: forbidden JetStream wildcard permission `$JS.>`"
            )

        if GENERIC_WILDCARD_PATTERN.search(line):
            violations.append(f"{path}:{line_number}: forbidden wildcard permission `>`")

    return violations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        default=["."],
        help="Files or directories to scan (defaults to current directory).",
    )
    args = parser.parse_args()

    candidates: list[Path] = []
    for raw_path in args.paths:
        path = Path(raw_path)
        if not path.exists():
            print(f"missing path: {path}", file=sys.stderr)
            return 2
        candidates.extend(iter_candidates(path))

    violations: list[str] = []
    scanned = 0
    for candidate in sorted(set(candidates)):
        file_violations = find_violations(candidate)
        if file_violations or should_scan(
            candidate.read_text(encoding="utf-8", errors="ignore"), candidate
        ):
            scanned += 1
        violations.extend(file_violations)

    if violations:
        print("NATS permission audit failed:")
        for violation in violations:
            print(violation)
        return 1

    if scanned == 0:
        print("No NATS authorization config files found.")
    else:
        print(f"NATS permission audit passed ({scanned} file(s) scanned).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
