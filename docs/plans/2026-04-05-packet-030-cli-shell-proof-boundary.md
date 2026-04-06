# Packet 030 CLI Shell Proof Boundary

Status: implemented locally on `codex/030-session-first-cli-shell` on April 5, 2026

## Scope

This note records the deterministic implementation evidence for packet
`030-session-first-cli-shell`.

The landed scope is limited to the CLI shell path:

- open the CLI shell
- start a new session
- resume the last session
- resume a specific session
- browse recent sessions
- steer a live session in place with session-shell controls

This note does not claim GUI parity, cross-surface continuity, or live runtime proof beyond the
commands listed under validation.

## Implemented Surface

- `mister-smith` with no args now renders a recent-first startup home instead of defaulting to the
  runtime-first path
- `mister-smith <prompt>` starts a new session directly when the first non-global token is not a
  top-level subcommand (e.g., `run`, `resume`, `sessions`, `conversation`, `autonomy`, `auth`) or
  option flag; the parser's `split_prompt_words()` determines when direct-prompt mode activates;
  use `--` to force direct-prompt mode (e.g., `mister-smith -- run my workflow` treats "run" as
  part of the prompt instead of the subcommand)
- `mister-smith resume --last` and `mister-smith resume <session_id>` reopen retained sessions
- `mister-smith sessions list` and `mister-smith sessions open <session_id>` expose recent-session
  browse and reopen flows
- the CLI session view now surfaces session title, control state, and support notices
- the HTTP session payloads now expose CLI shell title/control/notice fields plus a control update
  route
- live session slash controls now cover `model`, `permissions`, `config`, `status`, `mcp`,
  `resume`, `new`, and `sessions`

## Deterministic Validation

The following commands were run successfully in the repository root:

- `cargo test -p mister-smith-app`
- `cargo test -p mister-smith-http`
- `cargo build --workspace`
- `SPECIFY_FEATURE=030-session-first-cli-shell ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
- `npx markdownlint-cli2 "specs/030-session-first-cli-shell/**/*.md" --config .markdownlint.json`
- `git diff --check`

Focused packet-owned CLI coverage now includes:

- default no-arg home rendering on non-tty
- direct prompt entry against a mock session server
- resume-last, resume-by-session-id, sessions list, and sessions open flows against a mock session
  server
- session-control HTTP helper coverage and shell render coverage

## Proof Boundary

This packet proof is deterministic and repo-local only.

It does not claim:

- a live runtime proof against a real long-running runtime
- provider-backed end-to-end proof for new session turns after shell-control changes
- any GUI or operator-console validation
- clean git closure or upstream sync proof

## Remaining Closure Item

`scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` was run and failed
because the task-owned diff is still uncommitted and the branch has no upstream yet.