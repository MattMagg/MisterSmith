## PR Review: #109 - 008 agent system

### Review Scope

- `BRANCH_NAME=work` (compared against base commit `419ca2e`, the previous merge-to-main commit in local history).
- `REVIEW_DEPTH=comprehensive`.
- `FOCUS_AREAS=code-quality,testing,documentation,security`.

### Summary

This PR introduces Phase 7 agent-system specifications (`specs/008-agent-system/*`) and adds durable transport primitives for at-least-once messaging (`DurableTransport`, `DurableMessage`, and backend wiring in `mister-smith-nats`). It also includes message-idempotency persistence migration work and roadmap/spec cross-reference updates.

Overall this is a substantial, coherent change set with meaningful test coverage in the transport crate and no obvious high-risk security regressions in the reviewed diff.

### Recommendation

**COMMENT**

(Ready from a correctness/security perspective based on local verification, with one commit-history quality concern and one maintainability suggestion.)

### Key Findings

#### 🔴 Blocking Issues (must fix)

- None identified in local review.

#### 🟡 Suggestions (should consider)

1. **Commit message quality does not match repository convention**  
   The only commit in scope is `008 agent system (#109)`, which is not in the documented style (`docs: ...`, `fix(spec): ...`, `feat: ...`, etc.). Consider rewriting/squashing to a conventional message before merge for long-term history clarity.

2. **Add unit coverage for durable ack error mapping paths**  
   Transport tests passed for `mister-smith-transport`, but the new JetStream durable ack bridge (`ack/nak/term/in_progress`) in `mister-smith-nats` would benefit from focused tests that assert consistent `TransportError` conversion across all ack operations.

#### 🟢 Nitpicks (optional)

- Consider adding a brief cross-link from the new `specs/008-agent-system/quickstart.md` into the canonical `spec/` architecture index to improve discoverability for contributors who only start from `spec/`.

### Tested

- [x] `cargo test -p mister-smith-transport --lib` (pass: 56 passed)
- [x] `git diff 419ca2e..HEAD | rg -n "TODO|FIXME|HACK|XXX|password|secret|api_key|token|private_key"` (no concerning code-level markers in diff)
- [x] `npx markdownlint-cli2 "spec/**/*.md" "*.md" --config .markdownlint.json` (fails repo-wide with many pre-existing lint violations; not attributable solely to this PR)

### Checklist Summary

| Category | Status | Notes |
|----------|--------|-------|
| Code Quality | ✅ | Durable transport abstraction is coherent; no obvious logic defects found in sampled diff. |
| Security | ✅ | No hardcoded secrets or obvious unsafe input handling added in reviewed changes. |
| Testing | ⚠️ | Transport crate tests pass; no targeted tests run for all new durable ack conversion paths in `mister-smith-nats`. |
| Documentation | ✅ | Large spec expansion with roadmap/spec updates present. |
| Style | ⚠️ | Commit subject does not follow documented conventional commit pattern. |
| Performance | ✅ | No obvious high-cost algorithmic regressions introduced in reviewed Rust additions. |

### Commands Used

```bash
git log --oneline --decorate -n 5
git show --stat --oneline HEAD^..HEAD
git diff --name-only 419ca2e..HEAD
git log 419ca2e..HEAD --format=fuller
git diff 419ca2e..HEAD --stat
git diff 419ca2e..HEAD | rg -n "TODO|FIXME|HACK|XXX|password|secret|api_key|token|private_key"
cargo test -p mister-smith-transport --lib
npx markdownlint-cli2 "spec/**/*.md" "*.md" --config .markdownlint.json
```
