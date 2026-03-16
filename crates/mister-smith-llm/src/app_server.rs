use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::streaming::{ChunkDelta, StreamChunk};
use crate::types::{CompletionRequest, CompletionResponse, ContentBlock, StopReason, Usage};
use crate::LlmError;

const DEFAULT_CODEX_BIN: &str = "codex";
const CODEX_BIN_ENV: &str = "MISTER_SMITH_CODEX_BIN";
const CODEX_LOGIN_TIMEOUT_ENV: &str = "MISTER_SMITH_OPENAI_CHATGPT_LOGIN_TIMEOUT_MS";
const DEFAULT_LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const OPT_OUT_NOTIFICATION_METHODS: &[&str] = &[
    "codex/event/warning",
    "codex/event/mcp_startup_update",
    "codex/event/mcp_startup_complete",
    "codex/event/task_started",
    "codex/event/item_started",
    "codex/event/item_completed",
    "codex/event/user_message",
    "codex/event/agent_message_content_delta",
    "codex/event/agent_message_delta",
    "codex/event/agent_message",
    "codex/event/token_count",
    "codex/event/task_complete",
];
const NON_STREAMING_NOTIFICATION_OPT_OUT_METHODS: &[&str] = &[
    "thread/started",
    "thread/status/changed",
    "turn/started",
    "item/started",
    "item/agentMessage/delta",
];

/// Login flow handle returned by Codex app-server for ChatGPT browser authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatGptLoginHandle {
    /// Stable login identifier returned by `account/login/start`.
    pub login_id: String,
    /// Browser URL the operator should open to complete login.
    pub auth_url: String,
}

/// Normalized Codex app-server account state for the ChatGPT-backed provider path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppServerAccountStatus {
    /// Stable backend identifier for the ChatGPT-backed path.
    pub backend: String,
    /// Raw account type reported by Codex app-server, if any.
    pub account_type: Option<String>,
    /// Whether app-server currently has an authenticated ChatGPT session.
    pub authenticated: bool,
    /// ChatGPT account email when available.
    pub email: Option<String>,
    /// ChatGPT plan type when available.
    pub plan_type: Option<String>,
    /// Whether app-server indicates OpenAI authentication is still required.
    pub requires_openai_auth: bool,
}

impl AppServerAccountStatus {
    /// Convert the raw `account/read` JSON payload into a normalized status view.
    pub fn from_account_read_payload(payload: &Value) -> Result<Self, LlmError> {
        let requires_openai_auth = payload
            .get("requiresOpenaiAuth")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server account/read payload missing requiresOpenaiAuth".into(),
                )
            })?;

        let account = payload.get("account");
        let (account_type, authenticated, email, plan_type) = match account {
            Some(Value::Object(account)) => {
                let account_type = account
                    .get("type")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        LlmError::Serialization(
                            "Codex app-server account/read payload missing account.type".into(),
                        )
                    })?
                    .to_string();

                let (email, plan_type) = if account_type == "chatgpt" {
                    let email = account
                        .get("email")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                    let plan_type = account
                        .get("planType")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                    (email, plan_type)
                } else {
                    (None, None)
                };

                (Some(account_type), true, email, plan_type)
            }
            Some(Value::Null) | None => (None, false, None, None),
            Some(_) => {
                return Err(LlmError::Serialization(
                    "Codex app-server account/read payload contained invalid account shape".into(),
                ))
            }
        };

        Ok(Self {
            backend: "openai_chatgpt".to_string(),
            account_type,
            authenticated,
            email,
            plan_type,
            requires_openai_auth,
        })
    }
}

impl AppServerAccountStatus {
    /// Whether the current app-server account is a ChatGPT-backed session.
    pub fn is_chatgpt_session(&self) -> bool {
        self.account_type.as_deref() == Some("chatgpt")
    }
}

#[derive(Debug)]
enum RpcMessage {
    Response {
        id: u64,
        result: Value,
    },
    Notification {
        method: String,
        params: Value,
    },
    Request {
        id: Value,
        method: String,
        _params: Value,
    },
    Error {
        id: Option<u64>,
        error: Value,
    },
}

#[derive(Debug, Default)]
struct CompletionTurnState {
    rerouted_model_id: Option<String>,
    usage: Usage,
    agent_messages: HashMap<String, String>,
    completed_agent_message_ids: Vec<String>,
}

