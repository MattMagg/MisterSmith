# MS-38 Vet Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a working GitHub Actions vet workflow and repo-local vet environment integration so Codex sessions can run `vet` consistently without manual session-file discovery.

**Architecture:** Keep the change narrow and workflow-focused. Add a dedicated `.github/workflows/vet.yml` that uses the upstream `imbue-ai/vet` action for pull requests, add a repo-local `.vet/configs.toml` for stable CI defaults, and add a small shell wrapper that resolves the current Codex session file from `CODEX_THREAD_ID` before invoking the existing project-level export script.

**Tech Stack:** GitHub Actions YAML, TOML config, Bash, Markdown documentation, `gh` CLI for remote workflow verification, `vet` CLI for local integration proof.

---

### Task 1: Capture the missing-workflow regression and align the CI design

**Files:**
- Modify: `.github/workflows/README.md`
- Create: `.github/workflows/vet.yml`
- Reference: `https://raw.githubusercontent.com/imbue-ai/vet/main/README.md`
- Reference: `https://raw.githubusercontent.com/imbue-ai/vet/main/action.yml`

**Step 1: Record the observed regression**

Use the existing repo and GitHub workflow surfaces to confirm that no active vet workflow currently exists.

**Step 2: Add the dedicated PR workflow**

Create `.github/workflows/vet.yml` with:

- `pull_request` triggers for opened, edited, synchronize, reopened, and ready-for-review
- `contents: read` and `pull-requests: write` permissions
- `if: github.event.pull_request.draft == false`
- `actions/checkout@v6` with the PR head SHA and `fetch-depth: 0`
- `imbue-ai/vet@v0.2.7`
- CI config selection via `.vet/configs.toml`

**Step 3: Update the workflow inventory**

Document the new workflow in `.github/workflows/README.md`, including the existing `ANTHROPIC_API_KEY` dependency and the fact that vet now participates in pull-request review automation.

### Task 2: Add repo-local Codex environment integration

**Files:**
- Create: `.vet/configs.toml`
- Create: `scripts/run-vet.sh`
- Modify: `README.md`

**Step 1: Add stable vet defaults**

Create `.vet/configs.toml` with a `ci` profile that matches the upstream example closely enough for predictable PR runs and a repo-specific `codex` profile that is suitable for local review from this workspace.

**Step 2: Add a local wrapper**

Create `scripts/run-vet.sh` that:

- resolves the repo root
- prefers the project-level export script at `.codex/skills/vet/scripts/export_codex_session.py`
- finds the current Codex session file by preferring an exact
  `session_meta.payload.id == CODEX_THREAD_ID` match only when the recorded
  `cwd` is also this repository, otherwise falling back to the newest session
  whose recorded `cwd` is this repository
- falls back cleanly when `CODEX_THREAD_ID` is missing
- forwards user arguments to `vet`

**Step 3: Document the local path**

Update `README.md` with a short “Vet” section that explains:

- the PR workflow now runs automatically
- local Codex sessions should use `scripts/run-vet.sh "goal"`
- `ANTHROPIC_API_KEY` or another supported vet model configuration is still required for non-agentic local runs

### Task 3: Validate the workflow and environment integration

**Files:**
- Check: `.github/workflows/vet.yml`
- Check: `.vet/configs.toml`
- Check: `scripts/run-vet.sh`

**Step 1: Verify the wrapper path**

Run:

```bash
bash -n scripts/run-vet.sh
./scripts/run-vet.sh --help
```

Expected: shell syntax is clean and the wrapper forwards to `vet`.

**Step 2: Verify the GitHub workflow surface**

Run:

```bash
gh workflow list --repo MattMagg/Mister-Smith
```

Expected after push: a `Vet` workflow appears as active.

**Step 3: Run vet on the diff**

Run `vet` via `scripts/run-vet.sh` after each logical edit batch and once more after the final diff.

**Step 4: Run final scope validation**

Run:

```bash
cargo build --workspace
```

Expected: workspace build still succeeds after the workflow/doc/script changes.
