# 2026-04-05 Mister Smith Session-First CLI And GUI Proposal

For the deeper pre-SpecKit planning layer that should drive later packet authoring, use
`docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`.

## Objective

Define the user-facing Mister Smith shell.

This should be a session-first product for both terminal and desktop use. Runtime boot,
maintenance, proof, and admin actions still matter, but they should support the main job:
starting, resuming, steering, and reviewing AI work sessions.

## Bottom Line

Mister Smith should not feel like an admin tool with a few chat commands added on top.

It should feel like:

- a session-first interactive CLI by default
- a desktop app that uses the same session engine and state
- a small set of maintenance commands off to the side

The main product entry should be `mister-smith` opening a session shell, not `mister-smith run`
starting infrastructure.

## Research Summary

I checked:

- local `codex`, `claude`, and `gemini` help output
- OpenAI Codex docs and app documentation
- Gemini CLI documentation
- your screenshots of Gemini, Codex, and Claude Code

### Shared pattern across the strongest agent CLIs

The best agent CLIs all do these things:

1. open an interactive session by default
2. make resume and continue first-class actions
3. keep config, permissions, model changes, and MCP management available inside the session
4. show status inline instead of forcing users into separate admin commands
5. keep doctor, auth, update, and server functions available, but not in the center of the
   product

### Codex signals

- `codex` launches the terminal UI by default
- `codex resume` and `codex fork` are first-class
- slash commands control the live session:
  - `/model`
  - `/permissions`
  - `/plan`
  - `/status`
  - `/mcp`
  - `/new`
  - `/resume`
  - `/fork`
  - `/debug-config`
  - `/statusline`
- Codex CLI and Codex app share config, and the app picks up session history from the CLI

### Claude Code signals

- `claude` starts an interactive session by default
- continue and resume are top-level behaviors:
  - `--continue`
  - `--resume`
  - `--fork-session`
- `doctor`, `mcp`, `auth`, and plugin actions exist, but they sit beside the session flow instead
  of defining it

### Gemini CLI signals

- `gemini` starts an interactive REPL by default
- resume is first-class:
  - `--resume`
  - `--list-sessions`
  - `--delete-session`
- settings are editable in-session through `/settings`
- the docs explicitly support query-now-and-stay-interactive behavior

### What your screenshots reinforce

The screenshots point to the same product pattern:

- a welcome or startup card
- recent activity or recent sessions visible early
- warnings shown inline at startup
- a big central composer
- a persistent bottom status rail
- slash commands for settings and control
- a searchable in-session config dialog

That is the right direction for Mister Smith too.

## Product Positioning

Mister Smith should present one user shell with two front ends:

- terminal UI
- desktop GUI

Both should use the same:

- session store
- app protocol
- runtime client
- config model
- auth state
- MCP and capability registry

This means the CLI and GUI are not separate products. They are two ways to use the same session
system.

## Core Rule

Everything should be arranged around sessions.

That means:

- starting work
- resuming work
- switching work
- viewing recent work
- steering a live run
- checking model, permissions, MCP, and config without leaving the session

Admin and runtime commands still exist, but they are support features.

## Proposed Top-Level Product Shape

```text
mister-smith [prompt]
mister-smith resume [session_id|--last] [prompt]
mister-smith sessions <subcommand>
mister-smith app
mister-smith auth <subcommand>
mister-smith mcp <subcommand>
mister-smith config <subcommand>
mister-smith doctor
mister-smith runtime <subcommand>
mister-smith proof <subcommand>
```

## Proposed Meaning Of Each Entry

### 1. `mister-smith [prompt]`

This is the main entry.

Behavior:

- no args:
  - open the interactive session shell
- prompt supplied:
  - start a new interactive session with that prompt

Examples:

```text
mister-smith
mister-smith "explain this repo"
```

This should be the default experience.

### 2. `mister-smith resume`

This is the second most important entry.

Behavior:

- resume a known session by id
- resume the most recent session
- optionally append a new prompt when resuming

Examples:

```text
mister-smith resume --last
mister-smith resume 7f0d... "continue and fix the failing test"
```

### 3. `mister-smith sessions`

This is the session manager.

Suggested subcommands:

```text
mister-smith sessions list
mister-smith sessions open <session_id>
mister-smith sessions delete <session_id>
mister-smith sessions export <session_id>
```

Suggested fields in list view:

- title
- session id
- workspace
- model
- provider
- status
- last updated
- current branch or worktree when relevant

### 4. `mister-smith app`

Launch the desktop app.

The app should use the same session store and same app protocol as the CLI.

This is the right place for:

- recent sessions
- richer transcript browsing
- config dialogs
- run-detail views
- child runtime visibility later

### 5. Support commands

These still matter, but they are not the center:

```text
mister-smith auth ...
mister-smith mcp ...
mister-smith config ...
mister-smith doctor
mister-smith runtime ...
mister-smith proof ...
```

## Session Shell Design

