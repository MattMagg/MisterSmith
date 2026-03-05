//! Agent availability for transport-level presence tracking.
//!
//! Re-exports `AgentAvailability` from `mister-smith-core` and adds
//! transport-specific transition validation.

pub use mister_smith_core::AgentAvailability;

/// Validates whether transitioning from `current` to `next` is allowed.
///
/// Invalid transitions:
/// - `Offline` -> `Busy` (must go through `Idle` first)
pub fn is_valid_transition(current: AgentAvailability, next: AgentAvailability) -> bool {
    !matches!(
        (current, next),
        (AgentAvailability::Offline, AgentAvailability::Busy)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        assert!(is_valid_transition(
            AgentAvailability::Idle,
            AgentAvailability::Busy
        ));
        assert!(is_valid_transition(
            AgentAvailability::Busy,
            AgentAvailability::Idle
        ));
        assert!(is_valid_transition(
            AgentAvailability::Idle,
            AgentAvailability::Offline
        ));
        assert!(is_valid_transition(
            AgentAvailability::Busy,
            AgentAvailability::Offline
        ));
        assert!(is_valid_transition(
            AgentAvailability::Offline,
            AgentAvailability::Idle
        ));
    }

    #[test]
    fn invalid_transition_offline_to_busy() {
        assert!(!is_valid_transition(
            AgentAvailability::Offline,
            AgentAvailability::Busy
        ));
    }

    #[test]
    fn same_state_transition_is_valid() {
        assert!(is_valid_transition(
            AgentAvailability::Idle,
            AgentAvailability::Idle
        ));
        assert!(is_valid_transition(
            AgentAvailability::Busy,
            AgentAvailability::Busy
        ));
    }
}
