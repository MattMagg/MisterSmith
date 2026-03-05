//! Role-based access control (RBAC) with optional ABAC constraints.
//!
//! The [`PolicyEngine`] is the central authorization component. It stores
//! [`Role`]s in a concurrent [`DashMap`] and evaluates [`AuthorizationRequest`]s
//! against the loaded policy set, producing a [`PolicyDecision`].
//!
//! # Deny-wins semantics
//!
//! When a principal holds multiple roles, all effective permissions are
//! collected.  If **any** matching permission has `deny: true`, the request is
//! denied regardless of how many allow permissions also match.
//!
//! # Default roles
//!
//! [`PolicyEngine::new`] pre-populates four built-in roles:
//!
//! | Role | Permissions |
//! |------|-------------|
//! | `admin` | `*:*:*` |
//! | `developer` | `read:*:all`, `write:*:own`, `delete:*:own` |
//! | `operator` | `read:*:all`, `write:system:all` |
//! | `viewer` | `read:*:all` |

pub mod constraints;
pub mod permission;

pub use constraints::{PolicyConstraints, TimeWindow};
pub use permission::{Permission, Role};

use crate::config::RbacConfig;
use crate::jwt::AgentClaims;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// AuthorizationRequest
// ---------------------------------------------------------------------------

/// Input to [`PolicyEngine::evaluate`].
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// The authenticated principal making the request.
    pub principal: AgentClaims,
    /// Action being performed (e.g. `read`, `write`, `delete`).
    pub action: String,
    /// Target resource type (e.g. `agent`, `task`, `system`).
    pub resource: String,
    /// Optional specific resource instance identifier.
    pub resource_id: Option<String>,
    /// Additional context for ABAC constraint evaluation.
    ///
    /// Common keys: `hour`, `day`, `ip`, `is_owner`.
    pub context: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// PolicyDecision
// ---------------------------------------------------------------------------

/// Result of a policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Human-readable explanation of the decision.
    pub reason: String,
    /// The permission or role that produced this decision (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matching_policy: Option<String>,
}

// ---------------------------------------------------------------------------
// PolicyEngine
// ---------------------------------------------------------------------------

/// Concurrent RBAC policy engine.
///
/// Thread-safe (`Send + Sync`) — roles are stored in a [`DashMap`] for
/// lock-free concurrent reads and writes.
pub struct PolicyEngine {
    /// Role storage keyed by role name.
    roles: DashMap<String, Role>,
    /// Optional default role assigned to principals with no explicit roles.
    default_role: Option<String>,
}

impl PolicyEngine {
    /// Create a new `PolicyEngine` with default roles and the given
    /// configuration.
    ///
    /// Pre-populates the four built-in roles (`admin`, `developer`,
    /// `operator`, `viewer`).
    pub fn new(config: &RbacConfig) -> Self {
        let engine = Self {
            roles: DashMap::new(),
            default_role: config.default_role.clone(),
        };

        // -- Default roles --------------------------------------------------

        engine.add_role(Role {
            name: "admin".to_string(),
            description: Some("Full system access".to_string()),
            permissions: vec![Permission::parse("*:*:*").expect("static parse")],
            parent: None,
        });

        engine.add_role(Role {
            name: "developer".to_string(),
            description: Some("Read all, write/delete own resources".to_string()),
            permissions: vec![
                Permission::parse("read:*:all").expect("static parse"),
                Permission::parse("write:*:own").expect("static parse"),
                Permission::parse("delete:*:own").expect("static parse"),
            ],
            parent: None,
        });

        engine.add_role(Role {
            name: "operator".to_string(),
            description: Some("Read all, write system resources".to_string()),
            permissions: vec![
                Permission::parse("read:*:all").expect("static parse"),
                Permission::parse("write:system:all").expect("static parse"),
            ],
            parent: None,
        });

        engine.add_role(Role {
            name: "viewer".to_string(),
            description: Some("Read-only access".to_string()),
            permissions: vec![Permission::parse("read:*:all").expect("static parse")],
            parent: None,
        });

        debug!("PolicyEngine initialized with {} default roles", engine.roles.len());

        engine
    }

