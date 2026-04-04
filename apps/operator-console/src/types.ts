export type TabId = 'runs' | 'sessions' | 'agents' | 'health';

export interface StoredSettings {
  runtimeBaseUrl: string;
  natsMonitorUrl: string;
  reconnectEnabled: boolean;
  activeTab: TabId;
}

export type TimelineConnectionState = 'connecting' | 'connected' | 'disconnected';

export interface ComponentHealth {
  name: string;
  status: string;
  message?: string | null;
}

export interface HealthResponse {
  status: string;
  components: ComponentHealth[];
}

export interface ConfigResponse {
  version: string;
  config: Record<string, unknown>;
}

export interface ResultPreview {
  workflow_id: string;
  proof_outcome: string;
  preview_text?: string | null;
  payload_location: string;
  step_policy?: StepPolicySummary | null;
  coordinator_runtime_proof?: CoordinatorRuntimeProofView | null;
  provenance_lines: string[];
}

export interface RunSummary {
  task_id: string;
  status: string;
  priority: number;
  description: string;
  created_at: string;
  started_at?: string | null;
  completed_at?: string | null;
  session_id?: string | null;
  turn_index?: number | null;
  proof_outcome?: string | null;
  result_preview?: ResultPreview | null;
}

export interface TaskSupervisionTargetScope {
  kind: string;
  provider?: string | null;
  graph_id?: string | null;
  branch_id?: string | null;
  node_id?: string | null;
}

export interface TaskFingerprintRef {
  fingerprint_id: string;
  fingerprint_key: string;
  confidence: number;
  expires_at: string;
}

export interface TaskRepairLineageRef {
  source: string;
  checkpoint_ref?: string | null;
}

export interface TaskProfileSnapshot {
  health_state?: string | null;
  updated_at?: string | null;
}

export interface TaskInterventionRecord {
  rationale?: string | null;
  emitted_at?: string | null;
}

export interface RuntimeTruthProofBoundary {
  graph_execution: string;
  semantic_completion: string;
  grounded_tool_execution: string;
  task_proof: string;
}

export interface RuntimeTruthRunTrace {
  trace_root_id: string;
  workflow_id: string;
  graph_id?: string | null;
  branch_id?: string | null;
  node_id?: string | null;
  relationships: string[];
}

export interface RuntimeTruthGroundedEvidenceReference {
  kind: string;
  reference: string;
  label?: string | null;
}

export interface TaskRuntimeTruth {
  evidence_class: string;
  proof_boundary: RuntimeTruthProofBoundary;
  run_trace: RuntimeTruthRunTrace;
  grounded_evidence?: RuntimeTruthGroundedEvidenceReference[];
}

export interface CoordinatorDelegationRecord {
  delegation_id: string;
  workflow_id: string;
  session_id?: string | null;
  coordinator_agent_id: string;
  child_role: string;
  subagent_id: string;
  delegated_job_label: string;
  delegated_scope_ref: string;
  delegation_reason: string;
  allowed_follow_up_actions: string[];
  created_at: string;
  status: string;
}

export interface CoordinatorSubordinateInboxRecord {
  delegation_id: string;
  event_id: string;
  event_sequence: number;
  event_kind: string;
  event_payload_ref: string;
  recorded_at: string;
  visible_to: string;
}

export interface SubagentStateRecord {
  delegation_id: string;
  subagent_id: string;
  current_state: string;
  previous_state?: string | null;
  state_reason: string;
  state_updated_at: string;
  coordinator_action_ref?: string | null;
}

export interface DelegatedWorkEvidenceRef {
  delegation_id: string;
  evidence_kind: string;
  evidence_summary: string;
  artifact_refs: string[];
  proof_boundary_note: string;
  recorded_at: string;
}

export interface CoordinatorMergeDecision {
  decision_id: string;
  workflow_id: string;
  decision_kind: string;
  input_refs: string[];
  decision_reason: string;
  decision_outcome: string;
  decided_at: string;
}

export interface CoordinatorRuntimeProofView {
  workflow_id: string;
  coordinator_agent_id: string;
  delegation_records: CoordinatorDelegationRecord[];
  subordinate_inbox: CoordinatorSubordinateInboxRecord[];
  subagent_states: SubagentStateRecord[];
  delegated_work_evidence: DelegatedWorkEvidenceRef[];
  coordinator_decisions: CoordinatorMergeDecision[];
  proof_boundary: string;
  session_follow_up_note: string;
}

export interface StepPolicyDifficultyAssessment {
  workflow_id: string;
  step_id: string;
  difficulty_bucket: string;
  confidence_label: string;
  reason_codes: string[];
  verifier_ref?: string | null;
  routing_ref?: string | null;
  supervision_ref?: string | null;
  grounding_status_ref?: string | null;
}

export interface StepPolicyBudgetPressure {
  workflow_id: string;
  step_id: string;
  pressure_level: string;
  pressure_source: string;
  policy_hint: string;
  budget_root?: string | null;
  note?: string | null;
}

export interface StepPolicyDecision {
  workflow_id: string;
  step_id: string;
  chosen_action: string;
  action_reason: string;
  difficulty_ref?: string | null;
  budget_ref?: string | null;
  repair_lineage_ref?: string | null;
  requires_operator_attention: boolean;
}

export interface StepPolicyInputRefs {
  latest_step_evaluation?: string | null;
  latest_step_routing?: string | null;
  supervision_evidence?: string | null;
  runtime_truth?: string | null;
  boundary_evidence?: string | null;
}

export interface StepPolicyProofBoundaryRef {
  owner_packet: string;
  task_proof: string;
}

