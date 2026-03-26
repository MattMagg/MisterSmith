# Tasks: Budget-Backed Runtime Routing Control Loop

## T1. Freeze packet and state-doc routing

- [ ] Add packet `019` artifacts
- [ ] Add repo planning note in `docs/plans/`
- [ ] Update current router docs to point at this packet as the next bounded phase

## T2. Add typed runtime routing profile

- [ ] Extend framework config with runtime routing profile types
- [ ] Add validation for shipped-provider tiers and fallback semantics
- [ ] Add config tests for parsing and defaults

## T3. Activate bounded multi-provider runtime bootstrap

- [ ] Build more than one runtime provider when the routing profile is configured
- [ ] Preserve today's single-provider boot path when the profile is absent
- [ ] Add targeted app tests for runtime router registration

## T4. Wire budget store and runtime control-loop behavior

- [ ] Add one JetStream-backed `BudgetStore` implementation
- [ ] Wire `BudgetEnforcer` into runtime router bootstrap
- [ ] Exercise cascade or downgrade behavior on the runtime task path

## T5. Extend routing evidence and validation boundaries

- [ ] Surface routing policy, accepted tier, and budget checkpoints on task/autonomy outputs
- [ ] Update proof guidance or harness only if the path can be exercised honestly
- [ ] Refresh state-bearing docs with explicit deterministic vs live-proof boundaries
