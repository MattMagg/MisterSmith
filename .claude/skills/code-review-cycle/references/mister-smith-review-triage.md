# Mister Smith — Review Triage & Consolidation

You are triaging code review findings from multiple independent review versions run against the same review scope in the Mister Smith multi-agent orchestration framework. Your job is to verify each finding, deduplicate across versions, and produce a single prioritized task list.

<review_versions>
<!-- Paste all version results here, maintaining the Version 1 / Version 2 / Version 3 / Version 4 structure -->
</review_versions>

<original_prompt>
<!-- What was reviewed: commit range, PR, feature area — same scope given to the reviewers -->
</original_prompt>

<additional_context>
<!-- Optional: known trade-offs, intentional decisions, areas where findings may be expected/acceptable -->
</additional_context>

---

## Phase 1: Inventory

Read every finding across all versions. For each finding, record:
- Version and issue number (e.g., 1.3 = Version 1, issue 3)
- The claimed severity and category
- The specific file:line cited
- A one-line summary of the claim

Do not evaluate yet — just catalog.

---

## Phase 2: Verify

For each finding, read the actual source code at the cited location and determine:

### Legitimacy
- **Is the code actually there?** Does the file:line reference match what the reviewer claims? Reviewers hallucinate line numbers and code quotes — verify.
- **Is the analysis correct?** Does the code actually behave the way the reviewer says it does? Trace the logic yourself.
- **Is the severity accurate?** A reviewer may call something "critical" that is actually minor, or miss that a "minor" issue is actually a correctness bug.

### Intentionality
- **Was this deliberate?** Check `<additional_context>` for known trade-offs. Check git commit messages, code comments, and spec documents for design decisions that explain the behavior.
- **Is the spec actually violated?** Read the governing spec yourself — don't trust the reviewer's characterization of what the spec says.
- **Is this deferred work, not missing work?** Some gaps are intentional scope boundaries, not bugs.

Mark each finding as:
- **Valid** — legitimate issue, should be fixed
- **Valid but intentional** — real gap but documented/deliberate trade-off; note why
- **Invalid** — wrong analysis, hallucinated code, or mischaracterized behavior
- **Downgrade/Upgrade** — correct finding but wrong severity; note the corrected severity

---

## Phase 3: Deduplicate

Group findings that describe the same underlying issue across versions. For each duplicate cluster:

- List all version.issue references (e.g., 1.2, 3.5, 4.1)
- Identify which version's report is strongest based on:
  - **Accuracy**: Does it cite the correct lines and quote actual code?
  - **Completeness**: Does it explain the full impact, not just the symptom?
  - **Actionability**: Does the suggested fix actually address the root cause?
  - **Context**: Does it reference the spec or design decision that's violated?
- Select the best version for each duplicate cluster

Also identify **gaps** — issues found by only one version that other versions missed. These are often the most interesting findings (either highly insightful or false positives — verify carefully).

---

## Phase 4: Prioritize

Produce the final ordered task list. Order by:

1. **Critical bugs** — correctness or safety issues
2. **Spec violations** — code doesn't match what the spec says it should do
3. **Missing implementation** — spec requirements not implemented
4. **Major issues** — significant but not correctness-breaking
5. **Minor issues and nits** — lowest priority

Within each priority band, order by blast radius (issues affecting more code paths or crates first).

---

## Output

Produce a single ordered task list. Each entry:

```
[priority]. [V.I] — [one-line description]
  Severity: [critical/major/minor/nit] (corrected if reviewer was wrong)
  Category: [bug/spec-violation/missing/optimization/convention]
  Location: [file:line]
  Also found in: [other V.I references, or "unique"]
  Why this version: [brief justification if duplicated — e.g., "V1 cites exact lines and root cause; V3 only describes symptom"]
  Notes: [any context on intentionality, trade-offs, or verification findings]
```

After the task list, include:

- **Rejected findings**: Issues marked invalid with brief explanation of why
- **Intentional gaps**: Issues marked valid-but-intentional with the justifying decision/trade-off
- **Cross-version observations**: Patterns in what versions found vs missed — useful for calibrating future review runs
