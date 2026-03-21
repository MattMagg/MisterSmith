# Mister Smith Operator Console

Local-only Tauri 2 operator cockpit for Mister Smith.

## Scope

- managed local desktop shell for Mister Smith
- boots local `postgres` + `nats` through `deploy/docker-compose.yml` when no loopback runtime is
  already reachable
- launches the bundled `mister-smith-runtime` sidecar and inspects its loopback HTTP/WebSocket
  surfaces
- React + TypeScript + Vite frontend
- Tauri 2 macOS shell with native auth-status commands
- runtime actions through the existing HTTP API
- live timeline through `/api/v1/events/ws`
- curated NATS monitor state through `http://127.0.0.1:8222/{varz,connz,jsz}`

## Default endpoints

- runtime: `http://127.0.0.1:8080`
- NATS monitor: `http://127.0.0.1:8222`

## Commands

```bash
npm install
npm run tauri dev
npm test -- --run
npm run build
npm run tauri build
npm run tauri build -- --debug
```

## Local prerequisites

- Docker/Compose must be available so the app can bring up `postgres` and `nats`
- the launcher waits for the NATS HTTP monitor on `http://127.0.0.1:8222/varz` before it reports
  managed runtime readiness

## Shell commands exposed to the UI

- `openai_chatgpt_status`
- `login_openai_chatgpt`
- `claude_subscription_status`
