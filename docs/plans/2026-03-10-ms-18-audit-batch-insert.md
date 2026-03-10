# MS-18 Audit Batch Insert Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the audit batch insert path with a real single-statement batched insert while preserving atomic failure semantics and adding explicit validation for the multi-row path.

**Architecture:** Keep the `AuditRepository` API unchanged and constrain the behavior change to `mister-smith-persistence` by rewriting `insert_audit_batch()` to issue one PostgreSQL batch insert statement. Add targeted persistence tests that prove the batched path inserts multiple rows and rolls the whole batch back when one row conflicts.

**Tech Stack:** Rust, `sqlx 0.8`, PostgreSQL, existing ignored integration tests in `crates/mister-smith-persistence/tests`

---

### Task 1: Red Phase For Audit Batch Coverage

**Files:**
- Modify: `crates/mister-smith-persistence/tests/repository_tests.rs`

**Step 1: Write the failing test**

- Add an ignored integration test that inserts a multi-entry audit batch, queries the rows back by id, and expects every entry to be present.
- Add an ignored integration test that submits a batch containing a duplicate primary key and expects the whole batch to fail with zero persisted rows.

**Step 2: Run the targeted tests to verify RED**

Run:
- `cargo test -p mister-smith-persistence audit_append_batch -- --ignored`
- `cargo test -p mister-smith-persistence audit_append_batch_is_atomic_on_failure -- --ignored`

Expected:
- The multi-row coverage passes or remains neutral against current behavior.
- The atomic failure test fails if the current batch path partially commits rows, otherwise it becomes the guard for the refactor.

---

### Task 2: Implement Single-Statement Audit Batch Insert

**Files:**
- Modify: `crates/mister-smith-persistence/src/postgres/queries.rs`

**Step 1: Build the batch statement inputs**

- Collect column-wise vectors for every audit entry field in `insert_audit_batch()`.
- Keep empty-batch handling unchanged.

**Step 2: Replace the per-row loop**

- Execute one PostgreSQL `INSERT INTO audit_log ... SELECT * FROM UNNEST(...)` statement inside the existing transaction.
- Keep bind types explicit where `sqlx` needs help with `Vec<Option<T>>` and JSON/UUID/timestamp arrays.

**Step 3: Verify GREEN**

Run:
- `cargo test -p mister-smith-persistence`

Expected:
- Unit tests pass locally.
- Ignored DB-backed tests are ready for an environment with `DATABASE_URL`.

---

### Task 3: Scope Validation

**Files:**
- No new files

**Step 1: Run affected crate validation**

Run:
- `cargo test -p mister-smith-persistence`

**Step 2: Run cross-workspace compile validation**

Run:
- `cargo build --workspace`

**Step 3: Reconcile Linear workpad**

- Check off completed plan and acceptance items.
- Record exact commands run, plus whether ignored DB-backed tests were executable in-session.
