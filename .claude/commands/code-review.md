# Code Review: [DESCRIPTION]

Review all source code, specs, and tests associated with the subject above. Read every file that was added or modified as part of this work to build a complete picture.

## Scope Discovery

1. Locate the spec directory under `specs/` and the corresponding crate(s) under `crates/` for the phase being reviewed
2. Read `specs/<feature>/tasks.md` and `specs/<feature>/plan.md` to understand what was supposed to be built
3. Read all source files in the relevant crate(s) — every `.rs` file, `Cargo.toml`, and test file
4. Read `CLAUDE.md` and `ROADMAP.md` for project conventions and phase context

## Review Dimensions

### Spec Compliance

- Every task in `tasks.md` marked complete — verify the implementation actually satisfies each one
- Compare `plan.md` architectural decisions against what was built — flag deviations
- Check that all acceptance criteria from the spec are met with corresponding tests
- Identify anything implemented that was NOT in the spec (scope creep)

### Correctness

- Trace data flow end-to-end through the new code paths
- Verify error variants are exhaustive and propagated correctly (no swallowed errors, no silent fallbacks)
- Check concurrency primitives: Are locks held across await points? Are there potential deadlocks? Is shared state properly synchronized?
- Validate unsafe code blocks if any exist — verify soundness invariants are documented and upheld
- Check for off-by-one errors, integer overflow potential, and boundary conditions
- Verify all `unwrap()` / `expect()` calls are justified (test code vs production code)

### Security

- Secrets: No hardcoded keys, tokens, or credentials in source (test fixtures with obviously-fake values are acceptable)
- Input validation at system boundaries — untrusted data is validated before use
- Authentication/authorization checks cannot be bypassed through alternate code paths
- Cryptographic choices are current and appropriate (no deprecated algorithms, sufficient key lengths)
- Error messages don't leak internal state or stack traces to external callers
- Dependencies: Check for known vulnerabilities in new dependencies (`cargo audit` or equivalent)

### API Design

- Public API surface is minimal — only expose what consumers need
- Types enforce invariants at compile time where possible (newtype wrappers, builder patterns, typestate)
- Breaking changes to existing public APIs are justified and documented
- Naming follows existing crate conventions (check 3+ examples of similar patterns in the codebase)
- `#[must_use]`, visibility modifiers, and documentation on all public items

### Error Handling

- Custom error types are specific enough to be actionable by callers
- Error chains preserve context (`source()` / `#[from]` / `.context()`)
- No `Box<dyn Error>` in library code unless justified
- Recovery paths exist where recovery is possible — errors don't just propagate to top-level panic
- Distinguish between "this should never happen" (panic) and "this can happen at runtime" (Result)

### Testing

- Unit tests cover core logic, edge cases, and error paths — not just happy paths
- Integration tests validate cross-crate interactions and full request/response flows
- Test helpers and fixtures are reusable and don't duplicate setup logic
- Tests are deterministic — no timing-dependent assertions without adequate margins
- Test names describe the behavior being verified, not the implementation

### Performance

- No unnecessary allocations in hot paths (cloning where borrowing suffices, String where &str works)
- Appropriate data structures for access patterns (HashMap vs BTreeMap, Vec vs VecDeque)
- Async code doesn't block the runtime (no blocking I/O, no long-running compute without `spawn_blocking`)
- Connection pools, caches, and buffers are bounded
- No O(n^2) or worse algorithms where O(n log n) or O(n) alternatives exist

### Dependency Hygiene

- New dependencies are justified — not added for trivial functionality that could be a few lines of code
- Feature flags are used to keep optional dependencies optional
- Workspace dependencies are used consistently (no version pinning in individual crates that conflicts with workspace)
- No duplicate dependencies at different versions (check `cargo tree -d`)

### Code Organization

- Module structure follows existing crate conventions
- No circular dependencies between modules or crates
- Public re-exports create a clean API surface (`pub use` in `lib.rs` / `mod.rs`)
- Dead code is removed, not commented out or `#[allow(dead_code)]`
- Files are appropriately sized — no 1000+ line modules without clear structure

### Documentation

- Module-level doc comments explain purpose and usage patterns
- Complex algorithms or non-obvious design decisions have inline comments explaining WHY
- Doc examples compile and demonstrate real usage (not just `// TODO`)
- CLAUDE.md and ROADMAP.md are updated to reflect the new state

## Output Format

Structure your review as:

**Summary**: 2-3 sentences on what was built and overall assessment.

**Strengths**: What was done well — be specific with file:line references.

**Issues**: Categorized as Critical (must fix before merge), Important (should fix soon), or Minor (nice to have). Each issue should include:
- File and line reference
- What the problem is
- Why it matters
- Suggested fix

**Spec Gaps**: Anything in the spec that wasn't implemented, or implemented differently than specified.

**Test Coverage Assessment**: Areas that lack test coverage or have weak assertions.

**Verdict**: Ship it / Ship with fixes / Needs rework.
