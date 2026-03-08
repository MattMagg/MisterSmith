#!/usr/bin/env python3
"""Shared constants, client factory, and utilities for mem0 hooks.

All mem0 hooks import from this module to avoid duplicating:
- Environment loading
- Client creation
- Entity constants
- Context stripping (prevents feedback loops)
- Custom instructions and categories
"""

import os
import re
from pathlib import Path

# ── Entity Constants ─────────────────────────────────────────────
USER_ID = "matthewmaggio"
APP_ID = "mister-smith"
AGENT_ID_MAIN = "claude-code"
AGENT_ID_SUBAGENT = "claude-code-subagent"
AGENT_ID_BOOTSTRAP = "bootstrap"

# ── Per-Request Custom Instructions ──────────────────────────────
CUSTOM_INSTRUCTIONS = """
Extract and retain:
- Architectural decisions and their reasoning
- Implementation patterns, conventions, and gotchas
- Bug patterns, root causes, and fixes
- Cross-module/cross-crate dependencies and interactions
- User preferences for tooling, workflow, and communication
- Technology choices, version constraints, and migration notes

Exclude:
- Raw code blocks longer than 10 lines
- Temporary debugging output
- API keys, secrets, credentials
- File contents that are just being read/displayed
- Recalled memories being re-injected (prevent feedback loops)
"""

# ── Per-Request Custom Categories (8 domain-specific) ────────────
CUSTOM_CATEGORIES = [
    {"architecture": "System design decisions, component relationships, data flow"},
    {"implementation": "Code patterns, conventions, build configuration, testing strategies"},
    {"llm_integration": "LLM provider patterns, model routing, streaming, tool use"},
    {"security_ops": "Authentication, authorization, TLS, audit logging patterns"},
    {"agent_system": "Agent orchestration, supervision, lifecycle, team coordination"},
    {"transport_persistence": "Messaging, storage, NATS, PostgreSQL, state management"},
    {"debugging": "Bug patterns, root causes, fixes, failure modes, diagnostics"},
    {"workflow_preferences": "User preferences, tooling choices, communication style, process"},
]

# ── Regex for stripping recalled memory blocks ───────────────────
_RECALLED_RE = re.compile(
    r"<recalled-memories>.*?</recalled-memories>", re.DOTALL
)


def load_env(cwd=None):
    """Load .env from the project root (idempotent)."""
    candidates = [Path(__file__).resolve().parent.parent / ".env"]
    if cwd:
        candidates.insert(0, Path(cwd) / ".env")

    for candidate in candidates:
        if candidate.exists():
            for line in candidate.read_text().splitlines():
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, _, value = line.partition("=")
                    os.environ.setdefault(key.strip(), value.strip())
            return


def get_client():
    """Create and return a MemoryClient using env vars."""
    api_key = os.environ.get("MEM0_API_KEY")
    if not api_key:
        return None

    from mem0 import MemoryClient

    return MemoryClient(
        api_key=api_key,
        org_id=os.environ.get("MEM0_ORG_ID"),
        project_id=os.environ.get("MEM0_PROJECT_ID"),
    )


def strip_recalled_context(text):
    """Remove <recalled-memories>...</recalled-memories> blocks.

    Prevents mem0 from re-ingesting its own output (feedback loop).
    """
    if not text:
        return text
    return _RECALLED_RE.sub("", text).strip()
