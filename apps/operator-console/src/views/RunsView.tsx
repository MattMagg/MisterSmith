import { type FormEvent } from 'react';
import type { RunSummary } from '../types';
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
  taskDetail: { task_id: string; status: string; result?: unknown } | null;
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
