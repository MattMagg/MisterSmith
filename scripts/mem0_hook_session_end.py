#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (SessionEnd event).

Thin shim that captures final session context on Ctrl+C or
session close. Uses the centralized mem0_claude library.

Note: SessionEnd hooks cannot return context to the user
(the session is ending). This is purely for capture.
"""

import json
import sys

from mem0_config import CONFIG

# Central library import (path set up by mem0_config)
from mem0_claude.capture import handle_session_end
from mem0_claude.client import load_env


def main():
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    load_env(hook_input.get("cwd", ""))

    try:
        result = handle_session_end(hook_input, CONFIG)
        if result:
            print("mem0 session-end capture: ok", file=sys.stderr)
    except Exception as exc:
        print(f"mem0 session-end capture error: {exc}", file=sys.stderr)


if __name__ == "__main__":
    main()
