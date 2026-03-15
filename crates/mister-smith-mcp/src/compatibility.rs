use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::config::McpServerConfig;
use crate::errors::McpError;
use crate::server::{ExposedTool, McpServer, ToolHandler};

const DEFAULT_SERVER_NAME: &str = "smith";
const DEFAULT_LINEAR_ENDPOINT: &str = "https://api.linear.app/graphql";
const DEFAULT_SYMPHONY_CHECKOUT: &str = "~/symphony";
const DEFAULT_CODEX_CONFIG: &str = "~/.codex/config.toml";

const LINEAR_WORKSPACE_QUERY: &str = r#"
query SmithControlPlaneWorkspace {
  projects(first: 100) {
    nodes {
      id
      name
      slugId
      state
    }
  }
  issues(first: 100) {
    nodes {
      id
      identifier
      title
      priority
      url
      state {
        id
        name
        type
      }
      project {
        id
        name
        slugId
        state
      }
      team {
        id
        key
        name
      }
      labels(first: 20) {
        nodes {
          name
        }
      }
      inverseRelations(first: 20) {
        nodes {
          type
          issue {
            id
            identifier
            state {
              name
            }
          }
        }
      }
    }
  }
  teams(first: 20) {
    nodes {
      id
      key
      name
      states(first: 30) {
        nodes {
          id
          name
          type
        }
      }
    }
  }
}
"#;

