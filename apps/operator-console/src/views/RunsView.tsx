import { type FormEvent } from 'react';
import type {
  RunSummary,
  StepPolicySummary,
  TaskInspectResponse,
  TaskRuntimeTruth,
  TaskSupervisionEvidence,
} from '../types';
import { formatTimestamp, prettyJson } from '../utils/format';
import { StatusPill } from '../components/StatusPill';
import { EmptyState } from '../components/EmptyState';
import { KeyValueGrid } from '../components/KeyValueGrid';
import { PrioritySelect } from '../components/PrioritySelect';
import { PreviewCard } from '../components/PreviewCard';

export interface RunsViewProps {
  busy: boolean;
  taskDescription: string;
  taskPriority: string;
  onTaskDescriptionChange: (value: string) => void;
  onTaskPriorityChange: (value: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  runs: RunSummary[];
  selectedRunId: string | null;
  onSelect: (runId: string) => void;
  selectedRunSummary?: RunSummary;
  taskDetail: TaskInspectResponse | null;
}

export function RunsView(props: RunsViewProps) {
  const {
    busy,
    taskDescription,
    taskPriority,
    onTaskDescriptionChange,
    onTaskPriorityChange,
    onSubmit,
    runs,
    selectedRunId,
    onSelect,
    selectedRunSummary,
    taskDetail,
  } = props;
  const supervision = taskDetail?.result?.supervision_evidence ?? null;
  const runtimeTruth = taskDetail?.result?.runtime_truth ?? null;
  const stepPolicy =
    taskDetail?.result?.step_policy ??
    selectedRunSummary?.result_preview?.step_policy ??
    null;

  return (
    <div className="tab-layout">
      <section className="panel collection-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Runs</p>
            <h2>Root workflow queue</h2>
          </div>
          <StatusPill label="Count" tone="neutral" value={`${runs.length} loaded`} />
        </div>

        <form className="compose-form" onSubmit={onSubmit}>
          <label className="field field-block">
            <span>Submit task</span>
            <textarea
              placeholder="Ask Mister Smith to do something concrete."
              value={taskDescription}
              onChange={(event) => onTaskDescriptionChange(event.target.value)}
            />
          </label>
          <div className="compose-actions">
            <PrioritySelect value={taskPriority} onChange={onTaskPriorityChange} />
            <button className="primary-button" disabled={busy || !taskDescription.trim()}>
              {busy ? 'Submitting' : 'Submit task'}
            </button>
          </div>
        </form>

        <div className="collection-list" data-testid="runs-list">
          {runs.length === 0 ? (
            <EmptyState
              title="No runs available"
              body="Create a task or wait for the runtime to report root workflows."
            />
          ) : (
            runs.map((run) => (
              <button
                key={run.task_id}
                className={`list-row ${run.task_id === selectedRunId ? 'selected' : ''}`}
                onClick={() => onSelect(run.task_id)}
              >
                <div className="list-row-copy">
                  <strong>{run.description}</strong>
                  <p>{run.task_id}</p>
                </div>
                <div className="list-row-meta">
                  <span>{run.status}</span>
                  <small>{formatTimestamp(run.created_at)}</small>
                </div>
              </button>
            ))
          )}
        </div>
      </section>

      <section className="panel detail-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Detail</p>
            <h2>Selected run</h2>
          </div>
        </div>
        {selectedRunSummary ? (
          <div className="detail-stack">
            <section className="detail-hero">
              <div className="detail-hero-copy">
                <p className="eyebrow">Workflow focus</p>
                <h3>{selectedRunSummary.description}</h3>
                <p>
                  Root workflow {selectedRunSummary.task_id} is currently{' '}
                  {selectedRunSummary.status}.
                </p>
              </div>
              <div className="detail-hero-aside">
                <StatusPill
                  label="Status"
                  tone={toneForRun(selectedRunSummary.status)}
                  value={selectedRunSummary.status}
                />
                <StatusPill
                  label="Proof"
                  tone={selectedRunSummary.proof_outcome ? 'good' : 'neutral'}
                  value={selectedRunSummary.proof_outcome ?? 'Not recorded'}
                />
              </div>
            </section>

            <KeyValueGrid
              rows={[
                ['Task ID', selectedRunSummary.task_id],
                ['Priority', String(selectedRunSummary.priority)],
                ['Created', formatTimestamp(selectedRunSummary.created_at)],
                ['Started', formatTimestamp(selectedRunSummary.started_at)],
                ['Completed', formatTimestamp(selectedRunSummary.completed_at)],
                ['Session', selectedRunSummary.session_id ?? 'none'],
              ]}
            />

            <PreviewCard preview={selectedRunSummary.result_preview} />

            {runtimeTruth ? (
              <section className="subpanel">
                <h3>Runtime truth</h3>
                <KeyValueGrid
                  rows={[
                    ['Evidence class', runtimeTruth.evidence_class],
                    ['Trace root', runtimeTruth.run_trace.trace_root_id],
                    ['Relationships', formatRelationships(runtimeTruth)],
                    ['Graph', runtimeTruth.run_trace.graph_id ?? 'not recorded'],
                    ['Branch', runtimeTruth.run_trace.branch_id ?? 'not recorded'],
                    ['Node', runtimeTruth.run_trace.node_id ?? 'not recorded'],
                  ]}
                />
                <KeyValueGrid
                  rows={[
                    ['Graph execution', runtimeTruth.proof_boundary.graph_execution],
                    ['Semantic completion', runtimeTruth.proof_boundary.semantic_completion],
                    ['Grounded tool execution', runtimeTruth.proof_boundary.grounded_tool_execution],
                    ['Task proof', runtimeTruth.proof_boundary.task_proof],
                  ]}
                />
                <p>Grounded evidence: {formatGroundedEvidence(runtimeTruth)}</p>
              </section>
            ) : null}

            {supervision ? (
              <section className="subpanel">
                <h3>Predictive supervision</h3>
                <KeyValueGrid
                  rows={[
                    ['Target scope', formatTargetScope(supervision)],
                    ['Decision basis', supervision.decision_basis ?? 'not recorded'],
                    ['Fingerprint', formatFingerprint(supervision)],
                    ['Repair lineage', formatRepairLineage(supervision)],
                    ['Proof boundary', supervision.proof_boundary ?? 'not recorded'],
                  ]}
                />
                {supervision.profile_snapshot?.health_state ? (
                  <p>
                    Latest profile health: {supervision.profile_snapshot.health_state}
                    {supervision.profile_snapshot.updated_at
                      ? ` at ${formatTimestamp(supervision.profile_snapshot.updated_at)}`
                      : ''}
                    .
                  </p>
                ) : null}
                {supervision.intervention_record?.rationale ? (
                  <p>Latest intervention rationale: {supervision.intervention_record.rationale}</p>
                ) : null}
              </section>
            ) : null}

            {stepPolicy ? (
              <section className="subpanel">
                <h3>Step policy</h3>
                <KeyValueGrid
                  rows={[
                    ['Step', stepPolicy.difficulty_assessment.step_id],
                    ['Difficulty', stepPolicy.difficulty_assessment.difficulty_bucket],
                    ['Confidence', stepPolicy.difficulty_assessment.confidence_label],
                    ['Chosen action', stepPolicy.policy_decision.chosen_action],
                    ['Action reason', stepPolicy.policy_decision.action_reason],
                    ['Budget pressure', formatStepPolicyBudget(stepPolicy)],
                    [
                      'Operator attention',
                      stepPolicy.policy_decision.requires_operator_attention
                        ? 'required'
                        : 'not required',
                    ],
                    [
                      'Runtime truth ref',
                      stepPolicy.input_refs.runtime_truth ?? 'not recorded',
                    ],
                  ]}
                />
                <p>Reason codes: {formatStepPolicyReasonCodes(stepPolicy)}</p>
                <p>Policy refs: {formatStepPolicyRefs(stepPolicy)}</p>
                <p>Input refs: {formatStepPolicyInputRefs(stepPolicy)}</p>
                <p>
                  Proof boundary: packet {stepPolicy.proof_boundary_ref.owner_packet} says{' '}
                  {stepPolicy.proof_boundary_ref.task_proof}
                </p>
                <p>{stepPolicy.display_note}</p>
              </section>
            ) : null}

            <section className="subpanel">
              <h3>Transcript and evidence</h3>
              <div className="terminal-shell">
                <div className="terminal-header">
                  <strong>workflow transcript</strong>
                  <span>{selectedRunSummary.status}</span>
                </div>
                <div className="terminal-body">
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">created</span>
                    <span className="terminal-entry-line">
                      {formatTimestamp(selectedRunSummary.created_at)}
                    </span>
                  </div>
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">task</span>
                    <span className="terminal-entry-line emphasis">
                      {selectedRunSummary.description}
                    </span>
                  </div>
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">workflow</span>
                    <span className="terminal-entry-line">
                      {selectedRunSummary.task_id}
                    </span>
                  </div>
                  {selectedRunSummary.result_preview?.preview_text ? (
                    <div className="terminal-entry">
                      <span className="terminal-entry-time">preview</span>
                      <span className="terminal-entry-line">
                        {selectedRunSummary.result_preview.preview_text}
                      </span>
                    </div>
                  ) : null}
                </div>
                <div className="terminal-footer">
                  <input
                    className="terminal-input"
                    readOnly
                    value="selected workflow inspect route"
                  />
                  <button className="ghost-button" type="button" disabled>
                    locked
                  </button>
                </div>
              </div>
            </section>

            <details className="payload-disclosure payload-disclosure-block">
              <summary>Inspect payload</summary>
              <pre>{prettyJson(taskDetail ?? selectedRunSummary)}</pre>
            </details>
          </div>
        ) : (
          <EmptyState
            title="Select a run"
            body="Choose a root workflow to inspect its timeline, preview, and payload."
          />
        )}
      </section>
    </div>
  );
}

function toneForRun(status: string): 'good' | 'warn' | 'bad' | 'neutral' {
  const normalized = status.toLowerCase();

  if (normalized.includes('complete') || normalized.includes('success')) {
    return 'good';
  }

  if (normalized.includes('fail') || normalized.includes('error')) {
    return 'bad';
  }

  if (
    normalized.includes('queue') ||
    normalized.includes('pending') ||
    normalized.includes('running')
  ) {
    return 'warn';
  }

  return 'neutral';
}

function formatTargetScope(supervision: TaskSupervisionEvidence): string {
  const scope = supervision.target_scope;
  const detail =
    scope.node_id ?? scope.branch_id ?? scope.graph_id ?? scope.provider ?? 'scope not recorded';
  return `${scope.kind}: ${detail}`;
}

function formatFingerprint(supervision: TaskSupervisionEvidence): string {
  const fingerprint = supervision.fingerprint_ref;
  if (!fingerprint) {
    return 'not recorded';
  }

  return `${fingerprint.fingerprint_key} (${Math.round(fingerprint.confidence * 100)}%)`;
}

function formatRepairLineage(supervision: TaskSupervisionEvidence): string {
  const lineage = supervision.repair_lineage_ref;
  if (!lineage) {
    return 'not recorded';
  }

  return lineage.checkpoint_ref
    ? `${lineage.source} via ${lineage.checkpoint_ref}`
    : lineage.source;
}

function formatRelationships(runtimeTruth: TaskRuntimeTruth): string {
  return runtimeTruth.run_trace.relationships.length > 0
    ? runtimeTruth.run_trace.relationships.join(', ')
    : 'not recorded';
}

function formatGroundedEvidence(runtimeTruth: TaskRuntimeTruth): string {
  return runtimeTruth.grounded_evidence.length > 0
    ? runtimeTruth.grounded_evidence
        .map((reference) => `${reference.source}: ${reference.reference}`)
        .join(', ')
    : 'none/minimal';
}

function formatStepPolicyBudget(stepPolicy: StepPolicySummary): string {
  const pressure = stepPolicy.budget_pressure;
  if (!pressure) {
    return 'not recorded';
  }

  return `${pressure.pressure_level} via ${pressure.pressure_source} (${pressure.policy_hint})`;
}

function formatStepPolicyReasonCodes(stepPolicy: StepPolicySummary): string {
  return stepPolicy.difficulty_assessment.reason_codes.length > 0
    ? stepPolicy.difficulty_assessment.reason_codes.join(', ')
    : 'none recorded';
}

function formatStepPolicyRefs(stepPolicy: StepPolicySummary): string {
  const refs = [
    stepPolicy.policy_decision.difficulty_ref
      ? `difficulty=${stepPolicy.policy_decision.difficulty_ref}`
      : null,
    stepPolicy.policy_decision.budget_ref
      ? `budget=${stepPolicy.policy_decision.budget_ref}`
      : null,
    stepPolicy.policy_decision.repair_lineage_ref
      ? `repair=${stepPolicy.policy_decision.repair_lineage_ref}`
      : null,
  ].filter(Boolean);

  return refs.length > 0 ? refs.join(', ') : 'not recorded';
}

function formatStepPolicyInputRefs(stepPolicy: StepPolicySummary): string {
  const refs = [
    stepPolicy.input_refs.latest_step_evaluation
      ? `evaluation=${stepPolicy.input_refs.latest_step_evaluation}`
      : null,
    stepPolicy.input_refs.latest_step_routing
      ? `routing=${stepPolicy.input_refs.latest_step_routing}`
      : null,
    stepPolicy.input_refs.supervision_evidence
      ? `supervision=${stepPolicy.input_refs.supervision_evidence}`
      : null,
    stepPolicy.input_refs.runtime_truth
      ? `runtime_truth=${stepPolicy.input_refs.runtime_truth}`
      : null,
    stepPolicy.input_refs.boundary_evidence
      ? `boundary=${stepPolicy.input_refs.boundary_evidence}`
      : null,
  ].filter(Boolean);

  return refs.length > 0 ? refs.join(', ') : 'not recorded';
}
