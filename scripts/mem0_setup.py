#!/usr/bin/env python3
"""Mem0 Platform setup and management for Mister Smith project."""

import json
import os
import sys
from pathlib import Path

# Load .env from project root
env_path = Path(__file__).resolve().parent.parent / ".env"
if env_path.exists():
    for line in env_path.read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            key, _, value = line.partition("=")
            os.environ.setdefault(key.strip(), value.strip())

API_KEY = os.environ.get("MEM0_API_KEY")
ORG_ID = os.environ.get("MEM0_ORG_ID")
PROJECT_ID = os.environ.get("MEM0_PROJECT_ID")

if not API_KEY:
    print("ERROR: MEM0_API_KEY not set in .env or environment")
    sys.exit(1)

from mem0 import MemoryClient


def get_client():
    return MemoryClient(api_key=API_KEY, org_id=ORG_ID, project_id=PROJECT_ID)


def configure():
    """Apply remaining project settings not available via Rube MCP."""
    client = get_client()

    print("Configuring Mister Smith project settings...")

    # Retrieval criteria — weighted scoring for domain-relevant retrieval
    retrieval_criteria = [
        {
            "name": "architectural_impact",
            "description": "How broadly does this affect system design across crates",
            "weight": 3,
        },
        {
            "name": "implementation_confidence",
            "description": "How well-verified is this information (tested, reviewed, confirmed vs speculative)",
            "weight": 3,
        },
        {
            "name": "recency_relevance",
            "description": "How relevant to the current active phase of development",
            "weight": 2,
        },
        {
            "name": "cross_crate_scope",
            "description": "Does this span multiple crates or affect the workspace-wide contract",
            "weight": 2,
        },
    ]

    client.project.update(retrieval_criteria=retrieval_criteria)
    print("  [OK] Retrieval criteria set (4 weighted dimensions)")

    print("\nDone. Use 'verify' to confirm all settings.")


def verify():
    """Verify project configuration matches expected state."""
    client = get_client()
    info = client.project.get()
    print("=== Mister Smith Project Configuration ===\n")

    checks = {
        "enable_graph": (info.get("enable_graph"), True),
        "custom_instructions": (bool(info.get("custom_instructions")), True),
        "custom_categories": (
            len(info.get("custom_categories") or []),
            8,
        ),
    }

    all_pass = True
    for key, (actual, expected) in checks.items():
        status = "PASS" if actual == expected else "FAIL"
        if status == "FAIL":
            all_pass = False
        print(f"  [{status}] {key}: {actual} (expected: {expected})")

    # Retrieval criteria: platform GET may return null even when set — treat as WARN
    criteria_count = len(info.get("retrieval_criterias") or [])
    if criteria_count == 4:
        print(f"  [PASS] retrieval_criterias: {criteria_count} (expected: 4)")
    elif criteria_count == 0:
        print(f"  [WARN] retrieval_criterias: {criteria_count} (expected: 4 — platform GET may not expose this field)")
    else:
        print(f"  [FAIL] retrieval_criterias: {criteria_count} (expected: 4)")
        all_pass = False

    # Print categories
    cats = info.get("custom_categories") or []
    if cats:
        print(f"\n  Categories ({len(cats)}):")
        for cat in cats:
            for k, v in cat.items():
                print(f"    - {k}: {v}")

    # Print retrieval criteria
    criteria = info.get("retrieval_criterias") or []
    if criteria:
        print(f"\n  Retrieval Criteria ({len(criteria)}):")
        for c in criteria:
            print(f"    - {c.get('name')} (weight={c.get('weight')}): {c.get('description')}")

    # Print instructions (truncated)
    instructions = info.get("custom_instructions", "")
    if instructions:
        lines = instructions.strip().split("\n")
        print(f"\n  Custom Instructions ({len(lines)} lines): first 3 lines...")
        for line in lines[:3]:
            print(f"    {line}")

    print(f"\n{'ALL CHECKS PASSED' if all_pass else 'SOME CHECKS FAILED'}")
    return all_pass


def seed():
    """Seed foundational memories for the project (light seed)."""
    client = get_client()

    common = {
        "user_id": "matthewmaggio",
        "app_id": "mister-smith",
        "agent_id": "bootstrap",
    }

    seeds = [
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

    print(f"Seeding {len(seeds)} foundational memories...\n")

    for i, seed_data in enumerate(seeds):
        messages = seed_data.pop("messages")
        params = {**common, **seed_data}
        # Use sync mode so we get confirmation
        params["async_mode"] = False
        result = client.add(messages, **params)

        # Extract memory text for display
        mem_text = messages[0]["content"][:80]
        event_count = len(result.get("results", [])) if isinstance(result, dict) else 0
        print(f"  [{i+1}/{len(seeds)}] {mem_text}...")
        if isinstance(result, dict) and "results" in result:
            for r in result["results"]:
                print(f"         -> {r.get('event', '?')}: {r.get('memory', '')[:60]}")
        else:
            print(f"         -> {result}")

    print(f"\nSeeded {len(seeds)} memories. Use 'verify' and 'stats' to confirm.")


def stats():
    """Show memory counts for the project."""
    client = get_client()

    filters = {"AND": [{"user_id": "matthewmaggio"}, {"app_id": "mister-smith"}]}
    memories = client.get_all(filters=filters)

    if isinstance(memories, list):
        items = memories
    elif isinstance(memories, dict):
        items = memories.get("results", memories.get("memories", []))
    else:
        items = []

    print(f"=== Mister Smith Memory Stats ===\n")
    print(f"Total memories: {len(items)}")

    # Count by category
    cat_counts = {}
    for mem in items:
        cats = mem.get("categories", [])
        if isinstance(cats, list):
            for c in cats:
                cat_counts[c] = cat_counts.get(c, 0) + 1
        elif cats:
            cat_counts[str(cats)] = cat_counts.get(str(cats), 0) + 1

    if cat_counts:
        print("\nBy category:")
        for cat, count in sorted(cat_counts.items(), key=lambda x: -x[1]):
            print(f"  {cat}: {count}")

    # Count by metadata.source
    source_counts = {}
    for mem in items:
        meta = mem.get("metadata") or {}
        source = meta.get("source", "unknown")
        source_counts[source] = source_counts.get(source, 0) + 1

    if source_counts:
        print("\nBy source:")
        for src, count in sorted(source_counts.items(), key=lambda x: -x[1]):
            print(f"  {src}: {count}")


def search(query_text):
    """Search memories with full advanced retrieval."""
    client = get_client()
    results = client.search(
        query_text,
        keyword_search=True,
        rerank=True,
        filter_memories=True,
        top_k=10,
        filters={"AND": [{"user_id": "matthewmaggio"}, {"app_id": "mister-smith"}]},
    )

    if isinstance(results, dict):
        items = results.get("results", results.get("memories", []))
    elif isinstance(results, list):
        items = results
    else:
        items = []

    print(f"=== Search: '{query_text}' ({len(items)} results) ===\n")
    for i, mem in enumerate(items):
        score = mem.get("score", "?")
        memory = mem.get("memory", "?")
        cats = mem.get("categories", [])
        print(f"  [{i+1}] (score={score}) {memory}")
        if cats:
            print(f"      categories: {cats}")
        print()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python mem0_setup.py <command> [args]")
        print("Commands: configure, verify, seed, stats, search <query>")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "configure":
        configure()
    elif cmd == "verify":
        verify()
    elif cmd == "seed":
        seed()
    elif cmd == "stats":
        stats()
    elif cmd == "search":
        query = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else "architecture"
        search(query)
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
