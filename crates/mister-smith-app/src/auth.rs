use mister_smith_llm::{
    AppServerAccountStatus, ClaudeOAuthCredentials, CodexAppServerClient, LlmError,
};

/// Start the ChatGPT browser login flow and return the resulting account status.
pub async fn login_openai_chatgpt() -> Result<AppServerAccountStatus, LlmError> {
    login_openai_chatgpt_with(open_browser).await
}

/// Read and normalize the current ChatGPT authentication status.
pub async fn openai_chatgpt_status() -> Result<AppServerAccountStatus, LlmError> {
    let mut client = CodexAppServerClient::connect().await?;
    client.account_status(true).await
}

/// Render a human-readable ChatGPT authentication status line.
pub fn render_openai_chatgpt_status(status: &AppServerAccountStatus) -> String {
    if status.is_chatgpt_session() {
        match (&status.email, &status.plan_type) {
            (Some(email), Some(plan_type)) => {
                format!("Authenticated ChatGPT account: {email} ({plan_type})")
            }
            (Some(email), None) => format!("Authenticated ChatGPT account: {email}"),
            _ => "Authenticated ChatGPT account".to_string(),
        }
    } else if status.account_type.as_deref() == Some("apiKey") {
        "Codex is authenticated with an API key, not a ChatGPT subscription. Run `mister-smith auth openai-chatgpt login` to switch.".to_string()
    } else if !status.requires_openai_auth {
        "OpenAI authentication is not required by the active Codex provider.".to_string()
    } else {
        "ChatGPT authentication required. Run `mister-smith auth openai-chatgpt login`.".to_string()
    }
}

// ---------------------------------------------------------------------------
// Claude subscription authentication
// ---------------------------------------------------------------------------

/// Read the current Claude subscription credential status.
pub fn claude_subscription_status() -> Result<ClaudeOAuthCredentials, LlmError> {
    mister_smith_llm::claude_credentials::read_credentials()
}

/// Render a human-readable Claude subscription status line.
pub fn render_claude_subscription_status(creds: &ClaudeOAuthCredentials) -> String {
    let source = &creds.source;
    let masked = creds.masked_token();
    let expiry = if creds.is_expired() {
        " (EXPIRED — re-run `claude setup-token` to refresh)"
    } else {
        ""
    };
    format!("Claude subscription authenticated via {source}: {masked}{expiry}")
}

/// Render Claude subscription auth guidance when credentials are missing.
pub fn render_claude_subscription_missing() -> String {
    "No Claude subscription credentials found.\n\
     \n\
     To authenticate:\n\
     1. Install Claude Code CLI: https://docs.anthropic.com/en/docs/claude-code\n\
     2. Run `claude setup-token` to complete the OAuth login flow\n\
     3. Re-run `mister-smith auth claude status` to verify"
        .to_string()
}

async fn login_openai_chatgpt_with<F>(open: F) -> Result<AppServerAccountStatus, LlmError>
where
    F: FnOnce(&str) -> Result<(), LlmError>,
{
    let mut client = CodexAppServerClient::connect().await?;
    let login_handle = client.start_chatgpt_login().await?;
    eprintln!(
        "Starting ChatGPT login.\nIf your browser did not open, navigate to this URL to authenticate:\n\n{}\n",
        login_handle.auth_url
    );
    if let Err(error) = open(&login_handle.auth_url) {
        eprintln!(
            "Could not open your browser automatically: {error}\nContinue by visiting the URL above."
        );
    }
    client.wait_for_chatgpt_login(&login_handle).await
}

