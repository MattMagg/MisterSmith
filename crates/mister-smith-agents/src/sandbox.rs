//! Agent-side sandbox integration and lifecycle-aware credential management.

use std::sync::Arc;
use std::time::Duration;

use mister_smith_actor::system::ActorSystem;
use mister_smith_core::{Actor, AgentId, AgentState, AgentType};
use mister_smith_supervision::SupervisedSystem;
use serde_json::Value;

pub use mister_smith_security::sandbox::{
    AgentClass, CrossingDecision, CrossingRule, IOFirewall, SandboxAccountConfig,
    SandboxCredentialIssuer, SandboxCredentials,
};
use mister_smith_security::TaintLabel;

use crate::agent::{
    spawn_agent as spawn_runtime, spawn_supervised as spawn_supervised_runtime, AgentRuntime,
};
use crate::config::AgentConfig;
use crate::errors::AgentSystemError;
use crate::quarantine::{QuarantineActor, QuarantineTransfer};

const DEFAULT_LONG_RUNNING_THRESHOLD: Duration = Duration::from_secs(300);
const CLEANUP_POLL_INTERVAL: Duration = Duration::from_millis(25);
const TERMINATION_TIMEOUT: Duration = Duration::from_secs(5);

/// Wrapper around an [`AgentRuntime`] with lifecycle-scoped sandbox credentials.
pub struct SandboxedAgentRuntime<M: Send + 'static, R: Send + 'static> {
    runtime: AgentRuntime<M, R>,
    credentials: SandboxCredentials,
    agent_class: AgentClass,
}

impl<M: Send + 'static, R: Send + 'static> SandboxedAgentRuntime<M, R> {
    /// Return the underlying runtime.
    pub fn runtime(&self) -> &AgentRuntime<M, R> {
        &self.runtime
    }

    /// Return the issued sandbox credentials.
    pub fn credentials(&self) -> &SandboxCredentials {
        &self.credentials
    }

    /// Return the assigned lifecycle class.
    pub fn agent_class(&self) -> AgentClass {
        self.agent_class
    }
}

/// High-level sandbox entry point for classifying agents and issuing credentials.
#[derive(Clone)]
pub struct AgentSandbox {
    issuer: SandboxCredentialIssuer,
    firewall: IOFirewall,
    quarantine_actor: Option<Arc<QuarantineActor>>,
    long_running_threshold: Duration,
}

impl AgentSandbox {
    /// Create a new agent sandbox.
    #[must_use]
    pub fn new(issuer: SandboxCredentialIssuer, firewall: IOFirewall) -> Self {
        Self {
            issuer,
            firewall,
            quarantine_actor: None,
            long_running_threshold: DEFAULT_LONG_RUNNING_THRESHOLD,
        }
    }

    /// Override the timeout threshold that upgrades long-running tasks to persistent.
    #[must_use]
    pub fn with_long_running_threshold(mut self, threshold: Duration) -> Self {
        self.long_running_threshold = threshold;
        self
    }

    /// Attach a quarantine actor used for required cross-boundary transfers.
    #[must_use]
    pub fn with_quarantine_actor(mut self, quarantine_actor: Arc<QuarantineActor>) -> Self {
        self.quarantine_actor = Some(quarantine_actor);
        self
    }

    /// Classify an agent from its runtime configuration.
    pub fn classify(&self, config: &AgentConfig) -> AgentClass {
        self.classify_with_override(config, None)
    }

    /// Classify an agent from its configuration, honoring an explicit override.
    pub fn classify_with_override(
        &self,
        config: &AgentConfig,
        class_override: Option<AgentClass>,
    ) -> AgentClass {
        if let Some(class_override) = class_override {
            return class_override;
        }

        match config.agent_type {
            AgentType::Supervisor
            | AgentType::Coordinator
            | AgentType::Monitor
            | AgentType::Planner
            | AgentType::Router
            | AgentType::Memory => AgentClass::Persistent,
            AgentType::Worker | AgentType::Executor | AgentType::Critic => {
                if config.task_timeout >= self.long_running_threshold {
                    AgentClass::Persistent
                } else {
                    AgentClass::Ephemeral
                }
            }
        }
    }

    /// Issue credentials explicitly for an agent and class.
    pub fn create_credentials(
        &self,
        agent_id: AgentId,
        agent_class: AgentClass,
    ) -> Result<SandboxCredentials, AgentSystemError> {
        self.issuer
            .create_credentials(agent_id.to_string(), agent_class)
            .map_err(|error| AgentSystemError::PermissionDenied(error.to_string()))
    }

    /// Return the active credentials for an agent, if present.
    #[must_use]
    pub fn credentials(&self, agent_id: &AgentId) -> Option<SandboxCredentials> {
        self.issuer.credentials(&agent_id.to_string())
    }

    /// Remove active credentials for an agent.
    ///
    /// Returns the removed credentials, or `None` if no credentials were active.
    pub fn cleanup(&self, agent_id: &AgentId) -> Option<SandboxCredentials> {
        self.issuer.cleanup(&agent_id.to_string())
    }

    /// Evaluate whether a subject crossing is permitted.
    pub fn check_crossing(
        &self,
        source_account: &str,
        target_account: &str,
        subject: &str,
    ) -> Result<CrossingDecision, AgentSystemError> {
        self.firewall
            .check_crossing(source_account, target_account, subject)
            .map_err(|error| AgentSystemError::PermissionDenied(error.to_string()))
    }

