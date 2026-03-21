# Mister Smith Operator Console

Local-only Tauri 2 operator cockpit for Mister Smith.

## Scope

- attach-first desktop shell for a locally running Mister Smith runtime
- React + TypeScript + Vite frontend
- Tauri 2 macOS shell with native auth-status commands
- runtime actions through the existing HTTP API
- live timeline through `/api/v1/events/ws`

## Default endpoints

- runtime: `http://127.0.0.1:8080`
- NATS monitor: `http://127.0.0.1:8222`

## Commands

```bash
npm install
npm run dev
npm test -- --run
npm run build
npm run tauri build -- --debug
```

## Shell commands exposed to the UI

- `openai_chatgpt_status`
- `login_openai_chatgpt`
- `claude_subscription_status`
