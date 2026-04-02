//! Bounded delegation capability issuance, validation, and revocation.

use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use uuid::Uuid;

use mister_smith_core::{
    AgentId, AuthorityPrincipal, CapabilityId, DelegatedAction, DelegationCapability,
    DelegationError, DelegationScope, ExternalDelegationEnvelope, ProvenanceChain, ProvenanceLink,
    RevocationState,
};

use crate::jwt::{AgentClaims, DEFAULT_MAX_DELEGATION_CHAIN_DEPTH};

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
#[derive(Debug)]
pub struct DelegationService {
    revoked_capabilities: DashMap<CapabilityId, DateTime<Utc>>,
    revoked_actions: DashMap<String, DateTime<Utc>>,
    max_delegation_chain_depth: usize,
}

impl Default for DelegationService {
    fn default() -> Self {
        Self::new_with_delegation_chain_max_depth(DEFAULT_MAX_DELEGATION_CHAIN_DEPTH)
    }
}

impl DelegationService {
    /// Create a new delegation service.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new delegation service with an explicit claim-chain depth limit.
    #[must_use]
    pub fn new_with_delegation_chain_max_depth(max_delegation_chain_depth: usize) -> Self {
        Self {
            revoked_capabilities: DashMap::new(),
            revoked_actions: DashMap::new(),
            max_delegation_chain_depth,
        }
    }

    /// Issue a bounded capability and its provenance chain.
    #[allow(clippy::too_many_arguments)]
    pub fn issue_capability(
        &self,
        issuer: AuthorityPrincipal,
        recipient: AgentId,
        scope: DelegationScope,
        descriptor_id: Option<String>,
        ttl: Duration,
        parent: Option<&DelegationCapability>,
        parent_chain: Option<&ProvenanceChain>,
    ) -> Result<(DelegationCapability, ProvenanceChain), DelegationError> {
        let ttl = chrono::Duration::from_std(ttl).map_err(|error| {
            DelegationError::InvalidChain(format!("invalid delegation ttl: {error}"))
        })?;
        let mut expires_at = Utc::now() + ttl;

        let (root_issuer, mut links, parent_capability, descriptor_id) = match parent {
            Some(parent) => {
                let parent_chain = parent_chain.ok_or_else(|| {
                    DelegationError::InvalidChain(
                        "delegated capability requires an existing provenance chain".to_string(),
                    )
                })?;
                let validated_parent =
                    self.validate_capability(parent, parent_chain, Some(parent.scope))?;
                let descriptor_id = match (
                    descriptor_id,
                    validated_parent.capability.descriptor_id.clone(),
                ) {
                    (Some(candidate), Some(parent_descriptor)) => {
                        if candidate != parent_descriptor {
                            return Err(DelegationError::InvalidChain(format!(
                                "delegation descriptor '{candidate}' does not match parent descriptor '{parent_descriptor}'"
                            )));
                        }
                        Some(candidate)
                    }
                    (Some(candidate), None) => Some(candidate),
                    (None, inherited) => inherited,
                };
                expires_at = expires_at.min(validated_parent.capability.expires_at);
                (
                    validated_parent.provenance.root_issuer,
                    validated_parent.provenance.links,
                    Some(parent.capability_id),
                    descriptor_id,
                )
            }
            None => (issuer.clone(), Vec::new(), None, descriptor_id),
        };

        let capability = DelegationCapability {
            capability_id: CapabilityId::new(),
            issuer: issuer.clone(),
            recipient,
            scope,
            expires_at,
            descriptor_id: descriptor_id.clone(),
            parent_capability,
            revocation_state: RevocationState::Active,
        };

        links.push(ProvenanceLink {
            issuer,
            recipient,
            capability_id: capability.capability_id,
            scope,
            expires_at,
            descriptor_id,
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

    /// Validate a capability for a typed delegated action.
    pub fn validate_action(
        &self,
        capability: &DelegationCapability,
        provenance: &ProvenanceChain,
        action: &DelegatedAction,
    ) -> Result<ValidatedDelegation, DelegationError> {
        let validated = self.validate_capability(capability, provenance, action.required_scope)?;
        validate_descriptor_binding(&validated.capability, action)?;
        self.validate_action_revocation(action)?;
        Ok(validated)
    }

    /// Validate delegated authority that was serialized across an external boundary.
    pub fn validate_external_envelope(
        &self,
        envelope: &ExternalDelegationEnvelope,
    ) -> Result<ValidatedDelegation, DelegationError> {
        match &envelope.action {
            Some(action) => {
                self.validate_action(&envelope.capability, &envelope.provenance, action)
            }
            None => self.validate_capability(&envelope.capability, &envelope.provenance, None),
        }
    }

    /// Validate the delegation metadata embedded in agent claims.
    pub fn validate_claims(
        &self,
        claims: &AgentClaims,
        required_scope: Option<DelegationScope>,
    ) -> Result<Option<ValidatedDelegation>, DelegationError> {
        claims
            .validate_delegation_chain(self.max_delegation_chain_depth)
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

    /// Validate the delegation metadata in claims for a typed delegated action.
    pub fn validate_claims_for_action(
        &self,
        claims: &AgentClaims,
        action: &DelegatedAction,
    ) -> Result<Option<ValidatedDelegation>, DelegationError> {
        claims
            .validate_delegation_chain(self.max_delegation_chain_depth)
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

                self.validate_action(capability, provenance, action)
                    .map(Some)
            }
        }
    }

    /// Explicitly revoke a capability.
    pub fn revoke_capability(&self, capability_id: CapabilityId) {
        self.revoked_capabilities.insert(capability_id, Utc::now());
    }

    /// Explicitly revoke a delegated action by its stable revocation key.
    pub fn revoke_action(&self, revocation_key: impl Into<String>) {
        self.revoked_actions
            .insert(revocation_key.into(), Utc::now());
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

    fn validate_action_revocation(&self, action: &DelegatedAction) -> Result<(), DelegationError> {
        if self.revoked_actions.contains_key(&action.revocation_key) {
            return Err(DelegationError::ActionRevoked {
                revocation_key: action.revocation_key.clone(),
            });
        }

        Ok(())
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
        || last.descriptor_id != capability.descriptor_id
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

fn validate_descriptor_binding(
    capability: &DelegationCapability,
    action: &DelegatedAction,
) -> Result<(), DelegationError> {
    match capability.descriptor_id.as_deref() {
        Some(descriptor_id) if descriptor_id == action.descriptor_id => Ok(()),
        Some(descriptor_id) => Err(DelegationError::InvalidChain(format!(
            "delegation descriptor '{descriptor_id}' does not authorize action descriptor '{}'",
            action.descriptor_id
        ))),
        None => Err(DelegationError::InvalidChain(format!(
            "delegation capability missing descriptor binding for action descriptor '{}'",
            action.descriptor_id
        ))),
    }
}

pub(crate) fn authority_principal_for_agent_id(agent_id: &str) -> AuthorityPrincipal {
    Uuid::parse_str(agent_id)
        .map(AgentId::from_uuid)
        .map(AuthorityPrincipal::Agent)
        .unwrap_or_else(|_| AuthorityPrincipal::Policy(format!("agent:{agent_id}")))
}

/// Build a transport-safe delegation envelope from validated local policy state.
#[must_use]
pub fn external_delegation_envelope(
    validated: &ValidatedDelegation,
    action: Option<&DelegatedAction>,
) -> ExternalDelegationEnvelope {
    let envelope =
        ExternalDelegationEnvelope::new(validated.capability.clone(), validated.provenance.clone());

    match action {
        Some(action) => envelope.with_action(action.clone()),
        None => envelope,
    }
}
