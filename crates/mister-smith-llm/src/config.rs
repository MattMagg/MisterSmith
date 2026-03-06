use std::fmt;

use serde::{Deserialize, Serialize};

use crate::LlmError;

/// Runtime configuration for a selected model provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Backend selector for the configured provider implementation.
    pub provider_kind: ProviderKind,
    /// Canonical provider/model identifier surfaced through the shared API.
    pub model_id: String,
    /// Optional base URL override for proxies or local test endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
    /// Environment-variable name containing the provider credential.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    /// Upper bound for a provider call in milliseconds.
    #[serde(default = "ProviderConfig::default_timeout_ms")]
    pub timeout_ms: u64,
    /// Retry budget for retryable provider failures.
    #[serde(default)]
    pub max_retries: u32,
    /// Provider-specific configuration that stays out of public request types.
    #[serde(default = "ProviderConfig::default_metadata")]
    pub metadata: serde_json::Value,
}

/// Supported LLM provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderKind {
    /// Anthropic API-key provider.
    #[serde(rename = "anthropic")]
    Anthropic,
    /// OpenAI usage-billed API-key provider.
    #[serde(rename = "openai")]
    OpenAi,
    /// ChatGPT-subscription provider backed by Codex app-server.
    #[serde(rename = "openai_chatgpt")]
    OpenAiChatGpt,
    /// Claude subscription provider using OAuth Bearer tokens from Claude Code CLI.
    #[serde(rename = "claude_subscription")]
    ClaudeSubscription,
    /// Deterministic mock provider used for tests and local development.
    #[default]
    #[serde(rename = "mock")]
    Mock,
}

impl ProviderKind {
    /// Return the canonical serialized provider name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::OpenAiChatGpt => "openai_chatgpt",
            Self::ClaudeSubscription => "claude_subscription",
            Self::Mock => "mock",
        }
    }

    /// Whether the provider requires an API key environment variable.
    pub const fn requires_api_key(self) -> bool {
        matches!(self, Self::Anthropic | Self::OpenAi)
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ProviderConfig {
    const fn default_timeout_ms() -> u64 {
        30_000
    }

    fn default_metadata() -> serde_json::Value {
        serde_json::json!({})
    }

    /// Validate that the provider selection and credentials are internally consistent.
    pub fn validate(&self) -> Result<(), LlmError> {
        if self.model_id.trim().is_empty() {
            return Err(LlmError::InvalidRequest(
                "model_id must not be empty".to_string(),
            ));
        }

        if self.timeout_ms == 0 {
            return Err(LlmError::InvalidRequest(
                "timeout_ms must be greater than 0".to_string(),
            ));
        }

        if let Some(api_key_env) = &self.api_key_env {
            if api_key_env.trim().is_empty() {
                return Err(LlmError::InvalidRequest(
                    "api_key_env must not be empty when provided".to_string(),
                ));
            }
        }

        if self.provider_kind.requires_api_key() && self.api_key_env.is_none() {
            return Err(LlmError::InvalidRequest(format!(
                "provider '{}' requires api_key_env",
                self.provider_kind
            )));
        }

        Ok(())
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_kind: ProviderKind::Mock,
            model_id: "mock-default".to_string(),
            api_base_url: None,
            api_key_env: None,
            timeout_ms: Self::default_timeout_ms(),
            max_retries: 0,
            metadata: Self::default_metadata(),
        }
    }
}