fn open_browser(url: &str) -> Result<(), LlmError> {
    webbrowser::open(url)
        .map(|_| ())
        .map_err(|error| LlmError::Network(format!("failed to open browser: {error}")))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn fake_codex_script_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mister-smith-app-fake-codex-{unique}.py"))
    }

    #[derive(Clone, Copy)]
    enum LoginNotificationMode {
        Completed,
        UpdatedOnly,
        MismatchedThenCompleted,
    }

    fn write_fake_codex_script(
        authenticated: bool,
        requires_openai_auth: bool,
        login_notification_mode: LoginNotificationMode,
    ) -> PathBuf {
        let path = fake_codex_script_path();
        let authenticated = if authenticated { "True" } else { "False" };
        let requires_openai_auth = if requires_openai_auth {
            "True"
        } else {
            "False"
        };
        let login_notifications = match login_notification_mode {
            LoginNotificationMode::Completed => {
                r#"
        send({
            "jsonrpc": "2.0",
            "method": "account/login/completed",
            "params": {
                "loginId": "login-1",
                "success": True,
                "error": None
            }
        })
"#
            }
            LoginNotificationMode::UpdatedOnly => {
                r#"
        send({
            "jsonrpc": "2.0",
            "method": "account/updated",
            "params": {
                "authMode": "chatgpt"
            }
        })
"#
            }
            LoginNotificationMode::MismatchedThenCompleted => {
                r#"
        send({
            "jsonrpc": "2.0",
            "method": "account/login/completed",
            "params": {
                "loginId": "other-login",
                "success": True,
                "error": None
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "account/login/completed",
            "params": {
                "loginId": "login-1",
                "success": True,
                "error": None
            }
        })
"#
            }
        };
        let script = format!(
            r#"#!/usr/bin/env python3
import json
import sys

AUTHENTICATED = {authenticated}

def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()

for raw in sys.stdin:
    if not raw.strip():
        continue

    message = json.loads(raw)
    method = message.get("method")

    if method == "initialize":
        send({{"jsonrpc": "2.0", "id": message["id"], "result": {{"userAgent": "fake-codex"}}}})
    elif method == "initialized":
        continue
    elif method == "account/read":
        account = {{"type": "chatgpt", "email": "ops@example.com", "planType": "team"}} if AUTHENTICATED else None
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "account": account,
                "requiresOpenaiAuth": {requires_openai_auth}
            }}
        }})
    elif method == "account/login/start":
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "type": "chatgpt",
                "loginId": "login-1",
                "authUrl": "https://example.test/login"
            }}
        }})
{login_notifications}
"#
        );

        fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }
        path
    }

    #[tokio::test]
    async fn login_flow_opens_browser_and_returns_status() {
        let _guard = env_lock().lock().unwrap();
        let script_path = write_fake_codex_script(true, true, LoginNotificationMode::Completed);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let mut opened_url = None;
        let status = login_openai_chatgpt_with(|url| {
            opened_url = Some(url.to_string());
            Ok(())
        })
        .await
        .unwrap();

        assert_eq!(opened_url.as_deref(), Some("https://example.test/login"));
        assert!(status.authenticated);
        assert_eq!(status.email.as_deref(), Some("ops@example.com"));
        assert_eq!(status.plan_type.as_deref(), Some("team"));

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }

    #[tokio::test]
    async fn status_reports_missing_login() {
        let _guard = env_lock().lock().unwrap();
        let script_path = write_fake_codex_script(false, true, LoginNotificationMode::Completed);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let status = openai_chatgpt_status().await.unwrap();
        assert!(!status.authenticated);
        assert!(status.requires_openai_auth);
        assert_eq!(
            render_openai_chatgpt_status(&status),
            "ChatGPT authentication required. Run `mister-smith auth openai-chatgpt login`."
        );

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }

    #[tokio::test]
    async fn login_flow_accepts_account_updated_notification() {
        let _guard = env_lock().lock().unwrap();
        let script_path = write_fake_codex_script(true, true, LoginNotificationMode::UpdatedOnly);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let status = login_openai_chatgpt_with(|_| Ok(())).await.unwrap();

        assert!(status.authenticated);
        assert_eq!(status.email.as_deref(), Some("ops@example.com"));
        assert_eq!(status.plan_type.as_deref(), Some("team"));

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }

    #[tokio::test]
    async fn status_reports_authenticated_chatgpt_when_openai_auth_is_required() {
        let _guard = env_lock().lock().unwrap();
        let script_path = write_fake_codex_script(true, true, LoginNotificationMode::Completed);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let status = openai_chatgpt_status().await.unwrap();

        assert!(status.authenticated);
        assert!(status.requires_openai_auth);
        assert_eq!(
            render_openai_chatgpt_status(&status),
            "Authenticated ChatGPT account: ops@example.com (team)"
        );

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }

    #[test]
    fn render_status_reports_api_key_mode_explicitly() {
        let status = AppServerAccountStatus {
            backend: "openai_chatgpt".to_string(),
            account_type: Some("apiKey".to_string()),
            authenticated: true,
            email: None,
            plan_type: None,
            requires_openai_auth: true,
        };

        assert_eq!(
            render_openai_chatgpt_status(&status),
            "Codex is authenticated with an API key, not a ChatGPT subscription. Run `mister-smith auth openai-chatgpt login` to switch."
        );
    }

    #[test]
    fn render_status_reports_when_openai_auth_is_not_required() {
        let status = AppServerAccountStatus {
            backend: "openai_chatgpt".to_string(),
            account_type: None,
            authenticated: false,
            email: None,
            plan_type: None,
            requires_openai_auth: false,
        };

        assert_eq!(
            render_openai_chatgpt_status(&status),
            "OpenAI authentication is not required by the active Codex provider."
        );
    }

    #[tokio::test]
    async fn login_flow_ignores_mismatched_login_completion_notifications() {
        let _guard = env_lock().lock().unwrap();
        let script_path =
            write_fake_codex_script(true, true, LoginNotificationMode::MismatchedThenCompleted);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let status = login_openai_chatgpt_with(|_| Ok(())).await.unwrap();

        assert!(status.authenticated);
        assert_eq!(status.account_type.as_deref(), Some("chatgpt"));
        assert_eq!(status.email.as_deref(), Some("ops@example.com"));

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }

    #[tokio::test]
    async fn login_flow_continues_when_browser_open_fails() {
        let _guard = env_lock().lock().unwrap();
        let script_path = write_fake_codex_script(true, true, LoginNotificationMode::Completed);
        std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

        let status = login_openai_chatgpt_with(|_| {
            Err(LlmError::Network("browser launch failed".to_string()))
        })
        .await
        .unwrap();

        assert!(status.authenticated);
        assert_eq!(status.email.as_deref(), Some("ops@example.com"));

        std::env::remove_var("MISTER_SMITH_CODEX_BIN");
        let _ = fs::remove_file(script_path);
    }
}