impl CompletionTurnState {
    fn apply_agent_delta(&mut self, params: &Value) -> Result<String, LlmError> {
        let item_id = params
            .get("itemId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server agent delta payload missing itemId".to_string(),
                )
            })?;
        let delta = params.get("delta").and_then(Value::as_str).ok_or_else(|| {
            LlmError::Serialization(
                "Codex app-server agent delta payload missing delta".to_string(),
            )
        })?;

        self.agent_messages
            .entry(item_id.to_string())
            .or_default()
            .push_str(delta);

        Ok(delta.to_string())
    }

    fn apply_item_completed(&mut self, params: &Value) -> Result<Option<String>, LlmError> {
        let item = params.get("item").ok_or_else(|| {
            LlmError::Serialization(
                "Codex app-server item/completed payload missing item".to_string(),
            )
        })?;
        if item.get("type").and_then(Value::as_str) != Some("agentMessage") {
            return Ok(None);
        }

        let item_id = item.get("id").and_then(Value::as_str).ok_or_else(|| {
            LlmError::Serialization("Codex app-server agentMessage item missing id".to_string())
        })?;
        let final_text = item.get("text").and_then(Value::as_str).ok_or_else(|| {
            LlmError::Serialization("Codex app-server agentMessage item missing text".to_string())
        })?;

        let streamed_text = self
            .agent_messages
            .get(item_id)
            .cloned()
            .unwrap_or_default();
        self.agent_messages
            .insert(item_id.to_string(), final_text.to_string());

        if !self
            .completed_agent_message_ids
            .iter()
            .any(|existing| existing == item_id)
        {
            self.completed_agent_message_ids.push(item_id.to_string());
        }

        if streamed_text.is_empty() {
            return Ok((!final_text.is_empty()).then(|| final_text.to_string()));
        }

        Ok(final_text
            .strip_prefix(&streamed_text)
            .filter(|suffix| !suffix.is_empty())
            .map(ToOwned::to_owned))
    }

    fn apply_model_rerouted(&mut self, params: &Value) -> Result<(), LlmError> {
        let to_model = params
            .get("toModel")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server model/rerouted payload missing toModel".to_string(),
                )
            })?;
        self.rerouted_model_id = Some(to_model.to_string());
        Ok(())
    }

    fn apply_token_usage(&mut self, params: &Value) -> Result<(), LlmError> {
        let total = params
            .get("tokenUsage")
            .and_then(|value| value.get("total"))
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server token usage payload missing tokenUsage.total".to_string(),
                )
            })?;

        self.usage = Usage {
            input_tokens: required_u64_field(total, "inputTokens")?,
            output_tokens: required_u64_field(total, "outputTokens")?,
            total_tokens: required_u64_field(total, "totalTokens")?,
        };
        Ok(())
    }

    fn final_content(&self, streamed_content: &str) -> String {
        if !self.completed_agent_message_ids.is_empty() {
            let final_text = self
                .completed_agent_message_ids
                .iter()
                .filter_map(|item_id| self.agent_messages.get(item_id))
                .fold(String::new(), |mut combined, text| {
                    combined.push_str(text);
                    combined
                });
            if !final_text.is_empty() {
                return final_text;
            }
        }

        if self.agent_messages.len() == 1 {
            return self
                .agent_messages
                .values()
                .next()
                .cloned()
                .unwrap_or_default();
        }

        streamed_content.to_string()
    }

    fn final_model_id<'a>(&'a self, requested_model_id: &'a str) -> &'a str {
        self.rerouted_model_id
            .as_deref()
            .unwrap_or(requested_model_id)
    }
}

/// Thin JSON-RPC client for the Codex app-server stdio transport.
#[derive(Debug)]
pub struct CodexAppServerClient {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    #[allow(dead_code)]
    stderr: Option<ChildStderr>,
    next_id: u64,
    buffered: VecDeque<RpcMessage>,
}

impl CodexAppServerClient {
    /// Spawn and initialize a new Codex app-server client.
    pub async fn connect() -> Result<Self, LlmError> {
        Self::connect_with_opt_out(NON_STREAMING_NOTIFICATION_OPT_OUT_METHODS).await
    }