    /// Evaluate an authorization request against the loaded policy set.
    ///
    /// Evaluation never panics or returns errors — edge cases produce a
    /// [`PolicyDecision`] with `allowed: false` and a descriptive `reason`.
    ///
    /// # Algorithm
    ///
    /// 1. Determine the principal's role names (from claims + default role).
    /// 2. Collect all effective permissions (including inherited from parents).
    /// 3. Find permissions matching the requested action and resource.
    /// 4. If any matching permission is a deny rule, deny immediately.
    /// 5. Evaluate ABAC constraints on matching allow permissions.
    /// 6. If at least one allow permission passes constraints, allow.
    /// 7. Otherwise, deny with "no matching policy".
    pub fn evaluate(&self, request: &AuthorizationRequest) -> PolicyDecision {
        // Determine role names from the principal's agent_type and any
        // permissions embedded in the claims.
        let mut role_names: Vec<String> = Vec::new();

        // Use agent_type as a role name if it maps to a known role.
        if !request.principal.agent_type.is_empty() {
            role_names.push(request.principal.agent_type.clone());
        }

        // Fall back to the default role if no roles matched.
        if role_names.is_empty() {
            if let Some(ref default) = self.default_role {
                role_names.push(default.clone());
            }
        }

        // If we still have no roles and no inline permissions, deny.
        if role_names.is_empty() && request.principal.permissions.is_empty() {
            return PolicyDecision {
                allowed: false,
                reason: "no matching policy".to_string(),
                matching_policy: None,
            };
        }

        // Collect effective permissions from roles.
        let mut effective = self.effective_permissions(&role_names);

        // Also include inline permissions from the claims.
        for perm_str in &request.principal.permissions {
            if let Ok(perm) = Permission::parse(perm_str) {
                effective.push(perm);
            } else {
                warn!(permission = %perm_str, "skipping unparseable inline permission");
            }
        }

        // Determine scope for matching.  When no scope is provided in the
        // context, any permission scope matches (the caller is not restricting
        // by scope).  When a scope IS provided, only permissions whose scope
        // is `*` or equal to the requested scope will match.
        let requested_scope: Option<&str> = request.context.get("scope").map(|s| s.as_str());

        // Separate matching allows and denies.
        let mut has_allow = false;
        let mut allow_policy: Option<String> = None;

        for perm in &effective {
            // Check if this permission matches the action + resource.
            let action_ok = perm.action == "*" || perm.action == request.action;
            let resource_ok = perm.resource == "*" || perm.resource == request.resource;
            if !action_ok || !resource_ok {
                continue;
            }

            // Check scope: if the caller specified a scope, enforce it.
            // If no scope was specified, any permission scope matches.
            if let Some(scope) = requested_scope {
                let scope_ok = perm.scope == "*" || perm.scope == scope;
                if !scope_ok {
                    continue;
                }
            }

            // Deny-wins: any matching deny immediately rejects.
            if perm.deny {
                return PolicyDecision {
                    allowed: false,
                    reason: format!(
                        "explicit deny: {}:{}:{}",
                        perm.action, perm.resource, perm.scope,
                    ),
                    matching_policy: Some(format!(
                        "deny:{}:{}:{}",
                        perm.action, perm.resource, perm.scope,
                    )),
                };
            }

            // Evaluate ABAC constraints if present.
            if let Some(ref constraints) = perm.constraints {
                if !constraints.evaluate(&request.context) {
                    continue;
                }
            }

            has_allow = true;
            if allow_policy.is_none() {
                allow_policy = Some(format!(
                    "{}:{}:{}",
                    perm.action, perm.resource, perm.scope,
                ));
            }
        }

        if has_allow {
            PolicyDecision {
                allowed: true,
                reason: "permission granted".to_string(),
                matching_policy: allow_policy,
            }
        } else {
            PolicyDecision {
                allowed: false,
                reason: "no matching policy".to_string(),
                matching_policy: None,
            }
        }
    }

    /// Convenience wrapper: check whether the given claims allow `action` on
    /// `resource`.
    ///
    /// Builds a minimal [`AuthorizationRequest`] with an empty context and
    /// delegates to [`evaluate`](Self::evaluate).
    pub fn check_permission(
        &self,
        claims: &AgentClaims,
        action: &str,
        resource: &str,
    ) -> bool {
        let request = AuthorizationRequest {
            principal: claims.clone(),
            action: action.to_string(),
            resource: resource.to_string(),
            resource_id: None,
            context: HashMap::new(),
        };
        self.evaluate(&request).allowed
    }

