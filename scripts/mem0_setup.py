#!/usr/bin/env python3
"""Mem0 Platform setup and management for Mister Smith project.

Commands:
  configure  - Apply project settings (instructions, categories, graph, retrieval criteria)
  verify     - Verify project configuration matches expected state
  seed       - Seed foundational memories
  stats      - Show memory counts by category and source
  search     - Search memories with advanced retrieval
  graph      - Query entity-relationship graph
  feedback   - Rate memory quality (POSITIVE/NEGATIVE/VERY_NEGATIVE)
  export     - Export all project memories as JSON
  expire     - Set expiration on a specific memory
  webhooks   - Manage webhook notifications
"""

import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path

from mem0_common import (
    APP_ID,
    AGENT_ID_BOOTSTRAP,
    CUSTOM_CATEGORIES,
    CUSTOM_INSTRUCTIONS,
    USER_ID,
    get_client,
    load_env,
)

# Load env at module level for CLI usage
load_env()

API_KEY = os.environ.get("MEM0_API_KEY")
if not API_KEY:
    print("ERROR: MEM0_API_KEY not set in .env or environment")
    sys.exit(1)


def configure():
    """Apply full project settings: instructions, categories, graph, retrieval criteria."""
    client = get_client()

    print("Configuring Mister Smith project settings...")

    # Custom instructions (coding/engineering domain)
    client.project.update(custom_instructions=CUSTOM_INSTRUCTIONS)
    print("  [OK] Custom instructions set")

    # Custom categories (8 domain-specific)
    client.project.update(custom_categories=CUSTOM_CATEGORIES)
    print(f"  [OK] Custom categories set ({len(CUSTOM_CATEGORIES)} categories)")

    # Enable graph memory
    client.project.update(enable_graph=True)
    print("  [OK] Graph memory enabled")

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

    # Check custom instructions content (hash-based)
    instructions = info.get("custom_instructions", "")
    if instructions:
        has_extract = "Extract and retain" in instructions
        has_exclude = "Exclude:" in instructions
        if has_extract and has_exclude:
            print("  [PASS] custom_instructions content: contains expected sections")
        else:
            print("  [WARN] custom_instructions content: may be outdated (missing expected sections)")

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
        "user_id": USER_ID,
        "app_id": APP_ID,
        "agent_id": AGENT_ID_BOOTSTRAP,
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

    filters = {"AND": [{"user_id": USER_ID}, {"app_id": APP_ID}]}
    memories = client.get_all(filters=filters, output_format="v1.1")

    if isinstance(memories, dict):
        items = memories.get("results", memories.get("memories", []))
    elif isinstance(memories, list):
        items = memories
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

    # Count by agent_id
    agent_counts = {}
    for mem in items:
        agent = mem.get("agent_id", "unknown")
        agent_counts[agent] = agent_counts.get(agent, 0) + 1

    if agent_counts:
        print("\nBy agent:")
        for agent, count in sorted(agent_counts.items(), key=lambda x: -x[1]):
            print(f"  {agent}: {count}")


def search(query_text):
    """Search memories with full advanced retrieval."""
    client = get_client()
    results = client.search(
        query_text,
        keyword_search=True,
        rerank=True,
        filter_memories=True,
        top_k=10,
        enable_graph=True,
        filters={"AND": [{"user_id": USER_ID}, {"app_id": APP_ID}]},
    )

    if isinstance(results, dict):
        items = results.get("results", results.get("memories", []))
        relations = results.get("relations", [])
    elif isinstance(results, list):
        items = results
        relations = []
    else:
        items = []
        relations = []

    print(f"=== Search: '{query_text}' ({len(items)} results) ===\n")
    for i, mem in enumerate(items):
        score = mem.get("score", "?")
        memory = mem.get("memory", "?")
        cats = mem.get("categories", [])
        mem_id = mem.get("id", "?")
        print(f"  [{i+1}] (score={score}) {memory}")
        if cats:
            print(f"      categories: {cats}")
        print(f"      id: {mem_id}")
        print()

    if relations:
        print(f"  Graph Relations ({len(relations)}):")
        for rel in relations:
            source = rel.get("source", "?")
            relationship = rel.get("relationship", "?")
            target = rel.get("target", "?")
            score = rel.get("score", "?")
            print(f"    {source} --[{relationship}]--> {target} (score={score})")
        print()


def graph(query_text=None):
    """Query entity-relationship graph for the project."""
    client = get_client()
    query = query_text or "project architecture"

    results = client.search(
        query,
        user_id=USER_ID,
        enable_graph=True,
        top_k=10,
        filters={"AND": [{"user_id": USER_ID}, {"app_id": APP_ID}]},
    )

    if isinstance(results, dict):
        relations = results.get("relations", [])
        items = results.get("results", [])
    else:
        relations = []
        items = []

    print(f"=== Graph Query: '{query}' ===\n")

    if relations:
        print(f"  Relations ({len(relations)}):")
        for rel in relations:
            source = rel.get("source", "?")
            relationship = rel.get("relationship", "?")
            target = rel.get("target", "?")
            score = rel.get("score", "?")
            print(f"    {source} --[{relationship}]--> {target} (score={score})")
    else:
        print("  No graph relations found.")

    if items:
        print(f"\n  Related Memories ({len(items)}):")
        for i, mem in enumerate(items):
            print(f"    [{i+1}] {mem.get('memory', '?')[:100]}")

    print()


