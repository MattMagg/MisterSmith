//! Claude subscription credential management.
//!
//! Reads OAuth tokens from the Claude Code CLI credential stores:
//! 1. `CLAUDE_CODE_OAUTH_TOKEN` environment variable (highest priority)
//! 2. macOS Keychain (`Claude Code-credentials`)
//! 3. `~/.claude/.credentials.json` (Linux / fallback)
//!
//! Token refresh uses the stored refresh token against Anthropic's OAuth endpoint.
//! The OAuth client ID is inferred from observed Claude Code CLI behavior — Anthropic
//! does not publicly document the subscription token refresh protocol.

use serde_json::Value;

use crate::LlmError;

const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
const CREDENTIALS_FILE: &str = ".claude/.credentials.json";
const OAUTH_TOKEN_ENV: &str = "CLAUDE_CODE_OAUTH_TOKEN";
const OAUTH_TOKEN_ENDPOINT: &str = "https://console.anthropic.com/api/oauth/token";
/// Claude Code CLI's registered OAuth client ID (public client, no secret).
/// Inferred from observed PKCE flow — not officially documented by Anthropic.
const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// OAuth credentials extracted from Claude Code's credential store.
#[derive(Debug, Clone)]
pub struct ClaudeOAuthCredentials {
    /// Bearer access token (`sk-ant-oat01-...`).
    pub access_token: String,
    /// Refresh token for token renewal (`sk-ant-ort01-...`).
    pub refresh_token: Option<String>,
    /// Expiry timestamp in milliseconds since Unix epoch.
    pub expires_at: Option<u64>,
    /// Where the credentials were loaded from.
    pub source: CredentialSource,
}

/// Where Claude credentials were loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    /// `CLAUDE_CODE_OAUTH_TOKEN` environment variable.
    Environment,
    /// macOS Keychain (`Claude Code-credentials`).
    Keychain,
    /// `~/.claude/.credentials.json` file.
    CredentialsFile,
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment => f.write_str("CLAUDE_CODE_OAUTH_TOKEN env var"),
            Self::Keychain => f.write_str("macOS Keychain"),
            Self::CredentialsFile => f.write_str("~/.claude/.credentials.json"),
        }
    }
}

impl ClaudeOAuthCredentials {
    /// Whether the access token has expired or will expire within 60 seconds.
    pub fn is_expired(&self) -> bool {
        let Some(expires_at) = self.expires_at else {
            return false;
        };
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now_ms + 60_000 >= expires_at
    }

    /// Return a masked display version of the access token for status output.
    pub fn masked_token(&self) -> String {
        if self.access_token.len() > 20 {
            format!("{}***", &self.access_token[..20])
        } else {
            "***".to_string()
        }
    }
}

/// Read Claude OAuth credentials from available stores.
///
/// Priority:
/// 1. `CLAUDE_CODE_OAUTH_TOKEN` environment variable
/// 2. macOS Keychain
/// 3. `~/.claude/.credentials.json`
pub fn read_credentials() -> Result<ClaudeOAuthCredentials, LlmError> {
    if let Ok(token) = std::env::var(OAUTH_TOKEN_ENV) {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(ClaudeOAuthCredentials {
                access_token: token,
                refresh_token: None,
                expires_at: None,
                source: CredentialSource::Environment,
            });
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(creds) = read_keychain_credentials()? {
        return Ok(creds);
    }

    if let Some(creds) = read_credentials_file()? {
        return Ok(creds);
    }

    Err(LlmError::Authentication(
        "No Claude subscription credentials found. Authenticate with Claude Code CLI first, or set CLAUDE_CODE_OAUTH_TOKEN.".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn read_keychain_credentials() -> Result<Option<ClaudeOAuthCredentials>, LlmError> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let json_str = String::from_utf8_lossy(&output.stdout);
            let json_str = json_str.trim();
            if json_str.is_empty() {
                return Ok(None);
            }
            parse_credentials_json(json_str, CredentialSource::Keychain).map(Some)
        }
        _ => Ok(None),
    }
}

fn read_credentials_file() -> Result<Option<ClaudeOAuthCredentials>, LlmError> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return Ok(None);
    }
    let path = std::path::PathBuf::from(&home).join(CREDENTIALS_FILE);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            parse_credentials_json(trimmed, CredentialSource::CredentialsFile).map(Some)
        }
        Err(_) => Ok(None),
    }
}

/// Parse the Claude credentials JSON blob.
///
/// Expected format:
/// ```json
/// {
///   "claudeAiOauth": {
///     "accessToken": "sk-ant-oat01-...",
///     "refreshToken": "sk-ant-ort01-...",
///     "expiresAt": 1748658860401
///   }
/// }
/// ```
fn parse_credentials_json(
    json_str: &str,
    source: CredentialSource,
) -> Result<ClaudeOAuthCredentials, LlmError> {
    let value: Value = serde_json::from_str(json_str).map_err(|error| {
        LlmError::Serialization(format!("Failed to parse Claude credentials JSON: {error}"))
    })?;

    let oauth = value.get("claudeAiOauth").ok_or_else(|| {
        LlmError::Authentication("Claude credentials missing 'claudeAiOauth' key".to_string())
    })?;

    let access_token = oauth
        .get("accessToken")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            LlmError::Authentication("Claude credentials missing 'accessToken'".to_string())
        })?
        .to_string();

    let refresh_token = oauth
        .get("refreshToken")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let expires_at = oauth.get("expiresAt").and_then(Value::as_u64);

    Ok(ClaudeOAuthCredentials {
        access_token,
        refresh_token,
        expires_at,
        source,
    })
}

