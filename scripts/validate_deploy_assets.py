#!/usr/bin/env python3
"""Validate repo-managed Grafana dashboards and Prometheus alert rules.

JSON dashboards are parsed with Python's stdlib `json` module. YAML alert rules
are parsed with Ruby `Psych`, so this validator requires `ruby` to be available
on `PATH`.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys


JSON_SUFFIXES = {".json"}
YAML_SUFFIXES = {".yml", ".yaml"}
DEFAULT_PATHS = ("deploy/dashboards", "deploy/alerts")


def iter_candidates(path: Path) -> list[Path]:
    if path.is_file():
        return [path] if is_candidate_file(path) else []

    candidates: list[Path] = []
    for candidate in path.rglob("*"):
        if candidate.is_file() and is_candidate_file(candidate):
            candidates.append(candidate)
    return candidates


def is_candidate_file(path: Path) -> bool:
    return path.suffix.lower() in JSON_SUFFIXES | YAML_SUFFIXES


def validate_json(path: Path) -> None:
    with path.open(encoding="utf-8") as handle:
        json.load(handle)


def validate_yaml(path: Path) -> None:
    ruby = shutil.which("ruby")
    if ruby is None:
        raise RuntimeError(
            "ruby is required to validate YAML deploy assets; "
            "install Ruby with Psych support or run the check in CI"
        )

    parser = (
        "require 'psych'; "
        "Psych.safe_load(File.read(ARGV[0]), aliases: true)"
    )
    subprocess.run(
        [ruby, "-e", parser, str(path)],
        check=True,
        capture_output=True,
        text=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        default=list(DEFAULT_PATHS),
        help="Files or directories to validate (defaults to deploy dashboards and alerts).",
    )
    args = parser.parse_args()

    candidates: list[Path] = []
    for raw_path in args.paths:
        path = Path(raw_path)
        if not path.exists():
            print(f"missing path: {path}", file=sys.stderr)
            return 2
        candidates.extend(iter_candidates(path))

    unique_candidates = sorted(set(candidates))
    if not unique_candidates:
        print("No deploy asset files found.", file=sys.stderr)
        return 2

    validated = 0
    for candidate in unique_candidates:
        try:
            if candidate.suffix.lower() in JSON_SUFFIXES:
                validate_json(candidate)
                print(f"json ok: {candidate}")
            else:
                validate_yaml(candidate)
                print(f"yaml ok: {candidate}")
            validated += 1
        except json.JSONDecodeError as exc:
            print(f"invalid json: {candidate}: {exc}", file=sys.stderr)
            return 1
        except subprocess.CalledProcessError as exc:
            message = exc.stderr.strip() if exc.stderr else "ruby Psych parse failed"
            print(f"invalid yaml: {candidate}: {message}", file=sys.stderr)
            return 1
        except RuntimeError as exc:
            print(str(exc), file=sys.stderr)
            return 2

    print(f"deploy asset validation passed ({validated} file(s) checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
