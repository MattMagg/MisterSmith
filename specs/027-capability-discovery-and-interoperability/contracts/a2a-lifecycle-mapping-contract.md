# Contract: A2A Lifecycle Mapping

## Purpose

Define the first remote lifecycle bridge frozen by packet `027`.

## Version And Scope

- packet scope: `027-capability-discovery-and-interoperability`
- remote protocol baseline: A2A `v0.3.0`
- inherited upstream inputs:
  - packet `022` lifecycle and durable identifier language
  - packet `023` proof-boundary language
  - packet `024` security and discover-vs-execute boundary language
- this contract is scaffold-only until packet `027` is refreshed against the completed packet
  `022`, `023`, and `024` outputs

## Source-Native A2A Task States

Packet `027` freezes the following A2A `v0.3.0` source-native task states as the input set:

- `submitted`
- `working`
- `input-required`
- `completed`
- `canceled`
- `failed`
- `rejected`
- `auth-required`
- `unknown`

## Mapping Intent

The bridge maps one source-native A2A task state into:

- Mister Smith workflow status
- result or autonomy projection language
- operator-visible proof-boundary text

The bridge is explicit and loss-aware. It does not assume every A2A state has a perfect local
equivalent.

## Minimum Mapping Rules

| A2A state | Local posture | Notes |
| --------- | ------------- | ----- |
| `submitted` | non-terminal accepted workflow state | remote task exists but has not started active work yet |
| `working` | active workflow state | remote agent is actively progressing the task |
| `input-required` | blocked-on-input workflow state | requires explicit local display of waiting-for-input posture |
| `completed` | completed workflow state | terminal successful state |
| `canceled` | canceled workflow state | terminal user- or caller-canceled state |
| `failed` | failed workflow state | terminal failed state |
| `rejected` | rejected-before-start boundary state | preserve that the remote agent did not start task execution |
| `auth-required` | blocked-on-auth boundary state | preserve that more credentials are needed before remote work can continue |
| `unknown` | explicit unknown mapping | must not be silently coerced into a terminal or successful local state |

## Boundary Rules

- the lifecycle binding MUST preserve proof-boundary text so local views do not overclaim what the
  remote state proves
- the lifecycle binding MUST preserve discovery-versus-execute separation
- `taskId` and `contextId` are source-native A2A continuity identifiers and MUST be recorded as
  remote provenance, not silently renamed into granted local authority
- packet `016` continuity references MAY inform provenance wording, but packet `027` MUST NOT use
  them to claim broad remote lifecycle proof

## Unsupported Or Deferred Areas

- mapping for remote protocols other than A2A `v0.3.0`
- live proof that the A2A bridge is already part of the default runtime path
- broad multi-protocol cancellation or restart choreography
