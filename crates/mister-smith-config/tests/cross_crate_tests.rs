//! Cross-crate integration tests verifying mister-smith-config can use
//! types from mister-smith-core.

use mister_smith_core::{AgentId, SupervisionStrategy, SystemError};

#[test]
fn can_use_core_agent_id() {
    let id = AgentId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn can_use_core_supervision_strategy() {
    let strategy = SupervisionStrategy::default();
    assert_eq!(strategy.max_failures, 3);
}

#[test]
fn can_reference_core_system_error() {
    fn _accepts_error(_e: SystemError) {}
}

#[test]
fn config_and_core_compose() {
    // Verify we can use config types alongside core types in the same scope
    let _config = mister_smith_config::FrameworkConfig::default();
    let _id = AgentId::new();
    let _strategy = SupervisionStrategy::default();
}
