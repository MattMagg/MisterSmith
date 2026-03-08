#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (PreCompact event).

Captures session context before Claude Code compacts the context window.
Preserves insights that would otherwise be lost to compression.

Uses `last_assistant_message` from hook input.
Sets 7-day expiration — pre-compact memories are ephemeral session aids.
"""

import json
import os
import sys
from datetime import datetime, timedelta

from mem0_common import (
    AGENT_ID_MAIN,
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
        sys.exit(0)

    cwd = hook_input.get("cwd", "")
    load_env(cwd)

    last_msg = hook_input.get("last_assistant_message", "")
    if not last_msg or len(last_msg.strip()) < 50:
        sys.exit(0)

    # Strip recalled context to prevent feedback loops
    cleaned = strip_recalled_context(last_msg)
    if not cleaned or len(cleaned.strip()) < 50:
        sys.exit(0)

    # Truncate very long messages
    if len(cleaned) > 8000:
        cleaned = cleaned[:8000] + "\n... [truncated]"

    messages = [
        {"role": "user", "content": "[Pre-compaction context for memory extraction]"},
        {"role": "assistant", "content": cleaned},
    ]

    # Derive session ID
    session_id = hook_input.get("session_id")
    if not session_id:
        tp = hook_input.get("transcript_path", "")
        if tp:
            session_id = os.path.basename(os.path.dirname(tp))

    # 7-day expiration for ephemeral pre-compact memories
    expiry = (datetime.now() + timedelta(days=7)).strftime("%Y-%m-%d")

    try:
        client = get_client()
        if client:
            add_kwargs = dict(
                user_id=USER_ID,
                agent_id=AGENT_ID_MAIN,
                app_id=APP_ID,
                enable_graph=True,
                version="v2",
                output_format="v1.1",
                custom_instructions=CUSTOM_INSTRUCTIONS,
                custom_categories=CUSTOM_CATEGORIES,
                expiration_date=expiry,
                metadata={
                    "source": "pre-compact",
                    "capture": "auto",
                },
            )
            if session_id:
                add_kwargs["run_id"] = session_id
                add_kwargs["metadata"]["session_id"] = session_id

            client.add(messages, **add_kwargs)
            print(
                f"mem0 pre-compact capture: sent {len(cleaned)} chars (expires {expiry})",
                file=sys.stderr,
            )
        else:
            print("mem0 pre-compact capture: no API key configured", file=sys.stderr)
    except Exception as exc:
        print(f"mem0 pre-compact capture error: {exc}", file=sys.stderr)

    # PreCompact hooks don't control the compact decision — just exit


if __name__ == "__main__":
    main()
