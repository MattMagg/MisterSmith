#!/usr/bin/env python3
"""mem0 Auto-Recall hook for Claude Code (SubagentStart event).

Thin shim that injects relevant memory context into subagents
before they begin work. Uses the subagent's task description
as the search query.
"""

import json
import sys

from mem0_config import CONFIG

# Central library import (path set up by mem0_config)
from mem0_claude.client import load_env
from mem0_claude.recall import recall_subagent_start


def main():
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    load_env(hook_input.get("cwd", ""))

    try:
        context = recall_subagent_start(hook_input, CONFIG)
    except Exception:
        sys.exit(0)

    if not context:
        sys.exit(0)

    output = {
        "hookSpecificOutput": {
            "hookEventName": "SubagentStart",
            "additionalContext": context,
        }
    }
    json.dump(output, sys.stdout)


if __name__ == "__main__":
    main()