/// Refresh an expired access token using the stored refresh token.
///
/// Uses Anthropic's OAuth token endpoint with the Claude Code CLI client ID.
/// The client ID and endpoint are inferred from observed CLI behavior.
pub async fn refresh_access_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<ClaudeOAuthCredentials, LlmError> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", OAUTH_CLIENT_ID),
    ];

    let response = client
        .post(OAUTH_TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|error| LlmError::Network(format!("Claude token refresh failed: {error}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(LlmError::Authentication(format!(
            "Claude token refresh returned {status}: {body}"
        )));
    }

    let payload: Value = response.json().await.map_err(|error| {
        LlmError::Serialization(format!("Failed to parse token refresh response: {error}"))
    })?;

    let access_token = payload
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            LlmError::Serialization(
                "Token refresh response missing access_token".to_string(),
            )
        })?
        .to_string();

    let new_refresh_token = payload
        .get("refresh_token")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let expires_in = payload.get("expires_in").and_then(Value::as_u64);

    let expires_at = expires_in.map(|secs| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64 + secs * 1000)
            .unwrap_or(0)
    });

    Ok(ClaudeOAuthCredentials {
        access_token,
        refresh_token: new_refresh_token.or_else(|| Some(refresh_token.to_string())),
        expires_at,
        source: CredentialSource::Keychain, // Refreshed tokens inherit source context
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_credentials_json() {
        let json = r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-test","refreshToken":"sk-ant-ort01-refresh","expiresAt":9999999999999}}"#;
        let creds = parse_credentials_json(json, CredentialSource::CredentialsFile).unwrap();
        assert_eq!(creds.access_token, "sk-ant-oat01-test");
        assert_eq!(creds.refresh_token.as_deref(), Some("sk-ant-ort01-refresh"));
        assert_eq!(creds.expires_at, Some(9999999999999));
        assert_eq!(creds.source, CredentialSource::CredentialsFile);
    }

    #[test]
    fn parse_credentials_without_refresh_token() {
        let json = r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-test"}}"#;
        let creds = parse_credentials_json(json, CredentialSource::Keychain).unwrap();
        assert_eq!(creds.access_token, "sk-ant-oat01-test");
        assert!(creds.refresh_token.is_none());
        assert!(creds.expires_at.is_none());
        assert!(!creds.is_expired());
    }

    #[test]
    fn parse_credentials_missing_oauth_key() {
        let json = r#"{"other": "data"}"#;
        let err = parse_credentials_json(json, CredentialSource::Keychain).unwrap_err();
        assert!(matches!(err, LlmError::Authentication(msg) if msg.contains("claudeAiOauth")));
    }

    #[test]
    fn parse_credentials_empty_access_token() {
        let json = r#"{"claudeAiOauth":{"accessToken":""}}"#;
        let err = parse_credentials_json(json, CredentialSource::Keychain).unwrap_err();
        assert!(matches!(err, LlmError::Authentication(msg) if msg.contains("accessToken")));
    }

    #[test]
    fn parse_credentials_invalid_json() {
        let err = parse_credentials_json("not json", CredentialSource::Keychain).unwrap_err();
        assert!(matches!(err, LlmError::Serialization(_)));
    }

    #[test]
    fn expired_token_detected() {
        let creds = ClaudeOAuthCredentials {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(1000), // Long past
            source: CredentialSource::Environment,
        };
        assert!(creds.is_expired());
    }

    #[test]
    fn future_token_not_expired() {
        let creds = ClaudeOAuthCredentials {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(9999999999999),
            source: CredentialSource::Environment,
        };
        assert!(!creds.is_expired());
    }

    #[test]
    fn no_expiry_not_expired() {
        let creds = ClaudeOAuthCredentials {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            source: CredentialSource::Environment,
        };
        assert!(!creds.is_expired());
    }

    #[test]
    fn masked_token_output() {
        let creds = ClaudeOAuthCredentials {
            access_token: "sk-ant-oat01-GY885m1l-ZN7zIkAkUKw7flPAUGnwVEe-RABCDEFGH".to_string(),
            refresh_token: None,
            expires_at: None,
            source: CredentialSource::Keychain,
        };
        let masked = creds.masked_token();
        assert!(masked.starts_with("sk-ant-oat01-GY885m1"));
        assert!(masked.ends_with("***"));
        assert!(!masked.contains("ABCDEFGH"));
    }

    #[test]
    fn credential_source_display() {
        assert_eq!(
            CredentialSource::Environment.to_string(),
            "CLAUDE_CODE_OAUTH_TOKEN env var"
        );
        assert_eq!(CredentialSource::Keychain.to_string(), "macOS Keychain");
        assert_eq!(
            CredentialSource::CredentialsFile.to_string(),
            "~/.claude/.credentials.json"
        );
    }
}
