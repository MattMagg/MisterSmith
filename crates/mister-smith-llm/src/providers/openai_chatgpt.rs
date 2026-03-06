use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::app_server::CodexAppServerClient;
use crate::config::{ProviderConfig, ProviderKind};
use crate::provider::{CompletionStream, ModelProvider};
use crate::types::{CompletionRequest, CompletionResponse, EmbeddingResponse, ModelCapabilities};
use crate::LlmError;

/// ChatGPT-subscription provider backed by the official Codex app-server flow.
#[derive(Debug, Clone)]
pub struct OpenAiChatGptProvider {
    config: ProviderConfig,
}

impl OpenAiChatGptProvider {
    /// Construct a new ChatGPT-backed OpenAI provider.
    pub fn new(config: ProviderConfig) -> Result<Self, LlmError> {
        config.validate()?;
        if config.provider_kind != ProviderKind::OpenAiChatGpt {
            return Err(LlmError::InvalidRequest(format!(
                "OpenAiChatGptProvider requires provider_kind 'openai_chatgpt', got '{}'",
                config.provider_kind
            )));
        }

        Ok(Self { config })
    }

    async fn ensure_authenticated(client: &mut CodexAppServerClient) -> Result<(), LlmError> {
        let status = client.account_status(false).await?;
        if status.is_chatgpt_session() {
            Ok(())
        } else if status.account_type.as_deref() == Some("apiKey") {
            Err(LlmError::Authentication(
                "Codex is authenticated with an API key; run `mister-smith auth openai-chatgpt login` to switch to ChatGPT subscription authentication"
                    .to_string(),
            ))
        } else {
            Err(LlmError::Authentication(
                "ChatGPT authentication required; run `mister-smith auth openai-chatgpt login`"
                    .to_string(),
            ))
        }
    }

    fn unsupported_capability(&self, capability: &str) -> LlmError {
        LlmError::UnsupportedCapability {
            capability: capability.to_string(),
            model: self.config.model_id.clone(),
        }
    }

    fn validate_request(&self, request: &CompletionRequest) -> Result<(), LlmError> {
        if request.tools.is_some() {
            return Err(self.unsupported_capability("tool_calling"));
        }

        if request.temperature.is_some() {
            return Err(self.unsupported_capability("temperature"));
        }

        if request.max_tokens.is_some() {
            return Err(self.unsupported_capability("max_tokens"));
        }

        if request
            .stop_sequences
            .as_ref()
            .is_some_and(|stop_sequences| !stop_sequences.is_empty())
        {
            return Err(self.unsupported_capability("stop_sequences"));
        }

        Ok(())
    }
}

#[async_trait]
impl ModelProvider for OpenAiChatGptProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.validate_request(&request)?;

        let mut client = CodexAppServerClient::connect().await?;
        Self::ensure_authenticated(&mut client).await?;
        client
            .run_completion(&self.config.model_id, request, None)
            .await
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        let provider = self.clone();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            if let Err(error) = provider.validate_request(&request) {
                let _ = tx.send(Err(error)).await;
                return;
            }

            match CodexAppServerClient::connect().await {
                Ok(mut client) => {
                    if let Err(error) = Self::ensure_authenticated(&mut client).await {
                        let _ = tx.send(Err(error)).await;
                        return;
                    }

                    if let Err(error) = client
                        .run_completion(&provider.config.model_id, request, Some(tx.clone()))
                        .await
                    {
                        let _ = tx.send(Err(error)).await;
                    }
                }
                Err(error) => {
                    let _ = tx.send(Err(error)).await;
                }
            }
        });

        Box::pin(futures::stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|item| (item, rx))
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".to_string(),
            model: self.config.model_id.clone(),
        })
    }

    fn model_id(&self) -> &str {
        &self.config.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            completion: true,
            streaming: true,
            embeddings: false,
            tool_calling: false,
        }
    }
}
