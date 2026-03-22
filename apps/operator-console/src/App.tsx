import { startTransition, useEffect, useState, type FormEvent } from 'react';
import './App.css';
import { RunsView } from './views/RunsView';
import { SessionsView } from './views/SessionsView';
import { AgentsView } from './views/AgentsView';
import { HealthView } from './views/HealthView';
import { StatusPill } from './components/StatusPill';
import { EmptyState } from './components/EmptyState';
import { formatTimestamp, prettyJson, formatError } from './utils/format';

import {
  defaultOperatorConsoleServices,
  type OperatorConsoleServices,
} from './services';
import type {
  AuthSnapshot,
  DashboardSelection,
  DashboardSnapshot,
  LocalRuntimeSnapshot,
  StoredSettings,
  TabId,
  TimelineConnectionState,
  TimelineEvent,
} from './types';

interface AppProps {
  services?: OperatorConsoleServices;
  initialSettings?: StoredSettings;
}

function App({
  services = defaultOperatorConsoleServices,
  initialSettings,
}: AppProps) {
  const [settings, setSettings] = useState<StoredSettings>(() =>
    initialSettings ?? services.loadSettings(),
  );
  const [selection, setSelection] = useState<DashboardSelection>({
    runId: null,
    sessionId: null,
    agentId: null,
  });
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [auth, setAuth] = useState<AuthSnapshot | null>(null);
  const [timeline, setTimeline] = useState<TimelineEvent[]>([]);
  const [timelineConnection, setTimelineConnection] =
    useState<TimelineConnectionState>('connecting');
  const [dashboardBusy, setDashboardBusy] = useState(true);
  const [actionBusy, setActionBusy] = useState<string | null>(null);
  const [bannerError, setBannerError] = useState<string | null>(null);
  const [dashboardRefreshToken, setDashboardRefreshToken] = useState(0);
  const [authRefreshToken, setAuthRefreshToken] = useState(0);
  const [timelineRefreshToken, setTimelineRefreshToken] = useState(0);
  const [taskDescription, setTaskDescription] = useState('');
  const [taskPriority, setTaskPriority] = useState('normal');
  const [sessionMessage, setSessionMessage] = useState('');
  const [sessionPriority, setSessionPriority] = useState('normal');
  const [continueMessage, setContinueMessage] = useState('');
  const [continuePriority, setContinuePriority] = useState('normal');
  const selectedRunId = selection.runId;
  const selectedSessionId = selection.sessionId;
  const selectedAgentId = selection.agentId;
  const hasSnapshot = snapshot !== null;
  const runtimeReachable = snapshot?.runtimeReachable ?? false;
  const localRuntimeState = snapshot?.localRuntime?.state ?? null;

  useEffect(() => {
    services.saveSettings(settings);
  }, [services, settings]);

  useEffect(() => {
    let cancelled = false;
    const dashboardSelection = {
      runId: selectedRunId,
      sessionId: selectedSessionId,
      agentId: selectedAgentId,
    };

    const refreshDashboard = async () => {
      try {
        const nextSnapshot = await services.fetchDashboard(
          settings,
          dashboardSelection,
        );
        if (cancelled) {
          return;
        }

        startTransition(() => {
          setSnapshot(nextSnapshot);
          setSelection({
            runId: nextSnapshot.selectedRunId,
            sessionId: nextSnapshot.selectedSessionId,
            agentId: nextSnapshot.selectedAgentId,
          });
        });

        setBannerError(nextSnapshot.errors[0] ?? null);
      } catch (error) {
        if (!cancelled) {
          setBannerError(formatError(error));
        }
      } finally {
        if (!cancelled) {
          setDashboardBusy(false);
        }
      }
    };

    if (!hasSnapshot) {
      setDashboardBusy(true);
    }

    void refreshDashboard();
    const intervalId = window.setInterval(() => {
      void refreshDashboard();
    }, 5000);

    return () => {
      cancelled = true;
      window.clearInterval(intervalId);
    };
  }, [
    services,
    settings,
    selectedRunId,
    selectedSessionId,
    selectedAgentId,
    hasSnapshot,
    dashboardRefreshToken,
  ]);

  useEffect(() => {
    let cancelled = false;

    const refreshAuth = async () => {
      const nextAuth = await services.fetchAuthSnapshot();
      if (!cancelled) {
        setAuth(nextAuth);
      }
    };

    void refreshAuth();

    return () => {
      cancelled = true;
    };
  }, [services, authRefreshToken]);

  useEffect(() => {
    if (!hasSnapshot) {
      setTimelineConnection('connecting');
      return;
    }

    const localRuntime = localRuntimeState
      ? { state: localRuntimeState }
      : null;
    const startupPending = isRuntimeStartupPending(localRuntime);
    const startupFailed = localRuntimeState === 'failed' && !runtimeReachable;

    if (startupPending || startupFailed) {
      setTimelineConnection(startupPending ? 'connecting' : 'disconnected');
      return;
    }

    let cancelled = false;
    let reconnectTimeout: number | undefined;
    let disconnect:
      | (() => void | Promise<void>)
      | undefined;

    const scheduleReconnect = () => {
      if (!settings.reconnectEnabled || cancelled) {
        return;
      }

      reconnectTimeout = window.setTimeout(() => {
        setTimelineRefreshToken((current) => current + 1);
      }, 1250);
    };

    void services
      .connectTimeline(
        settings.runtimeBaseUrl,
        (event) => {
          if (cancelled) {
            return;
          }

          startTransition(() => {
            setTimeline((current) => [event, ...current].slice(0, 500));
          });
          setDashboardRefreshToken((current) => current + 1);
        },
        (state) => {
          if (cancelled) {
            return;
          }

          setTimelineConnection(state);
          if (state === 'disconnected') {
            scheduleReconnect();
          }
        },
      )
      .then((cleanup) => {
        if (cancelled) {
          void cleanup();
          return;
        }

        disconnect = cleanup;
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        setTimelineConnection('disconnected');
        setBannerError(formatError(error));
        scheduleReconnect();
      });

    return () => {
      cancelled = true;
      if (reconnectTimeout !== undefined) {
        window.clearTimeout(reconnectTimeout);
      }
      if (disconnect) {
        void disconnect();
      }
    };
  }, [
    services,
    settings.runtimeBaseUrl,
    settings.reconnectEnabled,
    timelineRefreshToken,
    hasSnapshot,
    runtimeReachable,
    localRuntimeState,
  ]);
  const selectedRunSummary = snapshot?.runs.find((run) => run.task_id === selectedRunId);
  const selectedSessionSummary = snapshot?.sessions.find(
    (session) => session.session_id === selectedSessionId,
  );
  const selectedAgentSummary = snapshot?.agents.find(
    (agent) => agent.agent_id === selectedAgentId,
  );

  const handleManualRefresh = () => {
    setDashboardBusy(true);
    setDashboardRefreshToken((current) => current + 1);
    setAuthRefreshToken((current) => current + 1);
  };

  const handleTaskSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const description = taskDescription.trim();
    if (!description) {
      return;
    }

    setActionBusy('Submitting task');
    try {
      const created = await services.createTask(settings.runtimeBaseUrl, {
        description,
        priority: taskPriority,
      });
      setTaskDescription('');
      setSelection((current) => ({ ...current, runId: created.task_id }));
      setSettings((current) => ({ ...current, activeTab: 'runs' }));
      setDashboardBusy(true);
      setDashboardRefreshToken((current) => current + 1);
      setBannerError(null);
    } catch (error) {
      setBannerError(formatError(error));
    } finally {
      setActionBusy(null);
    }
  };

  const handleSessionCreate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const message = sessionMessage.trim();
    if (!message) {
      return;
    }

    setActionBusy('Starting session');
    try {
      const accepted = await services.createSession(settings.runtimeBaseUrl, {
        message,
        priority: sessionPriority,
      });
      setSessionMessage('');
      setSelection((current) => ({
        ...current,
        sessionId: accepted.session_id,
        runId: accepted.workflow_id,
      }));
      setSettings((current) => ({ ...current, activeTab: 'sessions' }));
      setDashboardBusy(true);
      setDashboardRefreshToken((current) => current + 1);
      setBannerError(null);
    } catch (error) {
      setBannerError(formatError(error));
    } finally {
      setActionBusy(null);
    }
  };

  const handleSessionContinue = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const sessionId = snapshot?.selectedSessionId;
    const message = continueMessage.trim();

    if (!sessionId || !message) {
      return;
    }

    setActionBusy('Continuing session');
    try {
      const accepted = await services.continueSession(settings.runtimeBaseUrl, {
        sessionId,
        message,
        priority: continuePriority,
      });
      setContinueMessage('');
      setSelection((current) => ({
        ...current,
        sessionId: accepted.session_id,
        runId: accepted.workflow_id,
      }));
      setDashboardBusy(true);
      setDashboardRefreshToken((current) => current + 1);
      setBannerError(null);
    } catch (error) {
      setBannerError(formatError(error));
    } finally {
      setActionBusy(null);
    }
  };

  const handleSessionEnd = async () => {
    const sessionId = snapshot?.selectedSessionId;
    if (!sessionId) {
      return;
    }

    setActionBusy('Ending session');
    try {
      await services.endSession(settings.runtimeBaseUrl, sessionId);
      setDashboardBusy(true);
      setDashboardRefreshToken((current) => current + 1);
      setBannerError(null);
    } catch (error) {
      setBannerError(formatError(error));
    } finally {
      setActionBusy(null);
    }
  };

  const handleOpenAiLogin = async () => {
    setActionBusy('Refreshing OpenAI auth');
    try {
      const nextAuth = await services.loginOpenAiChatGpt();
      setAuth(nextAuth);
      setBannerError(nextAuth.errors[0] ?? null);
    } catch (error) {
      setBannerError(formatError(error));
    } finally {
      setActionBusy(null);
    }
  };

  const activeTab = settings.activeTab;
  const localRuntime = snapshot?.localRuntime ?? null;
  const runtimeStartupPending = isRuntimeStartupPending(localRuntime);
  const runtimeStartupFailed =
    localRuntime?.state === 'failed' && !snapshot?.runtimeReachable;
  const runtimeSummary = snapshot?.runtimeSummary ?? 'Loading runtime state';
  const runtimeClass = snapshot?.runtimeReachable
    ? 'good'
    : runtimeStartupPending
      ? 'warn'
      : 'bad';
  const launcherSummary =
    localRuntime?.summary ?? 'Launcher unavailable in browser preview';
  const launcherClass = toneForLauncher(localRuntime);
  const natsSummary = snapshot?.nats.summary ?? 'Loading NATS monitor';
  const natsClass = runtimeStartupPending
    ? 'warn'
    : runtimeStartupFailed
      ? 'bad'
      : snapshot?.nats.available && !snapshot.nats.degraded
        ? 'good'
        : snapshot?.nats.available
          ? 'warn'
          : 'bad';
  const timelineSummary = runtimeStartupPending
    ? 'waiting on runtime'
    : runtimeStartupFailed
      ? 'blocked by launcher failure'
      : timelineConnection;
  const timelineClass =
    runtimeStartupPending
      ? 'warn'
      : runtimeStartupFailed
        ? 'bad'
        : timelineConnection === 'connected'
      ? 'good'
        : timelineConnection === 'connecting'
          ? 'warn'
          : 'bad';
  const refreshSummary = dashboardBusy ? 'Syncing dashboard' : 'Idle refresh';
  const openAiReady = auth?.openAi.authenticated ?? false;
  const activeTabName = tabLabel(activeTab);

  return (
    <div className="console-shell">
      <header className="topbar">
        <div className="topbar-brand">
          <div className="brand-mark">
            <span>MISTERSMITH</span>
            <small>operator cockpit</small>
          </div>
        </div>
        <div className="topbar-tools">
          <div className="topbar-chip">
            <span className={`status-dot tone-${runtimeClass}`}></span>
            <strong>{activeTabName}</strong>
          </div>
          <div className="topbar-chip">
            <span className={`status-dot tone-${openAiReady ? 'good' : 'warn'}`}></span>
            <strong>
              {openAiReady ? 'OpenAI authenticated' : 'OpenAI login required'}
            </strong>
          </div>
          <div className="topbar-chip topbar-chip-muted">
            <span className={`status-dot tone-${dashboardBusy ? 'warn' : 'neutral'}`}></span>
            <strong>{refreshSummary}</strong>
          </div>
          {!openAiReady ? (
            <button className="ghost-button" onClick={handleOpenAiLogin}>
              OpenAI login
            </button>
          ) : null}
        </div>
      </header>

      <div className="console-grid">
        <aside className="sidebar">
          <nav className="nav-stack" aria-label="Primary navigation">
            {(['runs', 'sessions', 'agents', 'health'] as TabId[]).map((tab) => (
              <button
                key={tab}
                className={`nav-button ${activeTab === tab ? 'active' : ''}`}
                onClick={() =>
                  setSettings((current) => ({ ...current, activeTab: tab }))
                }
              >
                <span>{tabLabel(tab)}</span>
                <small>{tabSummary(tab, snapshot)}</small>
              </button>
            ))}
          </nav>
          <div className="nav-footer">
            <div className="nav-version">
              <span>v2.4.0</span>
              <span>local</span>
            </div>
          </div>
        </aside>

        <div className="workspace">
          <section className="workspace-strip">
            <div className="strip-status">
              <StatusPill label="Launcher" tone={launcherClass} value={launcherSummary} />
              <StatusPill
                label="Runtime"
                tone={runtimeClass}
                value={runtimeSummary}
                testId="runtime-status"
              />
              <StatusPill
                label="Timeline"
                tone={timelineClass}
                value={timelineSummary}
              />
              <StatusPill label="NATS" tone={natsClass} value={natsSummary} />
            </div>
            <div className="strip-controls">
              <label className="field">
                <span>Runtime URL</span>
                <input
                  value={settings.runtimeBaseUrl}
                  onChange={(event) =>
                    setSettings((current) => ({
                      ...current,
                      runtimeBaseUrl: event.target.value,
                    }))
                  }
                />
              </label>
              <label className="field">
                <span>NATS monitor URL</span>
                <input
                  value={settings.natsMonitorUrl}
                  onChange={(event) =>
                    setSettings((current) => ({
                      ...current,
                      natsMonitorUrl: event.target.value,
                    }))
                  }
                />
              </label>
              <label className="toggle-field">
                <input
                  type="checkbox"
                  checked={settings.reconnectEnabled}
                  onChange={(event) =>
                    setSettings((current) => ({
                      ...current,
                      reconnectEnabled: event.target.checked,
                    }))
                  }
                />
                <span>Reconnect websocket</span>
              </label>
              <button className="secondary-button" onClick={handleManualRefresh}>
                Refresh
              </button>
            </div>
          </section>

          {bannerError ? (
            <div className="banner-error" role="alert">
              {bannerError}
            </div>
          ) : null}

          <div className="workspace-body">
            <main className="main-pane">
              {activeTab === 'runs' ? (
                <RunsView
                  busy={actionBusy === 'Submitting task'}
                  taskDescription={taskDescription}
                  taskPriority={taskPriority}
                  onTaskDescriptionChange={setTaskDescription}
                  onTaskPriorityChange={setTaskPriority}
                  onSubmit={handleTaskSubmit}
                  runs={snapshot?.runs ?? []}
                  selectedRunId={snapshot?.selectedRunId ?? null}
                  onSelect={(runId) =>
                    setSelection((current) => ({ ...current, runId }))
                  }
                  selectedRunSummary={selectedRunSummary}
                  taskDetail={snapshot?.runDetail ?? null}
                />
              ) : null}

              {activeTab === 'sessions' ? (
                <SessionsView
                  busy={actionBusy}
                  createMessage={sessionMessage}
                  createPriority={sessionPriority}
                  continueMessage={continueMessage}
                  continuePriority={continuePriority}
                  onCreateMessageChange={setSessionMessage}
                  onCreatePriorityChange={setSessionPriority}
                  onContinueMessageChange={setContinueMessage}
                  onContinuePriorityChange={setContinuePriority}
                  onCreateSession={handleSessionCreate}
                  onContinueSession={handleSessionContinue}
                  onEndSession={handleSessionEnd}
                  sessions={snapshot?.sessions ?? []}
                  selectedSessionId={snapshot?.selectedSessionId ?? null}
                  onSelect={(sessionId) =>
                    setSelection((current) => ({ ...current, sessionId }))
                  }
                  selectedSessionSummary={selectedSessionSummary}
                  sessionDetail={snapshot?.sessionDetail ?? null}
                />
              ) : null}

              {activeTab === 'agents' ? (
                <AgentsView
                  agents={snapshot?.agents ?? []}
                  selectedAgentId={snapshot?.selectedAgentId ?? null}
                  onSelect={(agentId) =>
                    setSelection((current) => ({ ...current, agentId }))
                  }
                  selectedAgentSummary={selectedAgentSummary}
                  agentDetail={snapshot?.agentDetail ?? null}
                />
              ) : null}

              {activeTab === 'health' ? (
                <HealthView
                  snapshot={snapshot}
                  auth={auth}
                  actionBusy={actionBusy}
                  onOpenAiLogin={handleOpenAiLogin}
                />
              ) : null}
            </main>

            <aside className="timeline-pane">
              <section className="panel timeline-panel">
                <div className="panel-header">
                  <div>
                    <p className="eyebrow">Live signal rail</p>
                    <h2>Runtime event stream</h2>
                  </div>
                  <StatusPill
                    label="WebSocket"
                    tone={timelineClass}
                    value={timelineSummary}
                  />
                </div>
                <div className="timeline-list" data-testid="timeline-list">
                  {timeline.length === 0 ? (
                    <EmptyState
                      title="No events yet"
                      body="Runtime events appear here once the websocket feed starts emitting."
                    />
                  ) : (
                    timeline.map((event, index) => (
                      <article className="timeline-item" key={`${event.timestamp}-${index}`}>
                        <div className="timeline-meta">
                          <strong>{event.event_type}</strong>
                          <span>{formatTimestamp(event.timestamp)}</span>
                        </div>
                        <p className="timeline-copy">
                          {summarizeTimelinePayload(event.payload)}
                        </p>
                        <details className="payload-disclosure">
                          <summary>Payload</summary>
                          <pre>{prettyJson(event.payload)}</pre>
                        </details>
                      </article>
                    ))
                  )}
                </div>
              </section>
            </aside>
          </div>
        </div>
      </div>
    </div>
  );
}

