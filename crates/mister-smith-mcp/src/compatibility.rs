use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use mister_smith_core::CapabilityActionKind;
use mister_smith_security::DelegationService;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::config::McpServerConfig;
use crate::errors::McpError;
use crate::server::{
    tool_boundary_action, CapabilityCatalogEntry, ExposedTool, McpServer, ToolCallRequest,
    ToolHandler,
};

const DEFAULT_SERVER_NAME: &str = "smith";
const DEFAULT_LINEAR_ENDPOINT: &str = "https://api.linear.app/graphql";
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
      description
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
      parent {
        id
        identifier
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
      labels(first: 100) {
        nodes {
          id
          name
        }
      }
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

const LINEAR_ISSUE_CREATE_MUTATION: &str = r#"
mutation SmithCreateIssue($input: IssueCreateInput!) {
  issueCreate(input: $input) {
    success
    issue {
      id
      identifier
      title
      url
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
      url
    }
  }
}
"#;

const LINEAR_ISSUE_RELATION_CREATE_MUTATION: &str = r#"
mutation SmithCreateIssueRelation($input: IssueRelationCreateInput!) {
  issueRelationCreate(input: $input) {
    success
    issueRelation {
      id
    }
  }
}
"#;

const LINEAR_COMMENT_CREATE_MUTATION: &str = r#"
mutation SmithCreateComment($input: CommentCreateInput!) {
  commentCreate(input: $input) {
    success
    comment {
      id
      body
      updatedAt
      parent {
        id
      }
    }
  }
}
"#;

const LINEAR_ISSUE_QUERY: &str = r#"
query SmithIssueSnapshot($id: String!) {
  issue(id: $id) {
    id
    identifier
    title
    description
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
    parent {
      id
      identifier
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
"#;

const LINEAR_COMMENT_UPDATE_MUTATION: &str = r#"
mutation SmithUpdateComment($id: String!, $input: CommentUpdateInput!) {
  commentUpdate(id: $id, input: $input) {
    success
    comment {
      id
      body
      updatedAt
      parent {
        id
      }
    }
  }
}
"#;

const LINEAR_ISSUE_COMMENTS_QUERY: &str = r#"
query SmithIssueComments($id: String!) {
  issue(id: $id) {
    id
    identifier
    comments(first: 100) {
      nodes {
        id
        body
        updatedAt
        parent {
          id
        }
      }
    }
  }
}
"#;

const LINEAR_PROJECT_MILESTONES_QUERY: &str = r#"
query SmithProjectMilestones {
  projectMilestones(first: 100) {
    nodes {
      id
      name
      project {
        id
      }
    }
  }
}
"#;

const CODEX_WORKPAD_HEADER: &str = "## Codex Workpad";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearLabelSnapshot {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearTeamSnapshot {
    pub id: Option<String>,
    pub key: String,
    pub name: String,
    pub labels: Vec<LinearLabelSnapshot>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessAudit {
    pub repo_root: String,
    pub codex_config_path: String,
    pub env_file_path: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRuntimeInfo {
    pub server_name: String,
    pub crate_version: String,
    pub started_at: String,
    pub last_reload_at: String,
    pub reload_nonce: u64,
    pub repo_root: String,
    pub codex_config_path: String,
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
pub struct LinearIssueParentSnapshot {
    pub id: Option<String>,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearIssueSnapshot {
    pub id: Option<String>,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i64>,
    pub url: Option<String>,
    pub state: Option<LinearStateSnapshot>,
    pub project: Option<LinearProjectSnapshot>,
    pub parent: Option<LinearIssueParentSnapshot>,
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
pub struct LinearCommentSnapshot {
    pub id: String,
    pub body: String,
    pub updated_at: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearWorkpadSnapshot {
    pub comment_id: String,
    pub updated_at: Option<String>,
    pub body: String,
    pub duplicate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearWorkspaceSummary {
    pub issue_count: usize,
    pub project_count: usize,
    pub team_count: usize,
    pub issues_by_state: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneSnapshot {
    pub repo: RepoSnapshot,
    pub readiness: ReadinessAudit,
    pub github: GitHubSnapshot,
    pub linear: Option<LinearWorkspaceSummary>,
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
pub struct ReviewMergeStatus {
    pub open_pull_request_count: usize,
    pub review_blockers: Vec<String>,
    pub merge_ready_pull_requests: Vec<String>,
    pub stale_pull_requests: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueExecutionSnapshot {
    pub issue: Option<LinearIssueSnapshot>,
    pub workpad: Option<LinearWorkpadSnapshot>,
    pub matching_pull_requests: Vec<GitHubPullRequest>,
    pub execution_state: String,
    pub blocker_summaries: Vec<String>,
    pub workpad_status: String,
    pub next_step_hint: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectExecutionPlan {
    pub issue: Option<LinearIssueSnapshot>,
    pub workpad: Option<LinearWorkpadSnapshot>,
    pub feature_dir: Option<String>,
    pub tasks_path: Option<String>,
    pub should_use_speckit: bool,
    pub docs_to_read: Vec<String>,
    pub validation_commands: Vec<String>,
    pub next_recommended_action: String,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssueSaveResult {
    pub action: String,
    pub issue: LinearIssueSnapshot,
    pub applied_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearWorkpadSaveResult {
    pub action: String,
    pub issue_identifier: String,
    pub comment_id: String,
    pub duplicate_count: usize,
    pub workpad: LinearWorkpadSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedBacklogSlice {
    pub title: String,
    pub issue_identifier: String,
    pub action: String,
    pub blocker_identifiers: Vec<String>,
    pub workpad_action: Option<String>,
    pub issue: LinearIssueSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogSliceMaterialization {
    pub parent_issue_identifier: String,
    pub milestone: Option<String>,
    pub results: Vec<MaterializedBacklogSlice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLifecycleResolution {
    pub issue: Option<LinearIssueSnapshot>,
    pub issue_identifier: String,
    pub next_recommended_action: String,
    pub required_mutations: Vec<String>,
    pub blocking_reasons: Vec<String>,
    pub review_state: String,
    pub pr_correlation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphPacket {
    pub issue_identifier: String,
    pub plan_path: Option<String>,
    pub mode: String,
    pub goal: String,
    pub current_context: String,
    pub source_docs: Vec<String>,
    pub workflow_requirements: Vec<String>,
    pub validation_requirements: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub definition_of_done: Vec<String>,
    pub rendered_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphOutcomeRecord {
    pub issue_identifier: String,
    pub outcome_status: String,
    pub workpad_action: String,
    pub target_state: Option<String>,
    pub comment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecKitContext {
    pub issue_identifier: Option<String>,
    pub should_use_speckit: bool,
    pub feature_dir: Option<String>,
    pub source_docs: Vec<String>,
    pub packet_summary: String,
    pub next_command_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedSpecKitSlice {
    pub title: String,
    pub description: String,
    pub blocked_by: Vec<String>,
    pub workpad_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecKitTranslation {
    pub feature_dir: Option<String>,
    pub tasks_path: String,
    pub apply_requested: bool,
    pub packet_summary: String,
    pub translated_slices: Vec<TranslatedSpecKitSlice>,
    pub materialization: Option<BacklogSliceMaterialization>,
}

#[derive(Debug, Clone, Default)]
struct LinearProjectMilestoneSnapshot {
    id: Option<String>,
    name: String,
    project_id: Option<String>,
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
}

#[derive(Debug, Clone)]
pub struct SmithCompatibilityOptions {
    pub server_name: String,
    pub repo_root: PathBuf,
    pub codex_config_path: PathBuf,
    pub env_file_path: PathBuf,
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
        let env_file_path = repo_root.join(".env");

        Self {
            server_name: DEFAULT_SERVER_NAME.to_string(),
            repo_root,
            codex_config_path,
            env_file_path,
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
    registered_capability_catalog: Vec<CapabilityCatalogEntry>,
}

impl Default for SmithRuntimeState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            started_at: now,
            last_reload_at: now,
            reload_nonce: 0,
            registered_tool_names: Vec::new(),
            registered_capability_catalog: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct CompatibilityCaches {
    linear_workspace: Option<LinearWorkspaceData>,
}

#[derive(Debug, Clone, Default)]
struct LinearWorkspaceData {
    projects: Vec<LinearProjectSnapshot>,
    issues: Vec<LinearIssueSnapshot>,
    teams: Vec<LinearTeamSnapshot>,
    states_by_team: BTreeMap<String, Vec<LinearStateSnapshot>>,
}

#[derive(Debug, Clone)]
struct IssueExecutionContext {
    issue: Option<LinearIssueSnapshot>,
    workpad: Option<LinearWorkpadSnapshot>,
    matching_pull_requests: Vec<GitHubPullRequest>,
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

    async fn set_registered_capability_catalog(&self, catalog: Vec<CapabilityCatalogEntry>) {
        self.runtime.write().await.registered_capability_catalog = catalog;
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
            codex_config_path: self.options.codex_config_path.display().to_string(),
            tool_names: runtime.registered_tool_names.clone(),
        }
    }

    async fn clear_caches(&self) {
        *self.caches.write().await = CompatibilityCaches::default();
    }

    async fn linear_workspace(&self) -> Result<LinearWorkspaceData, String> {
        if let Some(workspace) = self.caches.read().await.linear_workspace.clone() {
            return Ok(workspace);
        }

        let data = self
            .linear_graphql(LINEAR_WORKSPACE_QUERY, serde_json::json!({}))
            .await?;
        let workspace = parse_linear_workspace(&data);
        self.caches.write().await.linear_workspace = Some(workspace.clone());
        Ok(workspace)
    }

    async fn linear_workspace_summary(&self) -> Result<LinearWorkspaceSummary, String> {
        let workspace = self.linear_workspace().await?;
        let mut issues_by_state = BTreeMap::new();
        for issue in &workspace.issues {
            let key = issue
                .state
                .as_ref()
                .map(|state| state.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            *issues_by_state.entry(key).or_insert(0) += 1;
        }

        Ok(LinearWorkspaceSummary {
            issue_count: workspace.issues.len(),
            project_count: workspace.projects.len(),
            team_count: workspace.teams.len(),
            issues_by_state,
        })
    }

    async fn linear_graphql(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let api_key = self.linear_api_key().ok_or_else(|| {
            "LINEAR_API_KEY is not available from the environment or repo .env".to_string()
        })?;
        let response = reqwest::Client::new()
            .post(&self.options.linear_endpoint)
            .header(reqwest::header::AUTHORIZATION, api_key)
            .json(&serde_json::json!({
                "query": query,
                "variables": variables,
            }))
            .send()
            .await
            .map_err(|err| format!("failed to query Linear: {err}"))?;

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

        body.get("data")
            .cloned()
            .ok_or_else(|| "Linear response missing data".to_string())
    }

    fn linear_api_key(&self) -> Option<String> {
        env::var("LINEAR_API_KEY")
            .ok()
            .or_else(|| read_env_value(&self.options.env_file_path, "LINEAR_API_KEY"))
    }

    async fn linear_issue_create(
        &self,
        input: serde_json::Value,
    ) -> Result<LinearIssueSnapshot, String> {
        let data = self
            .linear_graphql(
                LINEAR_ISSUE_CREATE_MUTATION,
                serde_json::json!({ "input": input }),
            )
            .await?;
        let success = data
            .pointer("/issueCreate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !success {
            return Err(format!("Linear issueCreate did not report success: {data}"));
        }
        let issue = data
            .pointer("/issueCreate/issue")
            .ok_or_else(|| "Linear issueCreate response missing issue".to_string())?;
        Ok(parse_linear_issue(issue))
    }

    async fn linear_issue_update(
        &self,
        issue_id: Option<String>,
        input: serde_json::Value,
    ) -> Result<LinearIssueSnapshot, String> {
        let issue_id = issue_id.ok_or_else(|| "issue id is missing".to_string())?;
        let data = self
            .linear_graphql(
                LINEAR_ISSUE_UPDATE_MUTATION,
                serde_json::json!({
                    "id": issue_id,
                    "input": input,
                }),
            )
            .await?;
        let success = data
            .pointer("/issueUpdate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !success {
            return Err(format!("Linear issueUpdate did not report success: {data}"));
        }
        let issue = data
            .pointer("/issueUpdate/issue")
            .ok_or_else(|| "Linear issueUpdate response missing issue".to_string())?;
        Ok(parse_linear_issue(issue))
    }

    async fn linear_issue_relation_create(&self, input: serde_json::Value) -> Result<(), String> {
        let data = self
            .linear_graphql(
                LINEAR_ISSUE_RELATION_CREATE_MUTATION,
                serde_json::json!({ "input": input }),
            )
            .await?;
        let success = data
            .pointer("/issueRelationCreate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !success {
            return Err(format!(
                "Linear issueRelationCreate did not report success: {data}"
            ));
        }
        Ok(())
    }

    async fn linear_comment_create(
        &self,
        input: serde_json::Value,
    ) -> Result<LinearCommentSnapshot, String> {
        let data = self
            .linear_graphql(
                LINEAR_COMMENT_CREATE_MUTATION,
                serde_json::json!({ "input": input }),
            )
            .await?;
        let success = data
            .pointer("/commentCreate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !success {
            return Err(format!(
                "Linear commentCreate did not report success: {data}"
            ));
        }
        let comment = data
            .pointer("/commentCreate/comment")
            .ok_or_else(|| "Linear commentCreate response missing comment".to_string())?;
        Ok(parse_linear_comment(comment))
    }

    async fn linear_comment_update(
        &self,
        comment_id: &str,
        input: serde_json::Value,
    ) -> Result<LinearCommentSnapshot, String> {
        let data = self
            .linear_graphql(
                LINEAR_COMMENT_UPDATE_MUTATION,
                serde_json::json!({
                    "id": comment_id,
                    "input": input,
                }),
            )
            .await?;
        let success = data
            .pointer("/commentUpdate/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !success {
            return Err(format!(
                "Linear commentUpdate did not report success: {data}"
            ));
        }
        let comment = data
            .pointer("/commentUpdate/comment")
            .ok_or_else(|| "Linear commentUpdate response missing comment".to_string())?;
        Ok(parse_linear_comment(comment))
    }

    async fn linear_issue_comments(
        &self,
        issue_id: &str,
    ) -> Result<Vec<LinearCommentSnapshot>, String> {
        let data = self
            .linear_graphql(
                LINEAR_ISSUE_COMMENTS_QUERY,
                serde_json::json!({ "id": issue_id }),
            )
            .await?;
        Ok(data
            .pointer("/issue/comments/nodes")
            .and_then(serde_json::Value::as_array)
            .map(|nodes| nodes.iter().map(parse_linear_comment).collect())
            .unwrap_or_default())
    }

    async fn linear_project_milestones(
        &self,
    ) -> Result<Vec<LinearProjectMilestoneSnapshot>, String> {
        let data = self
            .linear_graphql(LINEAR_PROJECT_MILESTONES_QUERY, serde_json::json!({}))
            .await?;
        Ok(data
            .pointer("/projectMilestones/nodes")
            .and_then(serde_json::Value::as_array)
            .map(|nodes| {
                nodes
                    .iter()
                    .map(parse_linear_project_milestone)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default())
    }

    async fn linear_issue_snapshot(
        &self,
        issue_identifier: &str,
    ) -> Result<Option<LinearIssueSnapshot>, String> {
        let data = self
            .linear_graphql(
                LINEAR_ISSUE_QUERY,
                serde_json::json!({ "id": issue_identifier }),
            )
            .await?;
        Ok(data.get("issue").map(parse_linear_issue))
    }

    async fn load_issue_execution_context(
        &self,
        issue_identifier: &str,
    ) -> Result<IssueExecutionContext, McpError> {
        let issue = self
            .linear_issue_snapshot(issue_identifier)
            .await
            .map_err(McpError::ToolCallFailed)?;
        let workpad = if let Some(issue) = issue.as_ref() {
            if let Some(issue_id) = issue.id.as_deref() {
                self.linear_issue_comments(issue_id)
                    .await
                    .ok()
                    .and_then(|comments| select_current_workpad(&comments))
            } else {
                None
            }
        } else {
            None
        };
        let github = self.github_snapshot().await;
        let matching_pull_requests = github
            .open_pull_requests
            .into_iter()
            .filter(|pr| {
                pr.title.contains(issue_identifier)
                    || pr
                        .head_ref_name
                        .to_lowercase()
                        .contains(&issue_identifier.to_lowercase())
            })
            .collect::<Vec<_>>();

        Ok(IssueExecutionContext {
            issue,
            workpad,
            matching_pull_requests,
        })
    }

    async fn attach_blockers_to_issue(
        &self,
        issue: &LinearIssueSnapshot,
        blocker_identifiers: &[String],
    ) -> Result<Vec<String>, String> {
        let Some(issue_id) = issue.id.as_deref() else {
            return Err(format!(
                "issue {} is missing a Linear id for blocker mutation",
                issue.identifier
            ));
        };
        let workspace = self.linear_workspace().await?;
        let mut attached = Vec::new();

        for blocker_identifier in blocker_identifiers {
            if issue
                .blocked_by
                .iter()
                .any(|blocker| blocker.identifier.eq_ignore_ascii_case(blocker_identifier))
            {
                attached.push(blocker_identifier.clone());
                continue;
            }
            let Some(blocker_issue) =
                find_linear_issue(&workspace, None, Some(blocker_identifier.as_str()))
            else {
                return Err(format!(
                    "could not resolve blocker issue {}",
                    blocker_identifier
                ));
            };
            let Some(blocker_id) = blocker_issue.id.as_deref() else {
                return Err(format!(
                    "blocker issue {} is missing a Linear id",
                    blocker_identifier
                ));
            };

            self.linear_issue_relation_create(serde_json::json!({
                "type": "blocks",
                "issueId": blocker_id,
                "relatedIssueId": issue_id,
            }))
            .await?;
            attached.push(blocker_identifier.clone());
        }

        if !attached.is_empty() {
            self.clear_caches().await;
        }

        Ok(attached)
    }

    async fn audit_workflow_readiness(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        json_response(self.collect_readiness_audit().await)
    }

    async fn collect_readiness_audit(&self) -> ToolResponse<ReadinessAudit> {
        let config_status = inspect_codex_smith_config(&self.options.codex_config_path);
        let linear_key_available = self.linear_api_key().is_some();

        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut blockers = Vec::new();
        let mut evidence = vec![
            EvidenceItem {
                label: "repo_root".to_string(),
                detail: self.options.repo_root.display().to_string(),
            },
        ];

        checks.push(file_check(
            "repo_root",
            self.options.repo_root.exists(),
            "blocker",
            format!("repo root {}", self.options.repo_root.display()),
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
                env_file_path: self.options.env_file_path.display().to_string(),
                checks,
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

    async fn describe_external_capabilities(
        &self,
        request: ToolCallRequest,
    ) -> Result<serde_json::Value, McpError> {
        let runtime = self.runtime.read().await;
        let catalog = runtime.registered_capability_catalog.clone();
        drop(runtime);

        let observed_delegation = request.context.delegation.as_ref().map(|envelope| {
            serde_json::json!({
                "descriptor_id": envelope.descriptor_id(),
                "capability_id": envelope.capability.capability_id,
                "scope": envelope.capability.scope,
                "revocation_state": envelope.capability.revocation_state,
                "chain_depth": envelope.provenance.links.len(),
                "root_issuer": envelope.provenance.root_issuer,
                "terminal_capability": envelope.provenance.terminal_capability,
                "action": envelope.action,
            })
        });

        let discovery_surface = catalog
            .iter()
            .find(|entry| entry.tool_name == "describe_external_capabilities")
            .cloned();

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: format!(
                "described {} external MCP capability surfaces",
                catalog.len()
            ),
            evidence: vec![EvidenceItem {
                label: "capability_count".to_string(),
                detail: catalog.len().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec!["get_server_runtime_info".to_string()],
            blocking_issues: Vec::new(),
            data: serde_json::json!({
                "discovery_surface": discovery_surface,
                "observed_delegation": observed_delegation,
                "capabilities": catalog,
                "notes": [
                    "Capability descriptors are also published in MCP tools/list metadata under mister_smith_capability.",
                    "This catalog tool is the bounded discovery surface and requires a Discover action envelope.",
                    "Delegated execute calls preserve the same descriptor and policy tuple at the MCP tools/call boundary."
                ]
            }),
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
                    route: "direct_execution".to_string(),
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
            &[
                "codex workpad",
                "workpad comment",
                "update workpad",
                "save workpad",
                "create child issue",
                "child issue",
                "create issue",
                "update issue",
                "linear comment",
                "issue comment",
            ],
        ) {
            let preferred_tool = if contains_any(
                &normalized,
                &[
                    "codex workpad",
                    "workpad comment",
                    "update workpad",
                    "save workpad",
                    "linear comment",
                    "issue comment",
                ],
            ) {
                "save_issue_workpad"
            } else {
                "save_linear_issue"
            };
            (
                "linear_workflow".to_string(),
                "request targets Smith-managed Linear issue or workpad mutation".to_string(),
                preferred_tool.to_string(),
                vec![
                    "save_linear_issue".to_string(),
                    "save_issue_workpad".to_string(),
                    "get_issue_execution_snapshot".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &[
                "backlog slice",
                "backlog slicing",
                "child issue creation",
                "materialize slices",
                "translate speckit tasks",
                "task-pack translation",
                "sub-issue creation",
            ],
        ) {
            let preferred_tool = if contains_any(
                &normalized,
                &["speckit", "task-pack", "tasks.md", "translate"],
            ) {
                "translate_speckit_tasks"
            } else {
                "materialize_backlog_slices"
            };
            (
                "backlog_slicing".to_string(),
                "request targets Smith-managed backlog slicing or SpecKit task translation"
                    .to_string(),
                preferred_tool.to_string(),
                vec![
                    "materialize_backlog_slices".to_string(),
                    "translate_speckit_tasks".to_string(),
                    "prepare_direct_execution".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &[
                "implement",
                "execution",
                "direct execution",
                "continue execution",
                "fix issue",
                "packet",
                "spec",
                "task pack",
                "tasks.md",
                "speckit",
            ],
        ) {
            (
                "direct_execution".to_string(),
                "request targets direct Codex execution preparation".to_string(),
                "prepare_direct_execution".to_string(),
                vec![
                    "prepare_direct_execution".to_string(),
                    "get_issue_execution_snapshot".to_string(),
                    "prepare_speckit_context".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &[
                "issue lifecycle",
                "next action for issue",
                "execution recovery",
                "review state",
                "next step",
            ],
        ) {
            (
                "issue_lifecycle".to_string(),
                "request targets issue lifecycle resolution or recovery routing".to_string(),
                "resolve_issue_lifecycle".to_string(),
                vec![
                    "resolve_issue_lifecycle".to_string(),
                    "get_issue_execution_snapshot".to_string(),
                    "prepare_direct_execution".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &[
                "workflow system",
                "development workflow",
                "development system",
                "workflow architecture",
                "operating model",
                "smith-first",
                "ralph",
                "speckit",
                "prompt",
                "workpad",
                "handoff",
                "skills",
                "repo context",
                "context gathering",
            ],
        ) {
            (
                "direct_execution".to_string(),
                "request targets Smith-first direct execution design or chaining".to_string(),
                "prepare_direct_execution".to_string(),
                vec![
                    "prepare_direct_execution".to_string(),
                    "get_control_plane_snapshot".to_string(),
                    "prepare_speckit_context".to_string(),
                ],
            )
        } else if contains_any(
            &normalized,
            &["review", "merge", "pull request", "github pr"],
        ) {
            (
                "review_merge".to_string(),
                "request targets direct review or merge reconciliation".to_string(),
                "review_merge_status".to_string(),
                vec![
                    "review_merge_status".to_string(),
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
                "direct_execution".to_string(),
                "request targets direct issue state or execution routing".to_string(),
                "prepare_direct_execution".to_string(),
                vec![
                    "prepare_direct_execution".to_string(),
                    "get_issue_execution_snapshot".to_string(),
                ],
            )
        } else {
            (
                "direct_execution".to_string(),
                "request needs a broad control-plane snapshot first".to_string(),
                "get_control_plane_snapshot".to_string(),
                vec![
                    "get_control_plane_snapshot".to_string(),
                    "prepare_direct_execution".to_string(),
                ],
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
        let github = self.github_snapshot().await;
        let linear = self.linear_workspace_summary().await.ok();
        let repo = self.repo_snapshot().await;

        let mut warnings = readiness.warnings.clone();
        if linear.is_none() {
            warnings
                .push("Linear snapshot unavailable; returning repo/runtime state only".to_string());
        }

        let status = if matches!(readiness.status, CompatibilityStatus::Blocked) {
            CompatibilityStatus::Blocked
        } else if matches!(readiness.status, CompatibilityStatus::Ok) {
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
                    label: "open_pull_requests".to_string(),
                    detail: github.open_pull_requests.len().to_string(),
                },
            ],
            warnings,
            recommended_next_tools: vec![
                "audit_workflow_readiness".to_string(),
                "prepare_direct_execution".to_string(),
                "get_issue_execution_snapshot".to_string(),
            ],
            blocking_issues: readiness.blocking_issues.clone(),
            data: ControlPlaneSnapshot {
                repo,
                readiness: readiness.data,
                github,
                linear,
            },
        })
    }

    async fn plan_workspace_adjustments(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let apply = bool_param(&params, "apply");
        let readiness = self.collect_readiness_audit().await;
        let mut adjustments = Vec::new();
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
                description: "Restore repo .env with LINEAR_API_KEY for Smith MCP Linear access"
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
            recommended_next_tools: vec!["audit_workflow_readiness".to_string()],
            blocking_issues: readiness.blocking_issues,
            data: WorkspaceAdjustmentPlan {
                apply_requested: apply,
                adjustments,
            },
        })
    }

    async fn save_linear_issue(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let workspace = match self.linear_workspace().await {
            Ok(workspace) => workspace,
            Err(err) => {
                return json_response(blocked_response(
                    format!("cannot save a Linear issue without Linear workspace access: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err.clone()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: LinearIssueSnapshot::default(),
                        applied_fields: Vec::new(),
                    },
                ))
            }
        };

        let issue_identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let issue_id = string_param(&params, "issue_id");
        let existing_issue =
            find_linear_issue(&workspace, issue_id.as_deref(), issue_identifier.as_deref())
                .cloned();
        let creating = existing_issue.is_none();

        let title = string_param(&params, "title");
        if creating
            && title
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            return json_response(blocked_response(
                "save_linear_issue requires title when creating a new issue",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["title parameter is required for issue creation".to_string()],
                LinearIssueSaveResult {
                    action: "blocked".to_string(),
                    issue: LinearIssueSnapshot::default(),
                    applied_fields: Vec::new(),
                },
            ));
        }

        let requested_team_key = string_param(&params, "team_key");
        let team_key = requested_team_key
            .clone()
            .or_else(|| {
                existing_issue
                    .as_ref()
                    .and_then(|issue| issue.team_key.clone())
            })
            .unwrap_or_else(|| "MS".to_string());
        let team = find_linear_team(&workspace, &team_key).cloned();

        let project_name = string_param(&params, "project").or_else(|| {
            if creating {
                Some("MisterSmith Validated Backlog".to_string())
            } else {
                None
            }
        });
        let project = project_name
            .as_deref()
            .and_then(|name| find_linear_project(&workspace, name).cloned());
        if project_name.is_some() && project.is_none() {
            return json_response(blocked_response(
                format!(
                    "could not resolve Linear project {}",
                    project_name.unwrap_or_default()
                ),
                vec!["get_control_plane_snapshot".to_string()],
                vec!["project parameter did not match a cached Linear project".to_string()],
                LinearIssueSaveResult {
                    action: "blocked".to_string(),
                    issue: existing_issue.unwrap_or_default(),
                    applied_fields: Vec::new(),
                },
            ));
        }

        let state_name = string_param(&params, "state").or_else(|| {
            if creating {
                Some("Backlog".to_string())
            } else {
                None
            }
        });
        let state = state_name.as_deref().and_then(|name| {
            team.as_ref()
                .and_then(|team| find_linear_state(&workspace, &team.key, name).cloned())
        });
        if state_name.is_some() && state.is_none() {
            return json_response(blocked_response(
                format!(
                    "could not resolve state {} for team {}",
                    state_name.unwrap_or_default(),
                    team_key
                ),
                vec!["get_control_plane_snapshot".to_string()],
                vec!["state parameter did not match a cached Linear team state".to_string()],
                LinearIssueSaveResult {
                    action: "blocked".to_string(),
                    issue: existing_issue.unwrap_or_default(),
                    applied_fields: Vec::new(),
                },
            ));
        }

        let parent_identifier = string_param(&params, "parent_identifier");
        let parent_issue = parent_identifier
            .as_deref()
            .and_then(|identifier| find_linear_issue(&workspace, None, Some(identifier)).cloned());
        if parent_identifier.is_some() && parent_issue.is_none() {
            return json_response(blocked_response(
                format!(
                    "could not resolve parent issue {}",
                    parent_identifier.unwrap_or_default()
                ),
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["parent_identifier did not match a cached Linear issue".to_string()],
                LinearIssueSaveResult {
                    action: "blocked".to_string(),
                    issue: existing_issue.unwrap_or_default(),
                    applied_fields: Vec::new(),
                },
            ));
        }

        let labels = string_array_param(&params, "labels");
        let label_ids = if let Some(label_names) = labels.as_ref() {
            let Some(team) = team.as_ref() else {
                return json_response(blocked_response(
                    format!("could not resolve team {} for label mutation", team_key),
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["team resolution is required before mutating labels".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            let (resolved_ids, missing_labels) = resolve_linear_label_ids(team, label_names);
            if !missing_labels.is_empty() {
                return json_response(blocked_response(
                    format!("could not resolve label(s): {}", missing_labels.join(", ")),
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["labels parameter included unknown Linear labels".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            }
            Some(resolved_ids)
        } else {
            None
        };

        let priority = i64_param(&params, "priority");
        let milestone_name = string_param(&params, "milestone");
        let blocked_by = string_array_param(&params, "blocked_by").unwrap_or_default();
        let milestone_project = project.clone().or_else(|| {
            existing_issue
                .as_ref()
                .and_then(|issue| issue.project.clone())
        });
        let milestone = if let Some(name) = milestone_name.as_deref() {
            let Some(project) = milestone_project.as_ref() else {
                return json_response(blocked_response(
                    "milestone resolution requires a project",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["provide project when setting milestone".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            let Some(project_id) = project.id.as_deref() else {
                return json_response(blocked_response(
                    "resolved project is missing an id",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["project id is required to resolve a milestone".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            let milestones = match self.linear_project_milestones().await {
                Ok(milestones) => milestones,
                Err(err) => {
                    return json_response(blocked_response(
                        format!("failed to load project milestones: {err}"),
                        vec!["audit_workflow_readiness".to_string()],
                        vec![err],
                        LinearIssueSaveResult {
                            action: "blocked".to_string(),
                            issue: existing_issue.unwrap_or_default(),
                            applied_fields: Vec::new(),
                        },
                    ))
                }
            };
            let resolved = milestones.into_iter().find(|milestone| {
                milestone
                    .project_id
                    .as_deref()
                    .map(|candidate| candidate == project_id)
                    .unwrap_or(false)
                    && milestone.name.eq_ignore_ascii_case(name)
            });
            let Some(resolved) = resolved else {
                return json_response(blocked_response(
                    format!("could not resolve milestone {}", name),
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["milestone parameter did not match a project milestone".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            Some(resolved)
        } else {
            None
        };

        let mut input = serde_json::Map::new();
        let mut applied_fields = Vec::new();

        if let Some(title) = title {
            input.insert("title".to_string(), serde_json::json!(title));
            applied_fields.push("title".to_string());
        }
        if let Some(description) = string_param(&params, "description") {
            input.insert("description".to_string(), serde_json::json!(description));
            applied_fields.push("description".to_string());
        }
        if creating || requested_team_key.is_some() {
            let Some(team) = team.as_ref() else {
                return json_response(blocked_response(
                    format!("could not resolve team {}", team_key),
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["team_key did not match a cached Linear team".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            let Some(team_id) = team.id.clone() else {
                return json_response(blocked_response(
                    "resolved team is missing an id",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["team id is required for Linear issue mutation".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            input.insert("teamId".to_string(), serde_json::json!(team_id));
            applied_fields.push("team".to_string());
        }
        if let Some(project) = project.as_ref() {
            let Some(project_id) = project.id.clone() else {
                return json_response(blocked_response(
                    "resolved project is missing an id",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["project id is required for Linear issue mutation".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            input.insert("projectId".to_string(), serde_json::json!(project_id));
            applied_fields.push("project".to_string());
        }
        if let Some(state) = state.as_ref() {
            let Some(state_id) = state.id.clone() else {
                return json_response(blocked_response(
                    "resolved state is missing an id",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["state id is required for Linear issue mutation".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            input.insert("stateId".to_string(), serde_json::json!(state_id));
            applied_fields.push("state".to_string());
        }
        if let Some(parent_issue) = parent_issue.as_ref() {
            let Some(parent_id) = parent_issue.id.clone() else {
                return json_response(blocked_response(
                    "resolved parent issue is missing an id",
                    vec!["get_issue_execution_snapshot".to_string()],
                    vec!["parent issue id is required for Linear issue mutation".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            input.insert("parentId".to_string(), serde_json::json!(parent_id));
            applied_fields.push("parent".to_string());
        }
        if let Some(priority) = priority {
            input.insert("priority".to_string(), serde_json::json!(priority));
            applied_fields.push("priority".to_string());
        }
        if let Some(label_ids) = label_ids {
            input.insert("labelIds".to_string(), serde_json::json!(label_ids));
            applied_fields.push("labels".to_string());
        }
        if let Some(milestone) = milestone.as_ref() {
            let Some(milestone_id) = milestone.id.clone() else {
                return json_response(blocked_response(
                    "resolved milestone is missing an id",
                    vec!["get_control_plane_snapshot".to_string()],
                    vec!["milestone id is required for Linear issue mutation".to_string()],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields: Vec::new(),
                    },
                ));
            };
            input.insert(
                "projectMilestoneId".to_string(),
                serde_json::json!(milestone_id),
            );
            applied_fields.push("milestone".to_string());
        }

        if !creating && input.is_empty() && blocked_by.is_empty() {
            return json_response(blocked_response(
                "save_linear_issue received no fields to update",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["provide at least one mutable field".to_string()],
                LinearIssueSaveResult {
                    action: "blocked".to_string(),
                    issue: existing_issue.unwrap_or_default(),
                    applied_fields,
                },
            ));
        }

        let action = if creating { "created" } else { "updated" };
        let saved_issue = if creating {
            self.linear_issue_create(serde_json::Value::Object(input))
                .await
        } else {
            self.linear_issue_update(
                existing_issue.as_ref().and_then(|issue| issue.id.clone()),
                serde_json::Value::Object(input),
            )
            .await
        };

        let saved_issue = match saved_issue {
            Ok(issue) => issue,
            Err(err) => {
                return json_response(blocked_response(
                    format!("failed to {action} Linear issue: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: existing_issue.unwrap_or_default(),
                        applied_fields,
                    },
                ))
            }
        };

        self.clear_caches().await;
        let mut refreshed_issue = self
            .linear_workspace()
            .await
            .ok()
            .and_then(|workspace| {
                find_linear_issue(
                    &workspace,
                    saved_issue.id.as_deref(),
                    Some(saved_issue.identifier.as_str()),
                )
                .cloned()
            })
            .unwrap_or(saved_issue);

        if !blocked_by.is_empty() {
            let attached = self
                .attach_blockers_to_issue(&refreshed_issue, &blocked_by)
                .await;
            if let Err(err) = attached {
                return json_response(blocked_response(
                    format!(
                        "issue {} was saved but blocker attachment failed: {err}",
                        refreshed_issue.identifier
                    ),
                    vec!["get_issue_execution_snapshot".to_string()],
                    vec![err],
                    LinearIssueSaveResult {
                        action: "blocked".to_string(),
                        issue: refreshed_issue,
                        applied_fields,
                    },
                ));
            }
            refreshed_issue = self
                .linear_issue_snapshot(&refreshed_issue.identifier)
                .await
                .ok()
                .flatten()
                .unwrap_or(refreshed_issue);
            applied_fields.push("blocked_by".to_string());
        }

        json_response(ToolResponse {
            status: CompatibilityStatus::Applied,
            summary: format!("Smith {action} Linear issue {}", refreshed_issue.identifier),
            evidence: vec![
                EvidenceItem {
                    label: "issue_identifier".to_string(),
                    detail: refreshed_issue.identifier.clone(),
                },
                EvidenceItem {
                    label: "issue_action".to_string(),
                    detail: action.to_string(),
                },
            ],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "save_issue_workpad".to_string(),
                "get_issue_execution_snapshot".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: LinearIssueSaveResult {
                action: action.to_string(),
                issue: refreshed_issue,
                applied_fields,
            },
        })
    }

    async fn save_issue_workpad(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let issue_identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let issue_id = string_param(&params, "issue_id");
        let body = string_param(&params, "body");

        if body
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return json_response(blocked_response(
                "save_issue_workpad requires a non-empty body",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["body parameter is required".to_string()],
                LinearWorkpadSaveResult {
                    action: "blocked".to_string(),
                    issue_identifier: issue_identifier.unwrap_or_default(),
                    comment_id: String::new(),
                    duplicate_count: 0,
                    workpad: LinearWorkpadSnapshot::default(),
                },
            ));
        }

        let workspace = match self.linear_workspace().await {
            Ok(workspace) => workspace,
            Err(err) => {
                return json_response(blocked_response(
                    format!("cannot save a workpad without Linear workspace access: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err.clone()],
                    LinearWorkpadSaveResult {
                        action: "blocked".to_string(),
                        issue_identifier: issue_identifier.unwrap_or_default(),
                        comment_id: String::new(),
                        duplicate_count: 0,
                        workpad: LinearWorkpadSnapshot::default(),
                    },
                ))
            }
        };

        let issue = find_linear_issue(&workspace, issue_id.as_deref(), issue_identifier.as_deref())
            .cloned();
        let Some(issue) = issue else {
            return json_response(blocked_response(
                "save_issue_workpad could not resolve the issue",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["issue_identifier or issue_id must resolve to a Linear issue".to_string()],
                LinearWorkpadSaveResult {
                    action: "blocked".to_string(),
                    issue_identifier: issue_identifier.unwrap_or_default(),
                    comment_id: String::new(),
                    duplicate_count: 0,
                    workpad: LinearWorkpadSnapshot::default(),
                },
            ));
        };
        let Some(resolved_issue_id) = issue.id.clone() else {
            return json_response(blocked_response(
                format!("issue {} is missing a Linear id", issue.identifier),
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["issue id is required to mutate the workpad comment".to_string()],
                LinearWorkpadSaveResult {
                    action: "blocked".to_string(),
                    issue_identifier: issue.identifier,
                    comment_id: String::new(),
                    duplicate_count: 0,
                    workpad: LinearWorkpadSnapshot::default(),
                },
            ));
        };

        let comments = match self.linear_issue_comments(&resolved_issue_id).await {
            Ok(comments) => comments,
            Err(err) => {
                return json_response(blocked_response(
                    format!("failed to load comments for {}: {err}", issue.identifier),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err],
                    LinearWorkpadSaveResult {
                        action: "blocked".to_string(),
                        issue_identifier: issue.identifier,
                        comment_id: String::new(),
                        duplicate_count: 0,
                        workpad: LinearWorkpadSnapshot::default(),
                    },
                ))
            }
        };

        let normalized_body = normalize_workpad_body(body.as_deref().unwrap_or_default());
        let existing_workpad = select_current_workpad(&comments);
        let duplicate_count = existing_workpad
            .as_ref()
            .map(|workpad| workpad.duplicate_count)
            .unwrap_or(0);
        let (action_name, comment) = if let Some(workpad) = existing_workpad.as_ref() {
            let updated = self
                .linear_comment_update(
                    &workpad.comment_id,
                    serde_json::json!({ "body": normalized_body }),
                )
                .await;
            match updated {
                Ok(comment) => ("updated", comment),
                Err(err) => {
                    return json_response(blocked_response(
                        format!("failed to update workpad for {}: {err}", issue.identifier),
                        vec!["audit_workflow_readiness".to_string()],
                        vec![err],
                        LinearWorkpadSaveResult {
                            action: "blocked".to_string(),
                            issue_identifier: issue.identifier,
                            comment_id: workpad.comment_id.clone(),
                            duplicate_count,
                            workpad: workpad.clone(),
                        },
                    ))
                }
            }
        } else {
            let created = self
                .linear_comment_create(serde_json::json!({
                    "issueId": resolved_issue_id,
                    "body": normalized_body,
                }))
                .await;
            match created {
                Ok(comment) => ("created", comment),
                Err(err) => {
                    return json_response(blocked_response(
                        format!("failed to create workpad for {}: {err}", issue.identifier),
                        vec!["audit_workflow_readiness".to_string()],
                        vec![err],
                        LinearWorkpadSaveResult {
                            action: "blocked".to_string(),
                            issue_identifier: issue.identifier,
                            comment_id: String::new(),
                            duplicate_count,
                            workpad: LinearWorkpadSnapshot::default(),
                        },
                    ))
                }
            }
        };

        let workpad = LinearWorkpadSnapshot {
            comment_id: comment.id.clone(),
            updated_at: comment.updated_at.clone(),
            body: comment.body.clone(),
            duplicate_count,
        };

        json_response(ToolResponse {
            status: CompatibilityStatus::Applied,
            summary: format!(
                "Smith {} the Codex workpad for {}",
                action_name, issue.identifier
            ),
            evidence: vec![
                EvidenceItem {
                    label: "issue_identifier".to_string(),
                    detail: issue.identifier.clone(),
                },
                EvidenceItem {
                    label: "comment_id".to_string(),
                    detail: workpad.comment_id.clone(),
                },
            ],
            warnings: if duplicate_count > 0 {
                vec![format!(
                    "{} additional top-level Codex workpad comment(s) remain on {}",
                    duplicate_count, issue.identifier
                )]
            } else {
                Vec::new()
            },
            recommended_next_tools: vec!["get_issue_execution_snapshot".to_string()],
            blocking_issues: Vec::new(),
            data: LinearWorkpadSaveResult {
                action: action_name.to_string(),
                issue_identifier: issue.identifier,
                comment_id: workpad.comment_id.clone(),
                duplicate_count,
                workpad,
            },
        })
    }

    async fn materialize_backlog_slices(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let parent_issue_identifier = string_param(&params, "parent_issue_identifier")
            .or_else(|| string_param(&params, "parent_identifier"));
        let Some(parent_issue_identifier) = parent_issue_identifier else {
            return json_response(blocked_response(
                "parent_issue_identifier is required",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["provide the parent issue identifier such as MS-49".to_string()],
                BacklogSliceMaterialization {
                    parent_issue_identifier: String::new(),
                    milestone: string_param(&params, "milestone"),
                    results: Vec::new(),
                },
            ));
        };

        let slice_specs = object_array_param(&params, "slices").unwrap_or_default();
        if slice_specs.is_empty() {
            return json_response(blocked_response(
                "materialize_backlog_slices requires at least one slice object",
                vec!["save_linear_issue".to_string()],
                vec!["slices must be a non-empty array".to_string()],
                BacklogSliceMaterialization {
                    parent_issue_identifier,
                    milestone: string_param(&params, "milestone"),
                    results: Vec::new(),
                },
            ));
        }

        let workspace = match self.linear_workspace().await {
            Ok(workspace) => workspace,
            Err(err) => {
                return json_response(blocked_response(
                    format!("cannot materialize backlog slices without Linear access: {err}"),
                    vec!["audit_workflow_readiness".to_string()],
                    vec![err],
                    BacklogSliceMaterialization {
                        parent_issue_identifier,
                        milestone: string_param(&params, "milestone"),
                        results: Vec::new(),
                    },
                ))
            }
        };
        if find_linear_issue(&workspace, None, Some(parent_issue_identifier.as_str())).is_none() {
            return json_response(blocked_response(
                format!("could not resolve parent issue {}", parent_issue_identifier),
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["parent issue must exist before child slices can be materialized".to_string()],
                BacklogSliceMaterialization {
                    parent_issue_identifier,
                    milestone: string_param(&params, "milestone"),
                    results: Vec::new(),
                },
            ));
        }

        let default_labels = string_array_param(&params, "default_labels").unwrap_or_default();
        let default_priority = i64_param(&params, "default_priority");
        let milestone = string_param(&params, "milestone");
        let mut results = Vec::new();

        for slice in slice_specs {
            let title = slice
                .get("title")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let description = slice
                .get("description")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            if title.is_empty() || description.is_empty() {
                return json_response(blocked_response(
                    "each slice requires a non-empty title and description",
                    vec!["save_linear_issue".to_string()],
                    vec!["slice validation failed before any new backlog mutation".to_string()],
                    BacklogSliceMaterialization {
                        parent_issue_identifier,
                        milestone,
                        results,
                    },
                ));
            }

            let issue_identifier = slice
                .get("issue_identifier")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| {
                    find_linear_child_issue_by_parent_and_title(
                        &workspace,
                        &parent_issue_identifier,
                        &title,
                    )
                    .map(|issue| issue.identifier.clone())
                });

            let slice_labels = slice
                .get("labels")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let mut labels = default_labels.clone();
            labels.extend(slice_labels);
            labels.sort();
            labels.dedup();

            let blocked_by = slice
                .get("blocked_by")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let mut issue_params = serde_json::Map::new();
            if let Some(identifier) = issue_identifier.clone() {
                issue_params.insert(
                    "issue_identifier".to_string(),
                    serde_json::json!(identifier),
                );
            }
            issue_params.insert("title".to_string(), serde_json::json!(title));
            issue_params.insert("description".to_string(), serde_json::json!(description));
            issue_params.insert(
                "project".to_string(),
                serde_json::json!("MisterSmith Validated Backlog"),
            );
            issue_params.insert("state".to_string(), serde_json::json!("Backlog"));
            issue_params.insert(
                "parent_identifier".to_string(),
                serde_json::json!(parent_issue_identifier),
            );
            if !labels.is_empty() {
                issue_params.insert("labels".to_string(), serde_json::json!(labels));
            }
            if let Some(priority) = slice
                .get("priority")
                .and_then(serde_json::Value::as_i64)
                .or(default_priority)
            {
                issue_params.insert("priority".to_string(), serde_json::json!(priority));
            }
            if let Some(milestone_name) = milestone.clone() {
                issue_params.insert("milestone".to_string(), serde_json::json!(milestone_name));
            }
            if !blocked_by.is_empty() {
                issue_params.insert("blocked_by".to_string(), serde_json::json!(blocked_by));
            }

            let issue_response = parse_tool_response::<LinearIssueSaveResult>(
                self.save_linear_issue(serde_json::Value::Object(issue_params))
                    .await?,
            )?;
            if !matches!(issue_response.status, CompatibilityStatus::Applied) {
                return json_response(ToolResponse {
                    status: issue_response.status,
                    summary: issue_response.summary,
                    evidence: issue_response.evidence,
                    warnings: issue_response.warnings,
                    recommended_next_tools: issue_response.recommended_next_tools,
                    blocking_issues: issue_response.blocking_issues,
                    data: BacklogSliceMaterialization {
                        parent_issue_identifier,
                        milestone,
                        results,
                    },
                });
            }

            let workpad_action = if let Some(workpad_body) = slice
                .get("workpad_body")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|body| !body.is_empty())
            {
                let workpad_response = parse_tool_response::<LinearWorkpadSaveResult>(
                    self.save_issue_workpad(serde_json::json!({
                        "issue_identifier": issue_response.data.issue.identifier,
                        "body": workpad_body,
                    }))
                    .await?,
                )?;
                if !matches!(workpad_response.status, CompatibilityStatus::Applied) {
                    return json_response(ToolResponse {
                        status: workpad_response.status,
                        summary: workpad_response.summary,
                        evidence: workpad_response.evidence,
                        warnings: workpad_response.warnings,
                        recommended_next_tools: workpad_response.recommended_next_tools,
                        blocking_issues: workpad_response.blocking_issues,
                        data: BacklogSliceMaterialization {
                            parent_issue_identifier,
                            milestone,
                            results,
                        },
                    });
                }
                Some(workpad_response.data.action)
            } else {
                None
            };

            results.push(MaterializedBacklogSlice {
                title: issue_response.data.issue.title.clone(),
                issue_identifier: issue_response.data.issue.identifier.clone(),
                action: issue_response.data.action,
                blocker_identifiers: blocked_by,
                workpad_action,
                issue: issue_response.data.issue,
            });
        }

        json_response(ToolResponse {
            status: CompatibilityStatus::Applied,
            summary: format!(
                "Smith materialized {} backlog slice(s) under {}",
                results.len(),
                parent_issue_identifier
            ),
            evidence: vec![EvidenceItem {
                label: "parent_issue_identifier".to_string(),
                detail: parent_issue_identifier.clone(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "prepare_direct_execution".to_string(),
                "get_issue_execution_snapshot".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: BacklogSliceMaterialization {
                parent_issue_identifier,
                milestone,
                results,
            },
        })
    }

    async fn resolve_issue_lifecycle(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let Some(identifier) = identifier else {
            return json_response(blocked_response(
                "issue identifier is required",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["issue_identifier parameter is required".to_string()],
                IssueLifecycleResolution {
                    issue: None,
                    issue_identifier: String::new(),
                    next_recommended_action: "verify_issue_identifier".to_string(),
                    required_mutations: Vec::new(),
                    blocking_reasons: vec!["provide an issue identifier such as MS-51".to_string()],
                    review_state: "unknown".to_string(),
                    pr_correlation: Vec::new(),
                },
            ));
        };

        let context = self.load_issue_execution_context(&identifier).await?;
        let resolution = build_direct_issue_lifecycle_resolution(
            context.issue.clone(),
            context.workpad.as_ref(),
            &context.matching_pull_requests,
            &identifier,
        );

        json_response(ToolResponse {
            status: if resolution.issue.is_some() {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Degraded
            },
            summary: if resolution.issue.is_some() {
                format!("resolved lifecycle for {}", resolution.issue_identifier)
            } else {
                format!(
                    "could not resolve lifecycle for {}",
                    resolution.issue_identifier
                )
            },
            evidence: vec![EvidenceItem {
                label: "issue_identifier".to_string(),
                detail: resolution.issue_identifier.clone(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: match resolution.next_recommended_action.as_str() {
                "reconcile_workpad" => vec!["save_issue_workpad".to_string()],
                "merge_ready_pr" => vec!["review_merge_status".to_string()],
                "run_speckit_implement" => vec![
                    "prepare_direct_execution".to_string(),
                    "prepare_speckit_context".to_string(),
                ],
                _ => vec!["get_issue_execution_snapshot".to_string()],
            },
            blocking_issues: resolution.blocking_reasons.clone(),
            data: resolution,
        })
    }

    async fn prepare_ralph_packet(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let Some(identifier) = identifier else {
            return json_response(blocked_response(
                "issue identifier is required",
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["provide an issue identifier such as MS-54".to_string()],
                RalphPacket {
                    issue_identifier: String::new(),
                    plan_path: None,
                    mode: "implementation".to_string(),
                    goal: String::new(),
                    current_context: String::new(),
                    source_docs: Vec::new(),
                    workflow_requirements: Vec::new(),
                    validation_requirements: Vec::new(),
                    stop_conditions: Vec::new(),
                    definition_of_done: Vec::new(),
                    rendered_prompt: String::new(),
                },
            ));
        };

        let context = self.load_issue_execution_context(&identifier).await?;
        let Some(issue) = context.issue.clone() else {
            return json_response(blocked_response(
                format!("could not resolve issue {}", identifier),
                vec!["get_issue_execution_snapshot".to_string()],
                vec!["issue must exist before Smith can prepare a Ralph packet".to_string()],
                RalphPacket {
                    issue_identifier: identifier,
                    plan_path: None,
                    mode: "implementation".to_string(),
                    goal: String::new(),
                    current_context: String::new(),
                    source_docs: Vec::new(),
                    workflow_requirements: Vec::new(),
                    validation_requirements: Vec::new(),
                    stop_conditions: Vec::new(),
                    definition_of_done: Vec::new(),
                    rendered_prompt: String::new(),
                },
            ));
        };

        let plan_path = string_param(&params, "plan_path").map(|raw| {
            let raw_path = PathBuf::from(raw);
            let resolved = if raw_path.is_absolute() {
                raw_path
            } else {
                self.options.repo_root.join(raw_path)
            };
            resolved.display().to_string()
        });
        let mode = match state_name(&issue) {
            Some("Rework") => "rework",
            Some("Human Review") | Some("Merging") => "review",
            _ => "implementation",
        }
        .to_string();
        let source_docs = {
            let mut docs = vec![
                self.options.repo_root.join("AGENTS.md").display().to_string(),
                self.options
                    .repo_root
                    .join("docs/linear/LINEAR.md")
                    .display()
                    .to_string(),
                self.options
                    .repo_root
                    .join("docs/plans/2026-03-16-smith-first-development-system.md")
                    .display()
                    .to_string(),
                self.options
                    .repo_root
                    .join("docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md")
                    .display()
                    .to_string(),
            ];
            if let Some(plan_path) = plan_path.clone() {
                docs.push(plan_path);
            }
            docs
        };
        let workflow_requirements = vec![
            "Use Smith-first control-plane boundaries; do not replace Linear, Ralph, or SpecKit.".to_string(),
            "Generate the current issue/workpad packet, run ./scripts/ralph prompt --packet <packet.json>, and only then run Ralph.".to_string(),
            "Keep execution grounded in the active issue, the durable workpad, and current repo contracts.".to_string(),
        ];
        let validation_requirements = vec![
            "Run the narrowest deterministic validation that proves the touched behavior."
                .to_string(),
            "Record concrete commands and outcomes back into the durable workpad path.".to_string(),
            "Keep proof boundaries explicit when direct Codex execution is not the same as live runtime proof.".to_string(),
        ];
        let stop_conditions = vec![
            "Stop for missing auth, missing required tooling, or a true repo contract conflict.".to_string(),
            "Stop before destructive or externally risky actions that are not justified by the issue scope.".to_string(),
        ];
        let definition_of_done = vec![
            "Requested slice is implemented or the blocker is explicitly recorded.".to_string(),
            "Validation evidence is captured in the workpad.".to_string(),
            "Any necessary follow-up is tracked through the same Smith-owned issue/workpad path."
                .to_string(),
        ];
        let current_context = format!(
            "Issue: {} - {}\nState: {}\nProject: {}\nDescription:\n{}\n\nCurrent workpad:\n{}",
            issue.identifier,
            issue.title,
            state_name(&issue).unwrap_or("unknown"),
            issue
                .project
                .as_ref()
                .map(|project| project.name.as_str())
                .unwrap_or("unassigned"),
            issue.description.clone().unwrap_or_default(),
            context
                .workpad
                .as_ref()
                .map(|workpad| workpad.body.clone())
                .unwrap_or_else(|| "## Codex Workpad\n\nNo existing workpad content.".to_string())
        );
        let goal = format!("Advance {}: {}", issue.identifier, issue.title);
        let rendered_prompt = render_ralph_prompt(
            &mode,
            &goal,
            &current_context,
            RalphPromptSections {
                source_docs: &source_docs,
                workflow_requirements: &workflow_requirements,
                validation_requirements: &validation_requirements,
                stop_conditions: &stop_conditions,
                definition_of_done: &definition_of_done,
            },
        );

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: format!("prepared a Ralph packet for {}", issue.identifier),
            evidence: vec![EvidenceItem {
                label: "issue_identifier".to_string(),
                detail: issue.identifier.clone(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "record_ralph_outcome".to_string(),
                "save_issue_workpad".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: RalphPacket {
                issue_identifier: issue.identifier,
                plan_path,
                mode,
                goal,
                current_context,
                source_docs,
                workflow_requirements,
                validation_requirements,
                stop_conditions,
                definition_of_done,
                rendered_prompt,
            },
        })
    }

    async fn record_ralph_outcome(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let Some(identifier) = identifier else {
            return json_response(blocked_response(
                "issue identifier is required",
                vec!["prepare_ralph_packet".to_string()],
                vec!["provide an issue identifier such as MS-55".to_string()],
                RalphOutcomeRecord {
                    issue_identifier: String::new(),
                    outcome_status: String::new(),
                    workpad_action: "blocked".to_string(),
                    target_state: None,
                    comment_id: None,
                },
            ));
        };

        let outcome_status =
            string_param(&params, "outcome_status").unwrap_or_else(|| "completed".to_string());
        let evidence = string_array_param(&params, "evidence").unwrap_or_default();
        let validation = string_array_param(&params, "validation").unwrap_or_default();
        let next_recommended_action = string_param(&params, "next_recommended_action")
            .unwrap_or_else(|| "review_merge_status".to_string());
        let target_state =
            string_param(&params, "target_state").or_else(|| string_param(&params, "state"));

        let context = self.load_issue_execution_context(&identifier).await?;
        let existing_workpad = context
            .workpad
            .as_ref()
            .map(|workpad| workpad.body.clone())
            .unwrap_or_else(|| CODEX_WORKPAD_HEADER.to_string());
        let outcome_section = format!(
            "## Ralph Outcome\n\n- Status: {outcome_status}\n- Next recommended action: {next_recommended_action}\n{}\n{}",
            render_bullet_block("Evidence", &evidence),
            render_bullet_block("Validation", &validation),
        );
        let merged_body =
            upsert_markdown_section(&existing_workpad, "## Ralph Outcome", &outcome_section);
        let workpad_response = parse_tool_response::<LinearWorkpadSaveResult>(
            self.save_issue_workpad(serde_json::json!({
                "issue_identifier": identifier,
                "body": merged_body,
            }))
            .await?,
        )?;
        if !matches!(workpad_response.status, CompatibilityStatus::Applied) {
            return json_response(ToolResponse {
                status: workpad_response.status,
                summary: workpad_response.summary,
                evidence: workpad_response.evidence,
                warnings: workpad_response.warnings,
                recommended_next_tools: workpad_response.recommended_next_tools,
                blocking_issues: workpad_response.blocking_issues,
                data: RalphOutcomeRecord {
                    issue_identifier: identifier,
                    outcome_status,
                    workpad_action: "blocked".to_string(),
                    target_state,
                    comment_id: None,
                },
            });
        }

        if let Some(state) = target_state.clone() {
            let _ = parse_tool_response::<LinearIssueSaveResult>(
                self.save_linear_issue(serde_json::json!({
                    "issue_identifier": identifier,
                    "state": state,
                }))
                .await?,
            )?;
        }

        json_response(ToolResponse {
            status: CompatibilityStatus::Applied,
            summary: format!("recorded the Ralph outcome for {}", identifier),
            evidence: vec![EvidenceItem {
                label: "issue_identifier".to_string(),
                detail: identifier.clone(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "resolve_issue_lifecycle".to_string(),
                "get_issue_execution_snapshot".to_string(),
            ],
            blocking_issues: Vec::new(),
            data: RalphOutcomeRecord {
                issue_identifier: identifier,
                outcome_status,
                workpad_action: workpad_response.data.action,
                target_state,
                comment_id: Some(workpad_response.data.comment_id),
            },
        })
    }

    async fn prepare_speckit_context(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let request_text = string_param(&params, "request_text")
            .or_else(|| string_param(&params, "request"))
            .unwrap_or_default();
        let issue_identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let feature_dir = string_param(&params, "feature_dir")
            .or_else(|| string_param(&params, "feature_directory"));

        let issue_context = if let Some(identifier) = issue_identifier.as_deref() {
            self.load_issue_execution_context(identifier).await.ok()
        } else {
            None
        };
        let combined_text = format!(
            "{}\n{}",
            request_text,
            issue_context
                .as_ref()
                .and_then(|context| context.issue.as_ref())
                .and_then(|issue| issue.description.clone())
                .unwrap_or_default()
        );
        let resolved_feature_dir = feature_dir
            .as_deref()
            .map(|raw| resolve_repo_path(&self.options.repo_root, raw))
            .or_else(|| extract_feature_dir_from_text(&combined_text).map(PathBuf::from));
        let should_use_speckit = resolved_feature_dir
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false)
            || contains_any(
                &combined_text.to_lowercase(),
                &[
                    "speckit",
                    "spec",
                    "task pack",
                    "tasks.md",
                    "acceptance criteria",
                ],
            );
        let mut source_docs = vec![
            self.options
                .repo_root
                .join(".codex/prompts/speckit.plan.md")
                .display()
                .to_string(),
            self.options
                .repo_root
                .join(".codex/prompts/speckit.tasks.md")
                .display()
                .to_string(),
        ];
        if let Some(feature_dir) = resolved_feature_dir.as_ref() {
            source_docs.push(feature_dir.join("spec.md").display().to_string());
            source_docs.push(feature_dir.join("plan.md").display().to_string());
            source_docs.push(feature_dir.join("tasks.md").display().to_string());
        }
        let feature_dir_string = resolved_feature_dir
            .as_ref()
            .map(|path| path.display().to_string());
        let packet_summary = if should_use_speckit {
            if let Some(feature_dir) = feature_dir_string.as_deref() {
                format!("Use SpecKit packet at {}", feature_dir)
            } else {
                "Work appears spec/task-pack shaped, but the feature directory is not resolved yet"
                    .to_string()
            }
        } else {
            "Work should stay on the direct Smith workflow path".to_string()
        };
        let next_command_hint = if should_use_speckit {
            if let Some(feature_dir) = feature_dir_string.as_deref() {
                format!(
                    "Run the repo-local SpecKit plan/tasks flow for {}",
                    feature_dir
                )
            } else {
                "Resolve the feature directory before running the SpecKit tasks flow".to_string()
            }
        } else {
            "Continue on the direct Smith execution path without entering SpecKit".to_string()
        };

        json_response(ToolResponse {
            status: CompatibilityStatus::Ok,
            summary: "prepared the Smith SpecKit routing context".to_string(),
            evidence: issue_identifier
                .clone()
                .into_iter()
                .map(|identifier| EvidenceItem {
                    label: "issue_identifier".to_string(),
                    detail: identifier,
                })
                .collect(),
            warnings: Vec::new(),
            recommended_next_tools: if should_use_speckit {
                vec!["translate_speckit_tasks".to_string()]
            } else {
                vec!["prepare_direct_execution".to_string()]
            },
            blocking_issues: Vec::new(),
            data: SpecKitContext {
                issue_identifier,
                should_use_speckit,
                feature_dir: feature_dir_string,
                source_docs,
                packet_summary,
                next_command_hint,
            },
        })
    }

    async fn translate_speckit_tasks(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let feature_dir = string_param(&params, "feature_dir")
            .or_else(|| string_param(&params, "feature_directory"))
            .map(|raw| resolve_repo_path(&self.options.repo_root, &raw));
        let tasks_path = string_param(&params, "tasks_path")
            .map(|raw| resolve_repo_path(&self.options.repo_root, &raw));
        let resolved_tasks_path = tasks_path
            .clone()
            .or_else(|| feature_dir.clone().map(|path| path.join("tasks.md")));
        let Some(resolved_tasks_path) = resolved_tasks_path else {
            return json_response(blocked_response(
                "translate_speckit_tasks requires feature_dir or tasks_path",
                vec!["prepare_speckit_context".to_string()],
                vec!["provide a SpecKit feature directory or a tasks.md path".to_string()],
                SpecKitTranslation {
                    feature_dir: feature_dir.map(|path| path.display().to_string()),
                    tasks_path: String::new(),
                    apply_requested: bool_param(&params, "apply"),
                    packet_summary: String::new(),
                    translated_slices: Vec::new(),
                    materialization: None,
                },
            ));
        };
        let tasks_markdown = match fs::read_to_string(&resolved_tasks_path) {
            Ok(content) => content,
            Err(err) => {
                return json_response(blocked_response(
                    format!("failed to read {}: {err}", resolved_tasks_path.display()),
                    vec!["prepare_speckit_context".to_string()],
                    vec!["tasks.md must be readable before Smith can translate it".to_string()],
                    SpecKitTranslation {
                        feature_dir: feature_dir.map(|path| path.display().to_string()),
                        tasks_path: resolved_tasks_path.display().to_string(),
                        apply_requested: bool_param(&params, "apply"),
                        packet_summary: String::new(),
                        translated_slices: Vec::new(),
                        materialization: None,
                    },
                ));
            }
        };
        let translated_slices = translate_speckit_task_markdown(&tasks_markdown);
        let apply_requested = bool_param(&params, "apply");
        let packet_summary = translated_slices
            .first()
            .map(|_| {
                format!(
                    "Translated {} bounded slice(s) from {}",
                    translated_slices.len(),
                    resolved_tasks_path.display()
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "No bounded slices were derived from {}",
                    resolved_tasks_path.display()
                )
            });

        let materialization = if apply_requested {
            let parent_issue_identifier = string_param(&params, "parent_issue_identifier")
                .or_else(|| string_param(&params, "parent_identifier"));
            let Some(parent_issue_identifier) = parent_issue_identifier else {
                return json_response(blocked_response(
                    "parent_issue_identifier is required when apply=true",
                    vec!["materialize_backlog_slices".to_string()],
                    vec![
                        "provide the parent issue identifier before applying translated slices"
                            .to_string(),
                    ],
                    SpecKitTranslation {
                        feature_dir: feature_dir.as_ref().map(|path| path.display().to_string()),
                        tasks_path: resolved_tasks_path.display().to_string(),
                        apply_requested,
                        packet_summary,
                        translated_slices,
                        materialization: None,
                    },
                ));
            };
            let mut created_by_title = BTreeMap::new();
            let mut aggregate_results = Vec::new();
            for slice in &translated_slices {
                let blocked_by = slice
                    .blocked_by
                    .iter()
                    .filter_map(|title| created_by_title.get(title).cloned())
                    .collect::<Vec<_>>();
                let materialized = parse_tool_response::<BacklogSliceMaterialization>(
                    self.materialize_backlog_slices(serde_json::json!({
                        "parent_issue_identifier": parent_issue_identifier,
                        "milestone": string_param(&params, "milestone"),
                        "default_labels": string_array_param(&params, "default_labels").unwrap_or_default(),
                        "default_priority": i64_param(&params, "default_priority"),
                        "slices": [{
                            "title": slice.title,
                            "description": slice.description,
                            "blocked_by": blocked_by,
                            "workpad_body": slice.workpad_body,
                        }]
                    }))
                    .await?,
                )?;
                if !matches!(materialized.status, CompatibilityStatus::Applied) {
                    return json_response(ToolResponse {
                        status: materialized.status,
                        summary: materialized.summary,
                        evidence: materialized.evidence,
                        warnings: materialized.warnings,
                        recommended_next_tools: materialized.recommended_next_tools,
                        blocking_issues: materialized.blocking_issues,
                        data: SpecKitTranslation {
                            feature_dir: feature_dir
                                .as_ref()
                                .map(|path| path.display().to_string()),
                            tasks_path: resolved_tasks_path.display().to_string(),
                            apply_requested,
                            packet_summary,
                            translated_slices,
                            materialization: Some(BacklogSliceMaterialization {
                                parent_issue_identifier,
                                milestone: string_param(&params, "milestone"),
                                results: aggregate_results,
                            }),
                        },
                    });
                }
                if let Some(result) = materialized.data.results.first() {
                    created_by_title.insert(slice.title.clone(), result.issue_identifier.clone());
                    aggregate_results.push(result.clone());
                }
            }
            Some(BacklogSliceMaterialization {
                parent_issue_identifier,
                milestone: string_param(&params, "milestone"),
                results: aggregate_results,
            })
        } else {
            None
        };

        json_response(ToolResponse {
            status: if apply_requested {
                CompatibilityStatus::Applied
            } else {
                CompatibilityStatus::Ok
            },
            summary: packet_summary.clone(),
            evidence: vec![EvidenceItem {
                label: "tasks_path".to_string(),
                detail: resolved_tasks_path.display().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: if apply_requested {
                vec!["prepare_direct_execution".to_string()]
            } else {
                vec!["materialize_backlog_slices".to_string()]
            },
            blocking_issues: Vec::new(),
            data: SpecKitTranslation {
                feature_dir: feature_dir.map(|path| path.display().to_string()),
                tasks_path: resolved_tasks_path.display().to_string(),
                apply_requested,
                packet_summary,
                translated_slices,
                materialization,
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
                vec!["prepare_direct_execution".to_string()],
                vec!["issue_identifier parameter is required".to_string()],
                IssueExecutionSnapshot {
                    issue: None,
                    workpad: None,
                    matching_pull_requests: Vec::new(),
                    execution_state: "missing_input".to_string(),
                    blocker_summaries: Vec::new(),
                    workpad_status: "missing".to_string(),
                    next_step_hint: "verify_issue_identifier".to_string(),
                    notes: vec!["provide an issue identifier such as MS-33".to_string()],
                },
            ));
        };

        let context = self.load_issue_execution_context(&identifier).await?;
        let resolution = build_direct_issue_lifecycle_resolution(
            context.issue.clone(),
            context.workpad.as_ref(),
            &context.matching_pull_requests,
            &identifier,
        );

        let (status, summary, blocking_issues) = if context.issue.is_some() {
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
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "resolve_issue_lifecycle".to_string(),
                "review_merge_status".to_string(),
                "prepare_direct_execution".to_string(),
            ],
            blocking_issues,
            data: IssueExecutionSnapshot {
                execution_state: context
                    .issue
                    .as_ref()
                    .and_then(|issue| issue.state.as_ref().map(|state| state.name.clone()))
                    .unwrap_or_else(|| "unknown".to_string()),
                blocker_summaries: context
                    .issue
                    .as_ref()
                    .map(direct_blocker_summaries)
                    .unwrap_or_default(),
                workpad_status: workpad_status(context.workpad.as_ref()),
                next_step_hint: resolution.next_recommended_action,
                notes: if context.issue.is_some() {
                    context
                        .workpad
                        .as_ref()
                        .and_then(|workpad| {
                            if workpad.duplicate_count > 0 {
                                Some(vec![format!(
                                    "{} additional top-level Codex workpad comment(s) exist for {}",
                                    workpad.duplicate_count, identifier
                                )])
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default()
                } else {
                    vec!["refresh Linear auth or verify the identifier".to_string()]
                },
                issue: context.issue,
                workpad: context.workpad,
                matching_pull_requests: context.matching_pull_requests,
            },
        })
    }

    async fn prepare_direct_execution(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let issue_identifier = string_param(&params, "issue_identifier")
            .or_else(|| string_param(&params, "identifier"));
        let request_text = string_param(&params, "request_text")
            .or_else(|| string_param(&params, "request"))
            .unwrap_or_default();
        let feature_dir = string_param(&params, "feature_dir")
            .or_else(|| string_param(&params, "feature_directory"))
            .map(|raw| resolve_repo_path(&self.options.repo_root, &raw));

        let context = if let Some(identifier) = issue_identifier.as_deref() {
            Some(self.load_issue_execution_context(identifier).await?)
        } else {
            None
        };
        let combined_text = format!(
            "{}\n{}",
            request_text,
            context
                .as_ref()
                .and_then(|context| context.issue.as_ref())
                .and_then(|issue| issue.description.clone())
                .unwrap_or_default()
        );
        let resolved_feature_dir = feature_dir
            .or_else(|| extract_feature_dir_from_text(&combined_text).map(PathBuf::from));
        let issue = context.as_ref().and_then(|context| context.issue.clone());
        let workpad = context.as_ref().and_then(|context| context.workpad.clone());
        let matching_pull_requests = context
            .as_ref()
            .map(|context| context.matching_pull_requests.clone())
            .unwrap_or_default();
        let plan = build_direct_execution_plan(
            &self.options.repo_root,
            issue_identifier,
            &combined_text,
            issue,
            workpad,
            &matching_pull_requests,
            resolved_feature_dir,
        );

        json_response(ToolResponse {
            status: if plan.blocking_reasons.is_empty() {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Degraded
            },
            summary: "prepared direct Codex execution context".to_string(),
            evidence: plan
                .issue
                .as_ref()
                .map(|issue| {
                    vec![EvidenceItem {
                        label: "issue_identifier".to_string(),
                        detail: issue.identifier.clone(),
                    }]
                })
                .unwrap_or_default(),
            warnings: Vec::new(),
            recommended_next_tools: direct_execution_recommended_next_tools(&plan),
            blocking_issues: plan.blocking_reasons.clone(),
            data: plan,
        })
    }

    async fn review_merge_status(
        &self,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let github = self.github_snapshot().await;
        let workspace = self.linear_workspace().await.ok();
        let status = build_review_merge_status(&github.open_pull_requests, workspace.as_ref());

        json_response(ToolResponse {
            status: if status.review_blockers.is_empty() {
                CompatibilityStatus::Ok
            } else {
                CompatibilityStatus::Degraded
            },
            summary: "direct review and merge status generated".to_string(),
            evidence: vec![EvidenceItem {
                label: "open_pull_request_count".to_string(),
                detail: github.open_pull_requests.len().to_string(),
            }],
            warnings: Vec::new(),
            recommended_next_tools: vec![
                "get_issue_execution_snapshot".to_string(),
                "prepare_direct_execution".to_string(),
            ],
            blocking_issues: status.review_blockers.clone(),
            data: status,
        })
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
                "routing",
                "supervision",
                "provenance",
            ],
        );
        let (verdict, suggested_project, suggested_state) = if frontier_aligned {
            (
                "legitimate".to_string(),
                "validated_backlog".to_string(),
                "Backlog".to_string(),
            )
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
        if let Some(issue) = issue.as_ref() {
            rationale.push(format!(
                "issue currently lives in {}",
                issue
                    .project
                    .as_ref()
                    .map(|project| project.name.as_str())
                    .unwrap_or("no assigned project")
            ));
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
                    vec!["Validated".to_string()]
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

        let (classification, project, state, labels, reason) = if contains_any(
            &normalized,
            &[
                "phase 10",
                "autonomy",
                "delegation",
                "provenance",
                "control-plane",
            ],
        ) {
            (
                "validated_backlog".to_string(),
                "MisterSmith Validated Backlog".to_string(),
                "Backlog".to_string(),
                vec!["Validated".to_string()],
                "frontier/control-plane work should be validated before direct execution"
                    .to_string(),
            )
        } else if contains_any(&normalized, &["readme", "docs", "workflow", "guide"]) {
            (
                "docs_hub".to_string(),
                "MisterSmith Workspace Docs".to_string(),
                "Backlog".to_string(),
                vec!["Docs".to_string()],
                "documentation follow-up belongs in the docs hub, not the direct execution lane"
                    .to_string(),
            )
        } else {
            (
                "triage".to_string(),
                "Triage".to_string(),
                "Triage".to_string(),
                vec!["Research".to_string()],
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
    let server = Arc::new(
        McpServer::new(McpServerConfig {
            bind_address: "stdio://smith".to_string(),
            namespace_views: Vec::new(),
        })
        .with_delegation_service(Arc::new(DelegationService::new())),
    );

    register_compatibility_tool(
        &server,
        &compatibility,
        "audit_workflow_readiness",
        "Audit Mister Smith workflow readiness across repo, Codex config, Rust, and Linear prerequisites.",
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
    register_compatibility_request_tool(
        &server,
        &compatibility,
        "describe_external_capabilities",
        "Describe the bounded external MCP capability surfaces and discovery contract for external agents.",
        object_schema(&[], &[]),
        tool_boundary_action("describe_external_capabilities", "", CapabilityActionKind::Discover),
        |state, request| async move { state.describe_external_capabilities(request).await },
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
        "Collect repo, GitHub, Linear, and readiness state in one response.",
        object_schema(&[], &[]),
        |state, params| async move { state.get_control_plane_snapshot(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "plan_workspace_adjustments",
        "Plan safe local workspace adjustments needed for Codex and repo readiness.",
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
        "prepare_direct_execution",
        "Prepare direct Codex execution context for an issue, request, or SpecKit packet.",
        object_schema(
            &[
                ("issue_identifier", string_schema("Optional issue identifier such as MS-33")),
                ("identifier", string_schema("Alias for issue_identifier")),
                ("request_text", string_schema("Optional request text describing the work")),
                ("request", string_schema("Alias for request_text")),
                ("feature_dir", string_schema("Optional absolute or repo-relative feature directory")),
                ("feature_directory", string_schema("Alias for feature_dir")),
            ],
            &[],
        ),
        |state, params| async move { state.prepare_direct_execution(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "review_merge_status",
        "Inspect direct review and merge status for open pull requests.",
        object_schema(&[], &[]),
        |state, params| async move { state.review_merge_status(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "save_linear_issue",
        "Create or update a Linear issue through the Smith control plane.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-50"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
                ("issue_id", string_schema("Internal Linear issue id")),
                ("title", string_schema("Issue title")),
                ("description", string_schema("Markdown issue description")),
                ("team_key", string_schema("Linear team key such as MS")),
                ("project", string_schema("Linear project name or slug id")),
                ("state", string_schema("Linear state name")),
                (
                    "parent_identifier",
                    string_schema("Parent issue identifier"),
                ),
                (
                    "blocked_by",
                    string_array_schema("Blocking issue identifiers"),
                ),
                ("priority", integer_schema("Linear priority value")),
                ("milestone", string_schema("Project milestone name")),
                ("labels", string_array_schema("Label names to apply")),
            ],
            &[],
        ),
        |state, params| async move { state.save_linear_issue(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "save_issue_workpad",
        "Create or update the durable ## Codex Workpad comment for a Linear issue.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-50"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
                ("issue_id", string_schema("Internal Linear issue id")),
                ("body", string_schema("Workpad markdown body")),
            ],
            &["body"],
        ),
        |state, params| async move { state.save_issue_workpad(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "materialize_backlog_slices",
        "Create or update bounded backlog child issues under a parent issue and optionally seed their Codex workpads.",
        object_schema(
            &[
                (
                    "parent_issue_identifier",
                    string_schema("Parent issue identifier such as MS-49"),
                ),
                ("parent_identifier", string_schema("Alias for parent_issue_identifier")),
                ("milestone", string_schema("Optional project milestone name")),
                (
                    "default_labels",
                    string_array_schema("Default label names applied to every slice"),
                ),
                ("default_priority", integer_schema("Default Linear priority value")),
                (
                    "slices",
                    serde_json::json!({
                        "type": "array",
                        "description": "Backlog slice objects to create or update",
                        "items": object_schema(
                            &[
                                ("issue_identifier", string_schema("Optional existing child issue identifier")),
                                ("title", string_schema("Slice issue title")),
                                ("description", string_schema("Slice issue description")),
                                ("priority", integer_schema("Optional slice-specific priority")),
                                ("labels", string_array_schema("Optional slice-specific labels")),
                                ("blocked_by", string_array_schema("Blocking issue identifiers")),
                                ("workpad_body", string_schema("Optional initial Codex workpad body"))
                            ],
                            &["title", "description"]
                        )
                    }),
                ),
            ],
            &["parent_issue_identifier", "slices"],
        ),
        |state, params| async move { state.materialize_backlog_slices(params).await },
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
        "resolve_issue_lifecycle",
        "Resolve the next recommended Smith action for a specific issue based on workpad state, blockers, and PR correlation.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-53"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
            ],
            &["issue_identifier"],
        ),
        |state, params| async move { state.resolve_issue_lifecycle(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "prepare_ralph_packet",
        "Generate a Smith-owned Ralph packet from the active issue, workpad, and optional plan path.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-54"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
                ("plan_path", string_schema("Optional absolute or repo-relative plan path")),
            ],
            &["issue_identifier"],
        ),
        |state, params| async move { state.prepare_ralph_packet(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "record_ralph_outcome",
        "Write a Ralph execution outcome back into the durable Codex workpad path and optionally update the issue state.",
        object_schema(
            &[
                (
                    "issue_identifier",
                    string_schema("Issue identifier such as MS-55"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
                ("outcome_status", string_schema("Outcome status summary")),
                ("evidence", string_array_schema("Evidence bullets to record")),
                ("validation", string_array_schema("Validation bullets to record")),
                (
                    "next_recommended_action",
                    string_schema("Next recommended Smith action"),
                ),
                ("target_state", string_schema("Optional Linear state update")),
                ("state", string_schema("Alias for target_state")),
            ],
            &["issue_identifier", "outcome_status"],
        ),
        |state, params| async move { state.record_ralph_outcome(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "prepare_speckit_context",
        "Decide whether work should enter SpecKit and return the repo-local context packet.",
        object_schema(
            &[
                (
                    "request_text",
                    string_schema("Optional request text to classify"),
                ),
                ("request", string_schema("Alias for request_text")),
                (
                    "issue_identifier",
                    string_schema("Optional issue identifier for context"),
                ),
                ("identifier", string_schema("Alias for issue_identifier")),
                (
                    "feature_dir",
                    string_schema("Optional absolute or repo-relative feature directory"),
                ),
                ("feature_directory", string_schema("Alias for feature_dir")),
            ],
            &[],
        ),
        |state, params| async move { state.prepare_speckit_context(params).await },
    )
    .await;
    register_compatibility_tool(
        &server,
        &compatibility,
        "translate_speckit_tasks",
        "Translate a SpecKit tasks packet into bounded backlog slices and optionally materialize them in Linear.",
        object_schema(
            &[
                ("feature_dir", string_schema("Optional absolute or repo-relative feature directory")),
                ("feature_directory", string_schema("Alias for feature_dir")),
                ("tasks_path", string_schema("Optional absolute or repo-relative tasks.md path")),
                (
                    "parent_issue_identifier",
                    string_schema("Parent issue identifier when apply=true"),
                ),
                ("parent_identifier", string_schema("Alias for parent_issue_identifier")),
                ("milestone", string_schema("Optional project milestone name")),
                (
                    "default_labels",
                    string_array_schema("Default labels for translated slices"),
                ),
                ("default_priority", integer_schema("Default priority for translated slices")),
                ("apply", bool_schema("Materialize translated slices in Linear")),
            ],
            &[],
        ),
        |state, params| async move { state.translate_speckit_tasks(params).await },
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
    compatibility
        .set_registered_capability_catalog(server.capability_catalog().await)
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
                required_boundary_action: None,
            },
            handler,
        )
        .await;
}

async fn register_compatibility_request_tool<F, Fut>(
    server: &Arc<McpServer>,
    compatibility: &Arc<SmithCompatibilityServer>,
    name: &str,
    description: &str,
    input_schema: serde_json::Value,
    required_boundary_action: mister_smith_core::DelegatedAction,
    func: F,
) where
    F: Fn(Arc<SmithCompatibilityServer>, ToolCallRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<serde_json::Value, McpError>> + Send + 'static,
{
    let handler = compatibility_request_handler(compatibility.clone(), func);
    server
        .register_tool(
            ExposedTool {
                name: name.to_string(),
                description: description.to_string(),
                input_schema,
                namespace: String::new(),
                required_boundary_action: Some(required_boundary_action),
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
    Arc::new(move |request: ToolCallRequest| {
        let compatibility = compatibility.clone();
        Box::pin(func(compatibility, request.params))
    })
}

fn compatibility_request_handler<F, Fut>(
    compatibility: Arc<SmithCompatibilityServer>,
    func: F,
) -> ToolHandler
where
    F: Fn(Arc<SmithCompatibilityServer>, ToolCallRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<serde_json::Value, McpError>> + Send + 'static,
{
    Arc::new(move |request: ToolCallRequest| {
        let compatibility = compatibility.clone();
        Box::pin(func(compatibility, request))
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

fn integer_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "integer",
        "description": description
    })
}

fn string_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "string",
        "description": description
    })
}

fn string_array_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "description": description,
        "items": {
            "type": "string"
        }
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

fn expand_home(path: &str) -> PathBuf {
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

fn string_array_param(params: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
}

fn object_array_param<'a>(
    params: &'a serde_json::Value,
    key: &str,
) -> Option<Vec<&'a serde_json::Map<String, serde_json::Value>>> {
    params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_object)
                .collect::<Vec<_>>()
        })
}

fn i64_param(params: &serde_json::Value, key: &str) -> Option<i64> {
    params.get(key).and_then(serde_json::Value::as_i64)
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

fn parse_tool_response<T>(value: serde_json::Value) -> Result<ToolResponse<T>, McpError>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value).map_err(|err| {
        McpError::SerializationError(format!("failed to parse tool response: {err}"))
    })
}

#[derive(Debug, Clone)]
struct CommandResult {
    success: bool,
    stdout: String,
}

async fn run_command(program: &str, args: &[&str]) -> CommandResult {
    let mut command = Command::new(program);
    command.args(args);
    match command.output().await {
        Ok(output) => CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        },
        Err(err) => CommandResult {
            success: false,
            stdout: err.to_string(),
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
        },
        Err(err) => CommandResult {
            success: false,
            stdout: err.to_string(),
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
    let teams = data
        .pointer("/teams/nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| nodes.iter().map(parse_linear_team).collect())
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
        teams,
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

fn parse_linear_label(value: &serde_json::Value) -> LinearLabelSnapshot {
    LinearLabelSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

fn parse_linear_team(value: &serde_json::Value) -> LinearTeamSnapshot {
    LinearTeamSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        key: value
            .get("key")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        labels: value
            .pointer("/labels/nodes")
            .and_then(serde_json::Value::as_array)
            .map(|nodes| nodes.iter().map(parse_linear_label).collect())
            .unwrap_or_default(),
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
        description: value
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        priority: value.get("priority").and_then(serde_json::Value::as_i64),
        url: value
            .get("url")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        state: value.get("state").map(parse_linear_state),
        project: value.get("project").map(parse_linear_project),
        parent: value.get("parent").map(parse_linear_parent_issue),
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

fn parse_linear_parent_issue(value: &serde_json::Value) -> LinearIssueParentSnapshot {
    LinearIssueParentSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        identifier: value
            .get("identifier")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
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

fn parse_linear_comment(value: &serde_json::Value) -> LinearCommentSnapshot {
    LinearCommentSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        body: value
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        updated_at: value
            .get("updatedAt")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        parent_id: value
            .pointer("/parent/id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn parse_linear_project_milestone(value: &serde_json::Value) -> LinearProjectMilestoneSnapshot {
    LinearProjectMilestoneSnapshot {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        project_id: value
            .pointer("/project/id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn find_linear_issue<'a>(
    workspace: &'a LinearWorkspaceData,
    issue_id: Option<&str>,
    issue_identifier: Option<&str>,
) -> Option<&'a LinearIssueSnapshot> {
    workspace.issues.iter().find(|issue| {
        issue_id
            .map(|candidate| issue.id.as_deref() == Some(candidate))
            .unwrap_or(false)
            || issue_identifier
                .map(|candidate| issue.identifier.eq_ignore_ascii_case(candidate))
                .unwrap_or(false)
    })
}

fn find_linear_team<'a>(
    workspace: &'a LinearWorkspaceData,
    team_key_or_name: &str,
) -> Option<&'a LinearTeamSnapshot> {
    workspace.teams.iter().find(|team| {
        team.key.eq_ignore_ascii_case(team_key_or_name)
            || team.name.eq_ignore_ascii_case(team_key_or_name)
    })
}

fn find_linear_project<'a>(
    workspace: &'a LinearWorkspaceData,
    project_name_or_slug: &str,
) -> Option<&'a LinearProjectSnapshot> {
    workspace.projects.iter().find(|project| {
        project.name.eq_ignore_ascii_case(project_name_or_slug)
            || project.slug.eq_ignore_ascii_case(project_name_or_slug)
    })
}

fn find_linear_state<'a>(
    workspace: &'a LinearWorkspaceData,
    team_key: &str,
    state_name: &str,
) -> Option<&'a LinearStateSnapshot> {
    workspace.states_by_team.get(team_key).and_then(|states| {
        states
            .iter()
            .find(|state| state.name.eq_ignore_ascii_case(state_name))
    })
}

fn resolve_linear_label_ids(
    team: &LinearTeamSnapshot,
    label_names: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut resolved = Vec::new();
    let mut missing = Vec::new();

    for label_name in label_names {
        if let Some(label_id) = team
            .labels
            .iter()
            .find(|label| label.name.eq_ignore_ascii_case(label_name))
            .and_then(|label| label.id.clone())
        {
            resolved.push(label_id);
        } else {
            missing.push(label_name.clone());
        }
    }

    (resolved, missing)
}

fn normalize_workpad_body(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.starts_with(CODEX_WORKPAD_HEADER) {
        trimmed.to_string()
    } else if trimmed.is_empty() {
        CODEX_WORKPAD_HEADER.to_string()
    } else {
        format!("{CODEX_WORKPAD_HEADER}\n\n{trimmed}")
    }
}

fn select_current_workpad(comments: &[LinearCommentSnapshot]) -> Option<LinearWorkpadSnapshot> {
    let mut matching = comments
        .iter()
        .filter(|comment| comment.parent_id.is_none())
        .filter(|comment| comment.body.trim_start().starts_with(CODEX_WORKPAD_HEADER))
        .cloned()
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return None;
    }

    matching.sort_by(|left, right| {
        left.updated_at
            .cmp(&right.updated_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    let selected = matching.pop().unwrap_or_default();
    Some(LinearWorkpadSnapshot {
        comment_id: selected.id,
        updated_at: selected.updated_at,
        body: selected.body,
        duplicate_count: matching.len(),
    })
}

fn state_name(issue: &LinearIssueSnapshot) -> Option<&str> {
    issue.state.as_ref().map(|state| state.name.as_str())
}

fn state_type_name(issue: &LinearIssueSnapshot) -> Option<&str> {
    issue.state.as_ref().and_then(|state| state.state_type.as_deref())
}

fn is_terminal_state_name(state_name: &str) -> bool {
    matches!(
        state_name,
        "Done" | "Canceled" | "Cancelled" | "Duplicate" | "Complete" | "Completed"
    )
}

fn is_terminal_issue_state(issue: &LinearIssueSnapshot) -> bool {
    state_type_name(issue)
        .map(|state_type| state_type.eq_ignore_ascii_case("completed"))
        .unwrap_or(false)
        || state_name(issue)
            .map(is_terminal_state_name)
            .unwrap_or(false)
}

fn find_linear_child_issue_by_parent_and_title<'a>(
    workspace: &'a LinearWorkspaceData,
    parent_identifier: &str,
    title: &str,
) -> Option<&'a LinearIssueSnapshot> {
    workspace.issues.iter().find(|issue| {
        issue
            .parent
            .as_ref()
            .map(|parent| parent.identifier.eq_ignore_ascii_case(parent_identifier))
            .unwrap_or(false)
            && issue.title.eq_ignore_ascii_case(title)
    })
}

fn workpad_status(workpad: Option<&LinearWorkpadSnapshot>) -> String {
    match workpad {
        Some(workpad) if workpad.duplicate_count > 0 => "duplicate".to_string(),
        Some(_) => "present".to_string(),
        None => "missing".to_string(),
    }
}

fn lifecycle_review_state(
    issue: &LinearIssueSnapshot,
    matching_pull_requests: &[GitHubPullRequest],
) -> String {
    let has_requested_changes = matching_pull_requests.iter().any(|pr| {
        pr.review_decision
            .as_deref()
            .map(|decision| decision.eq_ignore_ascii_case("CHANGES_REQUESTED"))
            .unwrap_or(false)
    });
    let has_approved_pr = matching_pull_requests.iter().any(|pr| {
        !pr.is_draft
            && pr
                .review_decision
                .as_deref()
                .map(|decision| decision.eq_ignore_ascii_case("APPROVED"))
                .unwrap_or(false)
    });

    match state_name(issue) {
        Some("Human Review") if has_requested_changes => "changes_requested".to_string(),
        Some("Human Review") if has_approved_pr => "merge_ready".to_string(),
        Some("Human Review") => "review_pending".to_string(),
        Some("Rework") => "changes_requested".to_string(),
        Some("Merging") => "merge_ready".to_string(),
        Some("In Progress") if has_requested_changes => "changes_requested".to_string(),
        Some("In Progress") if has_approved_pr => "merge_ready".to_string(),
        Some("In Progress") if !matching_pull_requests.is_empty() => {
            "pull_request_open".to_string()
        }
        Some("In Progress") => "active_execution".to_string(),
        Some("Todo") => "ready_to_start".to_string(),
        Some("Backlog") => "backlog".to_string(),
        Some(state) => state.to_ascii_lowercase().replace(' ', "_"),
        None => "unknown".to_string(),
    }
}

fn direct_blocker_summaries(issue: &LinearIssueSnapshot) -> Vec<String> {
    issue
        .blocked_by
        .iter()
        .filter_map(|blocker| {
            let state = blocker.state.as_deref().unwrap_or("unknown");
            if is_terminal_state_name(state) {
                None
            } else {
                Some(format!("{} ({state})", blocker.identifier))
            }
        })
        .collect()
}

fn pr_correlation(matching_pull_requests: &[GitHubPullRequest]) -> Vec<String> {
    matching_pull_requests
        .iter()
        .map(|pr| format!("#{} {}", pr.number, pr.title))
        .collect()
}

fn build_direct_issue_lifecycle_resolution(
    issue: Option<LinearIssueSnapshot>,
    workpad: Option<&LinearWorkpadSnapshot>,
    matching_pull_requests: &[GitHubPullRequest],
    issue_identifier: &str,
) -> IssueLifecycleResolution {
    let Some(issue_value) = issue.clone() else {
        return IssueLifecycleResolution {
            issue: None,
            issue_identifier: issue_identifier.to_string(),
            next_recommended_action: "verify_issue_identifier".to_string(),
            required_mutations: Vec::new(),
            blocking_reasons: vec![format!(
                "issue {issue_identifier} was not found in the current Linear snapshot"
            )],
            review_state: "unknown".to_string(),
            pr_correlation: Vec::new(),
        };
    };

    let workpad_status_value = workpad_status(workpad);
    let blocker_details = direct_blocker_summaries(&issue_value);
    let review_state = lifecycle_review_state(&issue_value, matching_pull_requests);
    let mut required_mutations = Vec::new();

    if workpad_status_value == "missing" {
        required_mutations.push("create_or_reconcile_codex_workpad".to_string());
    }
    if workpad_status_value == "duplicate" {
        required_mutations.push("consolidate_duplicate_codex_workpads".to_string());
    }
    if !issue_value.labels.iter().any(|label| label == "Validated")
        && state_name(&issue_value) == Some("Backlog")
    {
        required_mutations.push("apply_validated_label".to_string());
    }

    let next_recommended_action = if workpad_status_value != "present" {
        "reconcile_workpad".to_string()
    } else if !blocker_details.is_empty() {
        "resolve_blockers".to_string()
    } else if review_state == "changes_requested" {
        "address_review_feedback".to_string()
    } else if review_state == "merge_ready"
        || matches!(state_name(&issue_value), Some("Merging"))
    {
        "merge_ready_pr".to_string()
    } else if state_name(&issue_value) == Some("Todo") {
        "update_issue_state".to_string()
    } else if state_name(&issue_value) == Some("Backlog") {
        "run_speckit_implement".to_string()
    } else if matches!(state_name(&issue_value), Some("Human Review") | Some("Rework")) {
        "address_review_feedback".to_string()
    } else if is_terminal_issue_state(&issue_value) {
        "no_further_action".to_string()
    } else {
        "continue_execution".to_string()
    };

    IssueLifecycleResolution {
        issue: Some(issue_value.clone()),
        issue_identifier: issue_value.identifier.clone(),
        next_recommended_action,
        required_mutations,
        blocking_reasons: blocker_details,
        review_state,
        pr_correlation: pr_correlation(matching_pull_requests),
    }
}

fn build_direct_execution_plan(
    repo_root: &Path,
    requested_issue_identifier: Option<String>,
    request_text: &str,
    issue: Option<LinearIssueSnapshot>,
    workpad: Option<LinearWorkpadSnapshot>,
    matching_pull_requests: &[GitHubPullRequest],
    resolved_feature_dir: Option<PathBuf>,
) -> DirectExecutionPlan {
    let tasks_path = resolved_feature_dir
        .as_ref()
        .map(|path| path.join("tasks.md"))
        .filter(|path| path.exists());
    let should_use_speckit = tasks_path.is_some()
        || contains_any(
            &request_text.to_lowercase(),
            &["speckit", "packet", "spec", "tasks.md", "task pack"],
        );
    let issue_identifier_for_resolution = requested_issue_identifier
        .or_else(|| issue.as_ref().map(|issue| issue.identifier.clone()))
        .unwrap_or_default();
    let resolution = build_direct_issue_lifecycle_resolution(
        issue.clone(),
        workpad.as_ref(),
        matching_pull_requests,
        &issue_identifier_for_resolution,
    );
    let mut blocking_reasons = resolution.blocking_reasons.clone();
    if should_use_speckit && tasks_path.is_none() {
        blocking_reasons
            .push("SpecKit execution was requested but no tasks.md path could be resolved".to_string());
    }
    if issue.is_none() && request_text.trim().is_empty() && resolved_feature_dir.is_none() {
        blocking_reasons.push(
            "provide an issue identifier, request text, or feature directory to prepare direct execution".to_string(),
        );
    }

    let mut docs_to_read = vec![
        repo_root.join("AGENTS.md").display().to_string(),
        repo_root.join("docs/current-state.md").display().to_string(),
    ];
    if should_use_speckit {
        docs_to_read.push(
            repo_root
                .join(".codex/commands/implement.md")
                .display()
                .to_string(),
        );
        docs_to_read.push(
            repo_root
                .join(".codex/prompts/speckit.implement.md")
                .display()
                .to_string(),
        );
    }
    if let Some(feature_dir) = resolved_feature_dir.as_ref() {
        for path in ["spec.md", "plan.md", "tasks.md"] {
            let candidate = feature_dir.join(path);
            if candidate.exists() {
                docs_to_read.push(candidate.display().to_string());
            }
        }
    }

    let validation_commands = vec![
        "cargo build -p mister-smith-mcp".to_string(),
        "cargo test -p mister-smith-mcp".to_string(),
        "git diff --check".to_string(),
    ];
    let next_recommended_action = if should_use_speckit && tasks_path.is_some() {
        "run_speckit_implement".to_string()
    } else {
        resolution.next_recommended_action.clone()
    };

    DirectExecutionPlan {
        issue,
        workpad,
        feature_dir: resolved_feature_dir.map(|path| path.display().to_string()),
        tasks_path: tasks_path.map(|path| path.display().to_string()),
        should_use_speckit,
        docs_to_read,
        validation_commands,
        next_recommended_action,
        blocking_reasons,
    }
}

fn direct_execution_recommended_next_tools(plan: &DirectExecutionPlan) -> Vec<String> {
    if plan.should_use_speckit {
        vec![
            "prepare_speckit_context".to_string(),
            "get_issue_execution_snapshot".to_string(),
        ]
    } else {
        vec![
            "get_issue_execution_snapshot".to_string(),
            "save_issue_workpad".to_string(),
        ]
    }
}

fn build_review_merge_status(
    pull_requests: &[GitHubPullRequest],
    workspace: Option<&LinearWorkspaceData>,
) -> ReviewMergeStatus {
    let review_blockers = pull_requests
        .iter()
        .filter_map(|pr| {
            if pr.is_draft {
                Some(format!("#{} {} is still a draft PR", pr.number, pr.title))
            } else if pr
                .review_decision
                .as_deref()
                .map(|decision| decision.eq_ignore_ascii_case("CHANGES_REQUESTED"))
                .unwrap_or(false)
            {
                Some(format!("#{} {} has requested changes", pr.number, pr.title))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let merge_ready_pull_requests = pull_requests
        .iter()
        .filter(|pr| {
            !pr.is_draft
                && pr
                    .review_decision
                    .as_deref()
                    .map(|decision| decision.eq_ignore_ascii_case("APPROVED"))
                    .unwrap_or(false)
        })
        .map(|pr| format!("#{} {}", pr.number, pr.title))
        .collect::<Vec<_>>();
    let stale_pull_requests = pull_requests
        .iter()
        .filter(|pr| {
            let identifiers = collect_issue_identifiers(&format!("{} {}", pr.title, pr.head_ref_name));
            if identifiers.is_empty() {
                return true;
            }
            workspace
                .map(|workspace| {
                    !identifiers.iter().any(|identifier| {
                        workspace
                            .issues
                            .iter()
                            .any(|issue| issue.identifier.eq_ignore_ascii_case(identifier))
                    })
                })
                .unwrap_or(false)
        })
        .map(|pr| format!("#{} {}", pr.number, pr.title))
        .collect::<Vec<_>>();

    let mut recommended_actions = Vec::new();
    if !review_blockers.is_empty() {
        recommended_actions
            .push("address requested changes or finish draft PRs before attempting merge".to_string());
    }
    if !merge_ready_pull_requests.is_empty() {
        recommended_actions
            .push("inspect merge-ready PRs and land the ones that pass final validation".to_string());
    }
    if !stale_pull_requests.is_empty() {
        recommended_actions.push("inspect stale PRs and close or relink them to active issues".to_string());
    }
    if recommended_actions.is_empty() {
        recommended_actions.push("no immediate review or merge action is required".to_string());
    }

    ReviewMergeStatus {
        open_pull_request_count: pull_requests.len(),
        review_blockers,
        merge_ready_pull_requests,
        stale_pull_requests,
        recommended_actions,
    }
}

fn resolve_repo_path(repo_root: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        repo_root.join(candidate)
    }
}

fn render_bullet_block(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        format!("- {title}: none recorded")
    } else {
        let rendered = items
            .iter()
            .map(|item| format!("  - {item}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("- {title}:\n{rendered}")
    }
}

fn upsert_markdown_section(body: &str, heading: &str, replacement: &str) -> String {
    let trimmed_body = body.trim();
    if let Some(start) = trimmed_body.find(heading) {
        let after_heading = &trimmed_body[start + heading.len()..];
        let next_section_offset = after_heading.find("\n## ");
        let prefix = trimmed_body[..start].trim_end();
        let suffix = next_section_offset
            .map(|offset| &after_heading[offset + 1..])
            .unwrap_or("")
            .trim_start();
        return match (prefix.is_empty(), suffix.is_empty()) {
            (true, true) => replacement.to_string(),
            (true, false) => format!("{replacement}\n\n{suffix}"),
            (false, true) => format!("{prefix}\n\n{replacement}"),
            (false, false) => format!("{prefix}\n\n{replacement}\n\n{suffix}"),
        };
    }

    if trimmed_body.is_empty() {
        replacement.to_string()
    } else {
        format!("{trimmed_body}\n\n{replacement}")
    }
}

struct RalphPromptSections<'a> {
    source_docs: &'a [String],
    workflow_requirements: &'a [String],
    validation_requirements: &'a [String],
    stop_conditions: &'a [String],
    definition_of_done: &'a [String],
}

fn render_ralph_prompt(
    mode: &str,
    goal: &str,
    current_context: &str,
    sections: RalphPromptSections<'_>,
) -> String {
    format!(
        "# Ralph Packet\n\n## Mode\n\n{mode}\n\n## Goal\n\n{goal}\n\n## Current Context\n\n{current_context}\n\n## Source Docs\n\n{}\n\n## Workflow Requirements\n\n{}\n\n## Validation Requirements\n\n{}\n\n## Stop Conditions\n\n{}\n\n## Definition Of Done\n\n{}",
        sections
            .source_docs
            .iter()
            .map(|doc| format!("- {doc}"))
            .collect::<Vec<_>>()
            .join("\n"),
        sections
            .workflow_requirements
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n"),
        sections
            .validation_requirements
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n"),
        sections
            .stop_conditions
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n"),
        sections
            .definition_of_done
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn extract_feature_dir_from_text(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|token| token.trim_matches(|ch: char| ch == '`' || ch == '"' || ch == '\''))
        .find(|token| token.starts_with("specs/") && token.contains("/tasks.md"))
        .map(|token| token.trim_end_matches("/tasks.md").to_string())
        .or_else(|| {
            text.split_whitespace()
                .map(|token| token.trim_matches(|ch: char| ch == '`' || ch == '"' || ch == '\''))
                .find(|token| token.starts_with("specs/"))
                .map(ToOwned::to_owned)
        })
}

fn collect_issue_identifiers(text: &str) -> Vec<String> {
    let mut identifiers = text
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .filter(|token| !token.is_empty())
        .filter(|token| token.contains('-'))
        .filter_map(|token| {
            let mut parts = token.splitn(2, '-');
            let prefix = parts.next()?;
            let suffix = parts.next()?;
            if prefix.chars().all(|ch| ch.is_ascii_uppercase())
                && suffix.chars().all(|ch| ch.is_ascii_digit())
            {
                Some(format!("{prefix}-{suffix}"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    identifiers.sort();
    identifiers.dedup();
    identifiers
}

fn translate_speckit_task_markdown(markdown: &str) -> Vec<TranslatedSpecKitSlice> {
    let mut sections = Vec::<(String, Vec<String>)>::new();
    let mut current_heading: Option<String> = None;
    let mut current_lines = Vec::new();

    for line in markdown.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(heading) = current_heading.take() {
                sections.push((heading, current_lines));
            }
            current_heading = Some(rest.trim().to_string());
            current_lines = Vec::new();
        } else if current_heading.is_some() {
            current_lines.push(line.to_string());
        }
    }
    if let Some(heading) = current_heading.take() {
        sections.push((heading, current_lines));
    }

    let mut translated = Vec::new();
    let mut prior_titles = Vec::<String>::new();
    for (heading, lines) in sections {
        if heading
            .to_ascii_lowercase()
            .contains("explicitly out of scope")
        {
            continue;
        }
        let tasks = lines
            .iter()
            .filter(|line| line.trim_start().starts_with("- [ ]"))
            .cloned()
            .collect::<Vec<_>>();
        if tasks.is_empty() {
            continue;
        }
        let goal = lines
            .iter()
            .find(|line| line.trim_start().starts_with("**Goal**:"))
            .cloned()
            .unwrap_or_default();
        let independent_test = lines
            .iter()
            .find(|line| line.trim_start().starts_with("**Independent Test**:"))
            .cloned()
            .unwrap_or_default();
        let checkpoint = lines
            .iter()
            .find(|line| line.trim_start().starts_with("**Checkpoint**:"))
            .cloned()
            .unwrap_or_default();
        let description = [
            format!("## {}", heading),
            goal,
            independent_test,
            checkpoint,
            "### Tasks".to_string(),
            tasks.join("\n"),
        ]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
        let workpad_body = format!(
            "{CODEX_WORKPAD_HEADER}\n\n- [ ] Translate the SpecKit slice `{}` into execution evidence\n- [ ] Record validation and follow-up notes here",
            heading
        );
        let title = format!("SpecKit: {}", heading.replace(" - ", " "));
        translated.push(TranslatedSpecKitSlice {
            title: title.clone(),
            description,
            blocked_by: prior_titles.last().cloned().into_iter().collect(),
            workpad_body: Some(workpad_body),
        });
        prior_titles.push(title);
    }

    translated
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use mister_smith_core::{
        AgentId, AuthorityPrincipal, CapabilityActionKind, DelegatedAction, DelegationScope,
        ExternalDelegationEnvelope,
    };
    use mister_smith_security::DelegationService;
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

    fn sample_external_delegation_for_action(
        action: DelegatedAction,
    ) -> ExternalDelegationEnvelope {
        let service = DelegationService::new();
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let (capability, provenance) = service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some(action.descriptor_id.clone()),
                Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        ExternalDelegationEnvelope::new(capability, provenance).with_action(action)
    }

    fn sample_external_delegation(descriptor_id: &str) -> ExternalDelegationEnvelope {
        let external_name = descriptor_id.trim_start_matches("tool:");
        sample_external_delegation_for_action(tool_boundary_action(
            external_name,
            "",
            CapabilityActionKind::Execute,
        ))
    }

    fn write_fixture_repo(root: &Path) {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(".env"), "LINEAR_API_KEY=dummy\n").unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(
            root.join("scripts/run-smith-mcp.sh"),
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

    fn test_options(repo_root: &Path, config_path: PathBuf) -> SmithCompatibilityOptions {
        SmithCompatibilityOptions {
            server_name: "smith".to_string(),
            repo_root: repo_root.to_path_buf(),
            codex_config_path: config_path,
            env_file_path: repo_root.join(".env"),
            linear_endpoint: DEFAULT_LINEAR_ENDPOINT.to_string(),
        }
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
            "project": {"id": "project-1", "name": "MisterSmith Validated Backlog", "slugId": "validated-backlog"},
            "team": {"key": "MS", "name": "MisterSmith"},
            "labels": {"nodes": [{"name": "Validated"}]},
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

        assert_eq!(issue.labels, vec!["Validated"]);
        assert_eq!(issue.blocked_by.len(), 1);
        assert_eq!(issue.blocked_by[0].identifier, "MS-34");
        assert_eq!(issue.blocked_by[0].state.as_deref(), Some("In Progress"));
    }

    #[test]
    fn build_direct_issue_lifecycle_resolution_requires_workpad_for_backlog_issue() {
        let issue = LinearIssueSnapshot {
            identifier: "MS-51".to_string(),
            title: "Backlog slicing".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Backlog".to_string(),
                ..LinearStateSnapshot::default()
            }),
            project: Some(LinearProjectSnapshot {
                name: "MisterSmith Validated Backlog".to_string(),
                slug: "validated-backlog".to_string(),
                ..LinearProjectSnapshot::default()
            }),
            labels: vec!["Validated".to_string()],
            ..LinearIssueSnapshot::default()
        };

        let resolution = build_direct_issue_lifecycle_resolution(
            Some(issue),
            None,
            &[],
            "MS-51",
        );

        assert_eq!(resolution.next_recommended_action, "reconcile_workpad");
        assert!(resolution
            .required_mutations
            .iter()
            .any(|item| item == "create_or_reconcile_codex_workpad"));
        assert!(!resolution
            .required_mutations
            .iter()
            .any(|item| item.contains("symphony")));
    }

    #[test]
    fn build_direct_issue_lifecycle_resolution_marks_changes_requested_prs() {
        let issue = LinearIssueSnapshot {
            identifier: "MS-61".to_string(),
            title: "Review follow-up".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Human Review".to_string(),
                ..LinearStateSnapshot::default()
            }),
            ..LinearIssueSnapshot::default()
        };
        let workpad = LinearWorkpadSnapshot {
            comment_id: "c1".to_string(),
            updated_at: Some("2026-04-05T12:00:00Z".to_string()),
            body: "## Codex Workpad\n\n- [x] ready".to_string(),
            duplicate_count: 0,
        };
        let prs = vec![GitHubPullRequest {
            number: 210,
            title: "MS-61 tighten direct lifecycle".to_string(),
            head_ref_name: "codex/ms-61-direct-lifecycle".to_string(),
            url: "https://example.invalid/pr/210".to_string(),
            review_decision: Some("CHANGES_REQUESTED".to_string()),
            is_draft: false,
        }];

        let resolution = build_direct_issue_lifecycle_resolution(
            Some(issue),
            Some(&workpad),
            &prs,
            "MS-61",
        );

        assert_eq!(resolution.review_state, "changes_requested");
        assert_eq!(resolution.next_recommended_action, "address_review_feedback");
    }

    #[test]
    fn build_direct_issue_lifecycle_resolution_marks_merge_ready_prs() {
        let issue = LinearIssueSnapshot {
            identifier: "MS-62".to_string(),
            title: "Merge ready".to_string(),
            state: Some(LinearStateSnapshot {
                name: "Human Review".to_string(),
                ..LinearStateSnapshot::default()
            }),
            ..LinearIssueSnapshot::default()
        };
        let workpad = LinearWorkpadSnapshot {
            comment_id: "c2".to_string(),
            updated_at: Some("2026-04-05T12:00:00Z".to_string()),
            body: "## Codex Workpad\n\n- [x] ready".to_string(),
            duplicate_count: 0,
        };
        let prs = vec![GitHubPullRequest {
            number: 211,
            title: "MS-62 merge ready".to_string(),
            head_ref_name: "codex/ms-62-merge-ready".to_string(),
            url: "https://example.invalid/pr/211".to_string(),
            review_decision: Some("APPROVED".to_string()),
            is_draft: false,
        }];

        let resolution = build_direct_issue_lifecycle_resolution(
            Some(issue),
            Some(&workpad),
            &prs,
            "MS-62",
        );

        assert_eq!(resolution.review_state, "merge_ready");
        assert_eq!(resolution.next_recommended_action, "merge_ready_pr");
    }

    #[test]
    fn build_review_merge_status_categorizes_prs() {
        let workspace = LinearWorkspaceData {
            issues: vec![LinearIssueSnapshot {
                identifier: "MS-80".to_string(),
                title: "Existing issue".to_string(),
                ..LinearIssueSnapshot::default()
            }],
            ..LinearWorkspaceData::default()
        };
        let pull_requests = vec![
            GitHubPullRequest {
                number: 1,
                title: "MS-80 tighten routing".to_string(),
                head_ref_name: "codex/ms-80-routing".to_string(),
                url: "https://example.invalid/pr/1".to_string(),
                review_decision: Some("APPROVED".to_string()),
                is_draft: false,
            },
            GitHubPullRequest {
                number: 2,
                title: "MS-81 follow-up".to_string(),
                head_ref_name: "codex/ms-81-follow-up".to_string(),
                url: "https://example.invalid/pr/2".to_string(),
                review_decision: Some("CHANGES_REQUESTED".to_string()),
                is_draft: false,
            },
            GitHubPullRequest {
                number: 3,
                title: "No linked issue".to_string(),
                head_ref_name: "codex/orphan-branch".to_string(),
                url: "https://example.invalid/pr/3".to_string(),
                review_decision: None,
                is_draft: true,
            },
        ];

        let status = build_review_merge_status(&pull_requests, Some(&workspace));

        assert_eq!(status.open_pull_request_count, 3);
        assert_eq!(status.merge_ready_pull_requests, vec!["#1 MS-80 tighten routing"]);
        assert!(status
            .review_blockers
            .iter()
            .any(|item| item.contains("#2 MS-81 follow-up has requested changes")));
        assert!(status
            .review_blockers
            .iter()
            .any(|item| item.contains("#3 No linked issue is still a draft PR")));
        assert!(status
            .stale_pull_requests
            .iter()
            .any(|item| item.contains("#3 No linked issue")));
    }

    #[test]
    fn build_direct_execution_plan_includes_packet_docs_and_existing_workpad() {
        let repo_root = temp_path("direct-execution-plan");
        write_fixture_repo(&repo_root);
        let feature_dir = repo_root.join("specs/999-direct-execution");
        fs::create_dir_all(&feature_dir).unwrap();
        for name in ["spec.md", "plan.md", "tasks.md"] {
            fs::write(feature_dir.join(name), format!("# {name}\n")).unwrap();
        }

        let issue = LinearIssueSnapshot {
            identifier: "MS-90".to_string(),
            title: "Direct execution packet".to_string(),
            description: Some("Implement specs/999-direct-execution/tasks.md".to_string()),
            state: Some(LinearStateSnapshot {
                name: "Backlog".to_string(),
                ..LinearStateSnapshot::default()
            }),
            labels: vec!["Validated".to_string()],
            ..LinearIssueSnapshot::default()
        };
        let workpad = LinearWorkpadSnapshot {
            comment_id: "c1".to_string(),
            updated_at: Some("2026-04-05T12:00:00Z".to_string()),
            body: "## Codex Workpad\n\n- [ ] implement".to_string(),
            duplicate_count: 0,
        };

        let plan = build_direct_execution_plan(
            &repo_root,
            Some("MS-90".to_string()),
            "Implement specs/999-direct-execution/tasks.md",
            Some(issue),
            Some(workpad.clone()),
            &[],
            Some(feature_dir.clone()),
        );

        let expected_tasks_path = feature_dir.join("tasks.md").display().to_string();
        let plan_workpad = plan.workpad.as_ref().expect("expected existing workpad");
        assert_eq!(plan_workpad.comment_id, workpad.comment_id);
        assert_eq!(plan_workpad.body, workpad.body);
        assert!(plan.should_use_speckit);
        assert_eq!(plan.tasks_path.as_deref(), Some(expected_tasks_path.as_str()));
        assert!(plan
            .docs_to_read
            .iter()
            .any(|path| path.ends_with(".codex/commands/implement.md")));
        assert!(plan
            .docs_to_read
            .iter()
            .any(|path| path.ends_with("specs/999-direct-execution/tasks.md")));
        assert_eq!(plan.next_recommended_action, "run_speckit_implement");
    }

    #[test]
    fn build_direct_execution_plan_blocks_when_speckit_path_is_missing() {
        let repo_root = temp_path("direct-execution-missing-tasks");
        write_fixture_repo(&repo_root);

        let plan = build_direct_execution_plan(
            &repo_root,
            None,
            "Implement the SpecKit packet for specs/999-missing/tasks.md",
            None,
            None,
            &[],
            None,
        );

        assert!(plan.should_use_speckit);
        assert!(plan.tasks_path.is_none());
        assert!(plan
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("no tasks.md path could be resolved")));
    }

    #[test]
    fn build_direct_execution_plan_blocks_without_any_context() {
        let repo_root = temp_path("direct-execution-missing-context");
        write_fixture_repo(&repo_root);

        let plan = build_direct_execution_plan(&repo_root, None, "", None, None, &[], None);

        assert!(!plan.should_use_speckit);
        assert!(plan
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("provide an issue identifier")));
    }

    #[test]
    fn normalize_workpad_body_adds_header_when_missing() {
        let normalized = normalize_workpad_body("- [ ] Investigate Smith write path");
        assert!(normalized.starts_with("## Codex Workpad"));
        assert!(normalized.contains("Investigate Smith write path"));
    }

    #[test]
    fn normalize_workpad_body_preserves_existing_header() {
        let normalized = normalize_workpad_body("## Codex Workpad\n\n- [x] Existing");
        assert_eq!(normalized, "## Codex Workpad\n\n- [x] Existing");
    }

    #[test]
    fn select_current_workpad_prefers_latest_top_level_comment() {
        let workpad = select_current_workpad(&[
            LinearCommentSnapshot {
                id: "c1".to_string(),
                body: "## Codex Workpad\n\nold".to_string(),
                updated_at: Some("2026-03-16T10:00:00.000Z".to_string()),
                parent_id: None,
            },
            LinearCommentSnapshot {
                id: "reply".to_string(),
                body: "## Codex Workpad\n\nreply".to_string(),
                updated_at: Some("2026-03-16T12:00:00.000Z".to_string()),
                parent_id: Some("c1".to_string()),
            },
            LinearCommentSnapshot {
                id: "c2".to_string(),
                body: "## Codex Workpad\n\nnew".to_string(),
                updated_at: Some("2026-03-16T11:00:00.000Z".to_string()),
                parent_id: None,
            },
        ])
        .unwrap();

        assert_eq!(workpad.comment_id, "c2");
        assert_eq!(workpad.duplicate_count, 1);
        assert_eq!(workpad.body, "## Codex Workpad\n\nnew");
    }

    #[test]
    fn upsert_markdown_section_replaces_existing_section() {
        let body =
            "## Codex Workpad\n\n- [ ] Existing\n\n## Ralph Outcome\n\nold\n\n## Notes\n\nkeep";
        let updated = upsert_markdown_section(
            body,
            "## Ralph Outcome",
            "## Ralph Outcome\n\n- Status: completed",
        );

        assert!(updated.contains("- Status: completed"));
        assert!(!updated.contains("\n\nold\n\n"));
        assert!(updated.contains("## Notes\n\nkeep"));
    }

    #[test]
    fn translate_speckit_task_markdown_creates_ordered_slice_chain() {
        let tasks = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../specs/013-multi-turn-same-agent-conversations/tasks.md"),
        )
        .unwrap();

        let translated = translate_speckit_task_markdown(&tasks);

        assert!(translated.len() >= 4);
        assert!(translated[0]
            .title
            .starts_with("SpecKit: Foundational Tasks"));
        assert_eq!(translated[1].blocked_by, vec![translated[0].title.clone()]);
        assert!(translated[0].description.contains("T001"));
        assert!(translated[0].description.contains("### Tasks"));
        let serialized = serde_json::to_value(&translated[0]).unwrap();
        assert!(serialized.get("symphony_candidate").is_none());
    }

    #[tokio::test]
    async fn route_workflow_request_recognizes_development_workflow_requests() {
        let repo_root = temp_path("route-development-workflow");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Prepare a Smith-first development workflow operating model and Ralph prompt chain for Mister Smith"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("direct_execution".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("prepare_direct_execution".to_string())
        );
        assert!(result["recommended_next_tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "prepare_direct_execution"));
    }

    #[tokio::test]
    async fn route_workflow_request_does_not_treat_prepare_requests_as_review_merge() {
        let repo_root = temp_path("route-prepare-direct-execution");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
            .await
            .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Prepare the direct execution plan for MS-90"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("direct_execution".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("prepare_direct_execution".to_string())
        );
    }

    #[tokio::test]
    async fn route_workflow_request_keeps_pull_request_requests_in_review_dispatch() {
        let repo_root = temp_path("route-review-dispatch");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Review the pull request and merge it if checks are green"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("review_merge".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("review_merge_status".to_string())
        );
    }

    #[tokio::test]
    async fn compatibility_server_accepts_valid_delegated_request() {
        let repo_root = temp_path("delegated-route-request");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                ToolCallRequest::new(serde_json::json!({
                    "request": "Prepare a Smith-first development workflow operating model and Ralph prompt chain for Mister Smith"
                }))
                .with_delegation(sample_external_delegation("tool:route_workflow_request"))
                .into_wire_params(),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("direct_execution".to_string())
        );
    }

    #[tokio::test]
    async fn route_workflow_request_routes_workpad_mutations_to_linear_workflow() {
        let repo_root = temp_path("route-workpad-mutations");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Update the Codex workpad comment for MS-50"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("linear_workflow".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("save_issue_workpad".to_string())
        );
    }

    #[tokio::test]
    async fn route_workflow_request_routes_child_issue_creation_to_linear_workflow() {
        let repo_root = temp_path("route-child-issue");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Create a child issue under MS-49 for backlog slicing"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("linear_workflow".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("save_linear_issue".to_string())
        );
    }

    #[tokio::test]
    async fn route_workflow_request_routes_backlog_slicing_requests() {
        let repo_root = temp_path("route-backlog-slicing");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Materialize backlog slices for this validated issue"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("backlog_slicing".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("materialize_backlog_slices".to_string())
        );
        let recommended_next_tools = result["recommended_next_tools"]
            .as_array()
            .expect("router should return next tools");
        assert!(!recommended_next_tools
            .iter()
            .any(|value| matches!(value.as_str(), Some("plan_queue_stage" | "apply_queue_stage"))));
    }

    #[tokio::test]
    async fn route_workflow_request_routes_speckit_translation_requests_to_backlog_slicing() {
        let repo_root = temp_path("route-speckit-translation");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
            .await
            .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Translate SpecKit tasks from specs/013-multi-turn-same-agent-conversations/tasks.md into backlog slices"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("backlog_slicing".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("translate_speckit_tasks".to_string())
        );
    }

    #[tokio::test]
    async fn route_workflow_request_routes_issue_lifecycle_requests() {
        let repo_root = temp_path("route-issue-lifecycle");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "route_workflow_request",
                serde_json::json!({
                    "request": "Resolve the issue lifecycle and next action for MS-53"
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["route"],
            serde_json::Value::String("issue_lifecycle".to_string())
        );
        assert_eq!(
            result["data"]["preferred_tool"],
            serde_json::Value::String("resolve_issue_lifecycle".to_string())
        );
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let tools = server.handle_tools_list(None).await.unwrap();
        assert!(tools
            .iter()
            .any(|tool| tool.name == "audit_workflow_readiness"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "describe_external_capabilities"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "get_server_runtime_info"));
        assert!(tools.iter().any(|tool| tool.name == "save_linear_issue"));
        assert!(tools.iter().any(|tool| tool.name == "save_issue_workpad"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "materialize_backlog_slices"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "resolve_issue_lifecycle"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "prepare_direct_execution"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "review_merge_status"));
        assert!(tools.iter().any(|tool| tool.name == "prepare_ralph_packet"));
        assert!(tools.iter().any(|tool| tool.name == "record_ralph_outcome"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "prepare_speckit_context"));
        assert!(tools
            .iter()
            .any(|tool| tool.name == "translate_speckit_tasks"));
        assert!(!tools.iter().any(|tool| tool.name == "plan_queue_stage"));
        assert!(!tools.iter().any(|tool| tool.name == "apply_queue_stage"));
        assert!(!tools
            .iter()
            .any(|tool| tool.name == "sync_linear_with_runtime"));
        assert!(!tools
            .iter()
            .any(|tool| tool.name == "review_merge_dispatch_cycle"));
        assert!(!tools.iter().any(|tool| tool.name.contains("smith.")));
    }

    #[tokio::test]
    async fn describe_external_capabilities_requires_discover_delegation() {
        let repo_root = temp_path("describe-capabilities");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let err = server
            .handle_tools_call("describe_external_capabilities", serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            McpError::ToolCallFailed(message)
                if message.contains("delegation envelope required for MCP tool 'describe_external_capabilities'")
        ));
    }

    #[tokio::test]
    async fn describe_external_capabilities_returns_catalog_with_matching_discover_delegation() {
        let repo_root = temp_path("describe-capabilities-authorized");
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let discover_action = tool_boundary_action(
            "describe_external_capabilities",
            "",
            CapabilityActionKind::Discover,
        );
        let result = server
            .handle_tools_call(
                "describe_external_capabilities",
                ToolCallRequest::new(serde_json::json!({}))
                    .with_delegation(sample_external_delegation_for_action(discover_action))
                    .into_wire_params(),
            )
            .await
            .unwrap();

        assert_eq!(
            result["data"]["discovery_surface"]["tool_name"],
            serde_json::json!("describe_external_capabilities")
        );
        assert_eq!(
            result["data"]["observed_delegation"]["action"]["action_id"],
            serde_json::json!("tool:describe_external_capabilities#discover")
        );
        assert_eq!(
            result["data"]["discovery_surface"]["capability_descriptor"]["discover_action"]
                ["action_id"],
            serde_json::json!("tool:describe_external_capabilities#discover")
        );
        assert_eq!(
            result["data"]["discovery_surface"]["capability_descriptor"]["execute_action"]
                ["action_id"],
            serde_json::json!("tool:describe_external_capabilities#execute")
        );
        let capabilities = result["data"]["capabilities"]
            .as_array()
            .expect("capability catalog should be an array");
        let save_linear_issue = capabilities
            .iter()
            .find(|entry| entry["tool_name"] == "save_linear_issue")
            .expect("catalog should include save_linear_issue");
        assert_eq!(
            save_linear_issue["capability_descriptor"]["discover_action"]["action_id"],
            serde_json::json!("tool:save_linear_issue#discover")
        );
        assert_eq!(
            save_linear_issue["capability_descriptor"]["execute_action"]["action_id"],
            serde_json::json!("tool:save_linear_issue#execute")
        );
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

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
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
        let config_path = repo_root.join("config.toml");
        fs::write(&config_path, rich_codex_config(&repo_root)).unwrap();

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
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
        let config_path = repo_root.join("config.toml");
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\n",
                repo_root.display()
            ),
        )
        .unwrap();

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
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

    #[tokio::test]
    async fn rmcp_round_trip_supports_new_workflow_tools() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let config_path = temp_path("workflow-tools-config.toml");
        fs::write(
            &config_path,
            format!(
                "[mcp_servers.smith]\ncommand = \"{}/scripts/run-smith-mcp.sh\"\n",
                repo_root.display()
            ),
        )
        .unwrap();

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
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

        let direct_execution = peer
            .call_tool(
                CallToolRequestParams::new("prepare_direct_execution".to_string()).with_arguments(
                    serde_json::json!({
                        "request_text": "Implement specs/013-multi-turn-same-agent-conversations/tasks.md"
                    })
                    .as_object()
                    .cloned()
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        assert!(!direct_execution.is_error.unwrap_or(false));
        assert!(
            direct_execution.structured_content.as_ref().unwrap()["data"]["should_use_speckit"]
                .as_bool()
                .unwrap()
        );

        let speckit_context = peer
            .call_tool(CallToolRequestParams::new(
                "prepare_speckit_context".to_string(),
            )
            .with_arguments(
                serde_json::json!({
                    "request_text": "Translate specs/013-multi-turn-same-agent-conversations/tasks.md into backlog slices"
                })
                .as_object()
                .cloned()
                .unwrap(),
            ))
            .await
            .unwrap();
        assert!(!speckit_context.is_error.unwrap_or(false));
        assert!(
            speckit_context.structured_content.as_ref().unwrap()["data"]["should_use_speckit"]
                .as_bool()
                .unwrap()
        );

        let translation = peer
            .call_tool(
                CallToolRequestParams::new("translate_speckit_tasks".to_string()).with_arguments(
                    serde_json::json!({
                        "feature_dir": "specs/013-multi-turn-same-agent-conversations",
                        "apply": false
                    })
                    .as_object()
                    .cloned()
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        assert!(!translation.is_error.unwrap_or(false));
        assert!(
            translation.structured_content.as_ref().unwrap()["data"]["translated_slices"]
                .as_array()
                .map(|slices| !slices.is_empty())
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    #[ignore = "manual live Linear mutation proof for Smith issue/workpad handlers"]
    async fn live_linear_issue_and_workpad_mutation_round_trip() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let target_issue =
            env::var("SMITH_LIVE_LINEAR_PROOF_ISSUE").unwrap_or_else(|_| "MS-50".to_string());
        let config_path = expand_home(DEFAULT_CODEX_CONFIG);

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let before = server
            .handle_tools_call(
                "get_issue_execution_snapshot",
                serde_json::json!({ "issue_identifier": target_issue }),
            )
            .await
            .unwrap();
        let current_title = before["data"]["issue"]["title"]
            .as_str()
            .unwrap()
            .to_string();

        let issue_result = server
            .handle_tools_call(
                "save_linear_issue",
                serde_json::json!({
                    "issue_identifier": target_issue,
                    "title": current_title,
                }),
            )
            .await
            .unwrap();
        assert_eq!(
            issue_result["status"],
            serde_json::Value::String("applied".to_string())
        );

        let workpad_body = format!(
            "## Codex Workpad\n\n- [x] Smith live proof via `save_issue_workpad`\n- [x] Verified issue path: {target_issue}\n"
        );
        let workpad_result = server
            .handle_tools_call(
                "save_issue_workpad",
                serde_json::json!({
                    "issue_identifier": target_issue,
                    "body": workpad_body,
                }),
            )
            .await
            .unwrap();
        assert_eq!(
            workpad_result["status"],
            serde_json::Value::String("applied".to_string())
        );

        let after = server
            .handle_tools_call(
                "get_issue_execution_snapshot",
                serde_json::json!({ "issue_identifier": target_issue }),
            )
            .await
            .unwrap();
        let workpad = &after["data"]["workpad"];
        assert!(!workpad["comment_id"].as_str().unwrap().is_empty());
        assert!(workpad["body"]
            .as_str()
            .unwrap()
            .starts_with("## Codex Workpad"));
    }

    #[tokio::test]
    #[ignore = "manual live Linear read proof for Ralph packet preparation"]
    async fn live_prepare_ralph_packet_reads_real_issue_context() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let target_issue =
            env::var("SMITH_LIVE_RALPH_PACKET_ISSUE").unwrap_or_else(|_| "MS-54".to_string());
        let config_path = expand_home(DEFAULT_CODEX_CONFIG);

        let server = build_smith_compatibility_server(test_options(&repo_root, config_path))
        .await
        .unwrap();

        let result = server
            .handle_tools_call(
                "prepare_ralph_packet",
                serde_json::json!({ "issue_identifier": target_issue }),
            )
            .await
            .unwrap();

        assert_eq!(
            result["status"],
            serde_json::Value::String("ok".to_string())
        );
        assert!(result["data"]["rendered_prompt"]
            .as_str()
            .unwrap()
            .contains("# Ralph Packet"));
    }
}
