#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (PreCompact event).

Thin shim that delegates to the centralized mem0_claude library.
Captures session context before compaction with 7-day expiry.
"""

import json
import sys

from mem0_config import CONFIG

# Central library import (path set up by mem0_config)
from mem0_claude.capture import handle_pre_compact
from mem0_claude.client import load_env


def main():
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    load_env(hook_input.get("cwd", ""))

    try:
        result = handle_pre_compact(hook_input, CONFIG)
        if result:
            print("mem0 pre-compact capture: ok", file=sys.stderr)
    except Exception as exc:
        print(f"mem0 pre-compact capture error: {exc}", file=sys.stderr)


if __name__ == "__main__":
    main()