def feedback(memory_id, rating, reason=None):
    """Rate a memory as POSITIVE, NEGATIVE, or VERY_NEGATIVE."""
    valid_ratings = {"POSITIVE", "NEGATIVE", "VERY_NEGATIVE"}
    rating = rating.upper()
    if rating not in valid_ratings:
        print(f"ERROR: rating must be one of {valid_ratings}")
        sys.exit(1)

    client = get_client()
    kwargs = {"memory_id": memory_id, "feedback": rating}
    if reason:
        kwargs["feedback_reason"] = reason

    client.feedback(**kwargs)
    print(f"  [OK] Feedback '{rating}' applied to memory {memory_id}")
    if reason:
        print(f"        Reason: {reason}")


def export_memories():
    """Export all project memories as structured JSON."""
    client = get_client()
    filters = {"AND": [{"user_id": USER_ID}, {"app_id": APP_ID}]}
    memories = client.get_all(filters=filters, output_format="v1.1")

    out_path = f"mem0_export_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    with open(out_path, "w") as f:
        json.dump(memories, f, indent=2)

    if isinstance(memories, dict):
        count = len(memories.get("results", []))
    elif isinstance(memories, list):
        count = len(memories)
    else:
        count = 0

    print(f"  [OK] Exported {count} memories to {out_path}")


def expire(memory_id, days=30):
    """Set expiration date on a memory (auto-deletes after)."""
    expiry = (datetime.now() + timedelta(days=int(days))).strftime("%Y-%m-%d")
    client = get_client()
    client.update(memory_id, expiration_date=expiry)
    print(f"  [OK] Memory {memory_id} expires on {expiry} ({days} days)")


def webhooks(action="list", url=None):
    """Manage mem0 webhooks for memory change notifications.

    Note: webhook management uses the REST API directly since
    the SDK may not have webhook methods.
    """
    import urllib.request

    base_url = "https://api.mem0.ai/v1/webhooks/"
    headers = {
        "Authorization": f"Token {API_KEY}",
        "Content-Type": "application/json",
    }

    if action == "list":
        req = urllib.request.Request(base_url, headers=headers, method="GET")
        try:
            with urllib.request.urlopen(req) as resp:
                data = json.loads(resp.read())
            hooks = data if isinstance(data, list) else data.get("results", [])
            print(f"=== Webhooks ({len(hooks)}) ===\n")
            for wh in hooks:
                print(f"  ID: {wh.get('id')}")
                print(f"  URL: {wh.get('url')}")
                print(f"  Events: {wh.get('events', [])}")
                print()
            if not hooks:
                print("  No webhooks configured.")
        except Exception as exc:
            print(f"  ERROR: {exc}")

    elif action == "create":
        if not url:
            print("ERROR: webhook URL required. Usage: webhooks create <url>")
            sys.exit(1)
        payload = json.dumps({
            "url": url,
            "events": ["memory_add", "memory_update", "memory_delete", "memory_categorize"],
        }).encode()
        req = urllib.request.Request(base_url, data=payload, headers=headers, method="POST")
        try:
            with urllib.request.urlopen(req) as resp:
                data = json.loads(resp.read())
            print(f"  [OK] Webhook created: {data.get('id', '?')}")
            print(f"       URL: {url}")
            print(f"       Events: {data.get('events', [])}")
        except Exception as exc:
            print(f"  ERROR: {exc}")

    elif action == "delete":
        if not url:
            print("ERROR: webhook ID required. Usage: webhooks delete <webhook_id>")
            sys.exit(1)
        delete_url = f"{base_url}{url}/"
        req = urllib.request.Request(delete_url, headers=headers, method="DELETE")
        try:
            with urllib.request.urlopen(req) as resp:
                print(f"  [OK] Webhook {url} deleted")
        except Exception as exc:
            print(f"  ERROR: {exc}")

    else:
        print(f"Unknown webhook action: {action}")
        print("Usage: webhooks [list|create <url>|delete <webhook_id>]")


def usage():
    print("Usage: python mem0_setup.py <command> [args]")
    print()
    print("Commands:")
    print("  configure                       - Apply project settings")
    print("  verify                          - Verify project configuration")
    print("  seed                            - Seed foundational memories")
    print("  stats                           - Show memory counts")
    print("  search <query>                  - Search memories")
    print("  graph [query]                   - Query entity-relationship graph")
    print("  feedback <memory_id> <rating> [reason]  - Rate a memory")
    print("  export                          - Export all memories as JSON")
    print("  expire <memory_id> [days]       - Set expiration (default: 30 days)")
    print("  webhooks [list|create <url>|delete <id>] - Manage webhooks")
    sys.exit(1)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        usage()

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
    elif cmd == "graph":
        query = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else None
        graph(query)
    elif cmd == "feedback":
        if len(sys.argv) < 4:
            print("Usage: feedback <memory_id> <POSITIVE|NEGATIVE|VERY_NEGATIVE> [reason]")
            sys.exit(1)
        reason = " ".join(sys.argv[4:]) if len(sys.argv) > 4 else None
        feedback(sys.argv[2], sys.argv[3], reason)
    elif cmd == "export":
        export_memories()
    elif cmd == "expire":
        if len(sys.argv) < 3:
            print("Usage: expire <memory_id> [days]")
            sys.exit(1)
        days = int(sys.argv[3]) if len(sys.argv) > 3 else 30
        expire(sys.argv[2], days)
    elif cmd == "webhooks":
        action = sys.argv[2] if len(sys.argv) > 2 else "list"
        url_or_id = sys.argv[3] if len(sys.argv) > 3 else None
        webhooks(action, url_or_id)
    else:
        print(f"Unknown command: {cmd}")
        usage()
