## PR Review: #108 - feat(persistence): add dual-store persistence layer (Phase 6)

### Summary

Adds the `mister-smith-persistence` crate implementing a dual-store architecture (PostgreSQL + JetStream KV) with repository abstractions, migration tooling, hybrid state management, and audit bridge. 54 files changed, +9,722 lines, 165 new tests.

### Recommendation

**REQUEST_CHANGES**

### Key Findings

#### Blocking Issues (must fix)

1. **`MigrationRunner::status()` and `applied_count()` silently swallow database errors** (CLAUDE.md says "No broad try/catch blocks or success-shaped fallbacks... Propagate or surface errors explicitly rather than swallowing them")

   `status()` at line 110 uses `.unwrap_or_default()` and `applied_count()` at line 207 uses `.unwrap_or((0,))`, both converting any database error (connection failure, permission denied, missing table) into a success-shaped default value. This makes `verify()` and migration status checks silently report "zero migrations applied" on infrastructure failures.

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/src/postgres/migrations.rs#L107-L110

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/src/postgres/migrations.rs#L204-L209

2. **`get_config` binds 3 parameters for a 2-placeholder query on the NULL branch** (runtime SQL error)

   When `agent_id` is `None`, the query uses `agent_id IS NULL` with only `$1` and `$2` placeholders, but line 1059 unconditionally binds `agent_id` as a third parameter. PostgreSQL will reject this at runtime with a bind-count mismatch error.

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/src/postgres/queries.rs#L1047-L1063

3. **`MessageRepository::delete()` uses status `"cancelled"` which violates the `valid_msg_status` CHECK constraint** (runtime SQL error)

   The soft-delete at line 89 sets status to `"cancelled"`, but the schema CHECK constraint at migration line 121 only allows: `'pending', 'sent', 'delivered', 'processed', 'failed', 'expired'`. Every call to `delete()` will fail with a constraint violation.

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/src/repository/message.rs#L87-L92

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/migrations/00001_initial_schema.sql#L119-L122

4. **`from_kv_version_error` hardcodes `actual: 0`, producing misleading error messages** (diagnostic correctness)

   The `VersionConflict` variant documents `actual` as "The actual revision found", but this function always returns `actual: 0`. Every version conflict error will display as `"expected N, actual 0"` regardless of the real server revision, misleading operators debugging concurrency issues.

   https://github.com/MattMagg/MisterSmith/blob/4f086371793932a602efe721943a50acf3dc3eee/crates/mister-smith-persistence/src/error.rs#L59-L65

### Other Notable Findings (below threshold, included for awareness)

| Issue | Score | Source | Description |
|-------|-------|--------|-------------|
| flush_to_sql drains dirty keys before SQL write; mid-batch failure loses keys | 75 | Internal | Data loss risk if SQL transaction fails mid-batch |
| AuditPersister TOCTOU race on concurrent flush | 75 | Internal | Duplicate audit rows possible |
| AuditPersister reset logic includes all ring buffer events | 75 | Internal | Deduplication broken at reset boundary |
| NULL-unsafe UNIQUE constraint on configurations | 75 | Both | PostgreSQL doesn't treat NULLs as equal in UNIQUE |
| Two incompatible PersistenceConfig types (stub vs full) | 75 | Internal | TOML config silently drops persistence subsections |
| from_sqlx_error re-exported unconditionally with different signatures | 75 | Internal | Public API varies by feature flag |
| Repository::update doc promises VersionConflict, none deliver | 75 | Internal | Documented contract not implemented |
| flush_to_sql inline SQL omits checksum column | 75 | Both | Data inconsistency vs upsert_state helper |
| FrameworkConfig::validate() skips persistence | 75 | Internal | Pattern violation vs other config sections |
| Unused mister-smith-config dependency | 75 | Internal | Build hygiene |
| Default safety_margin_secs equals cache_ttl_secs | 75 | Internal | Undocumented 10s floor trap |
| `re_mark` and `mark_dirty` are functionally identical | -- | External | `re_mark` resets `oldest_dirty_at` instead of preserving original timestamp, delaying retry flushes |
| `configurations` table in `public` schema | -- | External | Every other table uses named schemas; inconsistent |
| Background flush has no shutdown mechanism | -- | External | Only `abort()` available; no graceful final flush |
| `check_partition_coverage` `range_end` always `''` | -- | External | Column is misleading; all data in `range_start` via `pg_get_expr()` |
| PersistenceHealthChecker has different constructor signatures per feature flag | 75 | Both | With sqlx: 2-arg `new(pg_pool, kv_context)`, without: 1-arg `new(kv_context)` — breaks downstream code toggling features |
| `from_kv_error` "revision" substring match too broad | 50 | Internal | `msg.contains("revision")` misclassifies unrelated NATS errors as VersionConflict |
| `write_state` returns `Ok(0)` revision in KV-degraded mode | -- | External | Callers treating revision as meaningful silently get 0; no indication of degraded path |

### Cross-Review Comparison

#### Issues unique to this review (missed by external report)
1. **MigrationRunner unwrap_or_default()** (score 100) -- runtime silent failure
2. **get_config extra bind parameter** (score 100) -- runtime crash on NULL agent_id
3. **MessageRepository::delete() CHECK violation** (score 100) -- runtime crash on every delete

#### Issues found by both reviews
1. **NULL-unsafe UNIQUE constraint on configurations** -- global config upserts create duplicates
2. **flush_to_sql inline SQL omits checksum** -- data inconsistency vs upsert_state helper
3. **PersistenceHealthChecker inconsistent constructor signatures** -- breaks code toggling features
4. **AuditPersister ID dedup drift** -- deduplication weaker than documented

#### Issues unique to external report (validated)
1. **`re_mark`/`mark_dirty` identical** -- valid; `re_mark` should preserve original timestamp
2. **`configurations` in `public` schema** -- valid consistency issue
3. **Background flush no shutdown** -- valid production readiness concern
4. **`check_partition_coverage` range_end always `''`** -- valid; misleading column
5. **`write_state` returns `Ok(0)` in KV-degraded mode** -- valid; callers get silent zero revision

#### Issues rejected from external report
- **`parse_kv_key` accepts empty state keys** -- test explicitly asserts this behavior (intentional)

### Checklist Summary

| Category | Status | Notes |
|----------|--------|-------|
| Code Quality | Needs fixes | Silent error swallowing, parameter mismatch, constraint violation |
| Security | OK | No secrets, SQL parameterized |
| Testing | OK | 165 new tests, good coverage for non-DB paths |
| Documentation | Needs fixes | VersionConflict actual field semantics incorrect |
| Style | OK | Follows codebase patterns |
| Performance | OK | Reasonable indexing and partitioning strategy |
