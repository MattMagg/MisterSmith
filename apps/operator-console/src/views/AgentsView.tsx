
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
  const { agents, selectedAgentId, onSelect, selectedAgentSummary, agentDetail } =
    props;

  return (
    <div className="tab-layout">
      <section className="panel collection-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Agents</p>
            <h2>Registry-backed runtime fleet</h2>
          </div>
          <StatusPill
            label="Count"
            tone="neutral"
            value={`${agents.length} loaded`}
          />
        </div>
        <div className="collection-list" data-testid="agents-list">
          {agents.length === 0 ? (
            <EmptyState
              title="No agents registered"
              body="Once the runtime registers agents, the real registry-backed list will appear here."
            />
          ) : (
            agents.map((agent) => (
              <button
                key={agent.agent_id}
                className={`list-row ${agent.agent_id === selectedAgentId ? 'selected' : ''}`}
                onClick={() => onSelect(agent.agent_id)}
              >
                <div>
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
            <KeyValueGrid
              rows={[
                ['Agent ID', selectedAgentSummary.agent_id],
                ['Name', selectedAgentSummary.name],
                ['Type', selectedAgentSummary.agent_type],
                ['Availability', selectedAgentSummary.availability],
                ['Status', selectedAgentSummary.status],
                [
                  'Last heartbeat',
                  formatTimestamp(selectedAgentSummary.last_heartbeat),
                ],
              ]}
            />
            <section className="subpanel">
              <h3>Runtime posture</h3>
              <div className="metric-grid">
                <div className="metric-card">
                  <span>Availability</span>
                  <strong>{selectedAgentSummary.availability}</strong>
                  <small>{selectedAgentSummary.status}</small>
                </div>
                <div className="metric-card">
                  <span>Agent type</span>
                  <strong>{selectedAgentSummary.agent_type}</strong>
                  <small>{selectedAgentSummary.name}</small>
                </div>
              </div>
            </section>
            <section className="subpanel">
              <h3>Recent agent output</h3>
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
            <section className="subpanel">
              <h3>Metadata</h3>
              <pre>{prettyJson(agentDetail.metadata)}</pre>
            </section>
          </div>
        ) : (
          <EmptyState
            title="Select an agent"
            body="This pane shows the actual registry row instead of the prior placeholder payload."
          />
        )}
      </section>
    </div>
  );
}
