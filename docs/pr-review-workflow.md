# Pull Request Review Workflow

Use this workflow to perform consistent and high-quality PR reviews in Mister Smith.

## Workflow Variables

Set these inputs before starting:

| Variable | Required | Description |
| --- | --- | --- |
| `PR_NUMBER` | Yes* | PR number to review (for example `42`) |
| `BRANCH_NAME` | Yes* | Branch name if no PR exists (for example `feature/new-auth`) |
| `REVIEW_DEPTH` | No | `quick` (15 min) or `comprehensive` (45+ min). Default: `comprehensive` |
| `FOCUS_AREAS` | No | Comma-separated categories to prioritize (for example `security,performance`) |

\*One of `PR_NUMBER` or `BRANCH_NAME` is required.

## Phase 1: Context Gathering

### 1.1 Fetch PR Metadata (if PR exists)

```bash
# Get PR details
gh pr view $PR_NUMBER --json title,body,author,baseRefName,headRefName,additions,deletions,changedFiles,labels,reviewDecision

# Get PR diff
gh pr diff $PR_NUMBER

# List PR files changed
gh pr view $PR_NUMBER --json files --jq '.files[].path'

# Get existing review comments
gh pr view $PR_NUMBER --json reviews,comments
```

### 1.2 Fetch Commit History

```bash
# Get commits in PR
gh pr view $PR_NUMBER --json commits --jq '.commits[] | "\(.oid[:7]) \(.messageHeadline)"'

# Or for branch comparison
git log main..$BRANCH_NAME --oneline

# Get detailed commit info
git log main..$BRANCH_NAME --stat
```

### 1.3 Understand the Change

- [ ] Read PR title, description, and linked issues.
- [ ] Identify the purpose of the change (feature, bugfix, refactor, docs).
- [ ] Note special instructions from the author.
- [ ] Check whether the PR is draft or ready for review.

## Phase 2: Commit Analysis

### 2.1 Commit Message Quality

For each commit, verify:

- [ ] Format: subject line <=72 chars, imperative mood (`Add`, not `Added`).
- [ ] Clarity: message explains what and why, not only how.
- [ ] Scope: commit is atomic (single logical change).
- [ ] References: issue or ticket number included where applicable.

```bash
# View full commit messages
git log main..$BRANCH_NAME --format=fuller
```

### 2.2 Commit Structure

- [ ] Atomic commits: each commit compiles and passes tests independently.
- [ ] Logical ordering: commits build on each other sensibly.
- [ ] No fixup commits: squash or amend pre-review.
- [ ] No merge commits: prefer rebase for clean history (if project standard).

## Phase 3: Code Review Checklist

### 3.1 Code Quality

- [ ] Logic correctness: code does what it claims.
- [ ] Edge cases: boundary conditions are handled.
- [ ] Error handling: errors are caught, logged, and handled gracefully.
- [ ] DRY principle: no unnecessary duplication.
- [ ] Readability: another developer can understand this in 6 months.
- [ ] Naming: variables, functions, and classes are clear.
- [ ] Complexity: functions are reasonably short and complexity remains low.

```bash
# Search for TODO/FIXME/HACK markers
git diff main..$BRANCH_NAME | grep -E "(TODO|FIXME|HACK|XXX)"
```

### 3.2 Security

- [ ] Input validation: external inputs are validated/sanitized.
- [ ] SQL injection: queries are parameterized.
- [ ] XSS prevention: user content is escaped in templates.
- [ ] Secrets exposure: no hardcoded API keys, passwords, or tokens.
- [ ] Dependency vulnerabilities: newly introduced deps are audited.
- [ ] Authentication/Authorization: access controls are correctly applied.
- [ ] Sensitive data: no inappropriate PII/PHI exposure in logs or responses.

```bash
# Check for potential secrets
git diff main..$BRANCH_NAME | grep -iE "(password|secret|api_key|token|private_key)"

# Audit new dependencies
npm audit  # or yarn audit, pip-audit, etc.
```

### 3.3 Testing

- [ ] Coverage: new code paths are tested.
- [ ] Edge cases: tests cover boundary conditions.
- [ ] Negative cases: failure scenarios are tested.
- [ ] Test quality: tests are clear, maintainable, and not brittle.
- [ ] Test isolation: no shared mutable state between tests.
- [ ] Mocking: external dependencies are mocked appropriately.

```bash
# Run tests
npm test  # or project-specific command

# Check coverage impact
npm run test -- --coverage
```

### 3.4 Documentation

- [ ] Code comments: complex sections are explained.
- [ ] API docs/docstrings: public APIs are documented.
- [ ] README updates: required updates are included.
- [ ] API documentation: new endpoints are documented.
- [ ] Changelog: change is noted when required.
- [ ] Migration guides: breaking changes are documented.

