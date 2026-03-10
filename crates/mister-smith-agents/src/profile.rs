//! Supervisory profile helpers for Guard / Advisor decisions.

use mister_smith_core::ProfileSnapshot;

/// Profile data plus operator-facing notes gathered before a Guard decision.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileAssessment {
    snapshot: Option<ProfileSnapshot>,
    notes: Vec<String>,
}

impl ProfileAssessment {
    /// Create a new profile assessment.
    pub fn new(snapshot: Option<ProfileSnapshot>, notes: Vec<String>) -> Self {
        Self { snapshot, notes }
    }

    /// Borrow the underlying profile snapshot when available.
    pub fn snapshot(&self) -> Option<&ProfileSnapshot> {
        self.snapshot.as_ref()
    }

    /// Borrow operator-facing notes captured during profile assessment.
    pub fn notes(&self) -> &[String] {
        &self.notes
    }
}
