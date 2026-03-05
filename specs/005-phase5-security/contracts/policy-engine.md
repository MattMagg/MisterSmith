# Contract: PolicyEngine

**Module**: `mister_smith_security::rbac`

## Public API

### PolicyEngine

```rust
pub struct PolicyEngine { /* private */ }

impl PolicyEngine {
    /// Create a new PolicyEngine with the given roles and policies.
    pub fn new(config: &RbacConfig) -> Self;

    /// Evaluate an authorization request against loaded policies.
    /// Returns allow/deny with reason.
    pub fn evaluate(&self, request: &AuthorizationRequest) -> PolicyDecision;

    /// Check a specific permission for a principal.
    /// Convenience wrapper around evaluate().
    pub fn check_permission(
        &self,
        claims: &AgentClaims,
        action: &str,
        resource: &str,
    ) -> bool;

    /// Add a role to the engine.
    pub fn add_role(&self, role: Role);

    /// Remove a role from the engine.
    pub fn remove_role(&self, role_name: &str) -> bool;

    /// Get all effective permissions for a set of role names,
    /// including inherited permissions from parent roles.
    pub fn effective_permissions(&self, roles: &[String]) -> Vec<Permission>;
}
```

### Permission Matching Rules

1. Exact match: `read:agent:own` matches `read:agent:own`
2. Wildcard action: `*:agent:own` matches any action on agent with own scope
3. Wildcard resource: `read:*:own` matches read on any resource with own scope
4. Wildcard scope: `read:agent:*` matches read on agent with any scope
5. Full wildcard: `*:*:*` matches everything (admin)
6. Deny-wins: Any explicit deny overrides all allows

### Error Handling

`evaluate()` never panics or returns errors — it returns a `PolicyDecision` with `allowed: false` and a descriptive `reason` for any edge case:
- No matching roles → default deny ("no matching policy")
- Role not found → default deny ("role not found: X")
- Constraint evaluation timeout → deny ("evaluation timeout")
- Constraint evaluation failure → deny ("constraint evaluation failed: X")

### Thread Safety

`PolicyEngine` is `Send + Sync`. Roles stored in `DashMap` for concurrent access.

### Test Contract

```rust
#[test] fn worker_can_read_own_tasks();
#[test] fn worker_cannot_delete_system_resources();
#[test] fn admin_has_full_access();
#[test] fn explicit_deny_overrides_allow();
#[test] fn default_deny_for_unmatched_requests();
#[test] fn role_hierarchy_inherits_permissions();
#[test] fn time_window_constraint_enforced();
#[test] fn wildcard_permissions_match();
#[test] fn permission_parsing_from_string();
```
