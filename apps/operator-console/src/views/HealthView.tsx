import type { DashboardSnapshot, AuthSnapshot } from '../types';
import { prettyJson, readMetric } from '../utils/format';
import { KeyValueGrid } from '../components/KeyValueGrid';
import { AuthCard } from '../components/AuthCard';

export interface HealthViewProps {
  snapshot: DashboardSnapshot | null;
  auth: AuthSnapshot | null;
  actionBusy: string | null;
  onOpenAiLogin: () => void;
}

export function HealthView({
  snapshot,
  auth,
  actionBusy,
  onOpenAiLogin,
}: HealthViewProps) {
  const localRuntime = snapshot?.localRuntime ?? null;

  return (
    <div className="health-layout">
      <section className="health-primary-column">
        <section className="panel detail-panel health-scroll">
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
              ['Health status', snapshot?.probes.health?.status ?? 'unavailable'],
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
        </section>

        <section className="panel detail-panel health-scroll">
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
          <details className="payload-disclosure payload-disclosure-block">
            <summary>Monitor errors</summary>
            <pre>{prettyJson(snapshot?.nats.errors ?? [])}</pre>
          </details>
        </section>
      </section>

      <section className="health-secondary-column">
        <section className="panel detail-panel health-scroll">
          <div className="panel-header">
            <div>
              <p className="eyebrow">Desktop auth</p>
              <h2>Operator credentials</h2>
            </div>
          </div>
          <div className="subpanel-stack">
            <AuthCard
              title="OpenAI ChatGPT"
              summary={auth?.openAi.summary ?? 'Loading OpenAI session'}
              tone={auth?.openAi.authenticated ? 'good' : 'warn'}
              meta={[
                auth?.openAi.email ?? 'email unavailable',
                auth?.openAi.plan_type ?? 'plan unavailable',
              ]}
              actionLabel="Login"
              onAction={onOpenAiLogin}
              disabled={actionBusy !== null}
            />
            <AuthCard
              title="Claude subscription"
              summary={auth?.claude.summary ?? 'Loading Claude credentials'}
              tone={
                auth?.claude.authenticated
                  ? auth?.claude.expired
                    ? 'warn'
                    : 'good'
                  : 'warn'
              }
              meta={[
                auth?.claude.source ?? 'source unavailable',
                auth?.claude.masked_token ?? 'token unavailable',
              ]}
            />
          </div>
        </section>

        <section className="panel detail-panel health-scroll">
          <div className="panel-header">
            <div>
              <p className="eyebrow">Bootstrap</p>
              <h2>Launcher and config</h2>
            </div>
          </div>
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
          <details className="payload-disclosure payload-disclosure-block">
            <summary>Last runtime log line</summary>
            <pre>
              {prettyJson(
                localRuntime?.last_log_line ?? 'No runtime log line captured yet.',
              )}
            </pre>
          </details>
          <details className="payload-disclosure payload-disclosure-block">
            <summary>Config payload</summary>
            <pre>{prettyJson(snapshot?.probes.config?.config)}</pre>
          </details>
        </section>
      </section>
    </div>
  );
}
