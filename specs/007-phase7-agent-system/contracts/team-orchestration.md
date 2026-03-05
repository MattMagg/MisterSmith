# Contract: Team Orchestration

## Overview

Defines how Coordinators create teams, decompose tasks, assign subtasks, monitor progress, handle failures, and aggregate results.

## Team Creation

```
Coordinator::assemble_team(task, pattern, member_types) -> Result<Team, OrchestrationError>
```

### Flow

1. **Query registry** for available agents matching `member_types` and required capabilities
2. **Spawn or allocate** agents (spawn new if pool insufficient, allocate existing if available and idle)
3. **Create supervision subtree** under the Coordinator's supervisor with configured restart strategy
4. **Register team** in the team tracker with task binding
5. **Return** Team handle with member refs and supervisor ref

### Team Patterns

| Pattern | Assignment | Result Aggregation | Failure Mode |
|---------|------------|-------------------|--------------|
| SupervisorWorker | Fan-out: each subtask to one worker | Fan-in: collect all results, combine in dependency order | Supervisor restarts failed worker, Coordinator reassigns subtask |
| Pipeline | Sequential: output of step N is input to step N+1 | Last step's output is the final result | Supervisor restarts failed step, pipeline resumes from that step |
| Consensus | Parallel: same input to all members | Voting: majority result wins (configurable threshold) | Member restart, re-evaluate if quorum still met |

## Task Decomposition

```
Coordinator::decompose(task) -> Result<Vec<SubTask>, OrchestrationError>
```

- Decomposition logic is provided by a pluggable `TaskDecomposer` trait
- Returns a dependency graph of subtasks (Vec with parent_task_id links)
- Each subtask has: task_type, input, dependencies (list of subtask IDs that must complete first)
- Coordinator validates: no cycles in dependency graph, all dependencies reference known subtasks

## Subtask Assignment

```
Coordinator::assign_subtasks(team, subtasks) -> Result<(), OrchestrationError>
```

1. For each subtask with satisfied dependencies (all deps Completed):
   - Select an idle team member matching the subtask type
   - Publish task assignment via DurableTransport to member's command subject
   - Track assignment in TaskAssignment state (Pending → Assigned)
2. As subtasks complete, check if new subtasks have their deps satisfied → assign them
3. Continue until all subtasks assigned or a failure requires intervention

## Progress Monitoring

- Coordinator subscribes to `tasks.{id}.progress` for each subtask
- Progress events carry: subtask_id, percent_complete, status update
- Coordinator tracks overall task progress as weighted average of subtask progress
- Deadline monitoring: Coordinator checks `deadline` on each progress tick

## Failure Handling

### Worker Failure

1. Supervisor detects failure, applies restart strategy
2. Supervisor notifies Coordinator via `CoordinatorMessage::TeamMemberFailed(agent_id)`
3. Coordinator checks: was the failed agent assigned an incomplete subtask?
4. If yes: mark subtask as Pending, reassign to same (restarted) or different member
5. Already-completed subtasks are NOT re-executed (idempotent by message_id)

### Deadline Exceeded

1. Coordinator detects deadline for a subtask exceeded
2. Configurable action:
   - **Retry**: Mark subtask as Pending, reassign (up to max_retries)
   - **Reassign**: Assign to a different team member
   - **Fail**: Mark subtask as TimedOut, propagate failure
   - **PartialResult**: Continue aggregation with available results, mark final result as partial

### Coordinator Failure

1. Coordinator's supervisor restarts the Coordinator
2. On `pre_start`, Coordinator loads persisted team and task state from persistence layer
3. Coordinator resumes monitoring from the last known subtask states
4. In-flight assignments are safe (workers will complete or timeout independently)

## Result Aggregation

```
Coordinator::aggregate(task, subtask_results) -> Result<Value, OrchestrationError>
```

- Aggregation logic is provided by a pluggable `ResultAggregator` trait
- Default aggregator: collect results into a JSON array ordered by dependency completion
- Custom aggregators can merge, filter, or transform results
- Final result published to `tasks.{task_id}.result`

## Team Disbanding

```
Coordinator::disband(team) -> Result<(), OrchestrationError>
```

1. Cancel any in-flight subtasks (publish cancel to assigned agents)
2. Stop all team member agents (graceful shutdown)
3. Remove supervision subtree
4. Deregister team from tracker
5. Mark team as disbanded with timestamp

## Concurrency Guarantees

- Subtask assignment is serialized within the Coordinator's message processing loop (no concurrent assignments to same subtask)
- Multiple Coordinators can run simultaneously with different teams (no shared mutable state between teams)
- Worker agent assignment is protected by durable message acknowledgment — a worker only processes a task after acking the assignment message
