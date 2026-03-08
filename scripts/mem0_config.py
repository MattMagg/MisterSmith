"""Mister Smith project configuration for mem0 Claude Code hooks.

This module defines the ProjectConfig with domain-specific custom
instructions, categories, and entity scoping for Mister Smith.

Imported by thin hook shims and the centralized CLI.
"""

import sys
from pathlib import Path

# Add the central library to the path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent / "Repos" / "mem0" / "claude-code"))

from mem0_claude.types import ProjectConfig

CONFIG = ProjectConfig(
    user_id="matthewmaggio",
    app_id="mister-smith",
    agent_id_main="claude-code",
    agent_id_subagent="claude-code-subagent",
    agent_id_bootstrap="bootstrap",
    custom_instructions="""
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
""",
    custom_categories=[
        {"architecture": "System design decisions, component relationships, data flow"},
        {"implementation": "Code patterns, conventions, build configuration, testing strategies"},
        {"llm_integration": "LLM provider patterns, model routing, streaming, tool use"},
        {"security_ops": "Authentication, authorization, TLS, audit logging patterns"},
        {"agent_system": "Agent orchestration, supervision, lifecycle, team coordination"},
        {"transport_persistence": "Messaging, storage, NATS, PostgreSQL, state management"},
        {"debugging": "Bug patterns, root causes, fixes, failure modes, diagnostics"},
        {"workflow_preferences": "User preferences, tooling choices, communication style, process"},
    ],
    includes="architectural decisions, implementation patterns, bug fixes, user preferences",
    excludes="raw code, API keys, recalled memories",
    capture_min_chars=50,
    subagent_min_chars=100,
    max_capture_chars=8000,
    recall_top_k=5,
    session_recall_top_k=3,
    startup_top_k=10,
    auto_capture_expiry_days=30,
    compact_expiry_days=7,
)

# Seeds for the seed command (used by: cli.py --config mem0_config.py seed)
SEEDS = [
    {
        "messages": [
            {
                "role": "user",
                "content": "Mister Smith is a model-agnostic multi-agent orchestration framework built in Rust with NATS messaging. It is NOT Claude-specific — it works with any LLM. Claude-specific files were archived to archive/claude-cli-research/.",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "architecture"},
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "The Mister Smith workspace has 19 crates across 9 phases: core, config, runtime, monitoring, events, async, resources, actor, supervision, transport, nats, http, grpc, mcp, security, persistence, llm, agents, app. Plus 1 integration-test crate. MSRV is 1.88.0, driven by async-nats 0.46.0.",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "architecture"},
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "spec/ contains canonical architecture specifications (the system contract: types, patterns, interfaces, 62 files). specs/ contains SpecKit-generated per-phase implementation artifacts (build instructions, 113 files). ROADMAP.md bridges them. These are different directories with different purposes.",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "architecture"},
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "Error types in Mister Smith follow a pattern: defined in mister-smith-core, re-exported from the domain crate (SecurityError, PersistenceError, etc). Feature flags gate selective compilation (e.g., jwt, rbac, tls, audit in security; sqlx, security in persistence; llm in agents).",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "implementation"},
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "As of March 2026, all 9 phases of Mister Smith are implementation-complete with 1100+ tests passing. Phase 9 LLM Provider Integration added mister-smith-llm crate with ModelProvider trait, MockProvider, OpenAI/Anthropic/Claude providers, ModelRouter with cascade routing, circuit breaker, budget enforcement, and dual-stream processing.",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "implementation", "phase": "9"},
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "Active code review triage identified critical findings in Phase 9: budget leak where failed completions don't release budget reservations (router.rs), tool ID mismatch in Anthropic streaming (anthropic.rs), tool name loss in DualStreamActor on completion (dual_stream.rs), and missing routing_hint not implemented end-to-end.",
            }
        ],
        "immutable": False,
        "metadata": {
            "confidence": "confirmed",
            "source": "code-review",
            "phase": "9",
            "crate": "mister-smith-llm",
        },
    },
    {
        "messages": [
            {
                "role": "user",
                "content": "The Mister Smith mem0 project uses these entity scoping conventions: user_id is always matthewmaggio (the developer — preferences, decisions, requests follow the user everywhere). agent_id identifies the AI tool that produced the memory (claude-code, opencode, codex, bootstrap, manual). app_id is always mister-smith. run_id is optional — only for session-scoped data like phase9-llm or review-2026-03-08. Metadata includes source, phase, crate, confidence fields.",
            }
        ],
        "immutable": True,
        "infer": False,
        "metadata": {"confidence": "confirmed", "source": "architecture"},
    },
]
