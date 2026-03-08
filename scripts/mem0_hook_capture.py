#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (Stop event).

Thin shim that delegates to the centralized mem0_claude library.
Always approves the stop — capture is best-effort.
"""

import json
import sys

from mem0_config import CONFIG

# Central library import (path set up by mem0_config)
from mem0_claude.capture import handle_stop
from mem0_claude.client import load_env


def main():
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        json.dump({"decision": "approve"}, sys.stdout)
        sys.exit(0)

    load_env(hook_input.get("cwd", ""))

    try:
        result = handle_stop(hook_input, CONFIG)
        if result:
            print("mem0 capture: ok", file=sys.stderr)
    except Exception as exc:
        print(f"mem0 capture error: {exc}", file=sys.stderr)

    json.dump({"decision": "approve"}, sys.stdout)


if __name__ == "__main__":
    main()
