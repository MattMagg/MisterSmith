# Mem0 Platform — Complete Feature Checklist (for IDE Agent Prompt)

> **Context**: This checklist covers every configurable Mem0 Platform feature
> your coding agent should incorporate. Each item includes a brief description,
> the relevant SDK parameter or method, and a documentation link.
> Use the Python SDK (`mem0` package) or the JS SDK (`mem0ai` package).
> Initialize with: `MemoryClient(api_key="...", org_id="...", project_id="...")`

> **Status**: ✅ = implemented, ⬜ = not yet used

---

## 1 · Memory Customization & Extraction

### 1.1 Custom Instructions ✅
Control what Mem0 extracts/excludes using natural-language rules set at the project level.
```python
client.project.update(custom_instructions="Extract only health info. Exclude financial data.")
```
**Status**: Set via `configure` command + passed per-request in all hooks via `custom_instructions` param.
Docs: https://docs.mem0.ai/platform/features/custom-instructions

### 1.2 Custom Categories ✅
Replace the 15 default category labels (travel, sports, etc.) with your own domain-specific tags.
```python
client.project.update(custom_categories=[
    {"onboarding_status": "Tracks user onboarding progress"},
    {"feature_requests": "Captures product feature requests"}
])
```
**Status**: 8 domain-specific categories configured via `configure` + passed per-request in hooks.
Docs: https://docs.mem0.ai/platform/features/custom-categories

### 1.3 Direct Import (`infer=False`) ✅
Bypass the inference/deduction pipeline and store raw user messages verbatim. Skips dedup.
```python
client.add(messages, user_id="alice", infer=False)
```
**Status**: Used in `seed` command for foundational memories.
Docs: https://docs.mem0.ai/platform/features/direct-import

### 1.4 Contextual Memory Creation ✅
Mem0 automatically tracks prior context across multiple `add()` calls for the same user — no need to resend conversation history.
```python
client.add(messages1, user_id="sarah")
# Later — Mem0 already knows Sarah's prior context:
client.add(messages2, user_id="sarah")
```
**Status**: Active — all hooks use consistent `user_id`/`app_id` scoping.
Docs: https://docs.mem0.ai/platform/features/contextual-add

### 1.5 Per-Request Includes / Excludes ✅
Fine-tune extraction on a single `add()` call without changing project-level instructions.
```python
client.add(messages, user_id="u1", includes="dietary preferences", excludes="financial info")
```
**Status**: Used in capture hook (`includes`/`excludes` for domain-specific extraction).
Docs: https://docs.mem0.ai/api-reference/memory/add-memories (see `includes` / `excludes` params)

### 1.6 Immutable Memories (`immutable=True`) ✅
Lock a memory so it can never be updated or overwritten after creation.
```python
client.add(messages, user_id="u1", immutable=True)
```
**Status**: Used in `seed` command for foundational architecture memories.
Docs: https://docs.mem0.ai/api-reference/memory/add-memories (see `immutable` param)

---

## 2 · Entity Scoping & Multi-Tenancy

### 2.1 Entity-Scoped Memory ✅
Scope every memory by `user_id`, `agent_id`, `app_id`, and/or `run_id` to isolate data per user, agent, app, or session.
```python
client.add(messages, user_id="customer_6412", agent_id="travel_planner", app_id="portal", run_id="session-42")
```
**Status**: All hooks use `user_id`, `agent_id`, `app_id`. Capture/subagent hooks use `run_id` for session scoping. Recall does dual-scope search (long-term + session).
Docs: https://docs.mem0.ai/platform/features/entity-scoped-memory

### 2.2 Organizations & Projects ✅
Multi-tenant isolation. Create orgs, projects, and manage members with role-based access.
```python
client = MemoryClient(api_key="...", org_id="org_abc", project_id="proj_xyz")
```
**Status**: All hooks use `org_id` + `project_id` from env vars.
Docs: https://docs.mem0.ai/api-reference/organizations-projects