    /// Spawn and initialize a new Codex app-server client with streaming notifications enabled.
    pub async fn connect_streaming() -> Result<Self, LlmError> {
        Self::connect_with_opt_out(&[]).await
    }

    async fn connect_with_opt_out(extra_opt_out: &[&str]) -> Result<Self, LlmError> {
        let isolated_cwd = isolated_codex_cwd()?;
        let mut command = Command::new(codex_binary_path());
        command
            .arg("app-server")
            .arg("--listen")
            .arg("stdio://")
            .current_dir(&isolated_cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|error| {
            LlmError::Network(format!("failed to launch codex app-server: {error}"))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            LlmError::Network("failed to capture codex app-server stdin".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            LlmError::Network("failed to capture codex app-server stdout".to_string())
        })?;
        let stderr = child.stderr.take();

        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            stderr,
            next_id: 1,
            buffered: VecDeque::new(),
        };

        client.initialize(extra_opt_out).await?;
        Ok(client)
    }

    /// Read the current ChatGPT authentication status from Codex app-server.
    pub async fn account_status(
        &mut self,
        refresh_token: bool,
    ) -> Result<AppServerAccountStatus, LlmError> {
        let payload = self
            .request("account/read", json!({ "refreshToken": refresh_token }))
            .await?;
        AppServerAccountStatus::from_account_read_payload(&payload)
    }

