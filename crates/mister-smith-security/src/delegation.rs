//! Bounded delegation capability issuance, validation, and revocation.

use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use uuid::Uuid;

use mister_smith_core::{
    AgentId, AuthorityPrincipal, CapabilityId, DelegationCapability, DelegationError,
    DelegationScope, ProvenanceChain, ProvenanceLink, RevocationState,
};

use crate::jwt::AgentClaims;

/// Validated delegation capability with reconstructable provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedDelegation {
    /// Capability used for the privileged action.
    pub capability: DelegationCapability,
    /// Ordered authority chain that led to the capability.
    pub provenance: ProvenanceChain,
    /// Depth of the authority chain.
    pub chain_depth: usize,
}

/// In-memory capability service for Phase 10 bounded delegation enforcement.
#[derive(Debug, Default)]
pub struct DelegationService {
    revoked_capabilities: DashMap<CapabilityId, DateTime<Utc>>,
}

impl DelegationService {
    /// Create a new delegation service.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Issue a bounded capability and its provenance chain.
    pub fn issue_capability(
        &self,
        issuer: AuthorityPrincipal,
        recipient: AgentId,
        scope: DelegationScope,
        ttl: Duration,
        parent: Option<&DelegationCapability>,
        parent_chain: Option<&ProvenanceChain>,
    ) -> Result<(DelegationCapability, ProvenanceChain), DelegationError> {
        let ttl = chrono::Duration::from_std(ttl).map_err(|error| {
            DelegationError::InvalidChain(format!("invalid delegation ttl: {error}"))
        })?;
        let mut expires_at = Utc::now() + ttl;

        let (root_issuer, mut links, parent_capability) = match parent {
            Some(parent) => {
                let parent_chain = parent_chain.ok_or_else(|| {
                    DelegationError::InvalidChain(
                        "delegated capability requires an existing provenance chain".to_string(),
                    )
                })?;
                let validated_parent =
                    self.validate_capability(parent, parent_chain, Some(parent.scope))?;
                expires_at = expires_at.min(validated_parent.capability.expires_at);
                (
                    validated_parent.provenance.root_issuer,
                    validated_parent.provenance.links,
                    Some(parent.capability_id),
                )
            }
            None => (issuer.clone(), Vec::new(), None),
        };

        let capability = DelegationCapability {
            capability_id: CapabilityId::new(),
            issuer: issuer.clone(),
            recipient,
            scope,
            expires_at,
            parent_capability,
            revocation_state: RevocationState::Active,
        };

        links.push(ProvenanceLink {
            issuer,
            recipient,
            capability_id: capability.capability_id,
            scope,
            expires_at,
        });

        let provenance = ProvenanceChain {
            root_issuer,
            terminal_capability: capability.capability_id,
            links,
        };

        self.validate_capability(&capability, &provenance, Some(scope))?;

        Ok((capability, provenance))
    }

    /// Validate a capability for the requested scope.
    pub fn validate_capability(
        &self,
        capability: &DelegationCapability,
        provenance: &ProvenanceChain,
        required_scope: Option<DelegationScope>,
    ) -> Result<ValidatedDelegation, DelegationError> {
        if let Some(required_scope) = required_scope {
            if capability.scope != required_scope {
                return Err(DelegationError::ScopeDenied {
                    capability_id: Some(capability.capability_id),
                    scope: required_scope,
                });
            }
        }

        match self.revocation_state(capability) {
            RevocationState::Active => {}
            RevocationState::Revoked => {
                return Err(DelegationError::Revoked {
                    capability_id: Some(capability.capability_id),
                })
            }
            RevocationState::Expired => {
                return Err(DelegationError::Expired {
                    capability_id: Some(capability.capability_id),
                })
            }
        }

        validate_provenance_chain(capability, provenance)?;

        Ok(ValidatedDelegation {
            capability: capability.clone(),
            provenance: provenance.clone(),
            chain_depth: provenance.links.len(),
        })
    }

