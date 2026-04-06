# Security Policy

## Supported Versions

Security fixes are applied to the latest `main` branch. Historical branches and
old pull request heads are not supported for coordinated vulnerability handling.

## Reporting a Vulnerability

Do not open a public GitHub issue or pull request for a suspected security
problem.

Use one of these private paths instead:

1. GitHub private vulnerability reporting or a security advisory, if that
   option is available to you.
2. Email `matthewtmaggio@gmail.com` with:
   - a concise summary of the issue
   - affected crates, files, endpoints, or deployment surfaces
   - reproduction steps or a proof of concept
   - impact assessment and any suggested mitigation

## Response Expectations

- Initial acknowledgement: best effort within 3 business days
- Follow-up status updates: best effort at least weekly until triage is clear

## Scope

This repository includes the Rust workspace, operator surfaces, deployment artifacts, and related
repo configuration. Reports that include exact file paths, commit SHAs, request traces, or failing
validation commands are much easier to triage quickly.
