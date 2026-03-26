# Research Notes: Budget-Backed Runtime Routing Control Loop

## Current router truth

- `mister-smith-llm` already ships `RoutingPolicy::Cascade` plus budget-aware routing checkpoints
- router tests already prove budget reservation, reconciliation, escalation, and hard-cap behavior
  in isolation
- the runtime app still builds `ModelRouter::new(RoutingPolicy::RoundRobin)` and registers exactly
  one provider

## Current budget truth

- `BudgetEnforcer` and `BudgetStore` already define the reserve-before-send / reconcile-after-send
  contract
- `InMemoryBudgetStore` exists for tests and local deterministic coverage
- the runtime app does not yet provide a production budget-store implementation or wire one into
  router bootstrap

## Bounded conclusion

The next legitimate runtime gap is not another provider or UI slice. It is wiring the already
landed router/budget substrate into the runtime-backed task path through a bounded multi-provider
profile and one JetStream-backed budget store.
