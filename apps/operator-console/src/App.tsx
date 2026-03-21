import { startTransition, useEffect, useState, type FormEvent } from 'react';
import './App.css';
import {
  defaultOperatorConsoleServices,
  type OperatorConsoleServices,
} from './services';
import type {
  AgentDetail,
  AgentSummary,
  AuthSnapshot,
  DashboardSelection,
  DashboardSnapshot,
  LocalRuntimeSnapshot,
  ResultPreview,
  RunSummary,
  SessionInspectResponse,
  SessionSummary,
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

  useEffect(() => {
    services.saveSettings(settings);
  }, [services, settings]);

  useEffect(() => {
    let cancelled = false;

    const refreshDashboard = async () => {
      try {
        const nextSnapshot = await services.fetchDashboard(settings, selection);
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

    if (snapshot === null) {
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
    selection.runId,
    selection.sessionId,
    selection.agentId,
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
    if (snapshot === null) {
      setTimelineConnection('connecting');
      return;
    }

    const localRuntime = snapshot.localRuntime ?? null;
    const startupPending = isRuntimeStartupPending(localRuntime);
    const startupFailed =
      localRuntime?.state === 'failed' && !snapshot.runtimeReachable;

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
    snapshot?.runtimeReachable,
    snapshot?.localRuntime?.state,
  ]);

  const selectedRunSummary = snapshot?.runs.find(
    (run) => run.task_id === snapshot.selectedRunId,
  );
  const selectedSessionSummary = snapshot?.sessions.find(
    (session) => session.session_id === snapshot.selectedSessionId,
  );
  const selectedAgentSummary = snapshot?.agents.find(
    (agent) => agent.agent_id === snapshot.selectedAgentId,
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
  const refreshSummary = dashboardBusy ? 'Syncing dashboard' : 'Steady';

  return (
    <div className="console-shell">
      <header className="topbar">
        <div className="topbar-main">
          <div className="topbar-hero">
            <p className="eyebrow">Mister Smith Operator Console</p>
            <h1>Local operator cockpit</h1>
            <p className="hero-copy">
              Attach to the local stack, inspect workflow state, and keep session
              handoffs visible without dropping into raw runtime logs.
            </p>
          </div>
          <div className="topbar-aside">
            <div className="status-strip">
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

            <section className="control-panel">
              <div className="control-panel-header">
                <div>
                  <p className="eyebrow">Connection</p>
                  <h2>Loopback runtime settings</h2>
                </div>
                <div className={`refresh-chip ${dashboardBusy ? 'warn' : 'neutral'}`}>
                  <span>Refresh</span>
                  <strong>{refreshSummary}</strong>
                </div>
              </div>

              <div className="topbar-controls">
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
                <button className="secondary-button control-button" onClick={handleManualRefresh}>
                  Refresh
                </button>
              </div>
            </section>
          </div>
        </div>

        <div className="auth-strip">
          <AuthCard
            title="OpenAI ChatGPT"
            summary={auth?.openAi.summary ?? 'Loading OpenAI session'}
            tone={auth?.openAi.authenticated ? 'good' : 'warn'}
            meta={[
              auth?.openAi.email ?? 'email unavailable',
              auth?.openAi.plan_type ?? 'plan unavailable',
            ]}
            actionLabel="Login"
            onAction={handleOpenAiLogin}
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

        {bannerError ? (
          <div className="banner-error" role="alert">
            {bannerError}
          </div>
        ) : null}
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
        </aside>

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
            <HealthView snapshot={snapshot} auth={auth} />
          ) : null}
        </main>

        <aside className="timeline-pane">
          <section className="panel timeline-panel">
            <div className="panel-header">
              <div>
                <p className="eyebrow">Live timeline</p>
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
                  body="Connect to the runtime websocket and event updates will accumulate here."
                />
              ) : (
                timeline.map((event, index) => (
                  <article className="timeline-item" key={`${event.timestamp}-${index}`}>
                    <div className="timeline-meta">
                      <strong>{event.event_type}</strong>
                      <span>{formatTimestamp(event.timestamp)}</span>
                    </div>
                    <pre>{prettyJson(event.payload)}</pre>
                  </article>
                ))
              )}
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}

interface RunsViewProps {
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

function RunsView(props: RunsViewProps) {
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

interface SessionsViewProps {
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

function SessionsView(props: SessionsViewProps) {
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
          <StatusPill
            label="Count"
            tone="neutral"
            value={`${sessions.length} loaded`}
          />
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
            <PrioritySelect
              value={createPriority}
              onChange={onCreatePriorityChange}
            />
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
              body="Start a session to get a retained coordinator, turn history, and restart-aware lineage."
            />
          ) : (
            sessions.map((session) => (
              <button
                key={session.session_id}
                className={`list-row ${session.session_id === selectedSessionId ? 'selected' : ''}`}
                onClick={() => onSelect(session.session_id)}
              >
                <div>
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
            <KeyValueGrid
              rows={[
                ['Session ID', selectedSessionSummary.session_id],
                ['Status', selectedSessionSummary.status],
                ['Coordinator', selectedSessionSummary.coordinator_agent_id],
                ['Provider', selectedSessionSummary.provider_kind],
                ['Model', selectedSessionSummary.model_id],
                ['Turns', String(selectedSessionSummary.turn_count)],
                [
                  'Active workflow',
                  selectedSessionSummary.active_workflow_id ?? 'none',
                ],
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
                  onChange={(event) =>
                    onContinueMessageChange(event.target.value)
                  }
                />
              </label>
              <div className="compose-actions">
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
                  {busy === 'Continuing session'
                    ? 'Sending'
                    : 'Continue session'}
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
              <h3>Retained result</h3>
              <pre>{prettyJson(sessionDetail.last_assistant_result)}</pre>
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
                    <pre>{prettyJson(turn.assistant_result ?? turn.resume_provenance)}</pre>
                  </article>
                ))}
              </div>
            </section>
          </div>
        ) : (
          <EmptyState
            title="Select a session"
            body="The session detail pane exposes retained result state, resume provenance, and turn lineage."
          />
        )}
      </section>
    </div>
  );
}

interface AgentsViewProps {
  agents: AgentSummary[];
  selectedAgentId: string | null;
  onSelect: (agentId: string) => void;
  selectedAgentSummary?: AgentSummary;
  agentDetail: AgentDetail | null;
}

function AgentsView(props: AgentsViewProps) {
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

interface HealthViewProps {
  snapshot: DashboardSnapshot | null;
  auth: AuthSnapshot | null;
}

function HealthView({ snapshot, auth }: HealthViewProps) {
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

interface StatusPillProps {
  label: string;
  value: string;
  tone: 'good' | 'warn' | 'bad' | 'neutral';
  testId?: string;
}

function StatusPill({ label, value, tone, testId }: StatusPillProps) {
  return (
    <div className={`status-pill ${tone}`} data-testid={testId}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

interface AuthCardProps {
  title: string;
  summary: string;
  tone: 'good' | 'warn' | 'bad' | 'neutral';
  meta: string[];
  actionLabel?: string;
  onAction?: () => void;
  disabled?: boolean;
}

function AuthCard(props: AuthCardProps) {
  const { title, summary, tone, meta, actionLabel, onAction, disabled } = props;
  const metaValues = Array.from(
    new Set(meta.map((value) => value.trim()).filter((value) => value.length > 0)),
  );

  return (
    <section className={`auth-card ${tone}`}>
      <div>
        <p className="eyebrow">{title}</p>
        <p className="auth-summary">{summary}</p>
      </div>
      <div className="auth-meta">
        {metaValues.map((value) => (
          <span key={value}>{value}</span>
        ))}
      </div>
      {actionLabel && onAction ? (
        <button
          className="secondary-button auth-action"
          onClick={onAction}
          disabled={disabled}
        >
          {actionLabel}
        </button>
      ) : null}
    </section>
  );
}

interface EmptyStateProps {
  title: string;
  body: string;
}

function EmptyState({ title, body }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <strong>{title}</strong>
      <p>{body}</p>
    </div>
  );
}

interface KeyValueGridProps {
  rows: Array<[string, string]>;
}

function KeyValueGrid({ rows }: KeyValueGridProps) {
  return (
    <dl className="key-value-grid">
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function PrioritySelect(props: {
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="field field-inline">
      <span>Priority</span>
      <select
        value={props.value}
        onChange={(event) => props.onChange(event.target.value)}
      >
        <option value="urgent">urgent</option>
        <option value="high">high</option>
        <option value="normal">normal</option>
        <option value="low">low</option>
        <option value="background">background</option>
      </select>
    </label>
  );
}

function PreviewCard({ preview }: { preview?: ResultPreview | null }) {
  if (!preview) {
    return (
      <section className="subpanel">
        <h3>Result preview</h3>
        <p>No shared result preview is available for this workflow yet.</p>
      </section>
    );
  }

  return (
    <section className="subpanel">
      <h3>Result preview</h3>
      <KeyValueGrid
        rows={[
          ['Workflow', preview.workflow_id],
          ['Proof outcome', preview.proof_outcome],
          ['Payload', preview.payload_location],
        ]}
      />
      <p className="preview-copy">{preview.preview_text ?? 'No preview text recorded.'}</p>
      <pre>{prettyJson(preview.provenance_lines)}</pre>
    </section>
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
      return `${snapshot?.runs.length ?? 0} root workflows`;
    case 'sessions':
      return `${snapshot?.sessions.length ?? 0} retained sessions`;
    case 'agents':
      return `${snapshot?.agents.length ?? 0} registry rows`;
    case 'health':
      return snapshot?.runtimeSummary ?? 'runtime probes';
    default:
      return '';
  }
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

function isRuntimeStartupPending(localRuntime: LocalRuntimeSnapshot | null): boolean {
  if (!localRuntime) {
    return false;
  }

  return (
    localRuntime.state === 'checking' ||
    localRuntime.state === 'starting_dependencies' ||
    localRuntime.state === 'starting_runtime'
  );
}

function formatTimestamp(value?: string | null): string {
  if (!value) {
    return 'not available';
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }

  return parsed.toLocaleString();
}

function prettyJson(value: unknown): string {
  if (value === undefined) {
    return 'undefined';
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function readMetric(
  source: Record<string, unknown> | null | undefined,
  key: string,
): string {
  if (!source || !(key in source)) {
    return 'n/a';
  }

  const value = source[key];
  return typeof value === 'string' || typeof value === 'number'
    ? String(value)
    : prettyJson(value);
}

function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

export default App;
