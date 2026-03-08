# Claude Code Infrastructure Bug Investigation — Root Cause Analysis

**Date:** 2026-03-08
**Claude Code Version:** 2.1.71
**Platform:** macOS (Darwin 25.3.0, arm64)

## Executive Summary

A filesystem-level audit of Claude Code's runtime artifacts revealed 12 distinct issues across telemetry, task management, file locking, MCP servers, plugin caching, and observability subsystems. Five are upstream bugs in Claude Code itself. Total reclaimable disk space: **~1.53 GB**.

## Severity Classification

| Severity | Issues | Impact |
|----------|--------|--------|
| **HIGH** | Plugin temp_git leak (#8) | 751 MB and growing ~60 MB/day |
| **MEDIUM** | Telemetry orphans (#1), Observer sessions (#9), Managed-settings noise (#6) | 73 MB stale, 705 MB stale, 30% debug log pollution |
| **LOW** | Todos (#4), Session-env (#5), Task dirs (#2), Lock files (#3), IDE MCP (#7), MCP logs (#11), Stats cache (#10), Backups (#12) | Cosmetic debris, no functional impact |

---

## Issue-by-Issue Root Causes

### 1. TELEMETRY: 73 MB of 100% failed events

**Root cause: HTTP 429 rate limiting + cross-session orphaning bug**

- `~/.claude/telemetry/` — 136 files, ALL named `1p_failed_events.*`, totaling 73 MB, containing 50,978 events
- 1P (first-party) telemetry is always-on, separate from the opt-in 3P system
- Events sent to `https://api.anthropic.com/api/claude_code/metrics`, authenticated via OAuth
- During heavy usage (Mar 3: 106 files in one day), Anthropic's endpoint returned **429 Too Many Requests**
- Failed events are written to disk for retry, but `retryPreviousBatches()` only processes files matching the *current* session's UUID prefix — files from prior sessions are permanently orphaned
- No TTL, no size cap, no age-based cleanup
- All 136 files are from version 2.1.63–2.1.68; **no new failures since upgrading to 2.1.71** (likely fixed server-side or in newer client)
- File naming pattern: `1p_failed_events.<session-UUID>.<batch-UUID>.json`
- 115 unique sessions across 136 files (some sessions produced multiple failed batches)
- 60+ distinct event types observed, all prefixed with `tengu_` (Anthropic's internal codename for Claude Code)

**The accumulation bug:** `retryPreviousBatches()` filters files by `1p_failed_events.<currentSessionId>.` prefix. Files from previous sessions are never retried by subsequent sessions. Since each session generates a new UUID, old failed files are permanently orphaned.

**3P telemetry status:** Disabled. `[3P telemetry] isTelemetryEnabled=false (CLAUDE_CODE_ENABLE_TELEMETRY=undefined)`. No user-facing opt-out exists for 1P telemetry.

| Attribute | Value |
|-----------|-------|
| **Impact** | 73 MB of dead weight. Zero functional impact. |
| **Fix** | `rm ~/.claude/telemetry/1p_failed_events.*.json` — safe, permanent |
| **Classification** | Upstream bug: Cross-session retry scoping + missing TTL cleanup |

---

### 2. TASK DIRS: 56 of 70 are empty shells

**Root cause: Normal end-state — consumed tasks are deleted, skeleton dirs are not**

- `~/.claude/tasks/` — 70 UUID directories. 56 contain ONLY `.lock` + `.highwatermark`, zero JSON data. 14 have actual data.
- Tasks use a DAG-based execution model with `.highwatermark` consumption cursors
- `.highwatermark` files contain a single integer: the highest task ID consumed. In populated dirs, JSON files start at hwm+1
- JSON task files are deleted as they're consumed; the 56 empty dirs had highwatermarks of 1–40 (average 11.9) — they DID process real tasks
- `.lock` files are zero-byte advisory mutexes, never cleaned up
- 6 of 56 empty dirs lack a `.highwatermark` — sessions created but never received tasks
- JSON task schema: `id`, `subject`, `description`, `activeForm`, `status` (completed/in_progress/pending), `blocks`/`blockedBy` (DAG dependency edges)

| Attribute | Value |
|-----------|-------|
| **Impact** | ~1 MB total. No functional impact. |
| **Fix** | Safe to delete empty dirs, but not worth the effort |
| **Classification** | Expected behavior — missing garbage collection, not a bug |

---

### 3. STALE LOCK FILES: Not actually stale

**Root cause: The IDE lock is active; task locks are inert debris**

- **`~/.claude/ide/25029.lock`** — The filename `25029` is a **WebSocket port number**, not a PID. The VS Code extension starts a WebSocket MCP server on a random localhost port and names the lock file after that port. The `pid` field inside the JSON is `process.ppid` (77076 = VS Code main process `/Applications/Visual Studio Code.app/Contents/MacOS/Code`). **PID 77076 confirmed running, started at 07:53:43 today.** This lock is active and valid.
- **Cleanup mechanism:** The extension calls `Ui(port)` on `dispose()`, which deletes the lock via `fs.unlinkSync`. Only fires on clean VS Code deactivation — crashes would orphan the file. Only 1 lock file present, confirming cleanup works normally.
- **`mcp-refresh-*.lock.lock/`** — A directory (not file), uses `mkdir`-based atomic locking (POSIX mkdir is atomic). Stale from Mar 3 but inert.
- **Task `.lock` files** — 67 zero-byte files, advisory mutexes, never cleaned up, harmless.

| Attribute | Value |
|-----------|-------|
| **Impact** | None. Initial audit misunderstood the IDE lock naming convention. |
| **Fix** | No action needed |
| **Classification** | Not a bug |

---

### 4. TODOS: 2,535 of 2,536 are empty `[]`

**Root cause: Eager initialization without cleanup**

- `~/.claude/todos/` — 2,536 JSON files, 5.6 KB actual data
- Claude Code creates an empty `[]` todo file for every agent in every session at startup, regardless of whether `TodoWrite` is ever invoked
- Filename pattern: `{session-UUID}-agent-{agent-UUID}.json`
  - 2,499 files (98.5%): both UUIDs identical (primary agent for session)
  - 37 files (1.5%): UUIDs differ (sub-agents within parent session)
- 99.96% are never populated. The single non-empty one contains a 4-item checklist:
  ```json
  [
    {"content": "Explore project context...", "status": "completed", "activeForm": "..."},
    {"content": "Ask clarifying questions...", "status": "completed", "activeForm": "..."},
    {"content": "Present design for approval", "status": "in_progress", "activeForm": "..."},
    {"content": "Write design doc...", "status": "pending", "activeForm": "..."}
  ]
  ```
- Date range: Jan 20 – Feb 28 (stopped accumulating — possibly fixed in newer version)

| Attribute | Value |
|-----------|-------|
| **Impact** | 5.6 KB total. 2,536 files in one directory could slow `ls` but doesn't affect runtime. |
| **Fix** | `rm ~/.claude/todos/*-agent-*.json` — safe |
| **Classification** | Upstream bug: Missing cleanup for unused todo files |

---

### 5. SESSION-ENV: 470 empty directories

**Root cause: Unconditional creation, no cleanup**

- `~/.claude/session-env/` — 471 empty subdirectories, all named with UUIDs matching session IDs
- `getSessionEnvDir()` calls `mkdir(dir, { recursive: true })` at session start, **before** checking if any scripts exist
- Purpose: hold hook-generated `.sh` scripts that are `source`-d before Bash tool invocations to inject environment variables
- Scripts can come from `CLAUDE_ENV_FILE` env var or `SessionStart`/`Setup` hooks writing `sessionstart-hook-<N>.sh` files
- Since no hooks write session-env scripts and `CLAUDE_ENV_FILE` is not set, every directory stays empty
- Debug log confirms: `"No session environment scripts found"` — the check happens AFTER creation
- ~15 new empty dirs per day (471 in ~30 days)
- No cleanup mechanism: `invalidateSessionEnvCache()` only clears an in-memory variable, does not touch filesystem

| Attribute | Value |
|-----------|-------|
| **Impact** | ~30 KB of empty directories. No functional impact. |
| **Fix** | `find ~/.claude/session-env -maxdepth 1 -type d -empty -not -name "session-env" -mtime +1 -exec rmdir {} \;` |
| **Classification** | Upstream bug: Should check for scripts before creating directory, or clean up on session end |

---

### 6. MANAGED-SETTINGS: 868 warnings in a single session

**Root cause: Enterprise MDM feature with no negative caching**

- `/Library/Application Support/ClaudeCode/managed-settings.json` is for IT-administered policy enforcement (MDM profiles)
- This path does not exist on non-enterprise machines
- Claude Code re-reads this path on **every** permission check and hook dispatch — not cached
- In a 46-minute session:
  - **1,933 debug lines** (868 "Broken symlink" warnings + 967 ENOENT errors)
  - = **29.4%** of the entire 6,572-line debug log
  - Peak: **300 warnings/minute** during active tool execution
- Triggers on every: PreToolUse hook dispatch, PostToolUse hook dispatch, SubagentStart, message deferred-value update, hook matcher resolution
- Initial MDM load completes in 13ms at startup
- Filesystem overhead is sub-millisecond per stat call, but debug log is nearly unusable for actual debugging

| Attribute | Value |
|-----------|-------|
| **Impact** | Debug log pollution makes real issues harder to find. Negligible latency impact. |
| **Fix** | Could create stub file to silence warnings, but risks enterprise feature conflicts. Better fix is upstream. |
| **Classification** | Upstream bug: Should cache negative lookup result per session |

---

### 7. IDE MCP SERVER: WebSocket reconnect storm

**Root cause: Two failure modes, both expected in context**

- The "ide" MCP server is the Claude Code VS Code extension, providing WebSocket-based access to `closeAllDiffTabs` and `getDiagnostics` tools
- Server identity: `"Claude Code VSCode MCP"` (v2.1.71)

**Mode 1 — Terminal sessions (expected failure):**
- CLI starts without VS Code WebSocket → fails in 22ms → gives up
- `MCP server "ide": Connection failed after 22ms: WebSocket is not open. Cannot start transport.`
- Graceful, no retry loop

**Mode 2 — VS Code sessions (reconnect storm):**
- WebSocket connects successfully in 1–7ms, then immediately disconnects
- Claude Code auto-reconnects → another disconnect → tight loop
- 9 connects, 7 disconnects, 1 error — all within the same second
- Error: `Failed to fetch tools: MCP error -32000: Connection closed`
- Likely caused by concurrent CLI sessions competing for the same WebSocket endpoint
- `getDiagnostics` tool intermittently fails with "Not connected" even after reconnect appears to succeed

**Log volume:** 73 log files, 2.2 MB across 6 days. Largest single file: 208 KB (1,115 lines, 3.5-hour session).

| Attribute | Value |
|-----------|-------|
| **Impact** | Log noise. `getDiagnostics` unreliable. No impact from terminal. |
| **Fix** | No user action needed |
| **Classification** | Upstream bug: No backoff on WebSocket reconnect; concurrent session contention |

---

### 8. PLUGIN AUTOUPDATE: temp_git dirs never cleaned — THE MOST IMPACTFUL BUG

**Root cause: `rename()` fails with ENOTEMPTY, cleanup never runs**

- **Only the `superpowers` plugin** is affected — sole plugin using a raw git URL source (`https://github.com/obra/superpowers.git`)
- All other plugins use marketplace versioned caches and skip git clones entirely

**The lifecycle (proven from debug logs):**

1. Every session start: plugin autoupdate clones `https://github.com/obra/superpowers.git` into `~/.claude/plugins/cache/temp_git_<timestamp>_<random>`
2. After cloning, attempts `rename()` to `~/.claude/plugins/cache/superpowers`
3. Rename fails with `ENOTEMPTY` (directory already exists from prior session's successful rename)
4. Error logged as WARN, execution continues
5. **temp_git directory is never cleaned up** — no try/finally, no error handler deletes it

**Direct evidence:**
```
Caching plugin from source: {"source":"url","url":"https://github.com/obra/superpowers.git"}
  to temporary path .../temp_git_1772968523595_h8er5d
WARN: Plugin autoupdate: error updating superpowers@superpowers-marketplace:
  ENOTEMPTY: directory not empty, rename 'temp_git_1772968523595_h8er5d' -> '/cache/superpowers'
```

**Race condition:** One session cloned the repo TWICE (temp dirs created 2ms apart — `temp_git_1772968523595_h8er5d` and `temp_git_1772968523597_135odz`).

**Statistics:**
- 449 orphaned directories, 751 MB total (~1.1 MB per clone)
- Accumulating since Mar 1 (~56 new dirs/day)
- The actual working cache at `superpowers-marketplace/superpowers/4.3.1` (760 KB) is intact and used at runtime

| Attribute | Value |
|-----------|-------|
| **Impact** | 751 MB growing at ~60 MB/day. Most impactful issue by disk usage and growth rate. |
| **Fix** | `rm -rf ~/.claude/plugins/cache/temp_git_*` — reclaims 751 MB immediately. **Leak resumes next session.** |
| **Classification** | Upstream bug: Autoupdate never cleans up failed renames for git-URL-sourced plugins. Needs try/finally or pre-clone cleanup. |

---

### 9. CLAUDE-MEM OBSERVER: 705 MB of phantom sessions

**Root cause: Plugin architecture spawns disposable Claude Code sessions with no transcript cleanup**

- The claude-mem plugin (`@thedotmack`, v10.5.2) hooks into 5 lifecycle events: `SessionStart`, `UserPromptSubmit`, `PostToolUse`, `Stop`, plus startup/clear/compact variants
- Each hook invocation spawns a full Claude Code SDK session using `~/.claude-mem/observer-sessions/` as the working directory
- Each session creates a `.jsonl` transcript in Claude Code's project storage at `~/.claude/projects/-Users-matthewmaggio--claude-mem-observer-sessions/`
- **1,910 JSONL files** (643.8 MB, average 345 KB, largest 16 MB) + **125 subagent directories** = **705 MB total**
- Creation rate: 60–167 sessions/day (peak 167 on Feb 25)
- The actual `~/.claude-mem/observer-sessions/` directory is **empty** — all data accumulates in Claude Code's internal project storage as a side effect
- The project directory `~/.claude/projects/-Users-matthewmaggio--claude-mem-observer-sessions/` maps to a path that no longer exists as a real filesystem directory

| Attribute | Value |
|-----------|-------|
| **Impact** | 705 MB of orphaned transcripts. 3x larger than the Mister Smith project's transcripts. |
| **Fix** | `rm -rf ~/.claude/projects/-Users-matthewmaggio--claude-mem-observer-sessions/` — safe |
| **Classification** | Plugin design issue: Should clean up disposable sessions or use a mechanism that doesn't create persistent transcripts |

---

### 10. STATS CACHE: Stale since Feb 20

**Root cause: Recomputation trigger unknown, possibly blocked by observer session volume**

- `~/.claude/stats-cache.json` — `lastComputedDate: "2026-02-20"` (16 days stale at time of audit)
- Contains daily activity from Jan 20 – Feb 20, model usage aggregates, 640 total sessions, 98,627 total messages
- `costUSD: 0` for all models (not implemented for CLI subscriptions)
- No "stats" or "recompute" entries found in any recent debug log — the mechanism leaves no trace
- **Hypothesis:** Stats scanner iterates all project directories/transcripts. The 2,036-file observer-sessions directory may be causing a timeout or silent failure. Alternatively, a version upgrade may have changed/disabled the trigger.

| Attribute | Value |
|-----------|-------|
| **Impact** | `/stats` view shows data only through Feb 20. No impact on functionality. |
| **Fix** | Deleting observer-sessions project dir (Issue #9) may unblock recomputation. Deleting `stats-cache.json` would force fresh computation on next trigger. |
| **Classification** | Likely blocked by observer session volume, not a standalone bug |

---

### 11. MCP SERVERS: Irrelevant services connecting per-session

**Root cause: Server-side account configuration, now partially resolved**

- Cloud MCP servers are fetched dynamically from `https://api.anthropic.com/v1/mcp_servers?limit=1000` on each session start
- They connect through Anthropic's MCP proxy at `https://mcp-proxy.anthropic.com/v1/mcp/<server_id>`
- **Previously (Mar 6–7):** 7 servers connecting every session (ClickUp, Context7, Gmail, Google Calendar, MS365, Notion, Tavily), each taking 350–1100ms, adding 2–8 seconds to startup
- **Now (Mar 8):** Reduced to 2 servers via server-side change. Total MCP startup: ~600ms for 4 local + 2 cloud servers
- Stale log directories from the 7-server era: ~768 KB, no longer growing
- Gamma MCP server showed `Token refresh failed with status 429: ThrottlerException: Too Many Requests` — suggesting concurrent sessions hitting token refresh rate limits

**Active MCP log dirs (today):** `mcp-logs-ide` (2.2M), `mcp-logs-claude-in-chrome` (392K), `mcp-logs-plugin-claude-mem-mcp-search` (304K), `mcp-logs-rube` (1.0M)

**Stale MCP log dirs (safe to delete):** `mcp-logs-claude-ai-Context7`, `mcp-logs-claude-ai-Gmail`, `mcp-logs-claude-ai-Google-Calendar`, `mcp-logs-claude-ai-Notion`, `mcp-logs-claude-ai-ms365`, `mcp-logs-claude-ai-tavily`, `mcp-logs-claude-ai-Midpage-Legal-Research`

| Attribute | Value |
|-----------|-------|
| **Impact** | Previously added 2–8 seconds to startup. Now resolved server-side. |
| **Fix** | Delete stale `mcp-logs-claude-ai-*` dirs for cleanup. Underlying issue is resolved. |
| **Classification** | Resolved server-side |

---

### 12. BACKUP CHURN: 5 backups in 17 minutes

**Root cause: Normal behavior — rolling backup on every `.claude.json` write**

- `~/.claude/backups/` — 5 files, all from today within a 17-minute window (08:33 to 08:50)
- All ~64 KB, named `.claude.json.backup.<timestamp>`
- Backups are triggered by atomic writes to `~/.claude.json` (the main Claude Code state file containing `numStartups`, `tipsHistory`, `promptQueueUseCount`, `cachedStatsigGates`, `cachedGrowthBookFeatures`, plugin metadata)
- Only 3 unique content hashes across 5 files (2 were duplicate writes with identical content)
- Triggers: prompt submission (increments queue count), plugin state updates, feature flag cache refreshes
- **Rolling retention**: Only 5 files present, suggesting old backups are pruned — not unbounded growth
- Debug log confirms: `"File /Users/matthewmaggio/.claude.json written atomically"` (17 writes in this session)

| Attribute | Value |
|-----------|-------|
| **Impact** | None. 320 KB total. Working as designed. |
| **Fix** | No action needed |
| **Classification** | Expected behavior |

---

## Safe Cleanup Commands

```bash
# Issue #8 — Most impactful: reclaim 751 MB (will regrow ~60 MB/day)
rm -rf ~/.claude/plugins/cache/temp_git_*

# Issue #9 — Reclaim 705 MB (orphaned plugin transcripts)
rm -rf ~/.claude/projects/-Users-matthewmaggio--claude-mem-observer-sessions/

# Issue #1 — Reclaim 73 MB (orphaned telemetry from old version)
rm ~/.claude/telemetry/1p_failed_events.*.json

# Issue #11 — Reclaim ~768 KB (stale MCP log dirs)
rm -rf ~/Library/Caches/claude-cli-nodejs/-Users-matthewmaggio-Mister-Smith/mcp-logs-claude-ai-*

# Issue #4 — Clean up 2,536 empty todo files
rm ~/.claude/todos/*-agent-*.json

# Issue #5 — Clean up 470 empty session-env dirs
find ~/.claude/session-env -maxdepth 1 -type d -empty -not -name "session-env" -mtime +1 -exec rmdir {} \;
```

**Total reclaimable: ~1.53 GB**

---

## Bugs to Report to Anthropic

| # | Bug | Severity | Details |
|---|-----|----------|---------|
| 1 | **Plugin autoupdate: ENOTEMPTY on rename leaves temp_git dirs** | High | Growing ~60 MB/day. Only affects git-URL-sourced plugins. `rename()` failure has no cleanup handler. |
| 2 | **1P telemetry: cross-session orphaning** | Medium | `retryPreviousBatches()` only retries current session's files. No TTL/age-based cleanup. No directory size cap. |
| 3 | **Managed-settings: no negative caching** | Medium | 30% of debug log is one repeated warning for a nonexistent MDM file. Should cache "file doesn't exist" per session. |
| 4 | **Session-env, todos, tasks: no cleanup of empty/consumed artifacts** | Low | Directories and empty JSON files accumulate indefinitely with no garbage collection. |
| 5 | **IDE MCP: no reconnect backoff on WebSocket disconnect** | Low | Tight connect-disconnect loop when concurrent sessions compete for same WebSocket endpoint. |
| 6 | **Agent teams: initialization race condition** | Low | `getTeammateModeFromSnapshot` called before capture during settings hot-reload. Non-fatal, self-heals in milliseconds. |

## Not Bugs

| Issue | Verdict |
|-------|---------|
| IDE lock file (`25029.lock`) | Active and valid — filename is WebSocket port number, not PID |
| Backup churn | Normal behavior, rolling retention works correctly |
| Cloud MCP servers | Resolved server-side (reduced from 7 to 2 servers) |
| Stats cache staleness | Likely blocked by observer session volume, not a standalone bug |
| Task directory skeletons | Normal end-state of consumed DAG task sessions |

---

## Key Locations Reference

| Location | Purpose | Size |
|----------|---------|------|
| `~/.claude/telemetry/` | 1P failed event queue | 73 MB (136 files) |
| `~/.claude/tasks/` | DAG task execution state | ~1 MB (70 dirs) |
| `~/.claude/todos/` | TodoWrite tool state | 5.6 KB (2,536 files) |
| `~/.claude/session-env/` | Session environment scripts | ~30 KB (471 empty dirs) |
| `~/.claude/ide/` | VS Code extension lock files | ~4 KB |
| `~/.claude/backups/` | Rolling `.claude.json` backups | 320 KB (5 files) |
| `~/.claude/debug/` | Session debug logs | 528 MB (3,488 files) |
| `~/.claude/plugins/cache/temp_git_*` | Orphaned plugin clone dirs | 751 MB (449 dirs) |
| `~/.claude/projects/-Users-matthewmaggio--claude-mem-observer-sessions/` | Phantom plugin transcripts | 705 MB (2,036 entries) |
| `~/.claude/stats-cache.json` | Usage statistics cache | 14 KB |
| `~/Library/Caches/claude-cli-nodejs/.../mcp-logs-*/` | MCP server logs | ~5 MB (16 dirs) |
| `/Library/Application Support/ClaudeCode/` | Enterprise MDM settings | Does not exist |
