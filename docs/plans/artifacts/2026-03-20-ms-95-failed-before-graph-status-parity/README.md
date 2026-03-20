# MS-95 Failed-Before-Graph Status Parity Artifacts

Date: March 20, 2026
Status: Captured during the `MS-95` fix validation pass

## Purpose

This directory holds the raw deterministic and live runtime evidence used to close the
`failed_before_graph` autonomy-status parity gap described in
`docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`.

## Contents

- `runtime.log`: runtime output from the isolated MS-95 live validation run
- `session-start-request.json`: session-start request submitted to the live runtime
- `session-start-response.json`: accepted session/workflow identifiers from that request
- `session-id.txt`: stable session identifier used for the captured proof
- `workflow-id.txt`: workflow identifier used for the captured proof
- `task-status.json`: task-surface view for the captured `failed_before_graph` workflow
- `session-inspect.json`: retained session view for the same workflow
- `autonomy-status.json`: post-fix autonomy status document for the same workflow

## Notes

- The live failure exercised a planner timeout before graph publication, which still lands in the
  bounded `failed_before_graph` taxonomy.
- Use the March 20 packet-015 evaluation note as the durable narrative source of truth.