const LINEAR_ISSUE_UPDATE_MUTATION: &str = r#"
mutation SmithUpdateIssue($id: String!, $input: IssueUpdateInput!) {
  issueUpdate(id: $id, input: $input) {
    success
    issue {
      id
      identifier
      title
    }
  }
}
"#;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Ok,
    Degraded,
    Blocked,
    DryRun,
    Applied,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse<T> {
    pub status: CompatibilityStatus,
    pub summary: String,
    pub evidence: Vec<EvidenceItem>,
    pub warnings: Vec<String>,
    pub recommended_next_tools: Vec<String>,
    pub blocking_issues: Vec<String>,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheck {
    pub name: String,
    pub status: CompatibilityStatus,
    pub severity: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowSummary {
    pub project_slug: Option<String>,
    pub active_states: Vec<String>,
    pub terminal_states: Vec<String>,
    pub workspace_root: Option<String>,
    pub codex_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessAudit {
    pub repo_root: String,
    pub codex_config_path: String,
    pub workflow_path: String,
    pub symphony_checkout: String,
    pub workspace_root: Option<String>,
    pub checks: Vec<ReadinessCheck>,
    pub workflow: WorkflowSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRuntimeInfo {
    pub server_name: String,
    pub crate_version: String,
    pub started_at: String,
    pub last_reload_at: String,
    pub reload_nonce: u64,
    pub repo_root: String,
    pub workflow_path: String,
    pub codex_config_path: String,
    pub symphony_checkout: String,
    pub tool_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRoute {
    pub route: String,
    pub reason: String,
    pub normalized_request: String,
    pub preferred_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub repo_root: String,
    pub git_branch: Option<String>,
    pub head_sha: Option<String>,
    pub upstream: Option<String>,
    pub remote_url: Option<String>,
    pub clean: bool,
    pub modified_count: usize,
    pub untracked_count: usize,
    pub recent_commit_subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitHubPullRequest {
    pub number: u64,
    pub title: String,
    pub head_ref_name: String,
    pub url: String,
    pub review_decision: Option<String>,
    pub is_draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubSnapshot {
    pub gh_available: bool,
    pub authenticated: bool,
    pub repo: Option<String>,
    pub default_branch: Option<String>,
    pub open_pull_requests: Vec<GitHubPullRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearProjectSnapshot {
    pub id: Option<String>,
    pub name: String,
    pub slug: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearStateSnapshot {
    pub id: Option<String>,
    pub name: String,
    pub state_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearIssueSnapshot {
    pub id: Option<String>,
    pub identifier: String,
    pub title: String,
    pub priority: Option<i64>,
    pub url: Option<String>,
    pub state: Option<LinearStateSnapshot>,
    pub project: Option<LinearProjectSnapshot>,
    pub team_key: Option<String>,
    pub team_name: Option<String>,
    pub labels: Vec<String>,
    pub blocked_by: Vec<LinearIssueBlockerSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearIssueBlockerSnapshot {
    pub id: Option<String>,
    pub identifier: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearRuntimeSync {
    pub configured_project_slug: Option<String>,
    pub queue_issue_count: usize,
    pub active_issue_count: usize,
    pub issues_by_state: BTreeMap<String, usize>,
    pub discrepancies: Vec<String>,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneSnapshot {
    pub repo: RepoSnapshot,
    pub workflow: WorkflowSummary,
    pub readiness: ReadinessAudit,
    pub symphony: SymphonyCheckoutSnapshot,
    pub github: GitHubSnapshot,
    pub linear: Option<LinearRuntimeSync>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymphonyCheckoutSnapshot {
    pub checkout_path: String,
    pub exists: bool,
    pub is_git_repo: bool,
    pub git_branch: Option<String>,
    pub head_sha: Option<String>,
    pub workspace_root: Option<String>,
    pub workspace_root_exists: bool,
    pub repo_env_present: bool,
    pub linear_api_key_available: bool,
    pub run_script_present: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAdjustment {
    pub id: String,
    pub action: String,
    pub description: String,
    pub path: Option<String>,
    pub requires_apply: bool,
    pub manually_required: bool,
    pub applied: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAdjustmentPlan {
    pub apply_requested: bool,
    pub adjustments: Vec<WorkspaceAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDispatchCycle {
    pub active_issue_count: usize,
    pub human_review_count: usize,
    pub merging_count: usize,
    pub open_pull_request_count: usize,
    pub refill_candidates: Vec<String>,
    pub stale_pull_requests: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSlice {
    pub name: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedLinearAction {
    pub action: String,
    pub issue_identifier: Option<String>,
    pub target_project_slug: Option<String>,
    pub target_state: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseExecutionPlan {
    pub phase: String,
    pub tasks_document: Option<String>,
    pub completed_work: Vec<String>,
    pub outstanding_work: Vec<String>,
    pub runnable_slices: Vec<PhaseSlice>,
    pub blocked_slices: Vec<PhaseSlice>,
    pub prep_slices: Vec<PhaseSlice>,
    pub recommended_linear_actions: Vec<PlannedLinearAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueExecutionSnapshot {
    pub issue: Option<LinearIssueSnapshot>,
    pub matching_pull_requests: Vec<GitHubPullRequest>,
    pub execution_state: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegitimacyAssessment {
    pub verdict: String,
    pub rationale: Vec<String>,
    pub suggested_project: String,
    pub suggested_state: String,
    pub suggested_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpClassification {
    pub classification: String,
    pub reason: String,
    pub suggested_title: Option<String>,
    pub suggested_project: String,
    pub suggested_state: String,
    pub suggested_labels: Vec<String>,
    pub should_stage: bool,
}

#[derive(Debug, Clone)]
pub struct SmithCompatibilityOptions {
    pub server_name: String,
    pub repo_root: PathBuf,
    pub codex_config_path: PathBuf,
    pub workflow_path: PathBuf,
    pub symphony_checkout: PathBuf,
    pub env_file_path: PathBuf,
    pub workspace_root_override: Option<PathBuf>,
    pub linear_endpoint: String,
}

impl SmithCompatibilityOptions {
    pub fn from_env() -> Self {
        let repo_root = env::var("MISTER_SMITH_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let codex_config_path = env::var("MISTER_SMITH_CODEX_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| expand_home(DEFAULT_CODEX_CONFIG));
        let symphony_checkout = env::var("MISTER_SMITH_SYMPHONY_CHECKOUT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| expand_home(DEFAULT_SYMPHONY_CHECKOUT));
        let workflow_path = repo_root.join("WORKFLOW.md");
        let env_file_path = repo_root.join(".env");
        let workspace_root_override = env::var("MISTER_SMITH_SYMPHONY_WORKSPACE_ROOT")
            .ok()
            .map(PathBuf::from);

        Self {
            server_name: DEFAULT_SERVER_NAME.to_string(),
            repo_root,
            codex_config_path,
            workflow_path,
            symphony_checkout,
            env_file_path,
            workspace_root_override,
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct SmithRuntimeState {
    started_at: DateTime<Utc>,
    last_reload_at: DateTime<Utc>,
    reload_nonce: u64,
    registered_tool_names: Vec<String>,
}

impl Default for SmithRuntimeState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            started_at: now,
            last_reload_at: now,
            reload_nonce: 0,
            registered_tool_names: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct CompatibilityCaches {
    workflow: Option<WorkflowSummary>,
    linear_workspace: Option<LinearWorkspaceData>,
}

#[derive(Debug, Clone, Default)]
struct LinearWorkspaceData {
    projects: Vec<LinearProjectSnapshot>,
    issues: Vec<LinearIssueSnapshot>,
    states_by_team: BTreeMap<String, Vec<LinearStateSnapshot>>,
}

#[derive(Debug)]
pub struct SmithCompatibilityServer {
    options: SmithCompatibilityOptions,
    runtime: RwLock<SmithRuntimeState>,
    caches: RwLock<CompatibilityCaches>,
}

impl SmithCompatibilityServer {
    fn new(options: SmithCompatibilityOptions) -> Self {
        Self {
            options,
            runtime: RwLock::new(SmithRuntimeState::default()),
            caches: RwLock::new(CompatibilityCaches::default()),
        }
    }

    async fn set_registered_tools(&self, tool_names: Vec<String>) {
        self.runtime.write().await.registered_tool_names = tool_names;
    }

    async fn runtime_info(&self) -> ServerRuntimeInfo {
        let runtime = self.runtime.read().await;
        ServerRuntimeInfo {
            server_name: self.options.server_name.clone(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: runtime.started_at.to_rfc3339(),
            last_reload_at: runtime.last_reload_at.to_rfc3339(),
            reload_nonce: runtime.reload_nonce,
            repo_root: self.options.repo_root.display().to_string(),
            workflow_path: self.options.workflow_path.display().to_string(),
            codex_config_path: self.options.codex_config_path.display().to_string(),
            symphony_checkout: self.options.symphony_checkout.display().to_string(),
            tool_names: runtime.registered_tool_names.clone(),
        }
    }

    async fn clear_caches(&self) {
        *self.caches.write().await = CompatibilityCaches::default();
    }

    async fn workflow_summary(&self) -> WorkflowSummary {
        if let Some(summary) = self.caches.read().await.workflow.clone() {
            return summary;
        }

        let summary = parse_workflow_summary(&self.options.workflow_path);
        self.caches.write().await.workflow = Some(summary.clone());
        summary
    }

    async fn linear_workspace(&self) -> Result<LinearWorkspaceData, String> {
        if let Some(workspace) = self.caches.read().await.linear_workspace.clone() {
            return Ok(workspace);
        }

        let api_key = self.linear_api_key().ok_or_else(|| {
            "LINEAR_API_KEY is not available from the environment or repo .env".to_string()
        })?;
        let response = reqwest::Client::new()
            .post(&self.options.linear_endpoint)
            .header(reqwest::header::AUTHORIZATION, api_key)
            .json(&serde_json::json!({
                "query": LINEAR_WORKSPACE_QUERY,
                "variables": {}
            }))
            .send()
            .await
            .map_err(|err| format!("failed to query Linear workspace: {err}"))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|err| format!("failed to decode Linear response: {err}"))?;

        if !status.is_success() {
            return Err(format!("Linear returned HTTP {status}: {body}"));
        }
        if let Some(errors) = body.get("errors") {
            return Err(format!("Linear GraphQL errors: {errors}"));
        }

        let data = body
            .get("data")
            .cloned()
            .ok_or_else(|| "Linear response missing data".to_string())?;
        let workspace = parse_linear_workspace(&data);
        self.caches.write().await.linear_workspace = Some(workspace.clone());
        Ok(workspace)
    }

    fn linear_api_key(&self) -> Option<String> {
        env::var("LINEAR_API_KEY")
            .ok()
            .or_else(|| read_env_value(&self.options.env_file_path, "LINEAR_API_KEY"))
    }

    async fn audit_workflow_readiness(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        json_response(self.collect_readiness_audit().await)
    }

    async fn collect_readiness_audit(&self) -> ToolResponse<ReadinessAudit> {
        let workflow = self.workflow_summary().await;
        let config_status = inspect_codex_smith_config(&self.options.codex_config_path);
        let workspace_root = self
            .resolved_workspace_root(Some(&workflow))
            .map(|path| path.display().to_string());
        let linear_key_available = self.linear_api_key().is_some();

        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut blockers = Vec::new();
        let mut evidence = vec![
            EvidenceItem {
                label: "repo_root".to_string(),
                detail: self.options.repo_root.display().to_string(),
            },
            EvidenceItem {
                label: "workflow_path".to_string(),
                detail: self.options.workflow_path.display().to_string(),
            },
        ];

        checks.push(file_check(
            "repo_root",
            self.options.repo_root.exists(),
            "blocker",
            format!("repo root {}", self.options.repo_root.display()),
        ));
        checks.push(file_check(
            "workflow_file",
            self.options.workflow_path.exists(),
            "blocker",
            format!("workflow file {}", self.options.workflow_path.display()),
        ));
        checks.push(file_check(
            "codex_config",
            self.options.codex_config_path.exists(),
            "blocker",
            format!("codex config {}", self.options.codex_config_path.display()),
        ));

        let smith_ready = config_status.configured && config_status.command.is_some();
        checks.push(ReadinessCheck {
            name: "smith_mcp_config".to_string(),
            status: if smith_ready {
                CompatibilityStatus::Ok
            } else if config_status.configured {
                CompatibilityStatus::Degraded
            } else {
                CompatibilityStatus::Blocked
            },
            severity: "blocker".to_string(),
            detail: if smith_ready {
                match config_status.inspection_source {
                    Some(source) => format!(
                        "smith MCP configured with command {} ({source} inspection)",
                        config_status.command.clone().unwrap_or_default()
                    ),
                    None => format!(
                        "smith MCP configured with command {}",
                        config_status.command.clone().unwrap_or_default()
                    ),
                }
            } else if config_status.configured {
                match config_status.inspection_source {
                    Some(source) => format!(
                        "smith MCP section exists but command is missing ({source} inspection)"
                    ),
                    None => "smith MCP section exists but command is missing".to_string(),
                }
            } else {
                "smith MCP server is not configured in Codex config".to_string()
            },
        });

        checks.push(command_check(
            "cargo",
            which("cargo").is_some(),
            "blocker",
            "cargo is required to run scripts/run-smith-mcp.sh",
        ));
        checks.push(command_check(
            "rustc",
            which("rustc").is_some(),
            "warning",
            "rustc is recommended for local validation",
        ));
        checks.push(command_check(
            "rustup",
            which("rustup").is_some(),
            "warning",
            "rustup is recommended for toolchain management",
        ));

        let gh_available = which("gh").is_some();
        let gh_authenticated = if gh_available {
            run_command("gh", &["auth", "status"]).await.success
        } else {
            false
        };
        checks.push(ReadinessCheck {
            name: "gh_auth".to_string(),
            status: if gh_authenticated {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Degraded
            },
            severity: "warning".to_string(),
            detail: if gh_authenticated {
                "gh is installed and authenticated".to_string()
            } else if gh_available {
                "gh is installed but not authenticated".to_string()
            } else {
                "gh is not installed".to_string()
            },
        });

        checks.push(file_check(
            "repo_env",
            self.options.env_file_path.exists(),
            "blocker",
            format!("repo env file {}", self.options.env_file_path.display()),
        ));
        checks.push(ReadinessCheck {
            name: "linear_api_key".to_string(),
            status: if linear_key_available {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Blocked
            },
            severity: "blocker".to_string(),
            detail: if linear_key_available {
                "LINEAR_API_KEY is available".to_string()
            } else {
                "LINEAR_API_KEY is missing from both the environment and repo .env".to_string()
            },
        });

        let symphony_snapshot = self.collect_symphony_checkout_snapshot().await;
        checks.push(ReadinessCheck {
            name: "symphony_checkout".to_string(),
            status: if symphony_snapshot.data.exists && symphony_snapshot.data.is_git_repo {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Blocked
            },
            severity: "blocker".to_string(),
            detail: symphony_snapshot.summary.clone(),
        });
        checks.push(ReadinessCheck {
            name: "symphony_workspace_root".to_string(),
            status: if symphony_snapshot.data.workspace_root_exists {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Blocked
            },
            severity: "blocker".to_string(),
            detail: symphony_snapshot
                .data
                .workspace_root
                .clone()
                .unwrap_or_else(|| "workspace root is not configured".to_string()),
        });

        for check in &checks {
            if matches!(check.status, CompatibilityStatus::Blocked) && check.severity == "blocker" {
                blockers.push(check.detail.clone());
            } else if !matches!(check.status, CompatibilityStatus::Ok) {
                warnings.push(check.detail.clone());
            }
        }

        if let Some(command) = config_status.command {
            evidence.push(EvidenceItem {
                label: "codex_smith_command".to_string(),
                detail: command,
            });
        }
        if let Some(source) = config_status.inspection_source {
            evidence.push(EvidenceItem {
                label: "codex_smith_inspection_source".to_string(),
                detail: source.to_string(),
            });
        }
        if let Some(cwd) = config_status.cwd {
            evidence.push(EvidenceItem {
                label: "codex_smith_cwd".to_string(),
                detail: cwd,
            });
        }
        if !config_status.args.is_empty() {
            evidence.push(EvidenceItem {
                label: "codex_smith_args".to_string(),
                detail: config_status.args.join(" "),
            });
        }
        if let Some(root) = &workspace_root {
            evidence.push(EvidenceItem {
                label: "workspace_root".to_string(),
                detail: root.clone(),
            });
        }

        let status = summarize_status(&checks);
        let summary = match status {
            CompatibilityStatus::Ok => {
                "Mister Smith control-plane prerequisites are ready".to_string()
            }
            CompatibilityStatus::Blocked => {
                "Mister Smith control-plane readiness is blocked by missing prerequisites"
                    .to_string()
            }
            _ => "Mister Smith control-plane readiness is degraded but partially inspectable"
                .to_string(),
        };

        ToolResponse {
            status,
            summary,
            evidence,
            warnings,
            recommended_next_tools: vec![
                "get_server_runtime_info".to_string(),
                "get_control_plane_snapshot".to_string(),
                "plan_workspace_adjustments".to_string(),
            ],
            blocking_issues: blockers,
            data: ReadinessAudit {
                repo_root: self.options.repo_root.display().to_string(),
                codex_config_path: self.options.codex_config_path.display().to_string(),
                workflow_path: self.options.workflow_path.display().to_string(),
                symphony_checkout: self.options.symphony_checkout.display().to_string(),
                workspace_root,
                checks,
                workflow,
            },
        }
    }

    async fn get_server_runtime_info(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: "smith MCP runtime metadata loaded".to_string(),
            evidence: vec![EvidenceItem {
                label: "tool_count".to_string(),
                detail: self
                    .runtime
                    .read()
                    .await
                    .registered_tool_names
                    .len()
                    .to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "audit_workflow_readiness".to_string(),
                "get_control_plane_snapshot".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: self.runtime_info().await,
        })
    }

    async fn reload_server(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let reason = string_param(&params, "reason").unwrap_or_else(|| "manual reload".to_string());
        self.clear_caches().await;
        {
            let mut runtime = self.runtime.write().await;
            runtime.reload_nonce += 1;
            runtime.last_reload_at = Utc::now();
        }
        let runtime = self.runtime_info().await;
        json_response(ToolResponse {
            status: CompatibilityStatus::Applied,
            summary: format!("smith MCP caches reloaded ({reason})"),
            evidence: vec![EvidenceItem {
                label: "reload_nonce".to_string(),
                detail: runtime.reload_nonce.to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "get_server_runtime_info".to_string(),
                "get_control_plane_snapshot".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: runtime,
        })
    }

    async fn route_workflow_request(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let request = string_param(&params, "request").unwrap_or_default();
        let normalized = request.to_lowercase();

        if normalized.trim().is_empty() {
            return json_response(ToolResponse {
                status: CompatibilityStatus::Blocked,
                summary: "route_workflow_request requires a request string".to_string(),
                evidence: Vec::new(),
                warnings: vec!["no operator request text was provided".to_string()],
                recommended_next_tools: vec!["get_control_plane_snapshot".to_string()],
                blocking_issues: vec!["request parameter is required".to_string()],
                data: WorkflowRoute {
                    route: "snapshot".to_string(),
                    reason: "missing request input".to_string(),
                    normalized_request: request,
                    preferred_tool: "get_control_plane_snapshot".to_string(),
                },
            });
        }

        let (route, reason, preferred_tool, next_tools) = if contains_any(
            &normalized,
            &[
                "bootstrap",
                "readiness",
                "install",
                "configure mcp",
                "setup",
            ],
        ) {
            (
                "bootstrap".to_string(),
                "request targets readiness or MCP bootstrap".to_string(),
                "audit_workflow_readiness".to_string(),
                vec![
                    "audit_workflow_readiness".to_string(),
                    "plan_workspace_adjustments".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &["symphony", "workspace", "checkout", "dispatch"],
        ) {
            (
                "symphony_runtime".to_string(),
                "request targets Symphony checkout or workspace hygiene".to_string(),
                "get_symphony_checkout_snapshot".to_string(),
                vec![
                    "get_symphony_checkout_snapshot".to_string(),
                    "plan_workspace_adjustments".to_string(),
                    "refresh_symphony".to_string(),
                ],
            )
        } else if contains_any(&normalized, &["phase", "slice", "stage", "milestone"]) {
            (
                "phase_execution".to_string(),
                "request targets phase planning or queue staging".to_string(),
                "plan_phase_execution".to_string(),
                vec![
                    "plan_phase_execution".to_string(),
                    "apply_phase_execution_plan".to_string(),
                ],
            )
        } else if contains_any(&normalized, &["review", "merge", "queue", "pr"]) {
            (
                "review_dispatch".to_string(),
                "request targets review/merge/dispatch reconciliation".to_string(),
                "review_merge_dispatch_cycle".to_string(),
                vec![
                    "review_merge_dispatch_cycle".to_string(),
                    "sync_linear_with_runtime".to_string(),
                    "get_control_plane_snapshot".to_string(),
                ],
            )
        } else if contains_any(&normalized, &["legitimacy", "frontier", "scope", "drift"]) {
            (
                "legitimacy".to_string(),
                "request targets legitimacy or scope judgment".to_string(),
                "evaluate_issue_legitimacy".to_string(),
                vec![
                    "evaluate_issue_legitimacy".to_string(),
                    "classify_follow_up_work".to_string(),
                ],
            )
        } else if contains_any(&normalized, &["issue", "linear", "ticket", "backlog"]) {
            (
                "linear_runtime".to_string(),
                "request targets Linear issue state or backlog routing".to_string(),
                "get_issue_execution_snapshot".to_string(),
                vec![
                    "get_issue_execution_snapshot".to_string(),
                    "sync_linear_with_runtime".to_string(),
                ],
            )
        } else {
            (
                "snapshot".to_string(),
                "request needs a broad control-plane snapshot first".to_string(),
                "get_control_plane_snapshot".to_string(),
                vec!["get_control_plane_snapshot".to_string()],
            )
        };

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: format!("routed workflow request to {route}"),
            evidence: vec![EvidenceItem {
                label: "request".to_string(),
                detail: request.clone(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: next_tools,
            blocking_issues: Vec::new(),
            data: WorkflowRoute {
                route,
                reason,
                normalized_request: normalized,
                preferred_tool,
            },
        })
    }

    async fn get_control_plane_snapshot(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let readiness = self.collect_readiness_audit().await;
        let workflow = self.workflow_summary().await;
        let symphony = self.collect_symphony_checkout_snapshot().await;
        let github = self.github_snapshot().await;
        let linear = self.linear_runtime_sync().await.ok();
        let repo = self.repo_snapshot().await;

        let mut warnings = readiness.warnings.clone();
        warnings.extend(symphony.warnings.clone());
        if linear.is_none() {
            warnings
                .push("Linear snapshot unavailable; returning repo/runtime state only".to_string());
        }

        let mut blocking_issues = readiness.blocking_issues.clone();
        blocking_issues.extend(symphony.blocking_issues.clone());
        blocking_issues.sort();
        blocking_issues.dedup();

        let status = if matches!(readiness.status, CompatibilityStatus::Blocked)
            || matches!(symphony.status, CompatibilityStatus::Blocked)
        {
            CompatibilityStatus::Blocked
        } else if matches!(readiness.status, CompatibilityStatus::Ok)
            && matches!(symphony.status, CompatibilityStatus::Ok)
        {
            CompatibilityStatus::Ok
        } else {
            CompatibilityStatus::Degraded
        };

        json_response(ToolResponse {
            status,
            summary: "loaded Mister Smith control-plane snapshot".to_string(),
            evidence: vec![
                EvidenceItem {
                    label: "git_branch".to_string(),
                    detail: repo
                        .git_branch
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                },
                EvidenceItem {
                    label: "project_slug".to_string(),
                    detail: workflow
                        .project_slug
                        .clone()
                        .unwrap_or_else(|| "unconfigured".to_string()),
                },
            ],
            warnings,
            recommended_next_tools: vec![
                "audit_workflow_readiness".to_string(),
                "get_symphony_checkout_snapshot".to_string(),
                "sync_linear_with_runtime".to_string(),
            ],
            blocking_issues,
            data: ControlPlaneSnapshot {
                repo,
                workflow,
                readiness: readiness.data,
                symphony: symphony.data,
                github,
                linear: linear.map(|response| response.data),
            },
        })
    }

    async fn get_symphony_checkout_snapshot(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        json_response(self.collect_symphony_checkout_snapshot().await)
    }

    async fn collect_symphony_checkout_snapshot(&self) -> ToolResponse<SymphonyCheckoutSnapshot> {
        let workflow = self.workflow_summary().await;
        let workspace_root = self.resolved_workspace_root(Some(&workflow));
        let checkout_path = self.options.symphony_checkout.clone();
        let exists = checkout_path.exists();
        let is_git_repo = checkout_path.join(".git").exists();
        let repo_env_present = self.options.env_file_path.exists();
        let linear_api_key_available = self.linear_api_key().is_some();
        let run_script_present = self
            .options
            .repo_root
            .join("scripts/run-symphony.sh")
            .exists();

        let git_branch = if exists && is_git_repo {
            trimmed_output(
                run_command_in_dir(
                    "git",
                    &["rev-parse", "--abbrev-ref", "HEAD"],
                    &checkout_path,
                )
                .await,
            )
        } else {
            None
        };
        let head_sha = if exists && is_git_repo {
            trimmed_output(
                run_command_in_dir("git", &["rev-parse", "--short", "HEAD"], &checkout_path).await,
            )
        } else {
            None
        };

        let workspace_root_exists = workspace_root
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false);
        let mut blockers = Vec::new();
        if !exists {
            blockers.push(format!(
                "Symphony checkout is missing at {}",
                checkout_path.display()
            ));
        } else if !is_git_repo {
            blockers.push(format!(
                "Symphony checkout exists but is not a git repository at {}",
                checkout_path.display()
            ));
        }
        if !workspace_root_exists {
            blockers.push(
                workspace_root
                    .as_ref()
                    .map(|path| format!("Symphony workspace root is missing at {}", path.display()))
                    .unwrap_or_else(|| "Symphony workspace root is not configured".to_string()),
            );
        }
        if !repo_env_present {
            blockers.push(format!(
                "Repo .env is missing at {}",
                self.options.env_file_path.display()
            ));
        }

        let status = if blockers.is_empty() {
            CompatibilityStatus::Ok
        } else {
            CompatibilityStatus::Blocked
        };
        let summary = if blockers.is_empty() {
            "Symphony checkout and workspace root are present".to_string()
        } else {
            "Symphony checkout prerequisites are incomplete".to_string()
        };

        ToolResponse {
            status,
            summary,
            evidence: vec![
                EvidenceItem {
                    label: "checkout_path".to_string(),
                    detail: checkout_path.display().to_string(),
                },
                EvidenceItem {
                    label: "workspace_root".to_string(),
                    detail: workspace_root
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "unset".to_string()),
                },
            ],
            warnings: if linear_api_key_available {
                Vec::new()
            } else {
                vec!["LINEAR_API_KEY is unavailable for Symphony launch".to_string()]
            },
            recommended_next_tools: vec![
                "plan_workspace_adjustments".to_string(),
                "refresh_symphony".to_string(),
                "sync_symphony_main".to_string(),
            ],
            blocking_issues: blockers.clone(),
            data: SymphonyCheckoutSnapshot {
                checkout_path: checkout_path.display().to_string(),
                exists,
                is_git_repo,
                git_branch,
                head_sha,
                workspace_root: workspace_root.map(|path| path.display().to_string()),
                workspace_root_exists,
                repo_env_present,
                linear_api_key_available,
                run_script_present,
                blockers,
            },
        }
    }

    async fn plan_workspace_adjustments(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let apply = bool_param(&params, "apply");
        let readiness = self.collect_readiness_audit().await;
        let mut adjustments = Vec::new();
        let workflow = readiness.data.workflow.clone();
        let workspace_root = self.resolved_workspace_root(Some(&workflow));
        let codex_config = inspect_codex_smith_config(&self.options.codex_config_path);
        let smith_snippet = self.codex_config_snippet();

        if !codex_config.configured || codex_config.command.is_none() {
            adjustments.push(WorkspaceAdjustment {
                id: "configure-codex-smith-server".to_string(),
                action: "update_codex_config".to_string(),
                description: "Add a [mcp_servers.smith] stdio server entry to Codex config"
                    .to_string(),
                path: Some(self.options.codex_config_path.display().to_string()),
                requires_apply: false,
                manually_required: true,
                applied: false,
                detail: Some(smith_snippet),
            });
        }

        if let Some(root) = workspace_root {
            let exists = root.exists();
            adjustments.push(WorkspaceAdjustment {
                id: "ensure-symphony-workspace-root".to_string(),
                action: "create_directory".to_string(),
                description: "Create the Symphony workspace root used by WORKFLOW.md".to_string(),
                path: Some(root.display().to_string()),
                requires_apply: true,
                manually_required: false,
                applied: false,
                detail: Some("safe local directory creation".to_string()),
            });

            if apply && !exists {
                if let Err(err) = fs::create_dir_all(&root) {
                    adjustments.push(WorkspaceAdjustment {
                        id: "ensure-symphony-workspace-root-error".to_string(),
                        action: "create_directory".to_string(),
                        description: "Failed to create the Symphony workspace root".to_string(),
                        path: Some(root.display().to_string()),
                        requires_apply: true,
                        manually_required: false,
                        applied: false,
                        detail: Some(err.to_string()),
                    });
                } else if let Some(existing) = adjustments.last_mut() {
                    existing.applied = true;
                }
            }
        }

        if !self.options.symphony_checkout.exists() {
            adjustments.push(WorkspaceAdjustment {
                id: "restore-symphony-checkout".to_string(),
                action: "checkout_repository".to_string(),
                description: "Restore the expected Symphony checkout at the configured path"
                    .to_string(),
                path: Some(self.options.symphony_checkout.display().to_string()),
                requires_apply: false,
                manually_required: true,
                applied: false,
                detail: Some(
                    "expected path from repo workflow: ~/Repos/symphony/elixir".to_string(),
                ),
            });
        }
        if which("cargo").is_none() {
            adjustments.push(WorkspaceAdjustment {
                id: "install-rust-toolchain".to_string(),
                action: "install_toolchain".to_string(),
                description: "Install a Rust toolchain so the smith MCP wrapper can run"
                    .to_string(),
                path: None,
                requires_apply: false,
                manually_required: true,
                applied: false,
                detail: Some("scripts/run-smith-mcp.sh uses cargo run".to_string()),
            });
        }
        if !self.options.env_file_path.exists() {
            adjustments.push(WorkspaceAdjustment {
                id: "restore-repo-env".to_string(),
                action: "restore_env_file".to_string(),
                description: "Restore repo .env with LINEAR_API_KEY for Symphony launches"
                    .to_string(),
                path: Some(self.options.env_file_path.display().to_string()),
                requires_apply: false,
                manually_required: true,
                applied: false,
                detail: None,
            });
        }

        let applied_count = adjustments.iter().filter(|item| item.applied).count();
        let status = if apply && applied_count > 0 {
            CompatibilityStatus::Applied
        } else {
            CompatibilityStatus::DryRun
        };

        json_response(ToolResponse {
            status,
            summary: if apply {
                format!("workspace adjustment pass completed; applied {applied_count} safe changes")
            } else {
                "workspace adjustment plan generated".to_string()
            },
            evidence: vec![EvidenceItem {
                label: "adjustment_count".to_string(),
                detail: adjustments.len().to_string(),
            }],
            warnings: readiness.warnings,
            recommended_next_tools: vec![
                "audit_workflow_readiness".to_string(),
                "get_symphony_checkout_snapshot".to_string(),
            ],
            blocking_issues: readiness.blocking_issues,
            data: WorkspaceAdjustmentPlan {
                apply_requested: apply,
                adjustments,
            },
        })
    }

    async fn refresh_symphony(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let apply = bool_param(&params, "apply");
        if !self.options.symphony_checkout.exists() {
            return json_response(blocked_response(
                "Symphony checkout is missing; refresh_symphony cannot run",
                vec!["plan_workspace_adjustments".to_string()],
                vec![format!(
                    "missing checkout at {}",
                    self.options.symphony_checkout.display()
                )],
                SymphonyCheckoutSnapshot {
                    checkout_path: self.options.symphony_checkout.display().to_string(),
                    exists: false,
                    is_git_repo: false,
                    git_branch: None,
                    head_sha: None,
                    workspace_root: self
                        .resolved_workspace_root(None)
                        .map(|path| path.display().to_string()),
                    workspace_root_exists: false,
                    repo_env_present: self.options.env_file_path.exists(),
                    linear_api_key_available: self.linear_api_key().is_some(),
                    run_script_present: self
                        .options
                        .repo_root
                        .join("scripts/run-symphony.sh")
                        .exists(),
                    blockers: vec!["missing Symphony checkout".to_string()],
                },
            ));
        }

        let fetch_command = "git -C <checkout> fetch --all --prune".to_string();
        let result = if apply {
            run_command_in_dir(
                "git",
                &["fetch", "--all", "--prune"],
                &self.options.symphony_checkout,
            )
            .await
        } else {
            CommandResult::dry_run(fetch_command.clone())
        };

        let snapshot = self.collect_symphony_checkout_snapshot().await;
        json_response(ToolResponse {
            status: if apply {
                if result.success {
                    CompatibilityStatus::Applied
                } else {
                    CompatibilityStatus::Blocked
                }
            } else {
                CompatibilityStatus::DryRun
            },
            summary: if apply {
                "Symphony fetch completed".to_string()
            } else {
                "Symphony fetch preview generated".to_string()
            },
            evidence: vec![EvidenceItem {
                label: "command".to_string(),
                detail: if apply {
                    format!(
                        "git -C {} fetch --all --prune",
                        self.options.symphony_checkout.display()
                    )
                } else {
                    fetch_command
                },
            }],
            warnings: if result.success {
                Vec::new()
            } else {
                vec![result.stderr]
            },
            recommended_next_tools: vec![
                "get_symphony_checkout_snapshot".to_string(),
                "sync_symphony_main".to_string(),
            ],
            blocking_issues: if result.success {
                Vec::new()
            } else {
                vec!["git fetch failed in Symphony checkout".to_string()]
            },
            data: snapshot.data,
        })
    }

    async fn sync_symphony_main(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let apply = bool_param(&params, "apply");
        if !self.options.symphony_checkout.exists() {
            return json_response(blocked_response(
                "Symphony checkout is missing; sync_symphony_main cannot run",
                vec!["plan_workspace_adjustments".to_string()],
                vec!["missing Symphony checkout".to_string()],
                self.collect_symphony_checkout_snapshot().await.data,
            ));
        }

        let branch = trimmed_output(
            run_command_in_dir(
                "git",
                &["rev-parse", "--abbrev-ref", "HEAD"],
                &self.options.symphony_checkout,
            )
            .await,
        );
        let status = run_command_in_dir(
            "git",
            &["status", "--short"],
            &self.options.symphony_checkout,
        )
        .await;
        let clean = status.stdout.trim().is_empty();

        if apply && (branch.as_deref() != Some("main") || !clean) {
            return json_response(blocked_response(
                "sync_symphony_main requires a clean main branch checkout",
                vec!["refresh_symphony".to_string()],
                vec![
                    format!(
                        "current branch: {}",
                        branch.unwrap_or_else(|| "unknown".to_string())
                    ),
                    if clean {
                        "working tree is clean".to_string()
                    } else {
                        "working tree is dirty".to_string()
                    },
                ],
                self.collect_symphony_checkout_snapshot().await.data,
            ));
        }

        let preview = format!(
            "git -C {} pull --ff-only origin main",
            self.options.symphony_checkout.display()
        );
        let result = if apply {
            run_command_in_dir(
                "git",
                &["pull", "--ff-only", "origin", "main"],
                &self.options.symphony_checkout,
            )
            .await
        } else {
            CommandResult::dry_run(preview.clone())
        };

        let snapshot = self.collect_symphony_checkout_snapshot().await;
        json_response(ToolResponse {
            status: if apply {
                if result.success {
                    CompatibilityStatus::Applied
                } else {
                    CompatibilityStatus::Blocked
                }
            } else {
                CompatibilityStatus::DryRun
            },
            summary: if apply {
                "Symphony main synchronization completed".to_string()
            } else {
                "Symphony main synchronization preview generated".to_string()
            },
            evidence: vec![EvidenceItem {
                label: "command".to_string(),
                detail: preview,
            }],
            warnings: if result.success {
                Vec::new()
            } else {
                vec![result.stderr]
            },
            recommended_next_tools: vec!["get_symphony_checkout_snapshot".to_string()],
            blocking_issues: if result.success {
                Vec::new()
            } else {
                vec!["failed to fast-forward Symphony main".to_string()]
            },
            data: snapshot.data,
        })
    }

    async fn sync_linear_with_runtime(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        json_response(self.linear_runtime_sync().await?)
    }

    async fn linear_runtime_sync(&self) -> Result<ToolResponse<LinearRuntimeSync>, McpError> {
        let workflow = self.workflow_summary().await;
        let workspace = match self.linear_workspace().await {
            Ok(workspace) => workspace,
            Err(err) => {
                return Ok(blocked_response(
                    format!("Linear workspace unavailable: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err],
                    LinearRuntimeSync {
                        configured_project_slug: workflow.project_slug,
                        queue_issue_count: 0,
                        active_issue_count: 0,
                        issues_by_state: BTreeMap::new(),
                        discrepancies: Vec::new(),
                        suggested_actions: vec!["restore LINEAR_API_KEY and rerun".to_string()],
                    },
                ))
            }
        };

        let mut issues_by_state = BTreeMap::new();
        let configured_slug = workflow.project_slug.clone();
        let queue_issues: Vec<&LinearIssueSnapshot> = workspace
            .issues
            .iter()
            .filter(|issue| issue_in_watched_project(issue, configured_slug.as_deref()))
            .collect();
        let active_issue_count = queue_issues
            .iter()
            .filter_map(|issue| state_name(issue))
            .filter(|state| is_active_state(state, &workflow))
            .count();

        for issue in &queue_issues {
            let key = issue
                .state
                .as_ref()
                .map(|state| state.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            *issues_by_state.entry(key).or_insert(0usize) += 1;
        }

        let mut discrepancies = Vec::new();
        let mut suggested_actions = Vec::new();
        if queue_issues.is_empty() {
            discrepancies
                .push("Execution queue has no issues in the configured project".to_string());
            suggested_actions.push(
                "Use plan_phase_execution and apply_phase_execution_plan to stage the next runnable slice".to_string(),
            );
        } else if active_issue_count == 0 {
            discrepancies.push(
                "Watched project has historical issues but no active issues in active workflow states"
                    .to_string(),
            );
            suggested_actions.push(
                "Stage the next runnable slice instead of relying on historical issues in the watched project".to_string(),
            );
        }

        let active_state_set: BTreeSet<_> = workflow.active_states.iter().cloned().collect();
        for issue in &queue_issues {
            if let Some(state) = &issue.state {
                if !active_state_set.contains(&state.name)
                    && !workflow.terminal_states.contains(&state.name)
                {
                    discrepancies.push(format!(
                        "{} is in unexpected state {} for the watched queue",
                        issue.identifier, state.name
                    ));
                }
            }
            if let Some(detail) = blocked_todo_detail(issue, &workflow) {
                discrepancies.push(detail);
                suggested_actions.push(format!(
                    "Remove {} from Todo or resolve its blockers before treating it as runnable capacity",
                    issue.identifier
                ));
            }
        }

        let status = if discrepancies.is_empty() {
            CompatibilityStatus::Ok
        } else {
            CompatibilityStatus::Degraded
        };

        Ok(ToolResponse {
            status,
            summary: "Linear/runtime synchronization snapshot generated".to_string(),
            evidence: vec![EvidenceItem {
                label: "queue_issue_count".to_string(),
                detail: queue_issues.len().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "get_issue_execution_snapshot".to_string(),
                "review_merge_dispatch_cycle".to_string(),
                "plan_phase_execution".to_string(),
            ],
            blocking_issues: if matches!(status, CompatibilityStatus::Blocked) {
                discrepancies.clone()
            } else {
                Vec::new()
            },
            data: LinearRuntimeSync {
                configured_project_slug: configured_slug,
                queue_issue_count: queue_issues.len(),
                active_issue_count,
                issues_by_state,
                discrepancies,
                suggested_actions,
            },
        })
    }

    async fn get_issue_execution_snapshot(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let Some(identifier) = identifier else {
            return json_response(blocked_response(
                "issue identifier is required",
                vec!["sync_linear_with_runtime".to_string()],
                vec!["issue_identifier parameter is required".to_string()],
                IssueExecutionSnapshot {
                    issue: None,
                    matching_pull_requests: Vec::new(),
                    execution_state: "missing_input".to_string(),
                    notes: vec!["provide an issue identifier such as MS-33".to_string()],
                },
            ));
        };

        let workspace = self.linear_workspace().await.ok();
        let issue = workspace.as_ref().and_then(|workspace| {
            workspace
                .issues
                .iter()
                .find(|issue| issue.identifier.eq_ignore_ascii_case(&identifier))
                .cloned()
        });
        let github = self.github_snapshot().await;
        let matching_pull_requests = github
            .open_pull_requests
            .into_iter()
            .filter(|pr| {
                pr.title.contains(&identifier)
                    || pr
                        .head_ref_name
                        .to_lowercase()
                        .contains(&identifier.to_lowercase())
            })
            .collect::<Vec<_>>();

        let (status, summary, blocking_issues) = if issue.is_some() {
            (
                CompatibilityStatus::Ok,
                format!("loaded execution snapshot for {identifier}"),
                Vec::new(),
            )
        } else {
            (
                CompatibilityStatus::Degraded,
                format!("could not resolve {identifier} from the current Linear snapshot"),
                vec![format!(
                    "issue {identifier} was not found in the current Linear snapshot"
                )],
            )
        };

        json_response(ToolResponse {
            status,
            summary,
            evidence: vec![EvidenceItem {
                label: "issue_identifier".to_string(),
                detail: identifier.clone(),
            }],
            warnings: if workspace.is_none() {
                vec!["Linear workspace unavailable; GitHub PR data may be partial".to_string()]
            } else {
                Vec::new()
            },
            recommended_next_tools: vec![
                "sync_linear_with_runtime".to_string(),
                "review_merge_dispatch_cycle".to_string(),
            ],
            blocking_issues,
            data: IssueExecutionSnapshot {
                execution_state: issue
                    .as_ref()
                    .and_then(|issue| issue.state.as_ref().map(|state| state.name.clone()))
                    .unwrap_or_else(|| "unknown".to_string()),
                notes: if issue.is_some() {
                    Vec::new()
                } else {
                    vec!["refresh Linear auth or verify the identifier".to_string()]
                },
                issue,
                matching_pull_requests,
            },
        })
    }

    async fn review_merge_dispatch_cycle(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let linear = self.linear_runtime_sync().await?;
        let workflow = self.workflow_summary().await;
        let workspace = self.linear_workspace().await.ok();
        let github = self.github_snapshot().await;
        let configured_slug = workflow.project_slug.clone();

        let human_review_count = workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .issues
                    .iter()
                    .filter(|issue| {
                        issue_in_watched_project(issue, configured_slug.as_deref())
                            && state_name(issue) == Some("Human Review")
                    })
                    .count()
            })
            .unwrap_or(0);
        let merging_count = workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .issues
                    .iter()
                    .filter(|issue| {
                        issue_in_watched_project(issue, configured_slug.as_deref())
                            && state_name(issue) == Some("Merging")
                    })
                    .count()
            })
            .unwrap_or(0);
        let blocked_todo_issues = workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .issues
                    .iter()
                    .filter(|issue| issue_in_watched_project(issue, configured_slug.as_deref()))
                    .filter_map(|issue| blocked_todo_detail(issue, &workflow))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let refill_candidates = workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .issues
                    .iter()
                    .filter(|issue| issue_is_honest_refill_candidate(issue))
                    .map(|issue| issue.identifier.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let stale_pull_requests = if linear.data.active_issue_count == 0 {
            github
                .open_pull_requests
                .iter()
                .map(|pr| format!("#{} {}", pr.number, pr.title))
                .collect()
        } else {
            Vec::new()
        };

        let mut recommended_actions = linear.data.suggested_actions.clone();
        if human_review_count > 0 {
            recommended_actions
                .push("inspect Human Review issues and land approved PRs".to_string());
        }
        if merging_count > 0 {
            recommended_actions
                .push("prioritize Merging issues before refilling the queue".to_string());
        }
        recommended_actions.extend(
            blocked_todo_issues.iter().map(|detail| {
                format!("remove blocked Todo noise from the watched queue: {detail}")
            }),
        );
        if linear.data.active_issue_count < 2 && !refill_candidates.is_empty() {
            recommended_actions.push(format!(
                "stage the next validated backlog issue: {}",
                refill_candidates[0]
            ));
        }
        if linear.data.active_issue_count < 2 && refill_candidates.is_empty() {
            recommended_actions.push(
                "validated backlog has no honest refill candidates; slice or validate more independent work to increase Symphony concurrency".to_string(),
            );
        }

        json_response(ToolResponse {
            status: if recommended_actions.is_empty() {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Degraded
            },
            summary: "review/merge/dispatch cycle snapshot generated".to_string(),
            evidence: vec![EvidenceItem {
                label: "open_pull_request_count".to_string(),
                detail: github.open_pull_requests.len().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "get_issue_execution_snapshot".to_string(),
                "plan_phase_execution".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: ReviewDispatchCycle {
                active_issue_count: linear.data.active_issue_count,
                human_review_count,
                merging_count,
                open_pull_request_count: github.open_pull_requests.len(),
                refill_candidates,
                stale_pull_requests,
                recommended_actions,
            },
        })
    }

    async fn plan_phase_execution(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let phase = string_param(&params, "phase").unwrap_or_else(|| "phase10".to_string());
        json_response(self.collect_phase_execution_plan(&phase).await)
    }

    async fn collect_phase_execution_plan(&self, phase: &str) -> ToolResponse<PhaseExecutionPlan> {
        let normalized = phase.to_lowercase();
        if normalized.contains("10") || normalized.contains("frontier") {
            let workspace = self.linear_workspace().await.ok();
            let workflow = self.workflow_summary().await;
            let issue_by_title = |needle: &str| {
                workspace.as_ref().and_then(|workspace| {
                    workspace
                        .issues
                        .iter()
                        .find(|issue| issue.title.contains(needle))
                        .cloned()
                })
            };

            let completed_work = vec![
                phase_marker(
                    &self
                        .options
                        .repo_root
                        .join("crates/mister-smith-agents/src/execution_graph.rs"),
                    "Phase 10.0 execution graph and topology compiler landed",
                ),
                phase_marker(
                    &self
                        .options
                        .repo_root
                        .join("crates/mister-smith-agents/src/branch_checkpoint.rs"),
                    "Phase 10.1 branch checkpoints landed",
                ),
                phase_marker(
                    &self
                        .options
                        .repo_root
                        .join("crates/mister-smith-agents/src/context_manager.rs"),
                    "Phase 10.2 managed memory and context snapshots landed",
                ),
                phase_marker(
                    &self
                        .options
                        .repo_root
                        .join("crates/mister-smith-agents/src/guard.rs"),
                    "Phase 10.3 predictive guard and intervention landed",
                ),
                phase_marker(
                    &self
                        .options
                        .repo_root
                        .join("crates/mister-smith-agents/src/orchestrator.rs"),
                    "Phase 10.4 autonomy status assembly exists in orchestrator",
                ),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            let mut outstanding_work = Vec::new();
            if !self
                .options
                .repo_root
                .join("crates/mister-smith-app/src/autonomy.rs")
                .exists()
            {
                outstanding_work
                    .push("Phase 10.5 app-level autonomy view is still missing".to_string());
            }
            if !self
                .options
                .repo_root
                .join("crates/mister-smith-app/tests/autonomy_status_tests.rs")
                .exists()
            {
                outstanding_work
                    .push("Phase 10.5 autonomy status tests are still missing".to_string());
            }
            if !self
                .options
                .repo_root
                .join("crates/mister-smith-security/src/delegation.rs")
                .exists()
            {
                outstanding_work.push(
                    "Phase 10.6 bounded delegation and provenance module is still missing"
                        .to_string(),
                );
            }
            outstanding_work.push("Phase 10 final verification/docs gate remains open".to_string());

            let ms33 = issue_by_title("Phase 10.5").or_else(|| issue_by_title("operator autonomy"));
            let ms34 =
                issue_by_title("Phase 10.6").or_else(|| issue_by_title("bounded delegation"));
            let ms35 =
                issue_by_title("Phase 10 gate").or_else(|| issue_by_title("verification and docs"));

            let mut runnable_slices = Vec::new();
            let mut blocked_slices = Vec::new();
            let mut recommended_linear_actions = Vec::new();

            if let Some(issue) = &ms33 {
                if !state_name(issue)
                    .map(|state| is_terminal_state(state, &workflow))
                    .unwrap_or(false)
                {
                    let slice = PhaseSlice {
                        name: issue.identifier.clone(),
                        description:
                            "Finish the operator autonomy view and alerts before later Phase 10 work"
                                .to_string(),
                        status: phase_slice_status(issue, &workflow),
                    };
                    if todo_issue_blocked_by_non_terminal(issue, &workflow) {
                        blocked_slices.push(PhaseSlice {
                            status: "blocked_by_non_terminal_dependencies".to_string(),
                            ..slice
                        });
                    } else {
                        if state_name(issue) == Some("Backlog") {
                            recommended_linear_actions.push(PlannedLinearAction {
                                action: "stage_issue".to_string(),
                                issue_identifier: Some(issue.identifier.clone()),
                                target_project_slug: workflow.project_slug.clone(),
                                target_state: Some("Todo".to_string()),
                                reason: "Phase 10.5 is the next unblocked slice".to_string(),
                            });
                        }
                        runnable_slices.push(slice);
                    }
                }
            }

            if let Some(issue) = &ms34 {
                let slice = PhaseSlice {
                    name: issue.identifier.clone(),
                    description:
                        "Bounded delegation and provenance should follow the autonomy view"
                            .to_string(),
                    status: phase_slice_status(issue, &workflow),
                };
                let blocked = todo_issue_blocked_by_non_terminal(issue, &workflow)
                    || ms33
                        .as_ref()
                        .and_then(state_name)
                        .map(|state| !is_terminal_state(state, &workflow))
                        .unwrap_or(false);
                if blocked {
                    blocked_slices.push(PhaseSlice {
                        status: "blocked_by_ms33".to_string(),
                        ..slice
                    });
                } else if !state_name(issue)
                    .map(|state| is_terminal_state(state, &workflow))
                    .unwrap_or(false)
                {
                    if state_name(issue) == Some("Backlog") {
                        recommended_linear_actions.push(PlannedLinearAction {
                            action: "stage_issue".to_string(),
                            issue_identifier: Some(issue.identifier.clone()),
                            target_project_slug: workflow.project_slug.clone(),
                            target_state: Some("Todo".to_string()),
                            reason: "Phase 10.6 is the next unblocked slice".to_string(),
                        });
                    }
                    runnable_slices.push(slice);
                }
            }

            if let Some(issue) = &ms35 {
                let slice = PhaseSlice {
                    name: issue.identifier.clone(),
                    description: "Final Phase 10 gate should run only after 10.6 lands".to_string(),
                    status: phase_slice_status(issue, &workflow),
                };
                let blocked = todo_issue_blocked_by_non_terminal(issue, &workflow)
                    || ms34
                        .as_ref()
                        .and_then(state_name)
                        .map(|state| !is_terminal_state(state, &workflow))
                        .unwrap_or(false);
                if blocked {
                    blocked_slices.push(PhaseSlice {
                        status: "blocked_by_ms34".to_string(),
                        ..slice
                    });
                } else if !state_name(issue)
                    .map(|state| is_terminal_state(state, &workflow))
                    .unwrap_or(false)
                {
                    if state_name(issue) == Some("Backlog") {
                        recommended_linear_actions.push(PlannedLinearAction {
                            action: "stage_issue".to_string(),
                            issue_identifier: Some(issue.identifier.clone()),
                            target_project_slug: workflow.project_slug.clone(),
                            target_state: Some("Todo".to_string()),
                            reason: "Phase 10 final gate is now unblocked".to_string(),
                        });
                    }
                    runnable_slices.push(slice);
                }
            }

            return ToolResponse {
                status: CompatibilityStatus::Ok,
                summary: "Phase 10 execution plan generated from repo and backlog state"
                    .to_string(),
                evidence: vec![EvidenceItem {
                    label: "tasks_document".to_string(),
                    detail: self
                        .options
                        .repo_root
                        .join("specs/012-phase10-frontier-autonomy/tasks.md")
                        .display()
                        .to_string(),
                }],
                warnings: vec![
                    "tasks.md remains stale and does not reflect landed 10.0-10.4 work".to_string(),
                ],
                recommended_next_tools: vec![
                    "apply_phase_execution_plan".to_string(),
                    "sync_linear_with_runtime".to_string(),
                ],
                blocking_issues: Vec::new(),
                data: PhaseExecutionPlan {
                    phase: "Phase 10 Frontier Autonomy".to_string(),
                    tasks_document: Some(
                        self.options
                            .repo_root
                            .join("specs/012-phase10-frontier-autonomy/tasks.md")
                            .display()
                            .to_string(),
                    ),
                    completed_work,
                    outstanding_work,
                    runnable_slices,
                    blocked_slices,
                    prep_slices: Vec::new(),
                    recommended_linear_actions,
                },
            };
        }

        let spec_dir = find_phase_spec_dir(&self.options.repo_root.join("specs"), &normalized);
        let tasks_document = spec_dir.as_ref().map(|dir| dir.join("tasks.md"));
        let (completed_count, outstanding_count) = tasks_document
            .as_ref()
            .map(|path| count_markdown_checkboxes(path))
            .unwrap_or((0, 0));

        ToolResponse {
            status: if spec_dir.is_some() {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Blocked
            },
            summary: if spec_dir.is_some() {
                format!("generic phase execution plan generated for {phase}")
            } else {
                format!("could not resolve a spec directory for {phase}")
            },
            evidence: tasks_document
                .as_ref()
                .map(|path| {
                    vec![EvidenceItem {
                        label: "tasks_document".to_string(),
                        detail: path.display().to_string(),
                    }]
                })
                .unwrap_or_default(),
            warnings: Vec::new(),
            recommended_next_tools: vec!["get_control_plane_snapshot".to_string()],
            blocking_issues: if spec_dir.is_some() {
                Vec::new()
            } else {
                vec![format!("no matching spec directory for {phase}")]
            },
            data: PhaseExecutionPlan {
                phase: phase.to_string(),
                tasks_document: tasks_document.map(|path| path.display().to_string()),
                completed_work: vec![format!("checked tasks: {completed_count}")],
                outstanding_work: vec![format!("unchecked tasks: {outstanding_count}")],
                runnable_slices: Vec::new(),
                blocked_slices: Vec::new(),
                prep_slices: Vec::new(),
                recommended_linear_actions: Vec::new(),
            },
        }
    }

    async fn apply_phase_execution_plan(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let apply = bool_param(&params, "apply");
        let phase = string_param(&params, "phase").unwrap_or_else(|| "phase10".to_string());
        let plan = self.collect_phase_execution_plan(&phase).await;
        if !apply {
            return json_response(ToolResponse {
                status: CompatibilityStatus::DryRun,
                summary: "phase execution plan preview generated".to_string(),
                evidence: plan.evidence.clone(),
                warnings: plan.warnings.clone(),
                recommended_next_tools: vec!["sync_linear_with_runtime".to_string()],
                blocking_issues: plan.blocking_issues.clone(),
                data: plan.data,
            });
        }

        let workspace = match self.linear_workspace().await {
            Ok(workspace) => workspace,
            Err(err) => {
                return json_response(blocked_response(
                    format!("cannot apply phase plan without Linear: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err],
                    plan.data,
                ))
            }
        };

        let mut applied = Vec::new();
        let mut warnings = Vec::new();
        for action in &plan.data.recommended_linear_actions {
            if action.action != "stage_issue" {
                continue;
            }

            let Some(issue_identifier) = &action.issue_identifier else {
                warnings.push("skipped stage_issue action with no issue identifier".to_string());
                continue;
            };
            let issue = workspace
                .issues
                .iter()
                .find(|issue| issue.identifier == *issue_identifier)
                .cloned();
            let project = action.target_project_slug.as_ref().and_then(|slug| {
                workspace
                    .projects
                    .iter()
                    .find(|project| project_slug_matches(Some(slug.as_str()), &project.slug))
                    .cloned()
            });
            let state = issue.as_ref().and_then(|issue| {
                let team_key = issue.team_key.clone().unwrap_or_else(|| "MS".to_string());
                workspace.states_by_team.get(&team_key).and_then(|states| {
                    states
                        .iter()
                        .find(|state| {
                            action
                                .target_state
                                .as_ref()
                                .map(|name| name == &state.name)
                                .unwrap_or(false)
                        })
                        .cloned()
                })
            });

            let Some(issue) = issue else {
                warnings.push(format!("issue {} was not found", issue_identifier));
                continue;
            };
            let Some(project) = project else {
                warnings.push(format!(
                    "target project {} was not found",
                    action.target_project_slug.clone().unwrap_or_default()
                ));
                continue;
            };
            let Some(state) = state else {
                warnings.push(format!(
                    "target state {} was not found for {}",
                    action.target_state.clone().unwrap_or_default(),
                    issue_identifier
                ));
                continue;
            };

            if let Err(err) = self
                .linear_issue_update(
                    issue.id.clone(),
                    serde_json::json!({
                        "projectId": project.id.clone(),
                        "stateId": state.id.clone(),
                    }),
                )
                .await
            {
                warnings.push(format!("failed to stage {}: {}", issue_identifier, err));
            } else {
                applied.push(format!(
                    "staged {} into {} / {}",
                    issue_identifier, project.slug, state.name
                ));
            }
        }

        if !applied.is_empty() {
            self.clear_caches().await;
        }

        json_response(ToolResponse {
            status: if applied.is_empty() {
                CompatibilityStatus::Blocked
            } else {
                CompatibilityStatus::Applied
            },
            summary: if applied.is_empty() {
                "phase execution plan did not apply any Linear changes".to_string()
            } else {
                format!("applied {} phase execution action(s)", applied.len())
            },
            evidence: applied
                .iter()
                .map(|detail| EvidenceItem {
                    label: "applied".to_string(),
                    detail: detail.clone(),
                })
                .collect(),
            warnings,
            recommended_next_tools: vec!["sync_linear_with_runtime".to_string()],
            blocking_issues: if applied.is_empty() {
                vec!["no phase actions were applied".to_string()]
            } else {
                Vec::new()
            },
            data: plan.data,
        })
    }

    async fn linear_issue_update(
        &self,
        issue_id: Option<String>,
        input: serde_json::Value,
    ) -> Result<(), String> {
        let issue_id = issue_id.ok_or_else(|| "issue id is missing".to_string())?;
        let api_key = self
            .linear_api_key()
            .ok_or_else(|| "LINEAR_API_KEY is unavailable".to_string())?;
        let response = reqwest::Client::new()
            .post(&self.options.linear_endpoint)
            .header(reqwest::header::AUTHORIZATION, api_key)
            .json(&serde_json::json!({
                "query": LINEAR_ISSUE_UPDATE_MUTATION,
                "variables": {
                    "id": issue_id,
                    "input": input,
                }
            }))
            .send()
            .await
            .map_err(|err| format!("Linear issue update failed: {err}"))?;
        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|err| format!("failed to decode Linear mutation response: {err}"))?;
        if !status.is_success() {
            return Err(format!("Linear mutation returned HTTP {status}: {body}"));
        }
        if let Some(errors) = body.get("errors") {
            return Err(format!("Linear mutation errors: {errors}"));
        }
        let success = body
            .pointer("/data/issueUpdate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if success {
            Ok(())
        } else {
            Err(format!("Linear mutation did not report success: {body}"))
        }
    }

    async fn evaluate_issue_legitimacy(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let issue_identifier = string_param(&params, "issue_identifier");
        let title = string_param(&params, "title");
        let summary = string_param(&params, "summary").unwrap_or_default();
        let issue = if let Some(identifier) = issue_identifier.clone() {
            self.linear_workspace().await.ok().and_then(|workspace| {
                workspace
                    .issues
                    .into_iter()
                    .find(|issue| issue.identifier.eq_ignore_ascii_case(&identifier))
            })
        } else {
            None
        };

        let title = issue
            .as_ref()
            .map(|issue| issue.title.clone())
            .or(title)
            .unwrap_or_default();
        let description = format!("{title} {summary}").to_lowercase();
        let frontier_aligned = contains_any(
            &description,
            &[
                "autonomy",
                "delegation",
                "control-plane",
                "symphony",
                "queue",
                "routing",
                "supervision",
                "provenance",
            ],
        );
        let watched_project = self.workflow_summary().await.project_slug;
        let currently_in_queue = issue
            .as_ref()
            .and_then(|issue| issue.project.as_ref().map(|project| project.slug.clone()))
            .map(|project_slug| project_slug_matches(watched_project.as_deref(), &project_slug))
            .unwrap_or(false);

        let (verdict, suggested_project, suggested_state) = if frontier_aligned {
            if currently_in_queue {
                (
                    "legitimate".to_string(),
                    "watched_queue".to_string(),
                    "Todo".to_string(),
                )
            } else {
                (
                    "legitimate_but_unstaged".to_string(),
                    "validated_backlog".to_string(),
                    "Backlog".to_string(),
                )
            }
        } else if description.contains("doc") || description.contains("readme") {
            (
                "docs_follow_up".to_string(),
                "workspace_docs".to_string(),
                "Backlog".to_string(),
            )
        } else {
            (
                "questionable".to_string(),
                "triage".to_string(),
                "Triage".to_string(),
            )
        };

        let mut rationale = Vec::new();
        if frontier_aligned {
            rationale
                .push("work strengthens the repo's autonomy/control-plane surfaces".to_string());
        } else {
            rationale.push(
                "work does not clearly map to the current frontier-autonomy mandate".to_string(),
            );
        }
        if currently_in_queue {
            rationale.push("issue is already in the watched execution queue".to_string());
        } else {
            rationale
                .push("issue is not currently staged in the watched execution queue".to_string());
        }

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: "issue legitimacy assessment generated".to_string(),
            evidence: vec![EvidenceItem {
                label: "title".to_string(),
                detail: title,
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec!["classify_follow_up_work".to_string()],
            blocking_issues: Vec::new(),
            data: LegitimacyAssessment {
                verdict,
                rationale,
                suggested_project,
                suggested_state,
                suggested_labels: if frontier_aligned {
                    vec!["Validated".to_string(), "Symphony Candidate".to_string()]
                } else {
                    vec!["Research".to_string()]
                },
            },
        })
    }

    async fn classify_follow_up_work(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let title = string_param(&params, "title")
            .or_else(|| string_param(&params, "issue_identifier"))
            .unwrap_or_default();
        let summary = string_param(&params, "summary").unwrap_or_default();
        let normalized = format!("{title} {summary}").to_lowercase();

        let (classification, project, state, labels, should_stage, reason) = if contains_any(
            &normalized,
            &[
                "phase 10",
                "autonomy",
                "delegation",
                "provenance",
                "control-plane",
                "queue",
            ],
        ) {
            (
                "validated_backlog".to_string(),
                "MisterSmith Validated Backlog".to_string(),
                "Backlog".to_string(),
                vec!["Validated".to_string(), "Symphony Candidate".to_string()],
                false,
                "frontier/control-plane work should be validated before it enters the watched queue".to_string(),
            )
        } else if contains_any(&normalized, &["readme", "docs", "workflow", "guide"]) {
            (
                "docs_hub".to_string(),
                "MisterSmith Workspace Docs".to_string(),
                "Backlog".to_string(),
                vec!["Docs".to_string()],
                false,
                "documentation follow-up belongs in the docs hub, not the watched queue"
                    .to_string(),
            )
        } else {
            (
                "triage".to_string(),
                "Triage".to_string(),
                "Triage".to_string(),
                vec!["Research".to_string()],
                false,
                "follow-up needs validation before it becomes runnable work".to_string(),
            )
        };

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: "follow-up work classification generated".to_string(),
            evidence: vec![EvidenceItem {
                label: "input".to_string(),
                detail: format!("{title} {summary}").trim().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec!["evaluate_issue_legitimacy".to_string()],
            blocking_issues: Vec::new(),
            data: FollowUpClassification {
                classification,
                reason,
                suggested_title: if title.is_empty() { None } else { Some(title) },
                suggested_project: project,
                suggested_state: state,
                suggested_labels: labels,
                should_stage,
            },
        })
    }

    async fn github_snapshot(&self) -> GitHubSnapshot {
        let gh_available = which("gh").is_some();
        if !gh_available {
            return GitHubSnapshot {
                gh_available,
                authenticated: false,
                repo: remote_owner_repo(&self.options.repo_root),
                default_branch: None,
                open_pull_requests: Vec::new(),
            };
        }

        let auth_status = run_command("gh", &["auth", "status"]).await;
        if !auth_status.success {
            return GitHubSnapshot {
                gh_available,
                authenticated: false,
                repo: remote_owner_repo(&self.options.repo_root),
                default_branch: None,
                open_pull_requests: Vec::new(),
            };
        }

        let repo_view = run_command(
            "gh",
            &["repo", "view", "--json", "nameWithOwner,defaultBranchRef"],
        )
        .await;
        let repo_json = serde_json::from_str::<serde_json::Value>(&repo_view.stdout).ok();
        let repo = repo_json
            .as_ref()
            .and_then(|json| json.get("nameWithOwner"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| remote_owner_repo(&self.options.repo_root));
        let default_branch = repo_json
            .as_ref()
            .and_then(|json| json.pointer("/defaultBranchRef/name"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);

        let prs = run_command(
            "gh",
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--limit",
                "50",
                "--json",
                "number,title,headRefName,url,reviewDecision,isDraft",
            ],
        )
        .await;
        let open_pull_requests =
            serde_json::from_str::<Vec<GitHubPullRequest>>(&prs.stdout).unwrap_or_default();

        GitHubSnapshot {
            gh_available,
            authenticated: true,
            repo,
            default_branch,
            open_pull_requests,
        }
    }

    async fn repo_snapshot(&self) -> RepoSnapshot {
        let branch = trimmed_output(
            run_command_in_dir(
                "git",
                &["rev-parse", "--abbrev-ref", "HEAD"],
                &self.options.repo_root,
            )
            .await,
        );
        let head_sha = trimmed_output(
            run_command_in_dir(
                "git",
                &["rev-parse", "--short", "HEAD"],
                &self.options.repo_root,
            )
            .await,
        );
        let upstream = trimmed_output(
            run_command_in_dir(
                "git",
                &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
                &self.options.repo_root,
            )
            .await,
        );
        let remote_url = trimmed_output(
            run_command_in_dir(
                "git",
                &["config", "--get", "remote.origin.url"],
                &self.options.repo_root,
            )
            .await,
        );
        let status_output =
            run_command_in_dir("git", &["status", "--short"], &self.options.repo_root).await;
        let recent_commit_subject = trimmed_output(
            run_command_in_dir(
                "git",
                &["log", "-1", "--pretty=%s"],
                &self.options.repo_root,
            )
            .await,
        );

        let mut modified_count = 0usize;
        let mut untracked_count = 0usize;
        for line in status_output.stdout.lines() {
            if line.starts_with("??") {
                untracked_count += 1;
            } else if !line.trim().is_empty() {
                modified_count += 1;
            }
        }

        RepoSnapshot {
            repo_root: self.options.repo_root.display().to_string(),
            git_branch: branch,
            head_sha,
            upstream,
            remote_url,
            clean: modified_count == 0 && untracked_count == 0,
            modified_count,
            untracked_count,
            recent_commit_subject,
        }
    }

    fn resolved_workspace_root(&self, workflow: Option<&WorkflowSummary>) -> Option<PathBuf> {
        self.options.workspace_root_override.clone().or_else(|| {
            workflow
                .and_then(|workflow| workflow.workspace_root.clone())
                .map(expand_home_from_string)
        })
    }

    fn codex_config_snippet(&self) -> String {
        format!(
            "[mcp_servers.smith]\ncommand = \"{}\"\ncwd = \"{}\"\nrequired = true\nstartup_timeout_sec = 30\ntool_timeout_sec = 120\nenv_vars = [\"LINEAR_API_KEY\"]\n",
            self.options
                .repo_root
                .join("scripts/run-smith-mcp.sh")
                .display(),
            self.options.repo_root.display(),
        )
    }
}

pub async fn build_smith_compatibility_server(
    options: SmithCompatibilityOptions,
) -> Result<Arc<McpServer>, McpError> {
    let compatibility = Arc::new(SmithCompatibilityServer::new(options));
    let server = Arc::new(McpServer::new(McpServerConfig {
        bind_address: "stdio://smith".to_string(),
        namespace_views: Vec::new(),
    }));

    register_compatibility_tool(
        &server,
        &compatibility,
        "audit_workflow_readiness",
        "Audit Mister Smith workflow readiness across repo, Codex config, Rust, Symphony, and Linear prerequisites.",
        object_schema(&[], &[]),
        |state, params| async move { state.audit_workflow_readiness(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "get_server_runtime_info",
        "Return smith MCP runtime metadata including registered tools and reload nonce.",
        object_schema(&[], &[]),
        |state, params| async move { state.get_server_runtime_info(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "reload_server",
        "Clear control-plane caches and refresh runtime metadata without restarting the process.",
        object_schema(&[("reason", string_schema("Optional reload reason"))], &[]),
        |state, params| async move { state.reload_server(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "route_workflow_request",
        "Route an operator request to the right smith control-plane tools.",
        object_schema(
            &[("request", string_schema("Operator request text"))],
            &["request"],
        ),
        |state, params| async move { state.route_workflow_request(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "get_control_plane_snapshot",
        "Collect repo, workflow, Symphony, GitHub, and Linear control-plane state in one response.",
        object_schema(&[], &[]),
        |state, params| async move { state.get_control_plane_snapshot(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "get_symphony_checkout_snapshot",
        "Inspect the configured Symphony checkout and workspace root.",
        object_schema(&[], &[]),
        |state, params| async move { state.get_symphony_checkout_snapshot(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "plan_workspace_adjustments",
        "Plan safe local workspace adjustments needed for Codex, Symphony, and repo readiness.",
        object_schema(
            &[("apply", bool_schema("Apply safe local adjustments"))],
            &[],
        ),
        |state, params| async move { state.plan_workspace_adjustments(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "sync_linear_with_runtime",
        "Compare watched Linear queue state to current runtime expectations.",
        object_schema(&[], &[]),
        |state, params| async move { state.sync_linear_with_runtime(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "refresh_symphony",
        "Refresh the configured Symphony checkout by fetching its remote state.",
        object_schema(
            &[(
                "apply",
                bool_schema("Run git fetch in the Symphony checkout"),
            )],
            &[],
        ),
        |state, params| async move { state.refresh_symphony(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "sync_symphony_main",
        "Fast-forward the configured Symphony main checkout when it is safe to do so.",
        object_schema(
            &[("apply", bool_schema("Run git pull --ff-only origin main"))],
            &[],
        ),
        |state, params| async move { state.sync_symphony_main(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "plan_phase_execution",
        "Generate a deterministic phase execution plan from repo state and backlog context.",
        object_schema(&[("phase", string_schema("Phase identifier or name"))], &[]),
        |state, params| async move { state.plan_phase_execution(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "apply_phase_execution_plan",
        "Apply the deterministic phase execution plan to Linear when explicitly requested.",
        object_schema(
            &[
                ("phase", string_schema("Phase identifier or name")),
                ("apply", bool_schema("Apply the planned Linear changes")),
            ],
            &[],
        ),
        |state, params| async move { state.apply_phase_execution_plan(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "review_merge_dispatch_cycle",
        "Inspect review, merge, PR, and queue state to recommend the next dispatch action.",
        object_schema(&[], &[]),
        |state, params| async move { state.review_merge_dispatch_cycle(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "get_issue_execution_snapshot",
        "Load the current Linear and GitHub execution snapshot for a specific issue identifier.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-33"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
            ],
            &[],
        ),
        |state, params| async move { state.get_issue_execution_snapshot(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "evaluate_issue_legitimacy",
        "Evaluate whether an issue or request is legitimate frontier/control-plane work for Mister Smith.",
        object_schema(
            &[
                ("issue_identifier", string_schema("Optional Linear issue identifier")),
                ("title", string_schema("Issue title or request title")),
                ("summary", string_schema("Optional description or summary")),
            ],
            &[],
        ),
        |state, params| async move { state.evaluate_issue_legitimacy(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "classify_follow_up_work",
        "Classify follow-up work into the appropriate Linear project/state and staging posture.",
        object_schema(
            &[
                ("title", string_schema("Follow-up title")),
                ("summary", string_schema("Follow-up summary")),
                (
                    "issue_identifier",
                    string_schema("Optional issue identifier to classify"),
                ),
            ],
            &[],
        ),
        |state, params| async move { state.classify_follow_up_work(params).await },
    )
    .await;

    compatibility
        .set_registered_tools(server.registered_tool_names().await)
        .await;
    Ok(server)
}

async fn register_compatibility_tool<F, Fut>(
    server: &Arc<McpServer>,
    compatibility: &Arc<SmithCompatibilityServer>,
    name: &str,
    description: &str,
    input_schema: serde_json::Value,
    func: F,
) where
    F: Fn(Arc<SmithCompatibilityServer>, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<serde_json::Value, McpError>> + Send + 'static,
{
    let handler = compatibility_handler(compatibility.clone(), func);
    server
        .register_tool(
            ExposedTool {
                name: name.to_string(),
                description: description.to_string(),
                input_schema,
                namespace: String::new(),
            },
            handler,
        )
        .await;
}

fn compatibility_handler<F, Fut>(
    compatibility: Arc<SmithCompatibilityServer>,
    func: F,
) -> ToolHandler
where
    F: Fn(Arc<SmithCompatibilityServer>, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<serde_json::Value, McpError>> + Send + 'static,
{
    Arc::new(move |params| {
        let compatibility = compatibility.clone();
        Box::pin(func(compatibility, params))
    })
}

fn object_schema(properties: &[(&str, serde_json::Value)], required: &[&str]) -> serde_json::Value {
    let props = properties
        .iter()
        .map(|(name, schema)| ((*name).to_string(), schema.clone()))
        .collect::<serde_json::Map<String, serde_json::Value>>();
    serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required,
        "additionalProperties": false
    })
}

fn bool_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "boolean",
        "description": description
    })
}

fn string_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "string",
        "description": description
    })
}

fn summarize_status(checks: &[ReadinessCheck]) -> CompatibilityStatus {
    if checks
        .iter()
        .any(|check| matches!(check.status, CompatibilityStatus::Blocked))
    {
        CompatibilityStatus::Blocked
    } else if checks
        .iter()
        .any(|check| !matches!(check.status, CompatibilityStatus::Ok))
    {
        CompatibilityStatus::Degraded
    } else {
        CompatibilityStatus::Ok
    }
}

fn file_check(name: &str, exists: bool, severity: &str, detail: String) -> ReadinessCheck {
    ReadinessCheck {
        name: name.to_string(),
        status: if exists {
            CompatibilityStatus::Ok
        } else {
            CompatibilityStatus::Blocked
        },
        severity: severity.to_string(),
        detail,
    }
}

fn command_check(name: &str, exists: bool, severity: &str, detail: &str) -> ReadinessCheck {
    ReadinessCheck {
        name: name.to_string(),
        status: if exists {
            CompatibilityStatus::Ok
        } else if severity == "warning" {
            CompatibilityStatus::Degraded
        } else {
            CompatibilityStatus::Blocked
        },
        severity: severity.to_string(),
        detail: detail.to_string(),
    }
}

fn blocked_response<T>(
    summary: impl Into<String>,
    recommended_next_tools: Vec<String>,
    blocking_issues: Vec<String>,
    data: T,
) -> ToolResponse<T> {
    ToolResponse {
        status: CompatibilityStatus::Blocked,
        summary: summary.into(),
        evidence: Vec::new(),
        warnings: Vec::new(),
        recommended_next_tools,
        blocking_issues,
        data,
    }
}

fn json_response<T: Serialize>(response: ToolResponse<T>) -> Result<serde_json::Value, McpError> {
    serde_json::to_value(response).map_err(|err| McpError::SerializationError(err.to_string()))
}

#[derive(Debug, Clone, Default)]
struct SmithCodexConfig {
    configured: bool,
    command: Option<String>,
    cwd: Option<String>,
    args: Vec<String>,
    inspection_source: Option<&'static str>,
}

fn inspect_codex_smith_config(path: &Path) -> SmithCodexConfig {
    let Ok(raw) = fs::read_to_string(path) else {
        return SmithCodexConfig::default();
    };
    if let Ok(value) = raw.parse::<toml::Value>() {
        if let Some(server) = find_smith_codex_server_value(&value) {
            return parse_smith_codex_config_value(server, "toml");
        }
    }

    inspect_codex_smith_config_heuristic(&raw)
}

fn find_smith_codex_server_value(value: &toml::Value) -> Option<&toml::Value> {
    value
        .get("mcp_servers")
        .and_then(|servers| servers.get("smith"))
        .or_else(|| value.get("mcp_servers.smith"))
}

fn parse_smith_codex_config_value(
    server: &toml::Value,
    inspection_source: &'static str,
) -> SmithCodexConfig {
    let args = server
        .get("args")
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    SmithCodexConfig {
        configured: true,
        command: server
            .get("command")
            .and_then(toml::Value::as_str)
            .map(ToOwned::to_owned),
        cwd: server
            .get("cwd")
            .and_then(toml::Value::as_str)
            .map(ToOwned::to_owned),
        args,
        inspection_source: Some(inspection_source),
    }
}

fn inspect_codex_smith_config_heuristic(raw: &str) -> SmithCodexConfig {
    let mut in_smith_section = false;
    let mut configured = false;
    let mut command = None;
    let mut cwd = None;
    let mut args = Vec::new();

    for line in raw.lines() {
        let trimmed = line
            .split_once('#')
            .map(|(prefix, _)| prefix)
            .unwrap_or(line)
            .trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_smith_section = trimmed == "[mcp_servers.smith]";
            configured |= in_smith_section;
            continue;
        }

        if !in_smith_section {
            continue;
        }

        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };

        match raw_key.trim() {
            "command" => {
                command = parse_toml_string_literal(raw_value.trim());
            }
            "cwd" => {
                cwd = parse_toml_string_literal(raw_value.trim());
            }
            "args" => {
                args = parse_toml_string_array(raw_value.trim());
            }
            _ => {}
        }
    }

    if configured {
        SmithCodexConfig {
            configured: true,
            command,
            cwd,
            args,
            inspection_source: Some("heuristic"),
        }
    } else {
        SmithCodexConfig::default()
    }
}

fn parse_toml_string_literal(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let quote = trimmed.chars().next()?;
    if (quote == '"' || quote == '\'') && trimmed.ends_with(quote) {
        return Some(trimmed[1..trimmed.len() - 1].to_string());
    }
    None
}

fn parse_toml_string_array(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };

    inner
        .split(',')
        .filter_map(parse_toml_string_literal)
        .collect()
}

fn parse_workflow_summary(path: &Path) -> WorkflowSummary {
    let Ok(raw) = fs::read_to_string(path) else {
        return WorkflowSummary::default();
    };
    let mut in_front_matter = false;
    let mut list_key: Option<String> = None;
    let mut summary = WorkflowSummary::default();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !in_front_matter {
                in_front_matter = true;
                continue;
            }
            break;
        }
        if !in_front_matter || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            match list_key.as_deref() {
                Some("active_states") => summary.active_states.push(strip_yaml_string(item)),
                Some("terminal_states") => summary.terminal_states.push(strip_yaml_string(item)),
                _ => {}
            }
            continue;
        }
        if trimmed.starts_with("active_states:") {
            list_key = Some("active_states".to_string());
            continue;
        }
        if trimmed.starts_with("terminal_states:") {
            list_key = Some("terminal_states".to_string());
            continue;
        }
        list_key = None;

        if let Some(value) = trimmed.strip_prefix("project_slug:") {
            summary.project_slug = Some(strip_yaml_string(value));
        } else if let Some(value) = trimmed.strip_prefix("root:") {
            if line.starts_with("workspace:") || !line.starts_with("  root:") {
                continue;
            }
            summary.workspace_root = Some(strip_yaml_string(value));
        } else if line.starts_with("  root:") {
            summary.workspace_root = Some(strip_yaml_string(line.trim_start_matches("  root:")));
        } else if line.starts_with("  command:") {
            summary.codex_command = Some(strip_yaml_string(line.trim_start_matches("  command:")));
        }
    }

    summary
}

fn strip_yaml_string(input: &str) -> String {
    input
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn project_slug_matches(configured: Option<&str>, actual: &str) -> bool {
    let Some(configured) = configured.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    let actual = actual.trim();
    if actual.is_empty() {
        return false;
    }
    if configured == actual {
        return true;
    }

    configured.ends_with(actual)
        || actual.ends_with(configured)
        || configured.rsplit('-').next() == Some(actual)
        || actual.rsplit('-').next() == Some(configured)
}

fn expand_home(path: &str) -> PathBuf {
    expand_home_from_string(path.to_string())
}

fn expand_home_from_string(path: String) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(path)
}

fn string_param(params: &serde_json::Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

fn bool_param(params: &serde_json::Value, key: &str) -> bool {
    params
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[derive(Debug, Clone)]
struct CommandResult {
    success: bool,
    stdout: String,
    stderr: String,
}

impl CommandResult {
    fn dry_run(command: String) -> Self {
        Self {
            success: true,
            stdout: command,
            stderr: String::new(),
        }
    }
}

async fn run_command(program: &str, args: &[&str]) -> CommandResult {
    let mut command = Command::new(program);
    command.args(args);
    match command.output().await {
        Ok(output) => CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(err) => CommandResult {
            success: false,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

async fn run_command_in_dir(program: &str, args: &[&str], cwd: &Path) -> CommandResult {
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    match command.output().await {
        Ok(output) => CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(err) => CommandResult {
            success: false,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn trimmed_output(result: CommandResult) -> Option<String> {
    if result.success {
        let trimmed = result.stdout.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}

fn which(program: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.exists())
}

fn read_env_value(path: &Path, key: &str) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some((found, value)) = trimmed.split_once('=') {
            if found.trim() == key {
                return Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
        }
    }
    None
}

fn remote_owner_repo(repo_root: &Path) -> Option<String> {
    let raw = fs::read_to_string(repo_root.join(".git/config")).ok()?;
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("url = ") {
            return None;
        }
        let url = trimmed.trim_start_matches("url = ").trim();
        if let Some(rest) = url.strip_prefix("https://github.com/") {
            return Some(rest.trim_end_matches(".git").to_string());
        }
        if let Some(rest) = url.strip_prefix("git@github.com:") {
            return Some(rest.trim_end_matches(".git").to_string());
        }
        None
    })
}

fn parse_linear_workspace(data: &serde_json::Value) -> LinearWorkspaceData {
    let projects = data
        .pointer("/projects/nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| nodes.iter().map(parse_linear_project).collect())
        .unwrap_or_default();
    let issues = data
        .pointer("/issues/nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| nodes.iter().map(parse_linear_issue).collect())
        .unwrap_or_default();
    let states_by_team = data
        .pointer("/teams/nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|team| {
                    let key = team
                        .get("key")
                        .and_then(serde_json::Value::as_str)?
                        .to_string();
                    let states = team
                        .pointer("/states/nodes")
                        .and_then(serde_json::Value::as_array)
                        .map(|nodes| nodes.iter().map(parse_linear_state).collect::<Vec<_>>())
                        .unwrap_or_default();
                    Some((key, states))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    LinearWorkspaceData {
        projects,
        issues,
        states_by_team,
    }
}

fn parse_linear_project(value: &serde_json::Value) -> LinearProjectSnapshot {
    LinearProjectSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        slug: value
            .get("slugId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        state: value
            .get("state")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn parse_linear_state(value: &serde_json::Value) -> LinearStateSnapshot {
    LinearStateSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        state_type: value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn parse_linear_issue(value: &serde_json::Value) -> LinearIssueSnapshot {
    LinearIssueSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        identifier: value
            .get("identifier")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        title: value
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        priority: value.get("priority").and_then(serde_json::Value::as_i64),
        url: value
            .get("url")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        state: value.get("state").map(parse_linear_state),
        project: value.get("project").map(parse_linear_project),
        team_key: value
            .pointer("/team/key")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        team_name: value
            .pointer("/team/name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        labels: value
            .pointer("/labels/nodes")
            .and_then(serde_json::Value::as_array)
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|node| node.get("name").and_then(serde_json::Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        blocked_by: value
            .pointer("/inverseRelations/nodes")
            .and_then(serde_json::Value::as_array)
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(parse_linear_blocker)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    }
}

fn parse_linear_blocker(value: &serde_json::Value) -> Option<LinearIssueBlockerSnapshot> {
    let relation_type = value.get("type")?.as_str()?.trim().to_ascii_lowercase();
    if relation_type != "blocks" {
        return None;
    }

    let blocker_issue = value.get("issue")?;
    Some(LinearIssueBlockerSnapshot {
        id: blocker_issue
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        identifier: blocker_issue
            .get("identifier")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        state: blocker_issue
            .pointer("/state/name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn state_name(issue: &LinearIssueSnapshot) -> Option<&str> {
    issue.state.as_ref().map(|state| state.name.as_str())
}

fn issue_in_watched_project(issue: &LinearIssueSnapshot, configured_slug: Option<&str>) -> bool {
    issue
        .project
        .as_ref()
        .map(|project| project_slug_matches(configured_slug, &project.slug))
        .unwrap_or(false)
}

fn is_active_state(state_name: &str, workflow: &WorkflowSummary) -> bool {
    workflow
        .active_states
        .iter()
        .any(|state| state == state_name)
}

fn is_terminal_state(state_name: &str, workflow: &WorkflowSummary) -> bool {
    workflow
        .terminal_states
        .iter()
        .any(|state| state == state_name)
}

fn todo_issue_blocked_by_non_terminal(
    issue: &LinearIssueSnapshot,
    workflow: &WorkflowSummary,
) -> bool {
    state_name(issue) == Some("Todo")
        && issue.blocked_by.iter().any(|blocker| {
            blocker
                .state
                .as_deref()
                .map(|state| !is_terminal_state(state, workflow))
                .unwrap_or(false)
        })
}

fn blocked_todo_detail(issue: &LinearIssueSnapshot, workflow: &WorkflowSummary) -> Option<String> {
    if !todo_issue_blocked_by_non_terminal(issue, workflow) {
        return None;
    }

    let blockers = issue
        .blocked_by
        .iter()
        .filter(|blocker| {
            blocker
                .state
                .as_deref()
                .map(|state| !is_terminal_state(state, workflow))
                .unwrap_or(false)
        })
        .map(|blocker| match blocker.state.as_deref() {
            Some(state) if !state.is_empty() => format!("{} ({state})", blocker.identifier),
            _ => blocker.identifier.clone(),
        })
        .collect::<Vec<_>>();

    Some(format!(
        "{} is in Todo but blocked by non-terminal issues: {}",
        issue.identifier,
        blockers.join(", ")
    ))
}

fn issue_is_honest_refill_candidate(issue: &LinearIssueSnapshot) -> bool {
    state_name(issue) == Some("Backlog")
        && issue.labels.iter().any(|label| label == "Validated")
        && issue
            .labels
            .iter()
            .any(|label| label == "Symphony Candidate")
        && issue
            .project
            .as_ref()
            .map(|project| project.name == "MisterSmith Validated Backlog")
            .unwrap_or(false)
}

fn phase_slice_status(issue: &LinearIssueSnapshot, workflow: &WorkflowSummary) -> String {
    match state_name(issue) {
        Some(state) if is_terminal_state(state, workflow) => "done".to_string(),
        Some("In Progress") => "in_progress".to_string(),
        Some("Todo") => "todo".to_string(),
        Some("Human Review") => "human_review".to_string(),
        Some("Merging") => "merging".to_string(),
        Some("Rework") => "rework".to_string(),
        Some(state) => state.to_ascii_lowercase().replace(' ', "_"),
        None => "unknown".to_string(),
    }
}

fn phase_marker(path: &Path, description: &str) -> Option<String> {
    if path.exists() {
        Some(description.to_string())
    } else {
        None
    }
}

fn count_markdown_checkboxes(path: &Path) -> (usize, usize) {
    let Ok(raw) = fs::read_to_string(path) else {
        return (0, 0);
    };
    let checked = raw.matches("- [X]").count();
    let unchecked = raw.matches("- [ ]").count();
    (checked, unchecked)
}

fn find_phase_spec_dir(specs_root: &Path, phase: &str) -> Option<PathBuf> {
    let mut entries = fs::read_dir(specs_root).ok()?;
    entries.find_map(|entry| {
        let path = entry.ok()?.path();
        let name = path.file_name()?.to_string_lossy().to_lowercase();
        if name.contains(phase) {
            Some(path)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        model::{CallToolRequestParams, ClientInfo},
        serve_server, ClientHandler, ServiceExt,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Clone)]
    struct TestClient;

    impl ClientHandler for TestClient {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        env::temp_dir().join(format!(
            "mister-smith-mcp-{name}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_fixture_repo(root: &Path) {
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("WORKFLOW.md"),
            r#"---
tracker:
  kind: linear
  project_slug: "320a0741920c"
  active_states:
    - Todo
    - In Progress
workspace:
  root: ~/.local/share/symphony-workspaces
codex:
  command: codex app-server
---
"#,
        )
        .unwrap();
        fs::write(root.join(".env"), "LINEAR_API_KEY=dummy\n").unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(
            root.join("scripts/run-symphony.sh"),
            "#!/usr/bin/env bash\n",
        )
        .unwrap();
    }

    fn rich_codex_config(repo_root: &Path) -> String {
        format!(
            r#"model = "gpt-5.4"
model_reasoning_effort = "xhigh"
approval_policy = "never"

[mcp_servers.linear]
url = "https://mcp.linear.app/mcp"

[mcp_servers.smith]
command = "{repo}/scripts/run-smith-mcp.sh"
cwd = "{repo}"
required = true
startup_timeout_sec = 30
tool_timeout_sec = 120
env_vars = ["LINEAR_API_KEY"]

[features]
apps = true
"#,
            repo = repo_root.display()
        )
    }

    #[test]
    fn inspect_codex_smith_config_reads_rich_codex_config() {
        let repo_root = temp_path("rich-config");
        let config_path = repo_root.join("config.toml");
        fs::create_dir_all(&repo_root).unwrap();
        fs::write(&config_path, rich_codex_config(&repo_root)).unwrap();

        let config = inspect_codex_smith_config(&config_path);
        let expected_command = format!("{}/scripts/run-smith-mcp.sh", repo_root.display());
        let expected_cwd = repo_root.display().to_string();

        assert!(config.configured);
        assert_eq!(config.command.as_deref(), Some(expected_command.as_str()));
        assert_eq!(config.cwd.as_deref(), Some(expected_cwd.as_str()));
        assert!(matches!(
            config.inspection_source,
            Some("toml" | "heuristic")
        ));
    }

    #[test]
    fn inspect_codex_smith_config_falls_back_to_heuristic() {
        let repo_root = temp_path("heuristic-config");
        let config_path = repo_root.join("config.toml");
        fs::create_dir_all(&repo_root).unwrap();
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\ncwd = \"{}\"\nargs = [\"--repo-root\", \"{}\"]\nthis is not valid toml\n",
                repo_root.display(),
                repo_root.display(),
                repo_root.display()
            ),
        )
        .unwrap();

        let config = inspect_codex_smith_config(&config_path);
        let expected_command = format!("{}/scripts/run-smith-mcp.sh", repo_root.display());

        assert!(config.configured);
        assert_eq!(config.inspection_source, Some("heuristic"));
        assert_eq!(config.command.as_deref(), Some(expected_command.as_str()));
        assert_eq!(
            config.args,
            vec!["--repo-root".to_string(), repo_root.display().to_string()]
        );
    }

    #[test]
    fn fixture_workflow_uses_slug_id_and_local_workspace_root() {
        let repo_root = temp_path("fixture-workflow");
        write_fixture_repo(&repo_root);

        let summary = parse_workflow_summary(&repo_root.join("WORKFLOW.md"));

        assert_eq!(summary.project_slug.as_deref(), Some("320a0741920c"));
        assert_eq!(
            summary.workspace_root.as_deref(),
            Some("~/.local/share/symphony-workspaces")
        );
        assert_eq!(summary.codex_command.as_deref(), Some("codex app-server"));
    }

    #[test]
    fn project_slug_matching_accepts_slug_id_and_legacy_slug() {
        assert!(project_slug_matches(Some("320a0741920c"), "320a0741920c"));
        assert!(project_slug_matches(
            Some("mistersmith-execution-queue-320a0741920c"),
            "320a0741920c"
        ));
        assert!(project_slug_matches(
            Some("320a0741920c"),
            "mistersmith-execution-queue-320a0741920c"
        ));
        assert!(!project_slug_matches(Some("a179384f32b2"), "320a0741920c"));
    }

    #[test]
    fn github_pull_request_deserializes_gh_json_fields() {
        let pull_request: GitHubPullRequest = serde_json::from_value(serde_json::json!({
            "number": 186,
            "title": "Example",
            "headRefName": "codex/example",
            "url": "https://github.com/example/repo/pull/186",
            "reviewDecision": null,
            "isDraft": false
        }))
        .unwrap();

        assert_eq!(pull_request.head_ref_name, "codex/example");
        assert_eq!(pull_request.review_decision, None);
        assert!(!pull_request.is_draft);
    }

    #[test]
    fn parse_linear_issue_extracts_blockers_and_labels() {
        let issue = parse_linear_issue(&serde_json::json!({
            "id": "issue-1",
            "identifier": "MS-35",
            "title": "Phase 10 gate",
            "priority": 3,
            "state": {"name": "Todo", "type": "unstarted"},
            "project": {"id": "project-1", "name": "MisterSmith Execution Queue", "slugId": "320a0741920c"},
            "team": {"key": "MS", "name": "MisterSmith"},
            "labels": {"nodes": [{"name": "Validated"}, {"name": "Symphony Candidate"}]},
            "inverseRelations": {"nodes": [
                {
                    "type": "blocks",
                    "issue": {
                        "id": "issue-2",
                        "identifier": "MS-34",
                        "state": {"name": "In Progress"}
                    }
                },
                {
                    "type": "relates_to",
                    "issue": {
                        "id": "issue-3",
                        "identifier": "MS-99",
                        "state": {"name": "Todo"}
                    }
                }
            ]}
        }));

        assert_eq!(issue.labels, vec!["Validated", "Symphony Candidate"]);
        assert_eq!(issue.blocked_by.len(), 1);
        assert_eq!(issue.blocked_by[0].identifier, "MS-34");
        assert_eq!(issue.blocked_by[0].state.as_deref(), Some("In Progress"));
    }

    #[test]
    fn helper_logic_distinguishes_blocked_todo_and_refill_candidates() {
        let workflow = WorkflowSummary {
            project_slug: Some("320a0741920c".to_string()),
            active_states: vec!["Todo".to_string(), "In Progress".to_string()],
            terminal_states: vec![
                "Done".to_string(),
                "Canceled".to_string(),
                "Duplicate".to_string(),
            ],
            workspace_root: None,
            codex_command: None,
        };
        let blocked_todo = LinearIssueSnapshot {
            identifier: "MS-35".to_string(),
            title: "Phase 10 gate".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Todo".to_string(),
                ..LinearStateSnapshot::default()
            }),
            project: Some(LinearProjectSnapshot {
                name: "MisterSmith Execution Queue".to_string(),
                slug: "320a0741920c".to_string(),
                ..LinearProjectSnapshot::default()
            }),
            blocked_by: vec![LinearIssueBlockerSnapshot {
                identifier: "MS-34".to_string(),
                state: Some("In Progress".to_string()),
                ..LinearIssueBlockerSnapshot::default()
            }],
            ..LinearIssueSnapshot::default()
        };
        let refill_candidate = LinearIssueSnapshot {
            identifier: "MS-90".to_string(),
            title: "Validated candidate".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Backlog".to_string(),
                ..LinearStateSnapshot::default()
            }),
            project: Some(LinearProjectSnapshot {
                name: "MisterSmith Validated Backlog".to_string(),
                slug: "validated-backlog".to_string(),
                ..LinearProjectSnapshot::default()
            }),
            labels: vec!["Validated".to_string(), "Symphony Candidate".to_string()],
            ..LinearIssueSnapshot::default()
        };
        let questionable_backlog = LinearIssueSnapshot {
            identifier: "MS-37".to_string(),
            title: "Questionable backlog item".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Backlog".to_string(),
                ..LinearStateSnapshot::default()
            }),
            project: Some(LinearProjectSnapshot {
                name: "MisterSmith Validated Backlog".to_string(),
                slug: "validated-backlog".to_string(),
                ..LinearProjectSnapshot::default()
            }),
            ..LinearIssueSnapshot::default()
        };

        assert!(todo_issue_blocked_by_non_terminal(&blocked_todo, &workflow));
        assert!(blocked_todo_detail(&blocked_todo, &workflow)
            .unwrap()
            .contains("MS-34 (In Progress)"));
        assert!(issue_is_honest_refill_candidate(&refill_candidate));
        assert!(!issue_is_honest_refill_candidate(&questionable_backlog));
    }

    #[tokio::test]
    async fn compatibility_server_lists_plain_tool_names() {
        let repo_root = temp_path("plain-tools");
        write_fixture_repo(&repo_root);
        let config_path = repo_root.join("config.toml");
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\n",
                repo_root.display()
            ),
        )
        .unwrap();

        let server = build_smith_compatibility_server(SmithCompatibilityOptions {
            server_name: "smith".to_string(),
            repo_root: repo_root.clone(),
            codex_config_path: config_path,
            workflow_path: repo_root.join("WORKFLOW.md"),
            symphony_checkout: temp_path("symphony"),
            env_file_path: repo_root.join(".env"),
            workspace_root_override: Some(temp_path("workspaces")),
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        })
        .await
        .unwrap();

        let tools = server.handle_tools_list(None).await.unwrap();
        assert!(tools
            .iter()
            .any(|tool| tool.name == "audit_workflow_readiness"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "get_server_runtime_info"));
        assert!(!tools.iter().any(|tool| tool.name.contains("smith.")));
    }

    #[tokio::test]
    async fn reload_server_increments_nonce() {
        let repo_root = temp_path("reload");
        write_fixture_repo(&repo_root);
        let config_path = repo_root.join("config.toml");
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\n",
                repo_root.display()
            ),
        )
        .unwrap();

        let server = build_smith_compatibility_server(SmithCompatibilityOptions {
            server_name: "smith".to_string(),
            repo_root: repo_root.clone(),
            codex_config_path: config_path,
            workflow_path: repo_root.join("WORKFLOW.md"),
            symphony_checkout: temp_path("symphony"),
            env_file_path: repo_root.join(".env"),
            workspace_root_override: Some(temp_path("workspaces")),
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        })
        .await
        .unwrap();

        let before = server
            .handle_tools_call("get_server_runtime_info", serde_json::json!({}))
            .await
            .unwrap();
        let before_nonce = before["data"]["reload_nonce"].as_u64().unwrap();

        server
            .handle_tools_call("reload_server", serde_json::json!({"reason": "test"}))
            .await
            .unwrap();

        let after = server
            .handle_tools_call("get_server_runtime_info", serde_json::json!({}))
            .await
            .unwrap();
        let after_nonce = after["data"]["reload_nonce"].as_u64().unwrap();

        assert_eq!(before_nonce + 1, after_nonce);
    }

    #[tokio::test]
    async fn readiness_audit_accepts_rich_codex_config() {
        let repo_root = temp_path("readiness-rich-config");
        write_fixture_repo(&repo_root);
        let symphony_checkout = temp_path("symphony-ready");
        let workspace_root = temp_path("workspaces-ready");
        fs::create_dir_all(symphony_checkout.join(".git")).unwrap();
        fs::create_dir_all(&workspace_root).unwrap();
        let config_path = repo_root.join("config.toml");
        fs::write(&config_path, rich_codex_config(&repo_root)).unwrap();

        let server = build_smith_compatibility_server(SmithCompatibilityOptions {
            server_name: "smith".to_string(),
            repo_root: repo_root.clone(),
            codex_config_path: config_path,
            workflow_path: repo_root.join("WORKFLOW.md"),
            symphony_checkout,
            env_file_path: repo_root.join(".env"),
            workspace_root_override: Some(workspace_root),
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        })
        .await
        .unwrap();

        let readiness = server
            .handle_tools_call("audit_workflow_readiness", serde_json::json!({}))
            .await
            .unwrap();
        let smith_check = readiness["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|value| value["name"] == "smith_mcp_config")
            .unwrap();

        assert_eq!(
            smith_check["status"],
            serde_json::Value::String("ok".to_string())
        );
        assert!(smith_check["detail"]
            .as_str()
            .unwrap()
            .contains("smith MCP configured with command"));
    }

    #[tokio::test]
    async fn rmcp_round_trip_supports_core_tools() {
        let repo_root = temp_path("rmcp");
        write_fixture_repo(&repo_root);
        let symphony_checkout = temp_path("symphony");
        fs::create_dir_all(symphony_checkout.join(".git")).unwrap();
        let workspace_root = temp_path("workspaces");
        fs::create_dir_all(&workspace_root).unwrap();
        let config_path = repo_root.join("config.toml");
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\n",
                repo_root.display()
            ),
        )
        .unwrap();

        let server = build_smith_compatibility_server(SmithCompatibilityOptions {
            server_name: "smith".to_string(),
            repo_root: repo_root.clone(),
            codex_config_path: config_path,
            workflow_path: repo_root.join("WORKFLOW.md"),
            symphony_checkout,
            env_file_path: repo_root.join(".env"),
            workspace_root_override: Some(workspace_root),
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        })
        .await
        .unwrap();

        let (server_transport, client_transport) = tokio::io::duplex(4096);
        tokio::spawn({
            let server = server.clone();
            async move {
                let running =
                    serve_server(crate::server::McpServerAdapter { server }, server_transport)
                        .await
                        .unwrap();
                let _ = running.waiting().await;
            }
        });

        let session = TestClient.serve(client_transport).await.unwrap();
        let peer = session.peer().clone();

        let tools = peer.list_all_tools().await.unwrap();
        assert!(tools
            .iter()
            .any(|tool| tool.name.as_ref() == "audit_workflow_readiness"));

        let result = peer
            .call_tool(CallToolRequestParams::new(
                "get_server_runtime_info".to_string(),
            ))
            .await
            .unwrap();
        assert!(!result.is_error.unwrap_or(false));
        assert_eq!(
            result.structured_content.unwrap()["data"]["server_name"]
                .as_str()
                .unwrap(),
            "smith"
        );
    }
}
