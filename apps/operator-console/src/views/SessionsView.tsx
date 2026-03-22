import { type FormEvent } from 'react';
import type { SessionSummary, SessionInspectResponse } from '../types';
import { formatTimestamp, prettyJson } from '../utils/format';
import { StatusPill } from '../components/StatusPill';
import { EmptyState } from '../components/EmptyState';
import { KeyValueGrid } from '../components/KeyValueGrid';
import { PrioritySelect } from '../components/PrioritySelect';

export interface SessionsViewProps {
  busy: string | null;
  createMessage: string;
  createPriority: string;
  continueMessage: string;
  continuePriority: string;
  onCreateMessageChange: (value: string) => void;
  onCreatePriorityChange: (value: string) => void;
  onContinueMessageChange: (value: string) => void;
  onContinuePriorityChange: (value: string) => void;
  onCreateSession: (event: FormEvent<HTMLFormElement>) => void;
  onContinueSession: (event: FormEvent<HTMLFormElement>) => void;
  onEndSession: () => void;
  sessions: SessionSummary[];
  selectedSessionId: string | null;
  onSelect: (sessionId: string) => void;
  selectedSessionSummary?: SessionSummary;
  sessionDetail: SessionInspectResponse | null;
}

export function SessionsView(props: SessionsViewProps) {
  const {
    busy,
    createMessage,
    createPriority,
    continueMessage,
    continuePriority,
    onCreateMessageChange,
    onCreatePriorityChange,
    onContinueMessageChange,
    onContinuePriorityChange,
    onCreateSession,
    onContinueSession,
    onEndSession,
    sessions,
    selectedSessionId,
    onSelect,
    selectedSessionSummary,
    sessionDetail,
  } = props;

  const sessionEnded = selectedSessionSummary
    ? selectedSessionSummary.status.toLowerCase() === 'ended'
    : false;

  return (
    <div className="tab-layout">
      <section className="panel collection-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Sessions</p>
            <h2>Persistent conversations</h2>
          </div>
          <StatusPill label="Count" tone="neutral" value={`${sessions.length} loaded`} />
        </div>

        <form className="compose-form" onSubmit={onCreateSession}>
          <label className="field field-block">
            <span>Start session</span>
            <textarea
              placeholder="Seed the first turn for a retained conversation."
              value={createMessage}
              onChange={(event) => onCreateMessageChange(event.target.value)}
            />
          </label>
          <div className="compose-actions">
            <PrioritySelect value={createPriority} onChange={onCreatePriorityChange} />
            <button
              className="primary-button"
              disabled={busy === 'Starting session' || !createMessage.trim()}
            >
              {busy === 'Starting session' ? 'Starting' : 'Create session'}
            </button>
          </div>
        </form>

        <div className="collection-list" data-testid="sessions-list">
          {sessions.length === 0 ? (
            <EmptyState
              title="No sessions available"
              body="Start a session to retain a coordinator, turn lineage, and restart-aware state."
            />
          ) : (
            sessions.map((session) => (
              <button
                key={session.session_id}
                className={`list-row ${session.session_id === selectedSessionId ? 'selected' : ''}`}
                onClick={() => onSelect(session.session_id)}
              >
                <div className="list-row-copy">
                  <strong>{session.session_id}</strong>
                  <p>{session.last_preview ?? 'No retained preview yet'}</p>
                </div>
                <div className="list-row-meta">
                  <span>{session.status}</span>
                  <small>{formatTimestamp(session.updated_at)}</small>
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
            <h2>Selected session</h2>
          </div>
        </div>

        {selectedSessionSummary && sessionDetail ? (
          <div className="detail-stack">
            <section className="detail-hero">
              <div className="detail-hero-copy">
                <p className="eyebrow">Session focus</p>
                <h3>{selectedSessionSummary.session_id}</h3>
                <p>
                  Coordinator {selectedSessionSummary.coordinator_agent_id} is retaining{' '}
                  {selectedSessionSummary.turn_count} turns on{' '}
                  {selectedSessionSummary.model_id}.
                </p>
              </div>
              <div className="detail-hero-aside">
                <StatusPill
                  label="Status"
                  tone={sessionEnded ? 'neutral' : 'good'}
                  value={selectedSessionSummary.status}
                />
                <StatusPill
                  label="Turns"
                  tone="neutral"
                  value={String(selectedSessionSummary.turn_count)}
                />
              </div>
            </section>

            <KeyValueGrid
              rows={[
                ['Coordinator', selectedSessionSummary.coordinator_agent_id],
                ['Provider', selectedSessionSummary.provider_kind],
                ['Model', selectedSessionSummary.model_id],
                ['Updated', formatTimestamp(selectedSessionSummary.updated_at)],
                ['Active workflow', selectedSessionSummary.active_workflow_id ?? 'none'],
                [
                  'Last completed',
                  selectedSessionSummary.last_completed_workflow_id ?? 'none',
                ],
              ]}
            />

            <form className="compose-form compact-form" onSubmit={onContinueSession}>
              <label className="field field-block">
                <span>Continue session</span>
                <textarea
                  placeholder="Add the next prompt to the selected session."
                  value={continueMessage}
                  onChange={(event) => onContinueMessageChange(event.target.value)}
                />
              </label>
              <div className="compose-actions compose-actions-wrap">
                <PrioritySelect
                  value={continuePriority}
                  onChange={onContinuePriorityChange}
                />
                <button
                  className="primary-button"
                  disabled={
                    sessionEnded ||
                    busy === 'Continuing session' ||
                    !continueMessage.trim()
                  }
                >
                  {busy === 'Continuing session' ? 'Sending' : 'Continue session'}
                </button>
                <button
                  type="button"
                  className="secondary-button"
                  disabled={sessionEnded || busy === 'Ending session'}
                  onClick={onEndSession}
                >
                  {busy === 'Ending session' ? 'Ending' : 'End session'}
                </button>
              </div>
            </form>

            <section className="subpanel">
              <h3>Retained transcript</h3>
              <div className="terminal-shell">
                <div className="terminal-header">
                  <strong>session transcript</strong>
                  <span>{selectedSessionSummary.status}</span>
                </div>
                <div className="terminal-body">
                  {sessionDetail.turns.map((turn) => (
                    <div className="terminal-entry" key={turn.workflow_id}>
                      <span className="terminal-entry-time">turn {turn.turn_index}</span>
                      <span className="terminal-entry-line">{turn.user_message}</span>
                    </div>
                  ))}
                </div>
                <div className="terminal-footer">
                  <input
                    className="terminal-input"
                    readOnly
                    value={sessionEnded ? 'session ended' : 'continue selected session'}
                  />
                  <button className="ghost-button" type="button" disabled={sessionEnded}>
                    armed
                  </button>
                </div>
              </div>
            </section>

            <section className="subpanel">
              <h3>Turn history</h3>
              <div className="turn-list">
                {sessionDetail.turns.map((turn) => (
                  <article className="turn-card" key={turn.workflow_id}>
                    <div className="timeline-meta">
                      <strong>Turn {turn.turn_index}</strong>
                      <span>{turn.status}</span>
                    </div>
                    <p>{turn.user_message}</p>
                    <details className="payload-disclosure">
                      <summary>Turn payload</summary>
                      <pre>{prettyJson(turn.assistant_result ?? turn.resume_provenance)}</pre>
                    </details>
                  </article>
                ))}
              </div>
            </section>

            <details className="payload-disclosure payload-disclosure-block">
              <summary>Retained payload</summary>
              <pre>{prettyJson(sessionDetail.last_assistant_result)}</pre>
            </details>
          </div>
        ) : (
          <EmptyState
            title="Select a session"
            body="Choose a retained session to inspect transcript, actions, and restart-aware payloads."
          />
        )}
      </section>
    </div>
  );
}
