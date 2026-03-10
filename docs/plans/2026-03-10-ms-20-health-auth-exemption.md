# MS-20 Health Auth Exemption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Keep `GET /api/v1/health` public even when HTTP security middleware is enabled, while preserving auth on protected API routes.

**Architecture:** Keep the auth middleware generic and localize the exemption in `mister-smith-http` by splitting the public health route onto its own router and merging it with the protected API router before middleware layering. Prove the behavior with a failing regression test first, then document the probe contract next to the health endpoint.

**Tech Stack:** Rust, Axum, tower, workspace feature-gated `mister-smith-security`

---

### Task 1: Capture the regression in `mister-smith-http`

**Files:**
- Modify: `crates/mister-smith-http/src/server.rs`
- Reference: `crates/mister-smith-security/src/middleware/mod.rs`

**Step 1: Write the failing test**

Add a `#[tokio::test]` in `crates/mister-smith-http/src/server.rs` that:
- builds a `SecurityLayer` with JWT auth enabled
- injects it via `AppState::with_security(...)`
- asserts unauthenticated `GET /api/v1/health` returns `200 OK`
- asserts unauthenticated `GET /api/v1/agents` still returns `401 Unauthorized`

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-http build_router_keeps_health_public_when_security_enabled`
Expected: FAIL because `/api/v1/health` currently inherits auth middleware and returns `401 Unauthorized`.

### Task 2: Implement the minimal router split

**Files:**
- Modify: `crates/mister-smith-http/src/routes.rs`
- Modify: `crates/mister-smith-http/src/server.rs`

**Step 3: Write minimal implementation**

- Extract `GET /api/v1/health` into a small public router function.
- Keep the remaining authenticated endpoints in the protected API router.
- In `build_router`, merge the public router with the protected router before request ID, CORS, and rate limiting layers are applied.
- Apply `security_middleware` only to the protected router so auth behavior for the other endpoints is unchanged.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-http build_router_keeps_health_public_when_security_enabled`
Expected: PASS with `/api/v1/health` public and `/api/v1/agents` still protected.

### Task 3: Document and validate

**Files:**
- Modify: `crates/mister-smith-http/src/handlers.rs`
- Optional: `crates/mister-smith-http/src/routes.rs`

**Step 5: Document intended probe behavior**

Update the health endpoint doc comment to state that `GET /api/v1/health` is intentionally public for liveness/readiness probes and does not require a bearer token.

**Step 6: Run scope validation**

Run:
- `cargo test -p mister-smith-http`
- `cargo build --workspace`
- `vet "MS-20 exempt /api/v1/health from auth middleware" ...`

Expected:
- all crate tests pass
- workspace build succeeds
- vet reports no actionable issues from this diff
