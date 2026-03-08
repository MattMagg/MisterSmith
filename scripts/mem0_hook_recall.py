#!/usr/bin/env python3
"""mem0 Auto-Recall hook for Claude Code (UserPromptSubmit + SessionStart).

Reads the user's prompt (or session start event) from stdin,
searches mem0 for relevant memories, and returns them as
additionalContext so Claude sees them before responding.

Hook events handled:
  - UserPromptSubmit: search using the user's prompt text
  - SessionStart: search using a broad project query
"""

import json
import os
import sys
from pathlib import Path


def load_env(cwd):
    """Load .env from the project root."""
    for candidate in [Path(cwd) / ".env", Path(__file__).resolve().parent.parent / ".env"]:
        if candidate.exists():
            for line in candidate.read_text().splitlines():
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, _, value = line.partition("=")
                    os.environ.setdefault(key.strip(), value.strip())
            return


def search_mem0(query, top_k=5):
    """Search mem0 and return formatted memory context string."""
    api_key = os.environ.get("MEM0_API_KEY")
    if not api_key:
        return None

    from mem0 import MemoryClient

    client = MemoryClient(
        api_key=api_key,
        org_id=os.environ.get("MEM0_ORG_ID"),
        project_id=os.environ.get("MEM0_PROJECT_ID"),
    )

    results = client.search(
        query,
        keyword_search=True,
        rerank=True,
        filter_memories=True,
        top_k=top_k,
        filters={"AND": [{"user_id": "matthewmaggio"}, {"app_id": "mister-smith"}]},
    )

    if isinstance(results, dict):
        items = results.get("results", [])
    elif isinstance(results, list):
        items = results
    else:
        items = []

    if not items:
        return None

    lines = ["## Recalled Memories (mem0)"]
    for mem in items:
        memory_text = mem.get("memory", "")
        categories = mem.get("categories", [])
        cat_str = f" [{', '.join(categories)}]" if categories else ""
        lines.append(f"- {memory_text}{cat_str}")

    return "\n".join(lines)


def main():
    # Read hook input from stdin
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    event = hook_input.get("hook_event_name", "")
    cwd = hook_input.get("cwd", os.getcwd())

    load_env(cwd)

    # Determine search query based on event type
    if event == "UserPromptSubmit":
        query = hook_input.get("prompt", "").strip()
        if not query or len(query) < 10:
            # Too short to search meaningfully
            sys.exit(0)
        top_k = 5
    elif event == "SessionStart":
        # Broad project recall on session start and compaction
        query = "Mister Smith architecture implementation status decisions"
        top_k = 10
    else:
        sys.exit(0)

    try:
        context = search_mem0(query, top_k=top_k)
    except Exception:
        # Never block the user on hook failure
        sys.exit(0)

    if not context:
        sys.exit(0)

    # Return context to Claude Code
    output = {
        "hookSpecificOutput": {
            "hookEventName": event,
            "additionalContext": context,
        }
    }
    json.dump(output, sys.stdout)


if __name__ == "__main__":
    main()
