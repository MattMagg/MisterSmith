# 2026-03-21 Operator Console Implementation Plan

## Objective

Implement the local Tauri 2 operator cockpit for Mister Smith as a real desktop executable that
can bootstrap the repo-native local runtime path on launch: start the existing `postgres` and
`nats` development dependencies, launch the bundled `mister-smith` runtime binary, and preserve
the existing operator lists, detail inspection, runtime event timeline streaming, and task/session
action flows from the desktop shell.

## Scope

- `crates/mister-smith-http/`
- `crates/mister-smith-app/`
- `crates/mister-smith-persistence/`
- `apps/operator-console/`

## Assumptions

- The existing local runtime remains the authority for task/session submission and execution.
- The desktop shell continues to target loopback runtime access only.
- Existing session/task write routes remain unchanged and are the action surface for the cockpit.
- The repo-native local dependency path remains `deploy/docker-compose.yml` for `postgres` and
  `nats`.

## Constraints

- Keep the implementation local-only and single-user.
- Use the existing Docker Compose development stack and the existing `mister-smith` runtime binary
  instead of introducing new infrastructure products or alternate runtime hosts.
- Do not widen this slice into signing, notarization, sandbox hardening, or shared multi-user
  concerns.
- Avoid touching unrelated local modifications already present in the worktree.

## Non-Goals

- Hosted/web operator access
- User-facing runtime lifecycle controls beyond booting the local stack on app launch
- Prometheus parsing in the desktop shell
- New auth HTTP endpoints

## Milestones

### 1. Runtime Operator Surfaces

Add list/read surfaces for workflows and sessions, replace mock agent inspection with registry
backing, and bridge runtime lifecycle/autonomy events into the existing websocket channel.

Validation:

- targeted `mister-smith-http` and `mister-smith-app` tests for list routes and websocket bridging
- `cargo build --workspace`

### 2. Desktop Runtime Bootstrap

Bundle the existing `mister-smith` binary as a Tauri sidecar, bundle the repo-local Compose file
as a resource, and let the app bring up `postgres` + `nats` before launching the runtime when no
loopback runtime is already reachable.

Validation:

- targeted desktop helper tests for launch-state helpers
- Tauri Rust backend build succeeds

### 3. Tauri Operator Shell

Keep the operator UI under `apps/operator-console/`, wire loopback HTTP/WebSocket flows, surface
launcher state alongside runtime health, persist local operator settings, and preserve the
runs/sessions/agents/health views plus task/session actions.

Validation:

- frontend tests for disconnected/connected/action flows
- Tauri frontend/backend build succeeds

## Stop Conditions

- The operator shell requires new runtime write routes instead of the existing task/session action
  routes.
- Runtime events still only publish to JetStream and do not reach the websocket feed.
- Agent list/detail remain mock-backed after the slice.
- The app cannot bootstrap the repo-native local runtime path without inventing new infra beyond
  the existing Compose services and bundled `mister-smith` binary.

## UI Polish Addendum

Objective:

- Tighten the operator-console presentation without changing the runtime/action model: reduce hero
  dominance, compact the status/control strip, clean up auth-card copy density, and soften noisy
  degraded-state banners that are expected in local managed-runtime use.

Scope:

- `apps/operator-console/src/App.tsx`
- `apps/operator-console/src/App.css`
- `apps/operator-console/src/index.css`

Validation:

- `npm test -- --run`
- `npm run build`

## Launcher Status Follow-Up

Objective:

- Keep startup-state reporting honest: do not show derivative websocket or NATS-monitor failures
  while the launcher is still booting local dependencies, and wait for the NATS HTTP monitor that
  the desktop shell actually queries before advertising managed readiness.

Scope:

- `apps/operator-console/src/App.tsx`
- `apps/operator-console/src/services.ts`
- `apps/operator-console/src/App.test.tsx`
- `apps/operator-console/src-tauri/src/managed_runtime.rs`

Validation:

- `npm test -- --run`
- `cargo test --manifest-path apps/operator-console/src-tauri/Cargo.toml`
- `npm run build`
