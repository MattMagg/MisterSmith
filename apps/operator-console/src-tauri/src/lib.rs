mod managed_runtime;

use managed_runtime::{ManagedRuntimeManager, ManagedRuntimeStatusPayload};
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

#[tauri::command]
async fn managed_runtime_status(
    runtime_manager: tauri::State<'_, ManagedRuntimeManager>,
) -> Result<ManagedRuntimeStatusPayload, String> {
    Ok(runtime_manager.snapshot().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime_manager = ManagedRuntimeManager::new();
    let setup_runtime_manager = runtime_manager.clone();
    let shutdown_runtime_manager = runtime_manager.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_websocket::init())
        .manage(runtime_manager)
        .setup(move |app| {
            setup_runtime_manager.ensure_started(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            openai_chatgpt_status,
            login_openai_chatgpt,
            claude_subscription_status,
            managed_runtime_status
        ])
        .build(tauri::generate_context!())
        .expect("error while building operator console");

    app.run(move |_app_handle, event| {
        if matches!(
            event,
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
        ) {
            shutdown_runtime_manager.shutdown();
        }
    });
}