    /// Start the browser-based ChatGPT authentication flow.
    pub async fn start_chatgpt_login(&mut self) -> Result<ChatGptLoginHandle, LlmError> {
        let payload = self
            .request("account/login/start", json!({ "type": "chatgpt" }))
            .await?;

        let login_id = payload
            .get("loginId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server login response missing loginId".to_string(),
                )
            })?;
        let auth_url = payload
            .get("authUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server login response missing authUrl".to_string(),
                )
            })?;

        Ok(ChatGptLoginHandle {
            login_id: login_id.to_string(),
            auth_url: auth_url.to_string(),
        })
    }

    async fn cancel_chatgpt_login(&mut self, login_id: &str) -> Result<(), LlmError> {
        self.request("account/login/cancel", json!({ "loginId": login_id }))
            .await
            .map(|_| ())
    }

    /// Wait for the browser-based ChatGPT login to complete and return the final account status.
    pub async fn wait_for_chatgpt_login(
        &mut self,
        login_handle: &ChatGptLoginHandle,
    ) -> Result<AppServerAccountStatus, LlmError> {
        let timeout_duration = login_timeout();
        let login_result = timeout(timeout_duration, async {
            loop {
                match self.next_message().await? {
                    RpcMessage::Notification { method, params }
                        if method == "account/login/completed" =>
                    {
                        let login_id_matches = params
                            .get("loginId")
                            .and_then(Value::as_str)
                            .map(|value| value == login_handle.login_id)
                            .unwrap_or(false);
                        if !login_id_matches {
                            continue;
                        }

                        let success = params
                            .get("success")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if !success {
                            let message = params
                                .get("error")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown ChatGPT login error");
                            return Err(LlmError::Authentication(message.to_string()));
                        }

                        return self.account_status(true).await;
                    }
                    RpcMessage::Notification { method, .. } if method == "account/updated" => {
                        let status = self.account_status(true).await?;
                        if status.is_chatgpt_session() {
                            return Ok(status);
                        }
                    }
                    RpcMessage::Notification { .. } | RpcMessage::Response { .. } => continue,
                    RpcMessage::Request { id, method, .. } => {
                        self.reply_with_error(
                            id,
                            -32_600,
                            format!("unexpected app-server request during login flow: {method}"),
                        )
                        .await?;
                    }
                    RpcMessage::Error { error, .. } => {
                        return Err(normalize_rpc_error("login flow", &error));
                    }
                }
            }
        })
        .await;

        match login_result {
            Ok(Ok(status)) => Ok(status),
            Ok(Err(error @ LlmError::Authentication(_))) => Err(error),
            Ok(Err(error)) => {
                let _ = self.cancel_chatgpt_login(&login_handle.login_id).await;
                Err(error)
            }
            Err(_) => {
                let _ = self.cancel_chatgpt_login(&login_handle.login_id).await;
                Err(LlmError::Authentication(format!(
                    "timed out waiting for ChatGPT login to complete after {}ms",
                    timeout_duration.as_millis()
                )))
            }
        }
    }

    /// Execute a ChatGPT-backed completion through Codex app-server.
    pub async fn run_completion(
        &mut self,
        model_id: &str,
        request: CompletionRequest,
        stream_tx: Option<mpsc::Sender<Result<StreamChunk, LlmError>>>,
    ) -> Result<CompletionResponse, LlmError> {
        let isolated_cwd = isolated_codex_cwd()?;
        let thread = self
            .request(
                "thread/start",
                json!({
                    "model": model_id,
                    "modelProvider": "openai",
                    "cwd": isolated_cwd.clone(),
                    "approvalPolicy": "never",
                    "sandbox": "read-only",
                    "serviceName": "mister-smith",
                    "baseInstructions": "You are serving as a provider-backed completion engine for Mister Smith. Do not execute tools, run commands, inspect files, or browse the current project. Respond only to the current conversational input.",
                    "developerInstructions": render_codex_developer_instructions(&request),
                    "ephemeral": true,
                    "experimentalRawEvents": false,
                    "persistExtendedHistory": false,
                    "personality": "pragmatic"
                }),
            )
            .await?;

        let thread_id = thread
            .get("thread")
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server thread/start response missing thread.id".to_string(),
                )
            })?;

        let turn = self
            .request(
                "turn/start",
                json!({
                    "threadId": thread_id,
                    "input": [{
                        "type": "text",
                        "text": render_codex_user_input(&request),
                        "text_elements": []
                    }],
                    "cwd": isolated_cwd,
                    "approvalPolicy": "never",
                    "sandboxPolicy": {
                        "type": "readOnly",
                        "access": {
                            "type": "fullAccess"
                        },
                        "networkAccess": false
                    },
                    "model": model_id,
                    "personality": "pragmatic"
                }),
            )
            .await?;

        let turn_id = turn
            .get("turn")
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LlmError::Serialization(
                    "Codex app-server turn/start response missing turn.id".to_string(),
                )
            })?;

        let mut content = String::new();
        let mut turn_state = CompletionTurnState::default();
        let mut index = 0usize;

        let stop_reason = loop {
            match self.next_message().await? {
                RpcMessage::Notification { method, params }
                    if method == "item/agentMessage/delta"
                        && notification_matches_turn(&params, thread_id, turn_id) =>
                {
                    if let Some(stream_tx) = &stream_tx {
                        let delta = turn_state.apply_agent_delta(&params)?;
                        content.push_str(&delta);
                        let _ = stream_tx
                            .send(Ok(StreamChunk {
                                index,
                                delta: ChunkDelta::Text { text: delta },
                            }))
                            .await;
                    }
                    index += 1;
                }
                RpcMessage::Notification { method, params }
                    if method == "item/completed"
                        && notification_matches_turn(&params, thread_id, turn_id) =>
                {
                    if let Some(missing_delta) = turn_state.apply_item_completed(&params)? {
                        content.push_str(&missing_delta);
                        if let Some(stream_tx) = &stream_tx {
                            let _ = stream_tx
                                .send(Ok(StreamChunk {
                                    index,
                                    delta: ChunkDelta::Text {
                                        text: missing_delta,
                                    },
                                }))
                                .await;
                        }
                        index += 1;
                    }
                }
                RpcMessage::Notification { method, params }
                    if method == "model/rerouted"
                        && notification_matches_turn(&params, thread_id, turn_id) =>
                {
                    turn_state.apply_model_rerouted(&params)?;
                }
                RpcMessage::Notification { method, params }
                    if method == "thread/tokenUsage/updated"
                        && notification_matches_turn(&params, thread_id, turn_id) =>
                {
                    turn_state.apply_token_usage(&params)?;
                }
                RpcMessage::Notification { method, params }
                    if method == "turn/completed"
                        && notification_matches_turn(&params, thread_id, turn_id) =>
                {
                    let status = params
                        .get("turn")
                        .and_then(|value| value.get("status"))
                        .and_then(Value::as_str)
                        .unwrap_or("completed");

                    break match status {
                        "completed" => StopReason::Completed,
                        "interrupted" => StopReason::Cancelled,
                        "failed" => {
                            let message = params
                                .get("turn")
                                .and_then(|value| value.get("error"))
                                .and_then(|value| value.get("message"))
                                .and_then(Value::as_str)
                                .unwrap_or("Codex app-server turn failed");
                            return Err(LlmError::ProviderError {
                                status: 500,
                                message: message.to_string(),
                                retryable: false,
                            });
                        }
                        other => {
                            return Err(LlmError::Serialization(format!(
                                "unexpected Codex turn status: {other}"
                            )));
                        }
                    };
                }
                RpcMessage::Notification { .. } | RpcMessage::Response { .. } => continue,
                RpcMessage::Request { id, method, .. } => {
                    self.reply_with_error(
                        id,
                        -32_600,
                        format!(
                            "unsupported app-server request for OpenAiChatGptProvider: {method}"
                        ),
                    )
                    .await?;
                }
                RpcMessage::Error { error, .. } => {
                    return Err(normalize_rpc_error("completion stream", &error));
                }
            }
        };

        if let Some(stream_tx) = &stream_tx {
            let _ = stream_tx
                .send(Ok(StreamChunk::stop(index, stop_reason.clone())))
                .await;
        }

        let final_content = turn_state.final_content(&content);

        Ok(CompletionResponse {
            content: vec![ContentBlock::Text {
                text: final_content,
            }],
            model_id: turn_state.final_model_id(model_id).to_string(),
            usage: turn_state.usage,
            stop_reason,
            tool_calls: Vec::new(),
        })
    }

    async fn initialize(&mut self, extra_opt_out: &[&str]) -> Result<(), LlmError> {
        let mut opt_out_notification_methods = OPT_OUT_NOTIFICATION_METHODS
            .iter()
            .copied()
            .collect::<Vec<_>>();
        opt_out_notification_methods.extend(extra_opt_out.iter().copied());

        self.request(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "mister-smith",
                    "title": "Mister Smith",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "optOutNotificationMethods": opt_out_notification_methods
                }
            }),
        )
        .await?;
        self.notify("initialized", None).await
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, LlmError> {
        let id = self.next_id;
        self.next_id += 1;

        self.write_json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))
        .await?;

        loop {
            match self.next_message().await? {
                RpcMessage::Response {
                    id: response_id,
                    result,
                } if response_id == id => return Ok(result),
                RpcMessage::Error {
                    id: Some(response_id),
                    error,
                } if response_id == id => {
                    return Err(normalize_rpc_error(method, &error));
                }
                other => self.buffered.push_back(other),
            }
        }
    }

    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<(), LlmError> {
        let mut payload = json!({
            "jsonrpc": "2.0",
            "method": method
        });
        if let Some(params) = params {
            payload["params"] = params;
        }
        self.write_json(&payload).await
    }

    async fn reply_with_error(
        &mut self,
        id: Value,
        code: i64,
        message: String,
    ) -> Result<(), LlmError> {
        self.write_json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        }))
        .await
    }

    async fn write_json(&mut self, value: &Value) -> Result<(), LlmError> {
        let serialized = serde_json::to_vec(value)
            .map_err(|error| LlmError::Serialization(error.to_string()))?;
        self.stdin
            .write_all(&serialized)
            .await
            .map_err(|error| LlmError::Network(error.to_string()))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|error| LlmError::Network(error.to_string()))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| LlmError::Network(error.to_string()))
    }

    async fn next_message(&mut self) -> Result<RpcMessage, LlmError> {
        if let Some(message) = self.buffered.pop_front() {
            return Ok(message);
        }

        let Some(line) = self
            .stdout
            .next_line()
            .await
            .map_err(|error| LlmError::Network(error.to_string()))?
        else {
            let exit = self
                .child
                .wait()
                .await
                .map_err(|error| LlmError::Network(error.to_string()))?;
            return Err(LlmError::Network(format!(
                "codex app-server exited unexpectedly with status {exit}"
            )));
        };

        let value: Value = serde_json::from_str(&line)
            .map_err(|error| LlmError::Serialization(error.to_string()))?;

        if let Some(method) = value.get("method").and_then(Value::as_str) {
            let params = value.get("params").cloned().unwrap_or(Value::Null);
            if value.get("id").is_some() {
                return Ok(RpcMessage::Request {
                    id: value.get("id").cloned().unwrap_or(Value::Null),
                    method: method.to_string(),
                    _params: params,
                });
            }

            return Ok(RpcMessage::Notification {
                method: method.to_string(),
                params,
            });
        }

        if let Some(error) = value.get("error").cloned() {
            return Ok(RpcMessage::Error {
                id: value.get("id").and_then(Value::as_u64),
                error,
            });
        }

        let id = value.get("id").and_then(Value::as_u64).ok_or_else(|| {
            LlmError::Serialization("JSON-RPC payload missing numeric id".to_string())
        })?;
        let result = value.get("result").cloned().ok_or_else(|| {
            LlmError::Serialization("JSON-RPC response missing result".to_string())
        })?;
        Ok(RpcMessage::Response { id, result })
    }
}

