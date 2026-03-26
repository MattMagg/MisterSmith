# Tasks: Budget-Backed Runtime Routing Control Loop

## T1. Freeze packet and state-doc routing

- [x] Add packet `019` artifacts
- [x] Add repo planning note in `docs/plans/`
- [x] Update current router docs to point at this packet as the next bounded phase

## T2. Add typed runtime routing profile

- [x] Extend framework config with runtime routing profile types
- [x] Add validation for shipped-provider tiers and fallback semantics
- [x] Add config tests for parsing and defaults

## T3. Activate bounded multi-provider runtime bootstrap

- [x] Build more than one runtime provider when the routing profile is configured
- [x] Preserve today's single-provider boot path when the profile is absent
- [x] Add targeted app tests for runtime router registration

## T4. Wire budget store and runtime control-loop behavior

- [x] Add one JetStream-backed `BudgetStore` implementation
- [x] Wire `BudgetEnforcer` into runtime router bootstrap
- [ ] Exercise cascade or downgrade behavior on the runtime task path

## T5. Extend routing evidence and validation boundaries

- [x] Surface routing policy, accepted tier, and budget checkpoints on task/autonomy outputs
- [ ] Update proof guidance or harness only if the path can be exercised honestly
- [x] Refresh state-bearing docs with explicit deterministic vs live-proof boundaries