The interactive shell should be the product heart.

### Startup screen

On launch, show:

- product name and version
- current account or auth state
- selected model or profile
- important warnings:
  - auth broken
  - MCP failed
  - runtime unavailable
  - low quota
- recent sessions
- quick actions:
  - new session
  - resume last
  - open sessions
  - open config

### Main layout

The session shell should have:

- a large central composer
- transcript above it
- a bottom status rail
- a slash-command menu
- inline warning area near the top

### Bottom status rail

Show a compact live footer like the best current tools do.

Suggested fields:

- workspace
- model
- provider
- approval mode
- sandbox mode
- MCP count and health
- context usage
- session id

### Session home view

Before the first message, show:

- recent sessions
- pinned sessions
- recent workspaces
- quick start prompts

The user should not have to remember commands to continue work.

## In-Session Slash Commands

These should be available inside the shell and later mirrored in the GUI command palette.

### Must-have slash commands

```text
/new
/resume
/sessions
/model
/provider
/permissions
/plan
/status
/config
/mcp
/doctor
/theme
/help
/quit
```

### Recommended later slash commands

```text
/fork
/share
/export
/runtime
/proof
```

## Suggested Config Experience

Do not force users to leave the session to manage basic settings.

Mister Smith should support:

- `/config` to open an in-session config view
- searchable settings
- tabs or sections for:
  - status
  - config
  - usage
  - stats

This matches the best current terminal-agent UX and the screenshot direction you shared.

## Suggested Command Priorities

### Tier 1: main product flow

- `mister-smith`
- `mister-smith resume`
- `mister-smith sessions ...`
- `mister-smith app`

### Tier 2: session steering

- slash commands for model, permissions, config, status, plan, and MCP

### Tier 3: support and maintenance

- `auth`
- `mcp`
- `doctor`
- `runtime`
- `proof`
- `config`

## Runtime And Maintenance Commands

These should exist, but they should be demoted from the product center.

Suggested shape:

```text
mister-smith runtime up
mister-smith runtime status
mister-smith runtime logs
mister-smith runtime down
```

This is clearer than making `run` or `serve` the primary identity of the tool.

Compatibility can stay for now:

- `mister-smith run`
- `mister-smith serve`

But the product should teach people the session-first path.

## Shared CLI And GUI Architecture

Use one shared session system under both front ends.

Suggested layers:

```text
session engine
  -> session storage
  -> runtime client
  -> config + auth + MCP state
  -> app protocol
      -> terminal UI
      -> desktop GUI
```

Important rule:

- the GUI should not invent a second session model
- the CLI should not be a thin admin shim over raw runtime commands

## What This Means For Mister Smith Naming

The product nouns should be:

- session
- run
- model
- permissions
- config
- mcp
- runtime

Avoid internal terms as the main navigation language when they are harder for users:

- `autonomy` is fine in technical views, but `session` and `run` are better first-level terms
- `conversation` should become `session`

## Recommended Command Model

```text
main:
  mister-smith [prompt]
  mister-smith resume [session_id|--last] [prompt]
  mister-smith sessions list|open|delete|export
  mister-smith app

support:
  mister-smith auth ...
  mister-smith mcp ...
  mister-smith config ...
  mister-smith doctor
  mister-smith runtime up|status|logs|down
  mister-smith proof smoke
```

## Rollout Plan

### Phase 1: session-first shell

Ship first:

- interactive default `mister-smith`
- `resume`
- `sessions list|open`
- startup home with recent sessions
- bottom status rail
- slash commands:
  - `/new`
  - `/resume`
  - `/model`
  - `/permissions`
  - `/status`
  - `/config`
  - `/mcp`

### Phase 2: desktop pairing

Ship next:

- `mister-smith app`
- shared session store between CLI and GUI
- GUI recent sessions and config views

### Phase 3: support surfaces

Ship after that:

- `doctor`
- `runtime ...`
- `proof smoke`
- richer MCP management

## Clear Recommendation

Reframe Mister Smith from:

- "runtime control tool with some chat commands"

to:

- "session-first AI workbench with runtime support built in"

That is the product shape users now expect from the best agent CLIs, and it matches the reference
screenshots you gave.

## Research Inputs

- Codex CLI local help: `codex --help`, `codex resume --help`, `codex app-server --help`
- Claude Code local help: `claude --help`
- Gemini CLI local help: `gemini --help`
- OpenAI Codex CLI docs:
  - [CLI reference](https://developers.openai.com/codex/cli/reference/)
  - [Slash commands](https://developers.openai.com/codex/cli/slash-commands/)
  - [CLI features](https://developers.openai.com/codex/cli/features/)
- OpenAI Codex app overview:
  - [Introducing the Codex app](https://openai.com/index/introducing-the-codex-app/)
- Gemini CLI docs:
  - [CLI reference](https://geminicli.com/docs/cli/cli-reference/)
  - [Settings](https://geminicli.com/docs/cli/settings/)