fn codex_binary_path() -> String {
    std::env::var(CODEX_BIN_ENV).unwrap_or_else(|_| DEFAULT_CODEX_BIN.to_string())
}

fn normalize_rpc_error(context: &str, error: &Value) -> LlmError {
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32_000);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unknown Codex app-server error");

    LlmError::ProviderError {
        status: rpc_status_code(code),
        message: format!("{context}: {message}"),
        retryable: false,
    }
}

fn rpc_status_code(code: i64) -> u16 {
    match code {
        -32_603 => 504,
        -32_602 => 500,
        -32_601 => 404,
        -32_700..=-32_600 => 400,
        _ => 500,
    }
}

impl Drop for CodexAppServerClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn login_timeout() -> Duration {
    std::env::var(CODEX_LOGIN_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_LOGIN_TIMEOUT)
}

fn required_u64_field(value: &Value, field: &str) -> Result<u64, LlmError> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| {
        LlmError::Serialization(format!(
            "Codex app-server payload missing numeric field '{field}'"
        ))
    })
}

fn notification_matches_turn(params: &Value, thread_id: &str, turn_id: &str) -> bool {
    let thread_matches = params
        .get("threadId")
        .and_then(Value::as_str)
        .is_some_and(|value| value == thread_id);
    let turn_matches = params
        .get("turnId")
        .and_then(Value::as_str)
        .or_else(|| {
            params
                .get("turn")
                .and_then(|value| value.get("id"))
                .and_then(Value::as_str)
        })
        .is_some_and(|value| value == turn_id);

    thread_matches && turn_matches
}

