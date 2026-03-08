#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (SubagentStop event).

Captures deep analysis from Explore, Plan, and general-purpose subagents
that would otherwise be lost when the subagent completes.

Uses `last_assistant_message` from the hook input.
Always approves — capture is best-effort.
"""

import json
import os
import sys

from mem0_common import (
    AGENT_ID_SUBAGENT,
    APP_ID,
    CUSTOM_CATEGORIES,
    CUSTOM_INSTRUCTIONS,
    USER_ID,
    get_client,
    load_env,
    strip_recalled_context,
)


def main():
    # Read hook input from stdin
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        json.dump({"decision": "approve"}, sys.stdout)
        sys.exit(0)

    cwd = hook_input.get("cwd", "")
    load_env(cwd)

    last_msg = hook_input.get("last_assistant_message", "")
    if not last_msg or len(last_msg.strip()) < 100:
        # Too short — trivial subagent result
        json.dump({"decision": "approve"}, sys.stdout)
        sys.exit(0)

    # Strip recalled context to prevent feedback loops
    cleaned = strip_recalled_context(last_msg)
    if not cleaned or len(cleaned.strip()) < 100:
        json.dump({"decision": "approve"}, sys.stdout)
        sys.exit(0)

    # Truncate very long subagent responses
    if len(cleaned) > 8000:
        cleaned = cleaned[:8000] + "\n... [truncated]"

    # Build message pair for extraction
    messages = [
        {"role": "user", "content": "[Subagent analysis for memory extraction]"},
        {"role": "assistant", "content": cleaned},
    ]

    # Derive session ID from transcript path
    session_id = hook_input.get("session_id")
    if not session_id:
        tp = hook_input.get("transcript_path", "")
        if tp:
            session_id = os.path.basename(os.path.dirname(tp))

    try:
        client = get_client()
        if client:
            add_kwargs = dict(
                user_id=USER_ID,
                agent_id=AGENT_ID_SUBAGENT,
                app_id=APP_ID,
                enable_graph=True,
                version="v2",
                output_format="v1.1",
                custom_instructions=CUSTOM_INSTRUCTIONS,
                custom_categories=CUSTOM_CATEGORIES,
                includes="architectural decisions, implementation patterns, research findings, codebase analysis",
                excludes="raw code, API keys, recalled memories, file listings",
                metadata={
                    "source": "claude-code-subagent",
                    "capture": "auto",
                },
            )
            if session_id:
                add_kwargs["run_id"] = session_id
                add_kwargs["metadata"]["session_id"] = session_id

            client.add(messages, **add_kwargs)
            print(f"mem0 subagent capture: sent {len(cleaned)} chars", file=sys.stderr)
        else:
            print("mem0 subagent capture: no API key configured", file=sys.stderr)
    except Exception as exc:
        print(f"mem0 subagent capture error: {exc}", file=sys.stderr)

    # Always approve
    json.dump({"decision": "approve"}, sys.stdout)


if __name__ == "__main__":
    main()