    /// Inspect and forward a cross-boundary payload when sandbox policy allows
    /// the subject crossing.
    pub fn inspect_cross_boundary_transfer(
        &self,
        principal: Option<&str>,
        source_account: &str,
        target_account: &str,
        subject: &str,
        state_type: &str,
        payload: &Value,
    ) -> Result<QuarantineTransfer, AgentSystemError> {
        match self.check_crossing(source_account, target_account, subject)? {
            CrossingDecision::Allow => Ok(QuarantineTransfer {
                action: mister_smith_security::QuarantineAction::Pass,
                taint_label: TaintLabel::Clean,
                payload: payload.clone(),
                schema_version: None,
                reason: None,
                monitored: false,
            }),
            CrossingDecision::Quarantine => {
                let actor = self.quarantine_actor.as_ref().ok_or_else(|| {
                    AgentSystemError::PermissionDenied(format!(
                        "cross-boundary subject '{subject}' requires a quarantine actor"
                    ))
                })?;
                actor
                    .inspect_cross_boundary_transfer(
                        principal,
                        source_account,
                        target_account,
                        subject,
                        state_type,
                        payload,
                    )
                    .map_err(|error| AgentSystemError::PermissionDenied(error.to_string()))
            }
        }
    }

    /// Spawn a sandboxed agent and attach lifecycle cleanup for ephemeral credentials.
    pub async fn spawn_agent<A>(
        &self,
        system: Arc<ActorSystem>,
        actor: A,
        initial_state: A::State,
        config: AgentConfig,
        class_override: Option<AgentClass>,
    ) -> Result<SandboxedAgentRuntime<A::Message, A::Response>, AgentSystemError>
    where
        A: Actor + 'static,
        A::Message: Send + 'static,
        A::State: Send + 'static,
    {
        let agent_class = self.classify_with_override(&config, class_override);
        let runtime = spawn_runtime(system.clone(), actor, initial_state, config.clone()).await?;
        let agent_id = runtime.context.agent_id;
        let credentials = match self.create_credentials(agent_id, agent_class) {
            Ok(credentials) => credentials,
            Err(error) => {
                let _ = system.stop_actor(&agent_id).await;
                let _ = tokio::time::timeout(
                    TERMINATION_TIMEOUT,
                    wait_for_termination(system, agent_id),
                )
                .await;
                return Err(error);
            }
        };
        self.attach_cleanup_if_ephemeral(agent_id, system, &config, agent_class);

        Ok(SandboxedAgentRuntime {
            runtime,
            credentials,
            agent_class,
        })
    }

    /// Spawn a sandboxed supervised agent and attach lifecycle cleanup for ephemeral credentials.
    pub async fn spawn_supervised<A, F>(
        &self,
        supervised: &SupervisedSystem,
        supervisor_id: AgentId,
        factory: F,
        config: AgentConfig,
        class_override: Option<AgentClass>,
    ) -> Result<SandboxedAgentRuntime<A::Message, A::Response>, AgentSystemError>
    where
        A: Actor + 'static,
        A::Message: Send + Clone + 'static,
        A::State: Send + 'static,
        A::Response: Send + Clone + 'static,
        F: Fn() -> (A, A::State) + Send + Sync + 'static,
    {
        let agent_class = self.classify_with_override(&config, class_override);
        let runtime =
            spawn_supervised_runtime(supervised, supervisor_id, factory, config.clone()).await?;
        let agent_id = runtime.context.agent_id;
        let system = supervised.system_arc();
        let credentials = match self.create_credentials(agent_id, agent_class) {
            Ok(credentials) => credentials,
            Err(error) => {
                let _ = system.stop_actor(&agent_id).await;
                let _ = tokio::time::timeout(
                    TERMINATION_TIMEOUT,
                    wait_for_termination(system.clone(), agent_id),
                )
                .await;
                return Err(error);
            }
        };
        self.attach_cleanup_if_ephemeral(agent_id, system, &config, agent_class);

        Ok(SandboxedAgentRuntime {
            runtime,
            credentials,
            agent_class,
        })
    }

    fn attach_cleanup_if_ephemeral(
        &self,
        agent_id: AgentId,
        system: Arc<ActorSystem>,
        config: &AgentConfig,
        agent_class: AgentClass,
    ) {
        if agent_class != AgentClass::Ephemeral {
            return;
        }

        let sandbox = self.clone();
        let timeout = config.task_timeout;

        tokio::spawn(async move {
            if tokio::time::timeout(timeout, wait_for_termination(system.clone(), agent_id))
                .await
                .is_err()
            {
                let _ = system.stop_actor(&agent_id).await;
                let _ = tokio::time::timeout(
                    TERMINATION_TIMEOUT,
                    wait_for_termination(system, agent_id),
                )
                .await;
            }

            let _ = sandbox.cleanup(&agent_id);
        });
    }
}

async fn wait_for_termination(system: Arc<ActorSystem>, agent_id: AgentId) {
    loop {
        match system.get_actor_state(&agent_id).await {
            None | Some(AgentState::Terminated) => return,
            _ => tokio::time::sleep(CLEANUP_POLL_INTERVAL).await,
        }
    }
}