    /// Collect all effective permissions for the given role names, including
    /// any permissions inherited from parent roles.
    ///
    /// Parent resolution is iterative with cycle detection (maximum depth of
    /// 16 levels) to guard against misconfigured circular hierarchies.
    pub fn effective_permissions(&self, roles: &[String]) -> Vec<Permission> {
        let mut perms = Vec::new();

        for role_name in roles {
            self.collect_role_permissions(role_name, &mut perms, 0);
        }

        perms
    }

    /// Add a role to the engine, replacing any existing role with the same
    /// name.
    pub fn add_role(&self, role: Role) {
        debug!(role = %role.name, "adding role");
        self.roles.insert(role.name.clone(), role);
    }

    /// Remove a role by name. Returns `true` if the role existed and was
    /// removed.
    pub fn remove_role(&self, role_name: &str) -> bool {
        let removed = self.roles.remove(role_name).is_some();
        if removed {
            debug!(role = %role_name, "removed role");
        }
        removed
    }

    // -- private helpers ----------------------------------------------------

    /// Recursively collect permissions from a role and its parent chain.
    fn collect_role_permissions(
        &self,
        role_name: &str,
        perms: &mut Vec<Permission>,
        depth: usize,
    ) {
        const MAX_DEPTH: usize = 16;
        if depth >= MAX_DEPTH {
            warn!(
                role = %role_name,
                depth,
                "role hierarchy depth limit reached, possible cycle"
            );
            return;
        }

        let role = match self.roles.get(role_name) {
            Some(r) => r,
            None => {
                if depth == 0 {
                    debug!(role = %role_name, "role not found");
                }
                return;
            }
        };

        perms.extend(role.permissions.clone());

        if let Some(ref parent) = role.parent {
            // Clone the parent name to release the DashMap ref before recursing.
            let parent_name = parent.clone();
            drop(role);
            self.collect_role_permissions(&parent_name, perms, depth + 1);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> PolicyEngine {
        PolicyEngine::new(&RbacConfig::default())
    }

    fn claims_with_type(agent_type: &str) -> AgentClaims {
        AgentClaims {
            agent_type: agent_type.to_string(),
            sub: "test-agent".to_string(),
            agent_id: "test-agent".to_string(),
            ..Default::default()
        }
    }

    fn claims_with_permissions(perms: &[&str]) -> AgentClaims {
        AgentClaims {
            sub: "test-agent".to_string(),
            agent_id: "test-agent".to_string(),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    // -- Default roles -----------------------------------------------------

    #[test]
    fn admin_has_full_access() {
        let engine = default_engine();
        let claims = claims_with_type("admin");
        assert!(engine.check_permission(&claims, "read", "agent"));
        assert!(engine.check_permission(&claims, "write", "system"));
        assert!(engine.check_permission(&claims, "delete", "task"));
        assert!(engine.check_permission(&claims, "admin", "everything"));
    }

    #[test]
    fn viewer_can_read() {
        let engine = default_engine();
        let claims = claims_with_type("viewer");
        assert!(engine.check_permission(&claims, "read", "agent"));
        assert!(engine.check_permission(&claims, "read", "task"));
    }

    #[test]
    fn viewer_cannot_write() {
        let engine = default_engine();
        let claims = claims_with_type("viewer");
        assert!(!engine.check_permission(&claims, "write", "agent"));
        assert!(!engine.check_permission(&claims, "delete", "task"));
    }

    #[test]
    fn developer_can_read_all() {
        let engine = default_engine();
        let claims = claims_with_type("developer");
        assert!(engine.check_permission(&claims, "read", "agent"));
    }

    #[test]
    fn developer_can_write_and_delete() {
        let engine = default_engine();
        let claims = claims_with_type("developer");
        // developer has write:*:own and delete:*:own
        assert!(engine.check_permission(&claims, "write", "agent"));
        assert!(engine.check_permission(&claims, "delete", "task"));
    }

    #[test]
    fn operator_can_write_system() {
        let engine = default_engine();
        let claims = claims_with_type("operator");
        assert!(engine.check_permission(&claims, "read", "agent"));
        assert!(engine.check_permission(&claims, "write", "system"));
    }

    #[test]
    fn operator_cannot_delete() {
        let engine = default_engine();
        let claims = claims_with_type("operator");
        assert!(!engine.check_permission(&claims, "delete", "agent"));
    }

    // -- Default deny / no matching policy ---------------------------------

    #[test]
    fn default_deny_for_unmatched_requests() {
        let engine = default_engine();
        let claims = AgentClaims::default(); // no agent_type, no permissions
        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims,
            action: "read".to_string(),
            resource: "agent".to_string(),
            resource_id: None,
            context: HashMap::new(),
        });
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "no matching policy");
    }

    #[test]
    fn unknown_role_denies() {
        let engine = default_engine();
        let claims = claims_with_type("nonexistent");
        assert!(!engine.check_permission(&claims, "read", "agent"));
    }

    // -- Explicit deny overrides allow -------------------------------------

    #[test]
    fn explicit_deny_overrides_allow() {
        let engine = default_engine();

        // Add a role with both an allow and a deny for the same target.
        let mut deny_perm = Permission::parse("read:secret:all").unwrap();
        deny_perm.deny = true;

        engine.add_role(Role {
            name: "restricted".to_string(),
            description: None,
            permissions: vec![
                Permission::parse("read:*:all").unwrap(),
                deny_perm,
            ],
            parent: None,
        });

        let claims = claims_with_type("restricted");
        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims,
            action: "read".to_string(),
            resource: "secret".to_string(),
            resource_id: None,
            context: HashMap::new(),
        });
        assert!(!decision.allowed);
        assert!(decision.reason.contains("explicit deny"));
    }

    // -- Inline permissions from claims ------------------------------------

    #[test]
    fn inline_permissions_from_claims() {
        let engine = default_engine();
        let claims = claims_with_permissions(&["write:agent:all"]);
        assert!(engine.check_permission(&claims, "write", "agent"));
    }

    #[test]
    fn inline_permissions_combined_with_role() {
        let engine = default_engine();
        let mut claims = claims_with_type("viewer");
        claims.permissions = vec!["write:agent:own".to_string()];
        assert!(engine.check_permission(&claims, "read", "task"));
        assert!(engine.check_permission(&claims, "write", "agent"));
    }

    // -- Role hierarchy (parent inheritance) --------------------------------

    #[test]
    fn role_hierarchy_inherits_permissions() {
        let engine = default_engine();

        // Create a child role that inherits from viewer.
        engine.add_role(Role {
            name: "junior_dev".to_string(),
            description: Some("Junior developer inheriting from viewer".to_string()),
            permissions: vec![Permission::parse("write:agent:own").unwrap()],
            parent: Some("viewer".to_string()),
        });

        let claims = claims_with_type("junior_dev");
        // Inherited from viewer
        assert!(engine.check_permission(&claims, "read", "agent"));
        // Own permission
        assert!(engine.check_permission(&claims, "write", "agent"));
        // Not granted
        assert!(!engine.check_permission(&claims, "delete", "agent"));
    }

    #[test]
    fn multi_level_inheritance() {
        let engine = default_engine();

        engine.add_role(Role {
            name: "level1".to_string(),
            description: None,
            permissions: vec![Permission::parse("read:*:all").unwrap()],
            parent: None,
        });
        engine.add_role(Role {
            name: "level2".to_string(),
            description: None,
            permissions: vec![Permission::parse("write:*:own").unwrap()],
            parent: Some("level1".to_string()),
        });
        engine.add_role(Role {
            name: "level3".to_string(),
            description: None,
            permissions: vec![Permission::parse("delete:*:own").unwrap()],
            parent: Some("level2".to_string()),
        });

        let perms = engine.effective_permissions(&["level3".to_string()]);
        assert_eq!(perms.len(), 3);
    }

    // -- Wildcard permissions -----------------------------------------------

    #[test]
    fn wildcard_permissions_match() {
        let engine = default_engine();
        let claims = claims_with_type("admin");
        // admin has *:*:* — should match anything
        assert!(engine.check_permission(&claims, "custom_action", "custom_resource"));
    }

    // -- Permission parsing from string ------------------------------------

    #[test]
    fn permission_parsing_from_string() {
        let perm = Permission::parse("read:agent:own").unwrap();
        assert_eq!(perm.action, "read");
        assert_eq!(perm.resource, "agent");
        assert_eq!(perm.scope, "own");
        assert!(!perm.deny);

        // Invalid formats
        assert!(Permission::parse("invalid").is_err());
        assert!(Permission::parse("a:b:c:d").is_err());
        assert!(Permission::parse("::").is_err());
    }

    // -- Time window constraint enforcement --------------------------------

    #[test]
    fn time_window_constraint_enforced() {
        let engine = default_engine();

        let mut perm = Permission::parse("read:agent:all").unwrap();
        perm.constraints = Some(PolicyConstraints {
            time_window: Some(TimeWindow {
                start_hour: 9,
                end_hour: 17,
                timezone: "UTC".to_string(),
                days: vec!["monday".to_string()],
            }),
            ip_ranges: None,
            resource_owner: None,
        });

        engine.add_role(Role {
            name: "business_hours".to_string(),
            description: None,
            permissions: vec![perm],
            parent: None,
        });

        let claims = claims_with_type("business_hours");

        // Within hours
        let mut context = HashMap::new();
        context.insert("hour".to_string(), "10".to_string());
        context.insert("day".to_string(), "monday".to_string());
        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims.clone(),
            action: "read".to_string(),
            resource: "agent".to_string(),
            resource_id: None,
            context,
        });
        assert!(decision.allowed);

        // Outside hours
        let mut context = HashMap::new();
        context.insert("hour".to_string(), "20".to_string());
        context.insert("day".to_string(), "monday".to_string());
        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims.clone(),
            action: "read".to_string(),
            resource: "agent".to_string(),
            resource_id: None,
            context,
        });
        assert!(!decision.allowed);

        // Wrong day
        let mut context = HashMap::new();
        context.insert("hour".to_string(), "10".to_string());
        context.insert("day".to_string(), "saturday".to_string());
        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims,
            action: "read".to_string(),
            resource: "agent".to_string(),
            resource_id: None,
            context,
        });
        assert!(!decision.allowed);
    }

    // -- Add / remove roles -----------------------------------------------

    #[test]
    fn add_and_remove_role() {
        let engine = default_engine();

        engine.add_role(Role {
            name: "custom".to_string(),
            description: None,
            permissions: vec![Permission::parse("read:custom:all").unwrap()],
            parent: None,
        });

        let claims = claims_with_type("custom");
        assert!(engine.check_permission(&claims, "read", "custom"));

        assert!(engine.remove_role("custom"));
        assert!(!engine.check_permission(&claims, "read", "custom"));

        // Removing again returns false
        assert!(!engine.remove_role("custom"));
    }

    // -- Default role from config ------------------------------------------

    #[test]
    fn default_role_from_config() {
        let config = RbacConfig {
            default_role: Some("viewer".to_string()),
        };
        let engine = PolicyEngine::new(&config);

        // Claims with no agent_type — should fall back to default role.
        let claims = AgentClaims {
            sub: "anon".to_string(),
            agent_id: "anon".to_string(),
            ..Default::default()
        };
        assert!(engine.check_permission(&claims, "read", "agent"));
        assert!(!engine.check_permission(&claims, "write", "agent"));
    }

    // -- Effective permissions count ---------------------------------------

    #[test]
    fn effective_permissions_for_multiple_roles() {
        let engine = default_engine();

        // developer has 3, operator has 2 — total 5 (some overlap but
        // effective_permissions returns all, dedup is caller's concern).
        let perms =
            engine.effective_permissions(&["developer".to_string(), "operator".to_string()]);
        assert_eq!(perms.len(), 5);
    }

    // -- Worker scenario from contract tests --------------------------------

    #[test]
    fn worker_can_read_own_tasks() {
        let engine = default_engine();

        // Simulate a worker with inline permissions.
        let claims = claims_with_permissions(&["read:task:own"]);
        let mut context = HashMap::new();
        context.insert("scope".to_string(), "own".to_string());

        let decision = engine.evaluate(&AuthorizationRequest {
            principal: claims,
            action: "read".to_string(),
            resource: "task".to_string(),
            resource_id: None,
            context,
        });
        assert!(decision.allowed);
    }

    #[test]
    fn worker_cannot_delete_system_resources() {
        let engine = default_engine();
        let claims = claims_with_permissions(&["read:task:own"]);
        assert!(!engine.check_permission(&claims, "delete", "system"));
    }
}
