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
          <StatusPill
            label="Count"
            tone="neutral"
            value={`${runs.length} loaded`}
          />
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
            <PrioritySelect
              value={taskPriority}
              onChange={onTaskPriorityChange}
            />
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
                <div>
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
            <KeyValueGrid
              rows={[
                ['Task ID', selectedRunSummary.task_id],
                ['Status', selectedRunSummary.status],
                ['Priority', String(selectedRunSummary.priority)],
                ['Created', formatTimestamp(selectedRunSummary.created_at)],
                ['Started', formatTimestamp(selectedRunSummary.started_at)],
                ['Completed', formatTimestamp(selectedRunSummary.completed_at)],
                ['Session', selectedRunSummary.session_id ?? 'none'],
                [
                  'Proof outcome',
                  selectedRunSummary.proof_outcome ?? 'not available',
                ],
              ]}
            />
            <PreviewCard preview={selectedRunSummary.result_preview} />
            <section className="subpanel">
              <h3>Runtime task response</h3>
              <pre>{prettyJson(taskDetail ?? selectedRunSummary)}</pre>
            </section>
          </div>
        ) : (
          <EmptyState
            title="Select a run"
            body="The detail pane shows the canonical inspect route plus the shared preview projection."
          />
        )}
      </section>
    </div>
  );
}
