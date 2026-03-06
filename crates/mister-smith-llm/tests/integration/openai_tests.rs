#[cfg(feature = "openai")]
use mister_smith_llm::OpenAiProvider;
use mister_smith_llm::{
    ChatMessage, CompletionRequest, ModelProvider, ProviderConfig, ProviderKind,
};
#[cfg(feature = "openai-chatgpt")]
use mister_smith_llm::{OpenAiChatGptProvider, StopReason};
use serde_json::json;

#[cfg(feature = "openai")]
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY and live OpenAI API access"]
async fn openai_provider_completes_against_live_api() {
    if std::env::var("OPENAI_API_KEY").is_err() {
        panic!("OPENAI_API_KEY must be set to run this ignored live-provider test");
    }

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        metadata: json!({
            "embedding_model_id": "text-embedding-3-small"
        }),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(CompletionRequest {
            messages: vec![ChatMessage::User {
                content: json!("Reply with the single word 'ack'."),
            }],
            max_tokens: Some(64),
            ..CompletionRequest::default()
        })
        .await
        .unwrap();

    assert!(!response.content.is_empty());
}

#[cfg(feature = "openai")]
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY and live OpenAI API access"]
async fn openai_provider_embeds_against_live_api() {
    if std::env::var("OPENAI_API_KEY").is_err() {
        panic!("OPENAI_API_KEY must be set to run this ignored live-provider test");
    }

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        metadata: json!({
            "embedding_model_id": "text-embedding-3-small"
        }),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .embed(vec!["Mister Smith".to_string()])
        .await
        .unwrap();

    assert_eq!(response.model_id, "text-embedding-3-small");
    assert_eq!(response.embeddings.len(), 1);
    assert!(!response.embeddings[0].is_empty());
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
#[ignore = "requires codex app-server with an active ChatGPT login"]
async fn openai_chatgpt_provider_completes_against_live_codex_app_server() {
    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(CompletionRequest {
            messages: vec![ChatMessage::User {
                content: json!("Reply with the single word 'ack'."),
            }],
            ..CompletionRequest::default()
        })
        .await
        .unwrap();

    assert!(!response.content.is_empty());
    assert_eq!(response.stop_reason, StopReason::Completed);
}