function tabLabel(tab: TabId): string {
  switch (tab) {
    case 'runs':
      return 'Runs';
    case 'sessions':
      return 'Sessions';
    case 'agents':
      return 'Agents';
    case 'health':
      return 'Health';
    default:
      return tab;
  }
}

function tabSummary(tab: TabId, snapshot: DashboardSnapshot | null): string {
  switch (tab) {
    case 'runs':
      return `${snapshot?.runs.length ?? 0} workflows`;
    case 'sessions':
      return `${snapshot?.sessions.length ?? 0} retained`;
    case 'agents':
      return `${snapshot?.agents.length ?? 0} registered`;
    case 'health':
      return snapshot?.runtimeSummary ?? 'runtime probes';
    default:
      return '';
  }
}

function summarizeTimelinePayload(payload: TimelineEvent['payload']): string {
  if (payload == null) {
    return 'No payload details recorded.';
  }

  if (typeof payload === 'string') {
    return payload;
  }

  if (Array.isArray(payload)) {
    return payload.slice(0, 6).join(', ');
  }

  const summaryPairs = Object.entries(payload)
    .slice(0, 3)
    .map(([key, value]) => {
      if (
        typeof value === 'string' ||
        typeof value === 'number' ||
        typeof value === 'boolean'
      ) {
        return `${key}: ${value}`;
      }

      if (Array.isArray(value)) {
        return `${key}: ${value.length} items`;
      }

      if (value && typeof value === 'object') {
        return `${key}: object`;
      }

      return `${key}: ${String(value)}`;
    });

  return summaryPairs.join(' · ') || 'Structured payload attached.';
}

function toneForLauncher(
  localRuntime: LocalRuntimeSnapshot | null,
): 'good' | 'warn' | 'bad' | 'neutral' {
  if (!localRuntime) {
    return 'neutral';
  }

  switch (localRuntime.state) {
    case 'external_ready':
    case 'managed_ready':
      return 'good';
    case 'checking':
    case 'starting_dependencies':
    case 'starting_runtime':
      return 'warn';
    case 'failed':
      return 'bad';
    default:
      return 'neutral';
  }
}

function isRuntimeStartupPending(
  localRuntime: Pick<LocalRuntimeSnapshot, 'state'> | null,
): boolean {
  if (!localRuntime) {
    return false;
  }

  return (
    localRuntime.state === 'checking' ||
    localRuntime.state === 'starting_dependencies' ||
    localRuntime.state === 'starting_runtime'
  );
}

export default App;
