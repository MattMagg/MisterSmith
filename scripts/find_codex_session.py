#!/usr/bin/env python3
"""Resolve the most appropriate Codex session file for the current repository."""

import argparse
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional


DEFAULT_SESSION_ROOT = Path.home() / ".codex" / "sessions"


@dataclass(frozen=True)
class SessionMeta:
    path: Path
    session_id: str
    cwd: str
    modified_at: float


def load_session_meta(path: Path) -> Optional[SessionMeta]:
    try:
        with path.open(encoding="utf-8") as handle:
            first_line = handle.readline()
    except OSError:
        return None

    if not first_line:
        return None

    try:
        entry = json.loads(first_line)
    except json.JSONDecodeError:
        return None

    if entry.get("type") != "session_meta":
        return None

    payload = entry.get("payload", {})
    session_id = payload.get("id")
    cwd = payload.get("cwd")
    if not isinstance(session_id, str) or not isinstance(cwd, str):
        return None

    return SessionMeta(
        path=path,
        session_id=session_id,
        cwd=cwd,
        modified_at=path.stat().st_mtime,
    )


def discover_sessions(session_root: Path) -> List[SessionMeta]:
    sessions = []
    for path in session_root.rglob("*.jsonl"):
        session = load_session_meta(path)
        if session is not None:
            sessions.append(session)
    return sessions


def newest_session(sessions: Iterable[SessionMeta]) -> Optional[SessionMeta]:
    return max(sessions, key=lambda session: session.modified_at, default=None)


def session_matches_cwd(session_cwd: str, requested_cwd: str) -> bool:
    if session_cwd == requested_cwd:
        return True

    try:
        return Path(session_cwd).resolve() == Path(requested_cwd).resolve()
    except OSError:
        return False


def select_session(
    sessions: Iterable[SessionMeta],
    cwd: str,
    thread_id: Optional[str],
) -> Optional[SessionMeta]:
    session_list = list(sessions)
    cwd_matches = [
        session
        for session in session_list
        if session_matches_cwd(session.cwd, cwd)
    ]

    if thread_id:
        # In app-server sessions CODEX_THREAD_ID can reference a parent thread, so
        # only treat it as authoritative when the recorded cwd also matches.
        exact_matches = [
            session
            for session in cwd_matches
            if session.session_id == thread_id
        ]
        exact_match = newest_session(exact_matches)
        if exact_match is not None:
            return exact_match

    return newest_session(cwd_matches)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Find the current Codex session JSONL file for this repository.",
    )
    parser.add_argument(
        "--session-root",
        default=str(DEFAULT_SESSION_ROOT),
        help="Root directory containing Codex session JSONL files.",
    )
    parser.add_argument(
        "--cwd",
        default=os.getcwd(),
        help="Repository working directory to match against session metadata.",
    )
    parser.add_argument(
        "--thread-id",
        default=os.environ.get("CODEX_THREAD_ID"),
        help="Optional Codex thread ID to prefer when it also matches the repository cwd.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    session_root = Path(args.session_root).expanduser()
    cwd = os.path.abspath(args.cwd)

    if not session_root.exists():
        print(
            f"Codex session root does not exist: {session_root}",
            file=sys.stderr,
        )
        return 1

    session = select_session(
        sessions=discover_sessions(session_root),
        cwd=cwd,
        thread_id=args.thread_id,
    )
    if session is None:
        print(
            f"No Codex session file found for cwd {cwd}",
            file=sys.stderr,
        )
        return 1

    print(session.path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
