# Packet 015 Live Runtime Evaluation Artifacts

Date: March 20, 2026
Status: Captured during packet-015 closure

## Purpose

This directory holds the raw host/runtime captures produced during the final packet-015 live
evaluation pass.

These files support the packet-015 closure note in:

- `docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`

## Contents

- `docker-ps.txt`: local container inventory at capture time
- `git-status.txt`: repo state captured during the live proof pass
- `health-live.json`: live health probe output
- `health-ready.json`: ready health probe output
- `nats-log-tail.txt`: tail of the NATS log used during the evaluation
- `openai-chatgpt-auth-status.txt`: provider-auth capture used for the run
- `postgres-health.txt`: PostgreSQL health capture
- `runtime.log`: runtime output from the live proof pass
- `session-collapse-request.json`: collapse-case request payload
- `session-success-request.json`: success-case request payload
- `task-failure-request.json`: failure-visible request payload

## Notes

- These are raw supporting artifacts, not the narrative source of truth.
- Use the packet-015 evaluation note for the durable summary, validation boundary, and final
  interpretation.