### 2.3 Group Chat ⬜
Multi-participant conversations. Add a `name` field to messages — Mem0 auto-attributes memories to each speaker.
```python
messages = [
    {"role": "user", "name": "Alice", "content": "I prefer React"},
    {"role": "user", "name": "Bob", "content": "I prefer Vue"},
]
client.add(messages, run_id="group_chat_1")
```
Docs: https://docs.mem0.ai/platform/features/group-chat

---

## 3 · Advanced Retrieval

### 3.1 Keyword Search (`keyword_search=True`) ✅
Expands results beyond semantic similarity to include term/entity matches. +~10ms latency, significantly increased recall.
```python
client.search("What foods should I avoid?", keyword_search=True, user_id="u1")
```
**Status**: Used in all recall searches.
Docs: https://docs.mem0.ai/platform/features/advanced-retrieval

### 3.2 Reranking (`rerank=True`) ✅
Deep semantic reranking of results for top-N precision. Use when result ordering is critical.
```python
client.search("upcoming travel plans", rerank=True, user_id="u1")
```
**Status**: Used in long-term recall searches.
Docs: https://docs.mem0.ai/platform/features/advanced-retrieval

### 3.3 Filter Memories (`filter_memories=True`) ✅
Strict relevance filtering to remove noise from results.
```python
client.search("travel plans", filter_memories=True, user_id="u1")
```
**Status**: Used in long-term recall searches.
Docs: https://docs.mem0.ai/platform/features/advanced-retrieval

### 3.4 Memory Filters v2 (AND/OR/NOT) ✅
Powerful filter logic across entity fields, time fields, categories, metadata, and keywords.
```python
filters = {
    "AND": [
        {"user_id": "alice"},
        {"created_at": {"gte": "2025-01-01"}},
        {"categories": {"in": ["onboarding_status"]}}
    ]
}
client.get_all(filters=filters)
```
**Status**: Used in all searches and stats commands.
Docs: https://docs.mem0.ai/platform/features/v2-memory-filters

### 3.5 Criteria Retrieval ✅
Define weighted custom criteria (e.g., "joy", "urgency", "confidence") to re-rank results by intent, not just content.
```python
client.project.update(retrieval_criteria=[
    {"name": "joy", "description": "Intensity of positive emotions", "weight": 3},
    {"name": "urgency", "description": "Time-sensitive signals", "weight": 2},
])
# Then search as normal — criteria apply project-wide
client.search("How is the user feeling?", user_id="u1")
```
**Status**: 4 weighted dimensions configured via `configure` command.
Docs: https://docs.mem0.ai/platform/features/criteria-retrieval

---

## 4 · Graph Memory

### 4.1 Graph Memory (`enable_graph=True`) ✅
Builds entity-relationship graphs. Graph relations are returned alongside vector search results for richer context.
```python
client.add(messages, user_id="joseph", enable_graph=True)
client.search("Joseph's work", user_id="joseph", enable_graph=True)
```
**Status**: Enabled in all capture hooks and recall searches. Graph relations displayed in recall context and `graph` command.
Docs: https://docs.mem0.ai/platform/features/graph-memory

### 4.2 Graph Threshold ⬜
Tune node-matching strictness (0.0–1.0). Higher = stricter (prevents false merges). Lower = more permissive (merges "Bob"/"Robert").
```python
config = {"graph_store": {"threshold": 0.95}}  # Strict for UUIDs
config = {"graph_store": {"threshold": 0.6}}   # Permissive for natural language
```
Docs: https://docs.mem0.ai/platform/features/graph-threshold

---

## 5 · Data Lifecycle & Governance

### 5.1 Custom Timestamps ⬜
Backdate memories to when events actually occurred (Unix timestamp).
```python
import time
from datetime import datetime, timedelta
five_days_ago = int((datetime.now() - timedelta(days=5)).timestamp())
client.add(messages, user_id="u1", timestamp=five_days_ago)
```
Docs: https://docs.mem0.ai/platform/features/timestamp

