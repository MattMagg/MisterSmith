//! Verify all core traits are implementable with dummy structs.
//!
//! These tests confirm that the trait signatures compile correctly and
//! can be implemented by downstream crates.

use async_trait::async_trait;
use mister_smith_core::*;
use std::any::TypeId;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Mock Actor
// ---------------------------------------------------------------------------

struct MockActor {
    id: AgentId,
}

#[async_trait]
impl Actor for MockActor {
    type Message = String;
    type State = Vec<String>;
    type Error = ActorError;
    type Response = ();

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<(), Self::Error> {
        state.push(message);
        Ok(())
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

// ---------------------------------------------------------------------------
// Mock Tool and Agent
// ---------------------------------------------------------------------------

struct MockTool {
    id: ToolId,
}

#[async_trait]
impl Tool for MockTool {
    async fn execute(&self, _params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema
    }

    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities
    }

    fn tool_id(&self) -> ToolId {
        self.id
    }

    fn version(&self) -> semver::Version {
        semver::Version::new(0, 1, 0)
    }
}

struct MockAgent {
    tool: MockTool,
    ctx: String,
}

#[async_trait]
impl Tool for MockAgent {
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        self.tool.execute(params).await
    }

    fn schema(&self) -> ToolSchema {
        self.tool.schema()
    }

    fn capabilities(&self) -> ToolCapabilities {
        self.tool.capabilities()
    }

    fn tool_id(&self) -> ToolId {
        self.tool.tool_id()
    }

    fn version(&self) -> semver::Version {
        self.tool.version()
    }
}

#[async_trait]
impl Agent for MockAgent {
    type Context = String;
    type Error = ToolError;

    async fn process(&self, _message: serde_json::Value) -> Result<serde_json::Value, Self::Error> {
        Ok(serde_json::json!({"agent": "mock"}))
    }

    fn role(&self) -> AgentType {
        AgentType::Worker
    }

    fn context(&self) -> &Self::Context {
        &self.ctx
    }

    async fn initialize(&mut self, context: Self::Context) -> Result<(), Self::Error> {
        self.ctx = context;
        Ok(())
    }

    fn dependencies() -> Vec<TypeId> {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Mock Resource
// ---------------------------------------------------------------------------

struct MockResource {
    id: ResourceId,
}

#[async_trait]
impl Resource for MockResource {
    type Config = String;
    type Error = ResourceError;

    async fn acquire(_config: Self::Config) -> Result<Self, Self::Error> {
        Ok(MockResource {
            id: ResourceId::new(),
        })
    }

    async fn release(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        true
    }

    async fn health_check(&self) -> Result<HealthStatus, Self::Error> {
        Ok(HealthStatus::Healthy)
    }

    fn resource_id(&self) -> ResourceId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Mock Supervisor
// ---------------------------------------------------------------------------

struct MockSupervisor {
    id: AgentId,
    strategy: SupervisionStrategy,
}

#[async_trait]
impl Supervisor for MockSupervisor {
    type Child = String;
    type Error = SupervisionError;

    async fn supervise(&self, _children: Vec<Self::Child>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn supervision_strategy(&self) -> &SupervisionStrategy {
        &self.strategy
    }

    fn restart_policy(&self) -> RestartPolicy {
        self.strategy.restart_policy
    }

    fn escalation_policy(&self) -> EscalationPolicy {
        self.strategy.escalation_policy
    }

    fn supervisor_id(&self) -> AgentId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Mock Transport
// ---------------------------------------------------------------------------

struct MockTransport {
    status: ConnectionStatus,
}

#[async_trait]
impl Transport for MockTransport {
    type Message = serde_json::Value;
    type Subscription = ();
    type ConnectionInfo = String;

    async fn send(&self, _destination: &str, _message: Self::Message) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn broadcast(&self, _topic: &str, _message: Self::Message) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn subscribe(&self, _pattern: &str) -> Result<Self::Subscription, NetworkError> {
        Ok(())
    }

    async fn request_response(
        &self,
        _destination: &str,
        _message: Self::Message,
        _timeout: Duration,
    ) -> Result<Self::Message, NetworkError> {
        Ok(serde_json::json!({}))
    }

    async fn connect(
        &mut self,
        _config: &TransportConfig,
    ) -> Result<Self::ConnectionInfo, NetworkError> {
        self.status = ConnectionStatus::Connected;
        Ok("connected".to_string())
    }

    async fn disconnect(&mut self) -> Result<(), NetworkError> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status
    }
}

// ---------------------------------------------------------------------------
// Mock EventPublisher
// ---------------------------------------------------------------------------

struct MockEventPublisher;

#[async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, _event: SystemEvent) -> Result<(), EventError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn actor_trait_compiles() {
    let actor = MockActor { id: AgentId::new() };
    assert_ne!(actor.actor_id().to_string(), "");
}

#[test]
fn tool_trait_compiles() {
    let tool = MockTool { id: ToolId::new() };
    assert_eq!(tool.version(), semver::Version::new(0, 1, 0));
}

#[test]
fn agent_extends_tool() {
    let agent = MockAgent {
        tool: MockTool { id: ToolId::new() },
        ctx: "test".to_string(),
    };
    assert_eq!(agent.role(), AgentType::Worker);
    assert_eq!(agent.context(), "test");
}

#[test]
fn supervisor_trait_compiles() {
    let supervisor = MockSupervisor {
        id: AgentId::new(),
        strategy: SupervisionStrategy::default(),
    };
    assert_eq!(supervisor.restart_policy(), RestartPolicy::OneForOne);
}

#[test]
fn resource_trait_compiles() {
    let resource = MockResource {
        id: ResourceId::new(),
    };
    assert!(resource.is_healthy());
    assert_ne!(resource.resource_id().to_string(), "");
}

#[test]
fn transport_trait_compiles() {
    let transport = MockTransport {
        status: ConnectionStatus::Disconnected,
    };
    assert_eq!(
        transport.connection_status(),
        ConnectionStatus::Disconnected
    );
}

#[test]
fn event_publisher_trait_compiles() {
    let _publisher = MockEventPublisher;
}

#[test]
fn trait_objects_are_object_safe() {
    // Verify Tool and EventPublisher can be used as trait objects
    fn _accepts_tool(_t: &dyn Tool) {}
    fn _accepts_publisher(_p: &dyn EventPublisher) {}
}
