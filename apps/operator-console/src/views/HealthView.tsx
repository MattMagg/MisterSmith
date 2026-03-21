
import type { DashboardSnapshot, AuthSnapshot } from '../types';
import { prettyJson, readMetric } from '../utils/format';
import { KeyValueGrid } from '../components/KeyValueGrid';

export interface HealthViewProps {
  snapshot: DashboardSnapshot | null;
  auth: AuthSnapshot | null;
}

export function HealthView({ snapshot, auth }: HealthViewProps) {
  const localRuntime = snapshot?.localRuntime ?? null;

  return (
    <div className="health-layout">
      <section className="panel detail-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Runtime health</p>
            <h2>Runtime and probes</h2>
          </div>
        </div>
        <KeyValueGrid
          rows={[
            ['Runtime summary', snapshot?.runtimeSummary ?? 'Loading'],
            ['Launcher', localRuntime?.summary ?? 'unavailable'],
            ['Launch state', localRuntime?.state ?? 'unknown'],
            [
              'Runtime owner',
              localRuntime
                ? localRuntime.managed_by_app
                  ? 'desktop app'
                  : 'existing local process'
                : 'unknown',
            ],
            ['Runtime URL', localRuntime?.runtime_url ?? 'unknown'],
            [
              'Health status',
              snapshot?.probes.health?.status ?? 'unavailable',
            ],
            ['Liveness', snapshot?.probes.live ? 'healthy' : 'offline'],
            ['Readiness', snapshot?.probes.ready ? 'ready' : 'not ready'],
            ['Version', snapshot?.probes.config?.version ?? 'unknown'],
          ]}
        />
        <section className="subpanel">
          <h3>Components</h3>
          <div className="metric-grid">
            {(snapshot?.probes.health?.components ?? []).map((component) => (
              <div className="metric-card" key={component.name}>
                <span>{component.name}</span>
                <strong>{component.status}</strong>
                <small>{component.message ?? 'No extra message'}</small>
              </div>
            ))}
          </div>
        </section>
        <section className="subpanel">
          <h3>Executable bootstrap</h3>
          <KeyValueGrid
            rows={[
              [
                'Dependencies bootstrapped',
                localRuntime?.dependencies_managed ? 'yes' : 'no',
              ],
              ['Database target', localRuntime?.database_target ?? 'unknown'],
              ['NATS target', localRuntime?.nats_target ?? 'unknown'],
              ['Last error', localRuntime?.last_error ?? 'none'],
            ]}
          />
          <pre>{prettyJson(localRuntime?.last_log_line ?? 'No runtime log line captured yet.')}</pre>
        </section>
      </section>

      <section className="panel detail-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">NATS monitor</p>
            <h2>Curated transport state</h2>
          </div>
        </div>
        <KeyValueGrid
          rows={[
            ['Summary', snapshot?.nats.summary ?? 'Loading'],
            ['Available', snapshot?.nats.available ? 'yes' : 'no'],
            ['Degraded', snapshot?.nats.degraded ? 'yes' : 'no'],
            ['Connections', readMetric(snapshot?.nats.connz, 'num_connections')],
            ['JetStream streams', readMetric(snapshot?.nats.jsz, 'streams')],
            ['JetStream consumers', readMetric(snapshot?.nats.jsz, 'consumers')],
          ]}
        />
        <section className="subpanel">
          <h3>Monitor errors</h3>
          <pre>{prettyJson(snapshot?.nats.errors ?? [])}</pre>
        </section>
      </section>

      <section className="panel detail-panel">
        <div className="panel-header">
          <div>
            <p className="eyebrow">Auth panel</p>
            <h2>Desktop auth state</h2>
          </div>
        </div>
        <KeyValueGrid
          rows={[
            ['OpenAI', auth?.openAi.summary ?? 'Loading'],
            ['OpenAI email', auth?.openAi.email ?? 'unknown'],
            ['Claude', auth?.claude.summary ?? 'Loading'],
            ['Claude source', auth?.claude.source ?? 'unknown'],
          ]}
        />
        <section className="subpanel">
          <h3>Config payload</h3>
          <pre>{prettyJson(snapshot?.probes.config?.config)}</pre>
        </section>
      </section>
    </div>
  );
}
