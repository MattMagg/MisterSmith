#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path


METADATA_PREFIX = "<!-- ralph:"
GENERATED_AT_PREFIX = "<!-- ralph:generated-at:"
SOURCE_PREFIX = "<!-- ralph:source:"


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Render PROMPT.md from a prepared prompt input and prepend Ralph freshness metadata."
        )
    )
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        "--input",
        help="Path to the prepared prompt input file, or '-' to read the prompt body from stdin.",
    )
    input_group.add_argument(
        "--packet",
        help=(
            "Path to a Ralph packet JSON file. Supports either the raw packet object or a Smith "
            "tool response envelope with the packet under top-level 'data'."
        ),
    )
    parser.add_argument(
        "--source",
        action="append",
        default=[],
        help=(
            "Source-of-record path to record in metadata. Repeat for multiple paths. "
            "Defaults to the --input file when --input is a file and no --source is provided."
        ),
    )
    parser.add_argument(
        "--output",
        default=str(repo_root() / "PROMPT.md"),
        help="Output prompt path. Defaults to the repo's PROMPT.md.",
    )
    parser.add_argument(
        "--generated-at",
        help="Optional UTC ISO 8601 timestamp override. Defaults to the current time.",
    )
    return parser.parse_args()


def normalize_generated_at(value: str | None) -> str:
    if value is None:
        return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")

    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:  # pragma: no cover - exercised through CLI exit path
        raise ValueError(f"Invalid --generated-at value: {value}") from exc

    return parsed.astimezone(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def strip_managed_metadata(prompt_text: str) -> str:
    lines = prompt_text.splitlines(keepends=True)
    index = 0

    while index < len(lines) and lines[index].startswith(METADATA_PREFIX):
        index += 1

    if index > 0 and index < len(lines) and lines[index].strip() == "":
        index += 1

    return "".join(lines[index:])


def read_prompt_body(input_value: str) -> tuple[str, Path | None]:
    if input_value == "-":
        prompt_text = sys.stdin.read()
        input_path = None
    else:
        input_path = Path(input_value).expanduser().resolve(strict=True)
        prompt_text = input_path.read_text(encoding="utf-8")

    prompt_body = strip_managed_metadata(prompt_text)
    if not prompt_body.strip():
        raise ValueError("Prompt input is empty after removing helper-managed metadata.")

    return prompt_body, input_path


def read_packet_prompt(packet_value: str) -> tuple[str, Path, list[str]]:
    packet_path = Path(packet_value).expanduser().resolve(strict=True)

    try:
        packet = json.loads(packet_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"Packet JSON is invalid: {packet_path}") from exc

    if not isinstance(packet, dict):
        raise ValueError("Packet JSON must be an object.")

    packet_data = packet.get("data", packet)
    if not isinstance(packet_data, dict):
        raise ValueError("Packet JSON must contain an object packet under 'data' or at the top level.")

    rendered_prompt = packet_data.get("rendered_prompt")
    if not isinstance(rendered_prompt, str) or not rendered_prompt.strip():
        raise ValueError("Packet JSON is missing a non-empty 'rendered_prompt' field.")

    source_docs = packet_data.get("source_docs", [])
    if not isinstance(source_docs, list) or not all(isinstance(item, str) for item in source_docs):
        raise ValueError("Packet JSON 'source_docs' must be a list of strings when present.")

    prompt_body = strip_managed_metadata(rendered_prompt)
    if not prompt_body.strip():
        raise ValueError("Packet rendered_prompt is empty after removing helper-managed metadata.")

    packet_sources = [str(packet_path), *source_docs]
    return prompt_body, packet_path, packet_sources


def format_source_path(source_path: Path, resolved_repo_root: Path) -> str:
    try:
        return source_path.relative_to(resolved_repo_root).as_posix()
    except ValueError:
        return str(source_path)


def collect_sources(raw_sources: list[str], input_path: Path | None) -> list[str]:
    resolved_repo_root = repo_root().resolve()
    source_paths = list(raw_sources)
    if not source_paths and input_path is not None:
        source_paths = [str(input_path)]

    if not source_paths:
        raise ValueError("At least one --source is required when --input is '-'.")

    seen: set[str] = set()
    formatted_sources: list[str] = []

    for source in source_paths:
        resolved_source = Path(source).expanduser().resolve(strict=True)
        formatted_source = format_source_path(resolved_source, resolved_repo_root)
        if formatted_source in seen:
            continue
        seen.add(formatted_source)
        formatted_sources.append(formatted_source)

    return formatted_sources


def render_prompt(prompt_body: str, generated_at: str, sources: list[str]) -> str:
    metadata_lines = [f"{GENERATED_AT_PREFIX} {generated_at} -->"]
    metadata_lines.extend(f"{SOURCE_PREFIX} {source} -->" for source in sources)
    return "\n".join(metadata_lines) + "\n\n" + prompt_body.rstrip("\n") + "\n"


def main() -> int:
    args = parse_args()

    try:
        if args.packet is not None:
            prompt_body, packet_path, packet_sources = read_packet_prompt(args.packet)
            source_values = args.source or packet_sources
            sources = collect_sources(source_values, packet_path)
        else:
            prompt_body, input_path = read_prompt_body(args.input)
            sources = collect_sources(args.source, input_path)
        generated_at = normalize_generated_at(args.generated_at)
        rendered_prompt = render_prompt(prompt_body, generated_at, sources)
    except (OSError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    output_path = Path(args.output).expanduser()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered_prompt, encoding="utf-8")
    print(output_path.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
