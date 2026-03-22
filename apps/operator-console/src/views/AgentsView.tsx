import type { AgentDetail, AgentSummary } from '../types';
import { formatTimestamp, prettyJson } from '../utils/format';
import { StatusPill } from '../components/StatusPill';
import { EmptyState } from '../components/EmptyState';
import { KeyValueGrid } from '../components/KeyValueGrid';

export interface AgentsViewProps {
  agents: AgentSummary[];
  selectedAgentId: string | null;
  onSelect: (agentId: string) => void;
  selectedAgentSummary?: AgentSummary;
  agentDetail: AgentDetail | null;
}

export function AgentsView(props: AgentsViewProps) {
  const { agents, selectedAgentId, onSelect, selectedAgentSummary, agentDetail } = props;

  return (
    <div className="tab-layout">
      <section className="panel collection-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Agents</p>
            <h2>Registry-backed runtime fleet</h2>
          </div>
          <StatusPill label="Count" tone="neutral" value={`${agents.length} loaded`} />
        </div>
        <div className="collection-list" data-testid="agents-list">
          {agents.length === 0 ? (
            <EmptyState
              title="No agents registered"
              body="Registered agents will appear here as soon as the runtime reports real registry rows."
            />
          ) : (
            agents.map((agent) => (
              <button
                key={agent.agent_id}
                className={`list-row ${agent.agent_id === selectedAgentId ? 'selected' : ''}`}
                onClick={() => onSelect(agent.agent_id)}
              >
                <div className="list-row-copy">
                  <strong>{agent.name}</strong>
                  <p>{agent.agent_id}</p>
                </div>
                <div className="list-row-meta">
                  <span>{agent.status}</span>
                  <small>{agent.agent_type}</small>
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
            <h2>Selected agent</h2>
          </div>
        </div>
        {selectedAgentSummary && agentDetail ? (
          <div className="detail-stack">
            <section className="detail-hero">
              <div className="detail-hero-copy">
                <p className="eyebrow">Runtime posture</p>
                <h3>{selectedAgentSummary.name}</h3>
                <p>
                  {selectedAgentSummary.agent_type} agent {selectedAgentSummary.agent_id}{' '}
                  last heartbeat {formatTimestamp(selectedAgentSummary.last_heartbeat)}.
                </p>
              </div>
              <div className="detail-hero-aside">
                <StatusPill
                  label="Availability"
                  tone={toneForAgent(selectedAgentSummary.availability)}
                  value={selectedAgentSummary.availability}
                />
                <StatusPill
                  label="Status"
                  tone={toneForAgent(selectedAgentSummary.status)}
                  value={selectedAgentSummary.status}
                />
              </div>
            </section>

            <KeyValueGrid
              rows={[
                ['Agent ID', selectedAgentSummary.agent_id],
                ['Type', selectedAgentSummary.agent_type],
                ['Last heartbeat', formatTimestamp(selectedAgentSummary.last_heartbeat)],
                ['Availability', selectedAgentSummary.availability],
              ]}
            />

            <section className="subpanel">
              <h3>Agent output and heartbeat</h3>
              <div className="terminal-shell">
                <div className="terminal-header">
                  <strong>agent command stream</strong>
                  <span>{selectedAgentSummary.status}</span>
                </div>
                <div className="terminal-body">
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">agent</span>
                    <span className="terminal-entry-line emphasis">
                      {selectedAgentSummary.name}
                    </span>
                  </div>
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">heartbeat</span>
                    <span className="terminal-entry-line">
                      {formatTimestamp(selectedAgentSummary.last_heartbeat)}
                    </span>
                  </div>
                  <div className="terminal-entry">
                    <span className="terminal-entry-time">status</span>
                    <span className="terminal-entry-line">
                      {selectedAgentSummary.availability} / {selectedAgentSummary.status}
                    </span>
                  </div>
                </div>
              </div>
            </section>

            <details className="payload-disclosure payload-disclosure-block">
              <summary>Metadata</summary>
              <pre>{prettyJson(agentDetail.metadata)}</pre>
            </details>
          </div>
        ) : (
          <EmptyState
            title="Select an agent"
            body="Choose an agent to inspect its runtime posture, heartbeat, and metadata."
          />
        )}
      </section>
    </div>
  );
}

function toneForAgent(value: string): 'good' | 'warn' | 'bad' | 'neutral' {
  const normalized = value.toLowerCase();

  if (normalized.includes('idle') || normalized.includes('ready') || normalized.includes('active')) {
    return 'good';
  }

  if (normalized.includes('error') || normalized.includes('fail') || normalized.includes('offline')) {
    return 'bad';
  }

  if (normalized.includes('busy') || normalized.includes('pending') || normalized.includes('running')) {
    return 'warn';
  }

  return 'neutral';
}
