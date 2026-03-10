use std::sync::Arc;
use std::time::Duration;

use mister_smith_actor::system::{ActorSystem, ActorSystemConfig};
use mister_smith_agents::config::AgentConfig;
use mister_smith_agents::sandbox::AgentSandbox;
use mister_smith_agents::AgentSystemError;
use mister_smith_core::{Actor, AgentId, AgentState, AgentType};
use mister_smith_security::sandbox::{
    AgentClass, IOFirewall, SandboxAccountConfig, SandboxCredentialIssuer,
};
use nkeys::KeyPair;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SandboxMessage {
    Ping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SandboxState;

#[derive(Debug, thiserror::Error)]
#[error("sandbox actor error: {0}")]
struct SandboxError(String);

struct SandboxActor {
    id: AgentId,
}

#[async_trait::async_trait]
impl Actor for SandboxActor {
    type Message = SandboxMessage;
    type State = SandboxState;
    type Error = SandboxError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        _message: Self::Message,
        _state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        Ok(serde_json::json!({"ok": true}))
    }

    fn pre_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post_stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

fn account_config(name: &str, ttl: Duration) -> SandboxAccountConfig {
    let signing_key = KeyPair::new_account();
    SandboxAccountConfig::new(name, signing_key.public_key(), signing_key, ttl)
}

fn sandbox() -> AgentSandbox {
    let issuer = SandboxCredentialIssuer::new(
        account_config("persistent-account", Duration::from_secs(900)),
        account_config("ephemeral-account", Duration::from_secs(60)),
    );
    let firewall = IOFirewall::with_default_rules("persistent-account", "ephemeral-account");
    AgentSandbox::new(issuer, firewall)
}

fn misconfigured_sandbox() -> AgentSandbox {
    let persistent_signing_key = KeyPair::new_user();
    let ephemeral_signing_key = KeyPair::new_account();
    let issuer = SandboxCredentialIssuer::new(
        SandboxAccountConfig::new(
            "persistent-account",
            persistent_signing_key.public_key(),
            persistent_signing_key,
            Duration::from_secs(900),
        ),
        SandboxAccountConfig::new(
            "ephemeral-account",
            ephemeral_signing_key.public_key(),
            ephemeral_signing_key,
            Duration::from_secs(60),
        ),
    );
    let firewall = IOFirewall::with_default_rules("persistent-account", "ephemeral-account");
    AgentSandbox::new(issuer, firewall)
}

async fn wait_for_cleanup(sandbox: &AgentSandbox, agent_id: AgentId) {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if sandbox.credentials(&agent_id).is_none() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("credentials should be cleaned up");
}

#[test]
fn classify_assigns_expected_defaults_and_honors_override() {
    let sandbox = sandbox();

    assert_eq!(
        sandbox.classify(&AgentConfig::for_type(AgentType::Coordinator)),
        AgentClass::Persistent
    );
    assert_eq!(
        sandbox.classify(&AgentConfig::for_type(AgentType::Worker)),
        AgentClass::Ephemeral
    );

    let mut long_running_critic = AgentConfig::for_type(AgentType::Critic);
    long_running_critic.task_timeout = Duration::from_secs(600);
    assert_eq!(
        sandbox.classify(&long_running_critic),
        AgentClass::Persistent
    );

    assert_eq!(
        sandbox.classify_with_override(&long_running_critic, Some(AgentClass::Ephemeral)),
        AgentClass::Ephemeral
    );
}

#[tokio::test]
async fn sandboxed_spawn_assigns_ephemeral_credentials_and_cleans_up_on_stop() {
    let sandbox = sandbox();
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let agent_id = AgentId::new();

    let runtime = sandbox
        .spawn_agent(
            system,
            SandboxActor { id: agent_id },
            SandboxState,
            AgentConfig::for_type(AgentType::Worker),
            None,
        )
        .await
        .expect("sandboxed spawn should succeed");

    assert_eq!(runtime.agent_class(), AgentClass::Ephemeral);
    assert_eq!(runtime.credentials().nats_account, "ephemeral-account");
    assert!(sandbox.credentials(&agent_id).is_some());

    runtime
        .runtime()
        .stop()
        .await
        .expect("explicit stop should succeed");
    wait_for_cleanup(&sandbox, agent_id).await;
}

#[tokio::test]
async fn sandboxed_spawn_times_out_ephemeral_agents_and_cleans_up_credentials() {
    let sandbox = sandbox();
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let agent_id = AgentId::new();
    let mut config = AgentConfig::for_type(AgentType::Worker);
    config.task_timeout = Duration::from_millis(50);

    let runtime = sandbox
        .spawn_agent(
            system,
            SandboxActor { id: agent_id },
            SandboxState,
            config,
            None,
        )
        .await
        .expect("sandboxed spawn should succeed");

    wait_for_cleanup(&sandbox, agent_id).await;
    assert_eq!(runtime.runtime().state().await, AgentState::Terminated);
}

#[tokio::test]
async fn persistent_agents_do_not_auto_cleanup_on_timeout_window() {
    let sandbox = sandbox();
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let agent_id = AgentId::new();
    let mut config = AgentConfig::for_type(AgentType::Coordinator);
    config.task_timeout = Duration::from_millis(50);

    let runtime = sandbox
        .spawn_agent(
            system,
            SandboxActor { id: agent_id },
            SandboxState,
            config,
            None,
        )
        .await
        .expect("sandboxed spawn should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(sandbox.credentials(&agent_id).is_some());

    assert!(
        sandbox.cleanup(&agent_id).is_some(),
        "manual cleanup should remove persistent credentials"
    );
    runtime
        .runtime()
        .stop()
        .await
        .expect("explicit stop should succeed");
}

#[tokio::test]
async fn sandboxed_spawn_stops_actor_when_credential_issue_fails() {
    let sandbox = misconfigured_sandbox();
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let agent_id = AgentId::new();

    let err = match sandbox
        .spawn_agent(
            system.clone(),
            SandboxActor { id: agent_id },
            SandboxState,
            AgentConfig::for_type(AgentType::Coordinator),
            None,
        )
        .await
    {
        Ok(_) => panic!("sandboxed spawn should fail with invalid signing key"),
        Err(err) => err,
    };
    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));

    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            match system.get_actor_state(&agent_id).await {
                None | Some(AgentState::Terminated) => break,
                _ => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        }
    })
    .await
    .expect("failed spawn should not leak a live actor");
    assert!(sandbox.credentials(&agent_id).is_none());
}