    /// Validate the delegation metadata embedded in agent claims.
    pub fn validate_claims(
        &self,
        claims: &AgentClaims,
        required_scope: Option<DelegationScope>,
    ) -> Result<Option<ValidatedDelegation>, DelegationError> {
        claims
            .validate_delegation_chain(claims.delegation_chain.len())
            .map_err(|error| DelegationError::InvalidChain(error.to_string()))?;

        match (&claims.delegation_capability, &claims.provenance_chain) {
            (None, None) => Ok(None),
            (Some(_), None) | (None, Some(_)) => Err(DelegationError::InvalidChain(
                "delegation capability and provenance chain must be present together".to_string(),
            )),
            (Some(capability), Some(provenance)) => {
                if capability.recipient.to_string() != claims.agent_id {
                    return Err(DelegationError::InvalidChain(format!(
                        "delegation capability recipient '{}' does not match claims agent '{}'",
                        capability.recipient, claims.agent_id
                    )));
                }

                self.validate_capability(capability, provenance, required_scope)
                    .map(Some)
            }
        }
    }

    /// Explicitly revoke a capability.
    pub fn revoke_capability(&self, capability_id: CapabilityId) {
        self.revoked_capabilities.insert(capability_id, Utc::now());
    }

    /// Return the current revocation state for a capability.
    #[must_use]
    pub fn revocation_state(&self, capability: &DelegationCapability) -> RevocationState {
        if capability.revocation_state == RevocationState::Revoked
            || self
                .revoked_capabilities
                .contains_key(&capability.capability_id)
        {
            return RevocationState::Revoked;
        }

        if capability.expires_at <= Utc::now() {
            return RevocationState::Expired;
        }

        RevocationState::Active
    }
}

fn validate_provenance_chain(
    capability: &DelegationCapability,
    provenance: &ProvenanceChain,
) -> Result<(), DelegationError> {
    let first = provenance.links.first().ok_or_else(|| {
        DelegationError::InvalidChain(
            "delegation provenance chain must contain at least one link".to_string(),
        )
    })?;
    let last = provenance.links.last().ok_or_else(|| {
        DelegationError::InvalidChain(
            "delegation provenance chain must contain at least one link".to_string(),
        )
    })?;

    if provenance.root_issuer != first.issuer {
        return Err(DelegationError::InvalidChain(
            "delegation provenance root issuer does not match first link".to_string(),
        ));
    }
    if provenance.terminal_capability != capability.capability_id {
        return Err(DelegationError::InvalidChain(
            "delegation provenance terminal capability does not match capability".to_string(),
        ));
    }
    if last.capability_id != capability.capability_id
        || last.recipient != capability.recipient
        || last.scope != capability.scope
        || last.expires_at != capability.expires_at
    {
        return Err(DelegationError::InvalidChain(
            "delegation provenance terminal link does not match capability".to_string(),
        ));
    }
    if let Some(parent_capability) = capability.parent_capability {
        let parent = provenance.links.iter().rev().nth(1).ok_or_else(|| {
            DelegationError::InvalidChain(
                "delegation parent capability missing from provenance".to_string(),
            )
        })?;
        if parent.capability_id != parent_capability {
            return Err(DelegationError::InvalidChain(
                "delegation parent capability does not match provenance chain".to_string(),
            ));
        }
    }

    let mut seen_capabilities = HashSet::with_capacity(provenance.links.len());
    let mut seen_recipients = HashSet::with_capacity(provenance.links.len());
    for link in &provenance.links {
        if !seen_capabilities.insert(link.capability_id) {
            return Err(DelegationError::InvalidChain(
                "delegation provenance reuses a capability identifier".to_string(),
            ));
        }
        if !seen_recipients.insert(link.recipient) {
            return Err(DelegationError::InvalidChain(
                "delegation provenance contains a cycle".to_string(),
            ));
        }
    }

    Ok(())
}

pub(crate) fn authority_principal_for_agent_id(agent_id: &str) -> AuthorityPrincipal {
    Uuid::parse_str(agent_id)
        .map(AgentId::from_uuid)
        .map(AuthorityPrincipal::Agent)
        .unwrap_or_else(|_| AuthorityPrincipal::Policy(format!("agent:{agent_id}")))
}
