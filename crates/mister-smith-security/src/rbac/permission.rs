//! Permission and Role types for role-based access control.
//!
//! A [`Permission`] encodes a single grant (or deny) in `action:resource:scope`
//! format.  Wildcards (`*`) match any value in the corresponding position.
//!
//! A [`Role`] groups permissions under a name and supports single-level
//! inheritance through an optional `parent` role.

use serde::{Deserialize, Serialize};

use super::constraints::PolicyConstraints;
use mister_smith_core::SecurityError;

// ---------------------------------------------------------------------------
// Permission
// ---------------------------------------------------------------------------

/// A single permission grant (or deny) within a [`Role`].
///
/// Permissions follow the `action:resource:scope` triple. Each component may
/// be a literal value or the wildcard `*`, which matches any value.
///
/// # Examples
///
/// ```
/// use mister_smith_security::rbac::permission::Permission;
///
/// let perm = Permission::parse("read:agent:own").unwrap();
/// assert!(perm.matches("read", "agent", "own"));
/// assert!(!perm.matches("write", "agent", "own"));
///
/// let admin = Permission::parse("*:*:*").unwrap();
/// assert!(admin.matches("delete", "system", "all"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Action verb (e.g. `read`, `write`, `delete`, `admin`, `*`).
    pub action: String,
    /// Resource type (e.g. `agent`, `task`, `system`, `*`).
    pub resource: String,
    /// Scope qualifier (e.g. `own`, `tenant`, `all`, `*`).
    pub scope: String,
    /// Optional ABAC constraints that must also be satisfied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<PolicyConstraints>,
    /// When `true` this is an explicit deny rule. Deny rules take precedence
    /// over all allow rules during policy evaluation (deny-wins).
    #[serde(default)]
    pub deny: bool,
}

impl Permission {
    /// Parse a permission string in `action:resource:scope` format.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InsufficientPermissions`] if the string does
    /// not contain exactly three colon-separated, non-empty components.
    pub fn parse(s: &str) -> Result<Self, SecurityError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(SecurityError::InsufficientPermissions(format!(
                "invalid permission format (expected action:resource:scope): {s}"
            )));
        }

        let action = parts[0].trim();
        let resource = parts[1].trim();
        let scope = parts[2].trim();

        if action.is_empty() || resource.is_empty() || scope.is_empty() {
            return Err(SecurityError::InsufficientPermissions(format!(
                "permission components must be non-empty: {s}"
            )));
        }

        Ok(Self {
            action: action.to_string(),
            resource: resource.to_string(),
            scope: scope.to_string(),
            constraints: None,
            deny: false,
        })
    }

    /// Check whether this permission covers the given `action`, `resource`,
    /// and `scope` triple.
    ///
    /// A wildcard `*` in any position matches every value.
    pub fn matches(&self, action: &str, resource: &str, scope: &str) -> bool {
        let action_ok = self.action == "*" || self.action == action;
        let resource_ok = self.resource == "*" || self.resource == resource;
        let scope_ok = self.scope == "*" || self.scope == scope;
        action_ok && resource_ok && scope_ok
    }
}

// ---------------------------------------------------------------------------
// Role
// ---------------------------------------------------------------------------

/// A named collection of [`Permission`]s with optional single-level
/// inheritance through a `parent` role.
///
/// Parent resolution is handled by [`super::PolicyEngine::effective_permissions`]
/// — the `Role` struct itself only stores the parent name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier (e.g. `admin`, `developer`).
    pub name: String,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Permissions directly granted by this role.
    pub permissions: Vec<Permission>,
    /// Optional parent role name for permission inheritance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_permission() {
        let perm = Permission::parse("read:agent:own").unwrap();
        assert_eq!(perm.action, "read");
        assert_eq!(perm.resource, "agent");
        assert_eq!(perm.scope, "own");
        assert!(!perm.deny);
        assert!(perm.constraints.is_none());
    }

    #[test]
    fn parse_wildcard_permission() {
        let perm = Permission::parse("*:*:*").unwrap();
        assert_eq!(perm.action, "*");
        assert_eq!(perm.resource, "*");
        assert_eq!(perm.scope, "*");
    }

    #[test]
    fn parse_rejects_too_few_parts() {
        let err = Permission::parse("read:agent").unwrap_err();
        assert!(err.to_string().contains("invalid permission format"));
    }

    #[test]
    fn parse_rejects_too_many_parts() {
        let err = Permission::parse("read:agent:own:extra").unwrap_err();
        assert!(err.to_string().contains("invalid permission format"));
    }

    #[test]
    fn parse_rejects_empty_components() {
        let err = Permission::parse("read::own").unwrap_err();
        assert!(err.to_string().contains("non-empty"));
    }

    #[test]
    fn matches_exact() {
        let perm = Permission::parse("read:agent:own").unwrap();
        assert!(perm.matches("read", "agent", "own"));
        assert!(!perm.matches("write", "agent", "own"));
        assert!(!perm.matches("read", "task", "own"));
        assert!(!perm.matches("read", "agent", "all"));
    }

    #[test]
    fn matches_wildcard_action() {
        let perm = Permission::parse("*:agent:own").unwrap();
        assert!(perm.matches("read", "agent", "own"));
        assert!(perm.matches("write", "agent", "own"));
        assert!(!perm.matches("write", "task", "own"));
    }

    #[test]
    fn matches_wildcard_resource() {
        let perm = Permission::parse("read:*:own").unwrap();
        assert!(perm.matches("read", "agent", "own"));
        assert!(perm.matches("read", "task", "own"));
        assert!(!perm.matches("write", "task", "own"));
    }

    #[test]
    fn matches_wildcard_scope() {
        let perm = Permission::parse("read:agent:*").unwrap();
        assert!(perm.matches("read", "agent", "own"));
        assert!(perm.matches("read", "agent", "all"));
        assert!(!perm.matches("write", "agent", "all"));
    }

    #[test]
    fn matches_full_wildcard() {
        let perm = Permission::parse("*:*:*").unwrap();
        assert!(perm.matches("delete", "system", "all"));
        assert!(perm.matches("read", "agent", "own"));
    }

    #[test]
    fn role_serde_roundtrip() {
        let role = Role {
            name: "developer".to_string(),
            description: Some("Developer role".to_string()),
            permissions: vec![
                Permission::parse("read:*:all").unwrap(),
                Permission::parse("write:*:own").unwrap(),
            ],
            parent: None,
        };

        let json = serde_json::to_string(&role).unwrap();
        let deserialized: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "developer");
        assert_eq!(deserialized.permissions.len(), 2);
    }
}
