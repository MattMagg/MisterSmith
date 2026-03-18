use serde::{Deserialize, Serialize};

use crate::types::{CompletionResponse, ContentBlock, StopReason};

/// Caller-supplied step metadata that identifies the workflow step being routed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepRoutingMetadata {
    pub step_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_kind: Option<String>,
}

/// Step-level action that downstream workflow consumers may take after routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepRoutingAction {
    Continue,
    Escalate,
    Downgrade,
    Fallback,
}

impl Default for StepRoutingAction {
    fn default() -> Self {
        Self::Continue
    }
}

/// Bounded checkpoint kinds emitted alongside a step routing signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepVerificationCheckpointKind {
    ConfidenceReview,
    BudgetPolicy,
    ProviderFailure,
    FinalTierGuard,
}

/// Outcome for a bounded verification checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepVerificationOutcome {
    Satisfied,
    Triggered,
    Skipped,
}

/// Verification checkpoint surfaced by the router before higher-level control loops react.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepVerificationCheckpoint {
    pub kind: StepVerificationCheckpointKind,
    pub outcome: StepVerificationOutcome,
    pub rationale: String,
}

/// Routing confidence signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceSignal {
    pub score: f32,
    pub source: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ConfidenceSignal {
    /// Heuristic confidence based on response properties.
    pub fn from_response(response: &CompletionResponse) -> Self {
        let mut score: f32 = 1.0;

        if response.stop_reason == StopReason::MaxTokens {
            score -= 0.3;
        }

        let total_text_len: usize = response
            .content
            .iter()
            .map(|block| match block {
                ContentBlock::Text { text } => text.len(),
                _ => 0,
            })
            .sum();
        if total_text_len < 10 {
            score -= 0.2;
        }

        if response.stop_reason == StopReason::ContentFilter {
            score -= 0.5;
        }

        Self {
            score: score.clamp(0.0, 1.0),
            source: "heuristic".to_string(),
            metadata: serde_json::json!({}),
        }
    }
}

/// Step-level routing signal returned by the live routing path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepRoutingSignal {
    pub metadata: StepRoutingMetadata,
    pub action: StepRoutingAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ConfidenceSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checkpoints: Vec<StepVerificationCheckpoint>,
}

impl Default for StepRoutingSignal {
    fn default() -> Self {
        Self {
            metadata: StepRoutingMetadata {
                step_id: "completion.request".to_string(),
                step_index: None,
                step_kind: Some("completion".to_string()),
            },
            action: StepRoutingAction::Continue,
            confidence: None,
            checkpoints: vec![],
        }
    }
}
