#!/usr/bin/env python3
"""mem0 Auto-Capture hook for Claude Code (Stop event).

When the agent considers stopping, this hook reads the session
transcript, extracts the last few user+assistant exchanges, and
sends them to mem0 for automatic extraction and categorization.

Mem0's extraction engine decides what's worth keeping based on
the project's custom instructions and categories.

Always approves the stop — capture is best-effort.
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


def extract_exchanges(transcript_path, max_exchanges=3):
    """Extract the last N user+assistant message pairs from the transcript JSONL.

    Claude Code transcripts are JSONL where each line is a message object.
    We look for objects with 'type'='human'/'assistant' or 'role'='user'/'assistant'.
    """
    if not transcript_path or not os.path.exists(transcript_path):
        return []

    entries = []
    try:
        with open(transcript_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                    entries.append(entry)
                except json.JSONDecodeError:
                    continue
    except (OSError, PermissionError):
        return []

    # Extract user and assistant messages
    messages = []
    for entry in entries:
        role = None
        content = None

        # Handle Claude Code transcript format
        if entry.get("type") == "human":
            role = "user"
            # Content can be a string or list of content blocks
            raw = entry.get("content", "")
            if isinstance(raw, list):
                # Extract text from content blocks
                parts = []
                for block in raw:
                    if isinstance(block, dict) and block.get("type") == "text":
                        parts.append(block.get("text", ""))
                    elif isinstance(block, str):
                        parts.append(block)
                content = "\n".join(parts)
            else:
                content = str(raw)
        elif entry.get("type") == "assistant":
            role = "assistant"
            raw = entry.get("content", "")
            if isinstance(raw, list):
                parts = []
                for block in raw:
                    if isinstance(block, dict) and block.get("type") == "text":
                        parts.append(block.get("text", ""))
                    elif isinstance(block, str):
                        parts.append(block)
                content = "\n".join(parts)
            else:
                content = str(raw)
        elif entry.get("role") in ("user", "assistant"):
            role = entry["role"]
            content = entry.get("content", "")
            if isinstance(content, list):
                parts = []
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "text":
                        parts.append(block.get("text", ""))
                    elif isinstance(block, str):
                        parts.append(block)
                content = "\n".join(parts)

        if role and content and len(content.strip()) > 0:
            messages.append({"role": role, "content": content.strip()})

    if not messages:
        return []

    # Take the last N exchanges (a user+assistant pair = 1 exchange)
    # Walk backwards to find complete pairs
    exchanges = []
    i = len(messages) - 1
    while i >= 0 and len(exchanges) < max_exchanges * 2:
        exchanges.insert(0, messages[i])
        i -= 1

    # Trim to max token budget — truncate individual messages if very long
    trimmed = []
    for msg in exchanges:
        content = msg["content"]
        if len(content) > 2000:
            content = content[:2000] + "\n... [truncated]"
        trimmed.append({"role": msg["role"], "content": content})

    return trimmed


def capture_to_mem0(messages):
    """Send messages to mem0 for extraction. Fire-and-forget (async mode)."""
    api_key = os.environ.get("MEM0_API_KEY")
    if not api_key:
        return

    from mem0 import MemoryClient

    client = MemoryClient(
        api_key=api_key,
        org_id=os.environ.get("MEM0_ORG_ID"),
        project_id=os.environ.get("MEM0_PROJECT_ID"),
    )

    # async_mode=True (default) — returns immediately with event_id
    client.add(
        messages,
        user_id="matthewmaggio",
        agent_id="claude-code",
        app_id="mister-smith",
        metadata={"source": "claude-code", "capture": "auto"},
    )


def main():
    # Read hook input from stdin
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        # Approve stop on any parse failure
        json.dump({"decision": "approve"}, sys.stdout)
        sys.exit(0)

    cwd = hook_input.get("cwd", os.getcwd())
    transcript_path = hook_input.get("transcript_path", "")

    load_env(cwd)

    # Extract and capture
    try:
        messages = extract_exchanges(transcript_path, max_exchanges=3)
        if messages:
            capture_to_mem0(messages)
    except Exception:
        # Never block the stop on capture failure
        pass

    # Always approve the stop
    output = {"decision": "approve"}
    json.dump(output, sys.stdout)


if __name__ == "__main__":
    main()
