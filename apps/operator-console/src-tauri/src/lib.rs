use mister_smith_app::auth;
use mister_smith_llm::LlmError;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct OpenAiChatGptStatusPayload {
    authenticated: bool,
    account_type: Option<String>,
    email: Option<String>,
    plan_type: Option<String>,
    requires_openai_auth: bool,
    summary: String,
}

#[derive(Debug, Serialize)]
struct ClaudeSubscriptionStatusPayload {
    authenticated: bool,
    expired: bool,
    source: Option<String>,
    masked_token: Option<String>,
    summary: String,
}

#[tauri::command]
async fn openai_chatgpt_status() -> Result<OpenAiChatGptStatusPayload, String> {
    let status = auth::openai_chatgpt_status()
        .await
        .map_err(|error| error.to_string())?;
    let summary = auth::render_openai_chatgpt_status(&status);

    Ok(OpenAiChatGptStatusPayload {
        authenticated: status.is_chatgpt_session(),
        account_type: status.account_type,
        email: status.email,
        plan_type: status.plan_type,
        requires_openai_auth: status.requires_openai_auth,
        summary,
    })
}

#[tauri::command]
async fn login_openai_chatgpt() -> Result<OpenAiChatGptStatusPayload, String> {
    let status = auth::login_openai_chatgpt()
        .await
        .map_err(|error| error.to_string())?;
    let summary = auth::render_openai_chatgpt_status(&status);

    Ok(OpenAiChatGptStatusPayload {
        authenticated: status.is_chatgpt_session(),
        account_type: status.account_type,
        email: status.email,
        plan_type: status.plan_type,
        requires_openai_auth: status.requires_openai_auth,
        summary,
    })
}

#[tauri::command]
fn claude_subscription_status() -> Result<ClaudeSubscriptionStatusPayload, String> {
    match auth::claude_subscription_status() {
        Ok(creds) => Ok(ClaudeSubscriptionStatusPayload {
            authenticated: true,
            expired: creds.is_expired(),
            source: Some(creds.source.to_string()),
            masked_token: Some(creds.masked_token()),
            summary: auth::render_claude_subscription_status(&creds),
        }),
        Err(LlmError::Authentication(_)) => Ok(ClaudeSubscriptionStatusPayload {
            authenticated: false,
            expired: false,
            source: None,
            masked_token: None,
            summary: auth::render_claude_subscription_missing(),
        }),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_websocket::init())
        .invoke_handler(tauri::generate_handler![
            openai_chatgpt_status,
            login_openai_chatgpt,
            claude_subscription_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running operator console");
}