### 5.2 Metadata ✅
Attach arbitrary key-value metadata for filtering, auditing, or post-processing.
```python
client.add(messages, user_id="u1", metadata={"source": "slack", "channel": "#support"})
```
**Status**: All hooks attach `source`, `capture`, `session_id` metadata.
Docs: https://docs.mem0.ai/api-reference/memory/add-memories (see `metadata` param)

### 5.3 Expiration ✅
Set auto-deletion date on memories.
**Status**: Pre-compact memories set 7-day expiration. `expire` CLI command available.
Docs: https://docs.mem0.ai/api-reference/memory/add-memories

---

## 6 · Async & Performance

### 6.1 Async Client (`AsyncMemoryClient`) ⬜
Non-blocking add/search/delete for high-concurrency applications.
```python
from mem0 import AsyncMemoryClient
client = AsyncMemoryClient()
await client.add(messages, user_id="alice")
await client.search("query", user_id="alice")
```
Docs: https://docs.mem0.ai/platform/features/async-client

### 6.2 Async Mode Default (`async_mode=True`) ✅
All `add()` calls process asynchronously by default (queued in background). Set `async_mode=False` if you need the processed memory object returned immediately.
```python
# Synchronous (wait for result):
client.add(messages, user_id="u1", async_mode=False)
# Async (default — returns event_id immediately):
client.add(messages, user_id="u1")
```
**Status**: Hooks use async mode (default) for non-blocking capture. Seed uses sync mode for confirmation.
Docs: https://docs.mem0.ai/platform/features/async-mode-default-change

---

## 7 · Feedback & Learning

### 7.1 Feedback Mechanism ✅
Rate memories as POSITIVE / NEGATIVE / VERY_NEGATIVE to continuously improve extraction accuracy. Supports bulk operations.
```python
client.feedback(memory_id="mem-123", feedback="NEGATIVE", feedback_reason="Outdated info")
# Remove feedback:
client.feedback(memory_id="mem-123", feedback=None, feedback_reason=None)
```
**Status**: Available via `feedback` CLI command.
Docs: https://docs.mem0.ai/platform/features/feedback-mechanism

---

## 8 · API Controls (Per-Request)

### 8.1 Output Format (`output_format="v1.1"`) ✅
Controls response shape. `v1.1` (recommended) wraps results in a `results` array and includes graph relations. `v1.0` is deprecated.
```python
client.add(messages, user_id="u1", output_format="v1.1")
```
**Status**: Used in all capture hooks and stats/export commands.
Docs: https://docs.mem0.ai/api-reference/memory/add-memories (see `output_format` param)

### 8.2 API Version (`version="v2"`) ✅
Opt into the v2 memory engine for latest extraction behavior. v1 is deprecated.
```python
client.add(messages, user_id="u1", version="v2")
```
**Status**: Used in all capture hooks.
Docs: https://docs.mem0.ai/api-reference/memory/add-memories (see `version` param)

---

## 9 · Webhooks ✅

### 9.1 Webhook Notifications
Receive POST callbacks when memories are added, updated, deleted, or categorized.
**Status**: Available via `webhooks` CLI command (list/create/delete).

---

## 10 · Export ✅

### 10.1 Memory Export
Export all project memories as structured JSON for backup or analysis.
**Status**: Available via `export` CLI command.

---

## Hook Coverage

| Hook Event | Script | Purpose |
|------------|--------|---------|
| SessionStart | `mem0_hook_recall.py` | Broad project recall at session start |
| UserPromptSubmit | `mem0_hook_recall.py` | Contextual recall per user prompt |
| Stop | `mem0_hook_capture.py` | Auto-capture from `last_assistant_message` |
| SubagentStop | `mem0_hook_subagent.py` | Capture deep analysis from Explore/Plan/general-purpose agents |
| PreCompact | `mem0_hook_compact.py` | Preserve context before compression (7-day expiration) |
