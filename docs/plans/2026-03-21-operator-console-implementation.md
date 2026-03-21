# 2026-03-21 Operator Console Implementation Plan

## Objective

Implement the attach-first local Tauri 2 operator cockpit for Mister Smith with the minimum
runtime/backend additions needed to support live operator lists, detail inspection, runtime event
timeline streaming, and the existing task/session action flows from a desktop shell.

## Scope

- `crates/mister-smith-http/`
- `crates/mister-smith-app/`
- `crates/mister-smith-persistence/`
- `apps/operator-console/`

## Assumptions

- The existing local runtime remains the authority for task/session submission and execution.
- The desktop shell targets loopback runtime access only and does not own the runtime process in
  v1.
- Existing session/task write routes remain unchanged and are the action surface for the cockpit.

## Constraints

- Keep the implementation attach-first and local-only.
- Do not add runtime lifecycle ownership, sidecars, Langfuse, or broader observability products.
- Do not widen this slice into signing, notarization, sandbox hardening, or shared multi-user
  concerns.
- Avoid touching unrelated local modifications already present in the worktree.

## Non-Goals

- Hosted/web operator access
- Runtime process start/stop controls
- Prometheus parsing in the desktop shell
- New auth HTTP endpoints

## Milestones

### 1. Runtime Operator Surfaces

Add list/read surfaces for workflows and sessions, replace mock agent inspection with registry
backing, and bridge runtime lifecycle/autonomy events into the existing websocket channel.

Validation:

- targeted `mister-smith-http` and `mister-smith-app` tests for list routes and websocket bridging
- `cargo build --workspace`

### 2. Desktop Rust Bridge

Expose the existing auth helpers to a Tauri shell without changing the runtime HTTP contract.

Validation:

- targeted desktop command tests if added
- Tauri Rust backend build succeeds

### 3. Tauri Operator Shell

Scaffold the Tauri 2 app under `apps/operator-console/`, wire loopback HTTP/WebSocket flows,
persist local operator settings, and build the runs/sessions/agents/health views plus task/session
actions.

Validation:

- frontend tests for disconnected/connected/action flows
- Tauri frontend/backend build succeeds

## Stop Conditions

- The operator shell requires new runtime write routes instead of the existing task/session action
  routes.
- Runtime events still only publish to JetStream and do not reach the websocket feed.
- Agent list/detail remain mock-backed after the slice.
- The app cannot inspect or act on local runtime state without widening scope beyond the attach-first
  cockpit model.
