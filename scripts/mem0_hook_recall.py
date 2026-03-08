#!/usr/bin/env python3
"""mem0 Auto-Recall hook for Claude Code (UserPromptSubmit + SessionStart).

Performs dual-scope search (long-term + session), includes graph relations,
and returns tagged context using <recalled-memories> for easy stripping
in the capture hook.

Hook events handled:
  - UserPromptSubmit: search using the user's prompt text
  - SessionStart: broad project recall (differentiated by source)
"""

import json
import os
import sys

from mem0_common import (
    APP_ID,
    USER_ID,
    get_client,
    load_env,
)

# Single-word commands that don't benefit from memory recall
_SKIP_PROMPTS = {"yes", "no", "y", "n", "continue", "ok", "done", "stop", "exit", "quit"}


def search_dual_scope(client, query, session_id=None, top_k=5):
    """Search both long-term (user-scoped) and session-scoped memories.

    Returns deduplicated results and graph relations.
    """
    # Long-term memories (user + app scoped)
    long_term = client.search(
        query,
        keyword_search=True,
        rerank=True,
        filter_memories=True,
        top_k=top_k,
        enable_graph=True,
        filters={"AND": [{"user_id": USER_ID}, {"app_id": APP_ID}]},
    )

    # Normalize response
    if isinstance(long_term, dict):
        lt_items = long_term.get("results", [])
        relations = long_term.get("relations", [])
    elif isinstance(long_term, list):
        lt_items = long_term
        relations = []
    else:
        lt_items = []
        relations = []

    # Session memories (run_id scoped) — only if session_id available
    session_items = []
    if session_id:
        try:
            session_results = client.search(
                query,
                keyword_search=True,
                top_k=3,
                filters={"AND": [{"user_id": USER_ID}, {"run_id": session_id}]},
            )
            if isinstance(session_results, dict):
                session_items = session_results.get("results", [])
            elif isinstance(session_results, list):
                session_items = session_results
        except Exception:
            pass  # Session search is best-effort

    # Deduplicate by memory ID
    seen_ids = set()
    deduped_lt = []
    for mem in lt_items:
        mid = mem.get("id")
        if mid and mid not in seen_ids:
            seen_ids.add(mid)
            deduped_lt.append(mem)
        elif not mid:
            deduped_lt.append(mem)

    deduped_session = []
    for mem in session_items:
        mid = mem.get("id")
        if mid and mid not in seen_ids:
            seen_ids.add(mid)
            deduped_session.append(mem)
        elif not mid:
            deduped_session.append(mem)

    return deduped_lt, deduped_session, relations


def format_context(lt_items, session_items, relations):
    """Format memories into tagged context for easy stripping."""
    lines = ["<recalled-memories>"]

    if lt_items:
        lines.append("Long-term:")
        for mem in lt_items:
            memory_text = mem.get("memory", "")
            categories = mem.get("categories", [])
            cat_str = f" [{', '.join(categories)}]" if categories else ""
            lines.append(f"- {memory_text}{cat_str}")

    if session_items:
        lines.append("")
        lines.append("Session:")
        for mem in session_items:
            memory_text = mem.get("memory", "")
            lines.append(f"- {memory_text}")

    if relations:
        lines.append("")
        lines.append("Relations:")
        for rel in relations:
            source = rel.get("source", "?")
            relationship = rel.get("relationship", "?")
            target = rel.get("target", "?")
            lines.append(f"- {source} -> {relationship} -> {target}")

    lines.append("</recalled-memories>")
    return "\n".join(lines)


def get_session_id(hook_input):
    """Extract session ID from hook input or transcript path."""
    sid = hook_input.get("session_id")
    if sid:
        return sid
    tp = hook_input.get("transcript_path", "")
    if tp:
        return os.path.basename(os.path.dirname(tp))
    return None


def main():
    # Read hook input from stdin
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    event = hook_input.get("hook_event_name", "")
    cwd = hook_input.get("cwd", "")

    load_env(cwd)

    # Determine search query and strategy based on event type
    if event == "UserPromptSubmit":
        query = hook_input.get("prompt", "").strip()
        if not query or len(query) < 10:
            sys.exit(0)
        # Skip slash commands
        if query.startswith("/"):
            sys.exit(0)
        # Skip single-word commands
        if query.lower() in _SKIP_PROMPTS:
            sys.exit(0)
        top_k = 5
    elif event == "SessionStart":
        # Differentiate by session source
        source = hook_input.get("session_source", "startup")
        if source == "resume":
            query = "recent session context architecture decisions implementation"
            top_k = 8
        elif source == "compact":
            query = "key architectural facts implementation patterns conventions"
            top_k = 10
        else:  # startup
            query = "Mister Smith architecture implementation status decisions patterns"
            top_k = 10
    else:
        sys.exit(0)

    session_id = get_session_id(hook_input)

    try:
        client = get_client()
        if not client:
            sys.exit(0)

        lt_items, session_items, relations = search_dual_scope(
            client, query, session_id=session_id, top_k=top_k
        )
    except Exception:
        # Never block the user on hook failure
        sys.exit(0)

    if not lt_items and not session_items:
        sys.exit(0)

    context = format_context(lt_items, session_items, relations)

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