fn render_codex_developer_instructions(request: &CompletionRequest) -> String {
    let mut lines = vec![
        "You are operating as a provider-backed completion adapter for Mister Smith.".to_string(),
        "Do not call tools, execute commands, browse files, or inspect the repository.".to_string(),
        "Answer only from the conversation transcript supplied by the user input.".to_string(),
    ];

    if let Some(system) = request.system.as_deref() {
        lines.push(format!("System instructions: {system}"));
    }

    for message in &request.messages {
        if let crate::types::ChatMessage::System { content } = message {
            lines.push(format!("Additional system instructions: {content}"));
        }
    }

    lines.join("\n")
}

fn render_codex_user_input(request: &CompletionRequest) -> String {
    let mut rendered = String::from("Conversation transcript:\n");

    for message in &request.messages {
        match message {
            crate::types::ChatMessage::System { content } => {
                rendered.push_str(&format!("System: {content}\n"));
            }
            crate::types::ChatMessage::User { content } => {
                rendered.push_str(&format!("User: {}\n", render_json_value(content)));
            }
            crate::types::ChatMessage::Assistant { content } => {
                rendered.push_str(&format!("Assistant: {}\n", render_json_value(content)));
            }
            crate::types::ChatMessage::Tool { result } => {
                let output = result
                    .output
                    .as_ref()
                    .map(render_json_value)
                    .unwrap_or_else(|| "null".to_string());
                let error = result.error.as_deref().unwrap_or("");
                rendered.push_str(&format!(
                    "Tool[{}]: output={} error={}\n",
                    result.call_id, output, error
                ));
            }
        }
    }

    rendered.push_str("\nReturn only the assistant response for the next turn.");
    rendered
}

fn render_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn isolated_codex_cwd() -> Result<String, LlmError> {
    let path = std::env::temp_dir().join("mister-smith-openai-chatgpt");
    std::fs::create_dir_all(&path).map_err(|error| {
        LlmError::Network(format!("failed to create isolated Codex cwd: {error}"))
    })?;
    Ok(path.to_string_lossy().into_owned())
}