export interface StepPolicySummary {
  difficulty_assessment: StepPolicyDifficultyAssessment;
  budget_pressure?: StepPolicyBudgetPressure | null;
  policy_decision: StepPolicyDecision;
  input_refs: StepPolicyInputRefs;
  proof_boundary_ref: StepPolicyProofBoundaryRef;
  display_note: string;
}

export interface TaskSupervisionEvidence {
  target_scope: TaskSupervisionTargetScope;
  decision_basis?: string | null;
  proof_boundary?: string | null;
  fingerprint_ref?: TaskFingerprintRef | null;
  repair_lineage_ref?: TaskRepairLineageRef | null;
  profile_snapshot?: TaskProfileSnapshot | null;
  guard_decision?: Record<string, unknown> | null;
  intervention_record?: TaskInterventionRecord | null;
}

export interface TaskResultDetail {
  workflow_id: string;
  status: string;
  proof_outcome?: string | null;
  orchestration_quality?: Record<string, unknown> | null;
  runtime_truth?: TaskRuntimeTruth | null;
  supervision_evidence?: TaskSupervisionEvidence | null;
  step_policy?: StepPolicySummary | null;
  coordinator_runtime_proof?: CoordinatorRuntimeProofView | null;
  result: Record<string, unknown>;
}

export interface TaskInspectResponse {
  task_id: string;
  status: string;
  result?: TaskResultDetail;
}

export interface CreateTaskResponse {
  task_id: string;
  assigned_agent_id: string;
  status: string;
}

export interface SessionSummary {
  session_id: string;
  status: string;
  coordinator_agent_id: string;
  provider_kind: string;
  model_id: string;
  active_workflow_id?: string | null;
  last_completed_workflow_id?: string | null;
  turn_count: number;
  updated_at: string;
  ended_at?: string | null;
  last_preview?: string | null;
}

export interface SessionResumeProvenance {
  recovered_after_restart?: boolean;
  resumed_after_restart?: boolean;
  recovered_at?: string | null;
  recovery_reason?: string | null;
  resumed_from_workflow_id?: string | null;
  resumed_from_turn_index?: number | null;
}

export interface SessionRetainedResult {
  workflow_id: string;
  turn_index: number;
  status: string;
  assistant_result: Record<string, unknown>;
  preview?: string | null;
  runtime_truth?: TaskRuntimeTruth | null;
  provenance: {
    runtime_execution_mode: Record<string, unknown>;
    graph_state?: string | null;
    graph_id?: string | null;
    source_fields: string[];
  };
}

export interface SessionTurnSummary {
  turn_index: number;
  workflow_id: string;
  status: string;
  user_message: string;
  assistant_result?: SessionRetainedResult | null;
  resume_provenance?: SessionResumeProvenance | null;
}

export interface SessionInspectResponse {
  session_id: string;
  status: string;
  coordinator_agent_id: string;
  provider_kind: string;
  model_id: string;
  active_workflow_id?: string | null;
  last_completed_workflow_id?: string | null;
  turn_count: number;
  last_assistant_result?: SessionRetainedResult | null;
  turns: SessionTurnSummary[];
  ended_at?: string | null;
}

export interface SessionTurnAcceptedResponse {
  session_id: string;
  workflow_id: string;
  coordinator_agent_id: string;
  turn_index: number;
  status: string;
}

export interface EndSessionResponse {
  session_id: string;
  status: string;
  ended_at: string;
}

export interface AgentSummary {
  agent_id: string;
  agent_type: string;
  availability: string;
  name: string;
  status: string;
  last_heartbeat?: string | null;
}

export interface AgentDetail extends AgentSummary {
  metadata: Record<string, unknown>;
}

export interface TimelineEvent {
  event_type: string;
  payload: Record<string, unknown> | string | number[] | null;
  timestamp: string;
}

export interface OpenAiChatGptStatusPayload {
  authenticated: boolean;
  account_type?: string | null;
  email?: string | null;
  plan_type?: string | null;
  requires_openai_auth: boolean;
  summary: string;
}

export interface ClaudeSubscriptionStatusPayload {
  authenticated: boolean;
  expired: boolean;
  source?: string | null;
  masked_token?: string | null;
  summary: string;
}

export interface NatsMonitorSnapshot {
  available: boolean;
  degraded: boolean;
  summary: string;
  varz?: Record<string, unknown> | null;
  connz?: Record<string, unknown> | null;
  jsz?: Record<string, unknown> | null;
  errors: string[];
}

export interface DashboardSelection {
  runId: string | null;
  sessionId: string | null;
  agentId: string | null;
}

export interface RuntimeProbeSnapshot {
  health: HealthResponse | null;
  config: ConfigResponse | null;
  live: boolean;
  ready: boolean;
}

export interface DashboardSnapshot {
  localRuntime: LocalRuntimeSnapshot | null;
  runtimeReachable: boolean;
  runtimeSummary: string;
  probes: RuntimeProbeSnapshot;
  runs: RunSummary[];
  selectedRunId: string | null;
  runDetail: TaskInspectResponse | null;
  sessions: SessionSummary[];
  selectedSessionId: string | null;
  sessionDetail: SessionInspectResponse | null;
  agents: AgentSummary[];
  selectedAgentId: string | null;
  agentDetail: AgentDetail | null;
  nats: NatsMonitorSnapshot;
  errors: string[];
}

export interface AuthSnapshot {
  openAi: OpenAiChatGptStatusPayload;
  claude: ClaudeSubscriptionStatusPayload;
  errors: string[];
}

export interface LocalRuntimeSnapshot {
  state: string;
  summary: string;
  managed_by_app: boolean;
  dependencies_managed: boolean;
  runtime_url: string;
  database_target: string;
  nats_target: string;
  last_error?: string | null;
  last_log_line?: string | null;
}