### 3.5 Style & Conventions

- [ ] Linting passes.
- [ ] Formatting follows project standards.
- [ ] Existing codebase patterns are followed.
- [ ] Files are in correct locations.
- [ ] Imports are ordered consistently.
- [ ] Type safety is maintained (no unnecessary `any`).

```bash
# Run linters
npm run lint

# Type check
npx tsc --noEmit
```

### 3.6 Performance

- [ ] N+1 queries are avoided.
- [ ] Resources are cleaned up (no leaks).
- [ ] Bundle size impact is acceptable.
- [ ] Cacheable operations are cached.
- [ ] Heavy resources are loaded on demand.
- [ ] Algorithmic complexity is appropriate for data sizes.

```bash
# Check bundle size impact (if applicable)
npm run build && du -sh dist/
```

## Phase 4: Contextual Verification

### 4.1 Cross-File Impact

```bash
# Find usages of modified functions
# Use rg or other codebase search tooling

# Check for breaking interface changes
git diff main..$BRANCH_NAME -- "*.ts" | grep -E "^[\+\-].*export (function|interface|type|class)"
```

- [ ] Shared function changes do not break callers.
- [ ] Interface changes are reflected in all implementations.
- [ ] Type changes are propagated correctly.

### 4.2 Integration Points

- [ ] Database migrations are reversible and tested.
- [ ] API contract changes are backward compatible.
- [ ] New env vars/configuration are documented.
- [ ] New functionality uses feature flags where needed.

## Phase 5: Review Depth Adjustments

### Quick Review (15 min)

Focus on:

1. Security issues (Phase 3.2).
2. Obvious logic bugs (Phase 3.1, logic correctness only).
3. Test existence (not full quality review).
4. Commit message format.

### Comprehensive Review (45+ min)

Complete all phases, plus:

- Run the code locally if possible.
- Test manual scenarios.
- Review performance implications.
- Check architectural concerns.

### Focused Review (specific areas)

If `FOCUS_AREAS` is set, prioritize those sections and do a cursory review of the rest.

## Phase 6: Compile Review Summary

### 6.1 Structure Findings

Use this template:

```markdown
## PR Review: #$PR_NUMBER - $TITLE

### Summary
[1-2 sentence overview of the change and its purpose]

### Recommendation
**[APPROVE | REQUEST_CHANGES | COMMENT]**

### Key Findings

#### 🔴 Blocking Issues (must fix)
- [Issue description with file:line reference]

#### 🟡 Suggestions (should consider)
- [Suggestion with rationale]

#### 🟢 Nitpicks (optional)
- [Minor style/preference notes]

### Tested
- [ ] Ran `npm run build`
- [ ] Ran `npm test`
- [ ] Manual verification: [describe what you tested]

### Checklist Summary
| Category | Status | Notes |
| --- | --- | --- |
| Code Quality | ✅/⚠️/❌ | |
| Security | ✅/⚠️/❌ | |
| Testing | ✅/⚠️/❌ | |
| Documentation | ✅/⚠️/❌ | |
| Style | ✅/⚠️/❌ | |
| Performance | ✅/⚠️/❌ | |
```

### 6.2 Decision Criteria

| Recommendation | When to Use |
| --- | --- |
| **APPROVE** | No blocking issues; suggestions are minor |
| **REQUEST_CHANGES** | Blocking issues exist (bugs, security, or missing critical tests) |
| **COMMENT** | Questions need answers before deciding, or feedback is non-blocking |

## Phase 7: Submit Review

### 7.1 Post Inline Comments

```bash
# Add line comment
gh pr review $PR_NUMBER --comment --body "Comment on specific line"

# Or use the GitHub web UI for inline comments
```

### 7.2 Submit Overall Review

```bash
# Approve
gh pr review $PR_NUMBER --approve --body "LGTM! [summary]"

# Request changes
gh pr review $PR_NUMBER --request-changes --body "[structured summary from Phase 6]"

# Comment only
gh pr review $PR_NUMBER --comment --body "[structured summary from Phase 6]"
```

## Quick Reference Commands

```bash
# View PR
gh pr view $PR_NUMBER

# View diff
gh pr diff $PR_NUMBER

# Checkout PR locally
gh pr checkout $PR_NUMBER

# List files changed
gh pr view $PR_NUMBER --json files --jq '.files[].path'

# View commits
gh pr view $PR_NUMBER --json commits

# Compare branches
git diff main..$BRANCH_NAME --stat
git log main..$BRANCH_NAME --oneline

# Run local verification
npm run lint && npm run build && npm test
```
