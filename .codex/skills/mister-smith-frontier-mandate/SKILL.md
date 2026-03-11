---
name: mister-smith-frontier-mandate
description: Use when a Mister Smith phase, issue, PR, or design decision needs a frontier-legitimacy judgment about supervised autonomy, scope drift, or security-versus-autonomy balance.
---

# Mister Smith Frontier Mandate

## Overview

This skill is the architectural legitimacy filter for Mister Smith. Start with the control-plane
judgment, then deepen the review with local docs when the answer is disputed or the work is novel.

## Use This When

- deciding whether work strengthens supervised autonomy or is drift
- judging security hardening against autonomy leverage
- reviewing frontier-autonomy issues, phases, or PRs
- deciding whether to advance, reshape, split, defer, or reject proposed work

## MCP-First Flow

1. `evaluate_issue_legitimacy` for concrete Linear issues
2. `classify_follow_up_work` for PR or follow-up posture
3. Read `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` and the relevant `spec.md` / `plan.md` / `tasks.md` when the judgment still needs human reasoning

## Judgment Standard

- advance work that strengthens supervised autonomy
- name the frontier axis that improves
- reject framework imitation, security monoculture, and arbitrary complexity
- split substrate hardening from autonomy-facing leverage when both are present but imbalanced
