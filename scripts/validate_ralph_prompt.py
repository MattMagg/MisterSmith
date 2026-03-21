#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from datetime import datetime, timezone
from pathlib import Path


METADATA_PREFIX = "<!-- ralph:"
GENERATED_AT_PREFIX = "<!-- ralph:generated-at:"
SOURCE_PREFIX = "<!-- ralph:source:"
HELP_FLAGS = {"-h", "--help"}
VERSION_FLAGS = {"-V", "--version"}
REFRESH_HINT = (
    "Re-run ./scripts/ralph prompt --packet <packet.json> or "
    "./scripts/prepare_ralph_prompt.py before ./scripts/ralph run."
)


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate repo-local Ralph prompt freshness metadata."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser(
        "validate",
        help="Validate one prompt file against its Ralph freshness metadata.",
    )
    add_common_args(validate_parser)

    guard_parser = subparsers.add_parser(
        "guard-run",
        help="Validate only when the Ralph argv targets the repo-default run prompt.",
    )
    add_common_args(guard_parser)
    guard_parser.add_argument(
        "ralph_args",
        nargs=argparse.REMAINDER,
        help="Arguments that will be passed to Ralph after '--'.",
    )

    return parser.parse_args()


def add_common_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--prompt-path",
        default=str(default_repo_root() / "PROMPT.md"),
        help="Prompt file to validate. Defaults to the repo's PROMPT.md.",
    )
    parser.add_argument(
        "--repo-root",
        default=str(default_repo_root()),
        help="Repo root used to resolve repo-relative source paths.",
    )


def normalize_generated_at(value: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ValueError(f"Prompt metadata has invalid generated-at timestamp: {value}") from exc
    return parsed.astimezone(timezone.utc)


def parse_metadata(prompt_text: str) -> tuple[datetime, list[str]]:
    lines = prompt_text.splitlines()
    index = 0
    generated_at: datetime | None = None
    sources: list[str] = []

    while index < len(lines) and lines[index].startswith(METADATA_PREFIX):
        line = lines[index]
        if line.startswith(GENERATED_AT_PREFIX):
            generated_at_value = line.removeprefix(GENERATED_AT_PREFIX).removesuffix("-->").strip()
            generated_at = normalize_generated_at(generated_at_value)
        elif line.startswith(SOURCE_PREFIX):
            source_value = line.removeprefix(SOURCE_PREFIX).removesuffix("-->").strip()
            if not source_value:
                raise ValueError("Prompt metadata contains an empty source entry.")
            sources.append(source_value)
        else:
            raise ValueError(f"Prompt metadata contains an unsupported line: {line}")
        index += 1

    if generated_at is None:
        raise ValueError(
            "Prompt freshness metadata is missing a generated-at line. "
            f"{REFRESH_HINT}"
        )
    if not sources:
        raise ValueError(
            "Prompt freshness metadata is missing source paths. "
            "Re-run ./scripts/ralph prompt --packet <packet.json> or "
            "./scripts/prepare_ralph_prompt.py with one or more --source values."
        )

    return generated_at, sources


def resolve_path(raw_path: str, base_dir: Path) -> Path:
    path = Path(raw_path).expanduser()
    if not path.is_absolute():
        path = base_dir / path
    return path.resolve()


def validate_prompt(prompt_path: Path, repo_root: Path) -> None:
    if not prompt_path.is_file():
        raise ValueError(
            f"Prompt file is missing at {prompt_path}. "
            f"{REFRESH_HINT}"
        )

    prompt_text = prompt_path.read_text(encoding="utf-8")
    generated_at, sources = parse_metadata(prompt_text)

    # generated-at metadata is second-resolution, so compare against whole seconds
    # to avoid false failures when a source and the prompt are written in the same second.
    generated_at_seconds = int(generated_at.timestamp())

    for source in sources:
        source_path = resolve_path(source, repo_root)
        if not source_path.exists():
            raise ValueError(
                f"Prompt freshness metadata references missing source path: {source_path}"
            )
        source_mtime_seconds = int(source_path.stat().st_mtime)
        if source_mtime_seconds > generated_at_seconds:
            raise ValueError(
                f"Prompt is stale: source {source_path} is newer than generated-at "
                f"{generated_at.replace(microsecond=0).isoformat().replace('+00:00', 'Z')}. "
                f"{REFRESH_HINT}"
            )


def parse_prompt_target(ralph_args: list[str], prompt_path: Path) -> Path | None:
    args = list(ralph_args)
    if args and args[0] == "--":
        args = args[1:]

    if not args:
        return prompt_path

    if args[0] == "run":
        args = args[1:]
    elif args[0] in HELP_FLAGS | VERSION_FLAGS:
        return None
    elif not args[0].startswith("-"):
        return None

    prompt_override = False
    prompt_file_override: str | None = None
    index = 0

    while index < len(args):
        arg = args[index]
        if arg == "--":
            break
        if arg in HELP_FLAGS | VERSION_FLAGS:
            return None
        if arg in {"-p", "--prompt"}:
            prompt_override = True
            if index + 1 < len(args):
                index += 2
                continue
            break
        if arg.startswith("--prompt="):
            prompt_override = True
            index += 1
            continue
        if arg.startswith("-p") and len(arg) > 2:
            prompt_override = True
            index += 1
            continue
        if arg in {"-P", "--prompt-file"}:
            if index + 1 >= len(args):
                break
            prompt_file_override = args[index + 1]
            index += 2
            continue
        if arg.startswith("--prompt-file="):
            prompt_file_override = arg.split("=", 1)[1]
            index += 1
            continue
        if arg.startswith("-P") and len(arg) > 2:
            prompt_file_override = arg[2:]
            index += 1
            continue
        index += 1

    if prompt_override:
        return None
    if prompt_file_override is None:
        return prompt_path

    resolved_override = resolve_path(prompt_file_override, Path.cwd())
    if resolved_override == prompt_path.resolve():
        return prompt_path
    return None


def main() -> int:
    args = parse_args()
    prompt_path = Path(args.prompt_path).expanduser().resolve()
    repo_root = Path(args.repo_root).expanduser().resolve()

    try:
        if args.command == "validate":
            validate_prompt(prompt_path, repo_root)
            return 0

        target_prompt = parse_prompt_target(args.ralph_args, prompt_path)
        if target_prompt is None:
            return 0

        validate_prompt(target_prompt, repo_root)
        return 0
    except (OSError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
