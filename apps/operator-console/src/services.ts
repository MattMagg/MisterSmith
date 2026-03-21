import { loadStoredSettings, normalizeUrl, saveStoredSettings } from './settings';
import type {
  AgentDetail,
  AgentSummary,
  AuthSnapshot,
  ClaudeSubscriptionStatusPayload,
  ConfigResponse,
  CreateTaskResponse,
  DashboardSelection,
  DashboardSnapshot,
  EndSessionResponse,
  HealthResponse,
  NatsMonitorSnapshot,
  OpenAiChatGptStatusPayload,
  RunSummary,
  SessionInspectResponse,
  SessionSummary,
  SessionTurnAcceptedResponse,
  StoredSettings,
  TaskInspectResponse,
  TimelineConnectionState,
  TimelineEvent,
} from './types';

export interface OperatorConsoleServices {
  loadSettings(): StoredSettings;
  saveSettings(settings: StoredSettings): void;
  fetchDashboard(
    settings: StoredSettings,
    selection: DashboardSelection,
  ): Promise<DashboardSnapshot>;
  fetchAuthSnapshot(): Promise<AuthSnapshot>;
  loginOpenAiChatGpt(): Promise<AuthSnapshot>;
  connectTimeline(
    runtimeBaseUrl: string,
    onEvent: (event: TimelineEvent) => void,
    onStateChange: (state: TimelineConnectionState) => void,
  ): Promise<() => void | Promise<void>>;
  createTask(
    runtimeBaseUrl: string,
    payload: { description: string; priority: string },
  ): Promise<CreateTaskResponse>;
  createSession(
    runtimeBaseUrl: string,
    payload: { message: string; priority: string },
  ): Promise<SessionTurnAcceptedResponse>;
  continueSession(
    runtimeBaseUrl: string,
    payload: { sessionId: string; message: string; priority: string },
  ): Promise<SessionTurnAcceptedResponse>;
  endSession(
    runtimeBaseUrl: string,
    sessionId: string,
  ): Promise<EndSessionResponse>;
}

export const defaultOperatorConsoleServices: OperatorConsoleServices = {
  loadSettings: loadStoredSettings,
  saveSettings: saveStoredSettings,
  fetchDashboard,
  fetchAuthSnapshot,
  loginOpenAiChatGpt,
  connectTimeline,
  createTask,
  createSession,
  continueSession,
  endSession,
};

async function fetchDashboard(
  settings: StoredSettings,
  selection: DashboardSelection,
): Promise<DashboardSnapshot> {
  const runtimeBaseUrl = normalizeUrl(settings.runtimeBaseUrl, 'http://127.0.0.1:8080');
  const errors: string[] = [];

  const runtimeResults = await Promise.allSettled([
    requestJson<HealthResponse>(buildUrl(runtimeBaseUrl, '/api/v1/health')),
    requestJson<ConfigResponse>(buildUrl(runtimeBaseUrl, '/api/v1/config')),
    requestStatusOk(buildUrl(runtimeBaseUrl, '/health/live')),
    requestStatusOk(buildUrl(runtimeBaseUrl, '/health/ready')),
    requestJson<RunSummary[]>(buildUrl(runtimeBaseUrl, '/api/v1/tasks')),
    requestJson<SessionSummary[]>(buildUrl(runtimeBaseUrl, '/api/v1/sessions')),
    requestJson<AgentSummary[]>(buildUrl(runtimeBaseUrl, '/api/v1/agents')),
  ]);

  const [
    healthResult,
    configResult,
    liveResult,
    readyResult,
    runsResult,
    sessionsResult,
    agentsResult,
  ] = runtimeResults;

  const runtimeReachable = runtimeResults.some(
    (result) => result.status === 'fulfilled',
  );
  const health = unwrapSettled(healthResult, errors, 'runtime health');
  const config = unwrapSettled(configResult, errors, 'runtime config');
  const live = unwrapSettled(liveResult, errors, 'runtime liveness') ?? false;
  const ready =
    unwrapSettled(readyResult, errors, 'runtime readiness') ?? false;
  const runs = unwrapSettled(runsResult, errors, 'runs') ?? [];
  const sessions = unwrapSettled(sessionsResult, errors, 'sessions') ?? [];
  const agents = unwrapSettled(agentsResult, errors, 'agents') ?? [];

  const selectedRunId = resolveSelection(selection.runId, runs, (run) => run.task_id);
  const selectedSessionId = resolveSelection(
    selection.sessionId,
    sessions,
    (session) => session.session_id,
  );
  const selectedAgentId = resolveSelection(
    selection.agentId,
    agents,
    (agent) => agent.agent_id,
  );

  const detailResults = await Promise.allSettled([
    selectedRunId
      ? requestJson<TaskInspectResponse>(
          buildUrl(runtimeBaseUrl, `/api/v1/tasks/${selectedRunId}`),
        )
      : Promise.resolve(null),
    selectedSessionId
      ? requestJson<SessionInspectResponse>(
          buildUrl(runtimeBaseUrl, `/api/v1/sessions/${selectedSessionId}`),
        )
      : Promise.resolve(null),
    selectedAgentId
      ? requestJson<AgentDetail>(
          buildUrl(runtimeBaseUrl, `/api/v1/agents/${selectedAgentId}`),
        )
      : Promise.resolve(null),
  ]);

  const runDetail = unwrapSettled(detailResults[0], errors, 'run detail');
  const sessionDetail = unwrapSettled(detailResults[1], errors, 'session detail');
  const agentDetail = unwrapSettled(detailResults[2], errors, 'agent detail');
  const nats = await fetchNatsSnapshot(settings.natsMonitorUrl);

  if (nats.errors.length > 0) {
    errors.push(...nats.errors);
  }

  return {
    runtimeReachable,
    runtimeSummary: summarizeRuntime(runtimeReachable, health, ready),
    probes: {
      health,
      config,
      live,
      ready,
    },
    runs,
    selectedRunId,
    runDetail,
    sessions,
    selectedSessionId,
    sessionDetail,
    agents,
    selectedAgentId,
    agentDetail,
    nats,
    errors,
  };
}

async function fetchAuthSnapshot(): Promise<AuthSnapshot> {
  const errors: string[] = [];
  const [openAiResult, claudeResult] = await Promise.allSettled([
    invokeCommand<OpenAiChatGptStatusPayload>('openai_chatgpt_status'),
    invokeCommand<ClaudeSubscriptionStatusPayload>('claude_subscription_status'),
  ]);

  return {
    openAi: unwrapSettled(
      openAiResult,
      errors,
      'OpenAI auth status',
      unavailableOpenAiStatus(),
    ),
    claude: unwrapSettled(
      claudeResult,
      errors,
      'Claude auth status',
      unavailableClaudeStatus(),
    ),
    errors,
  };
}

async function loginOpenAiChatGpt(): Promise<AuthSnapshot> {
  const errors: string[] = [];
  const [openAiResult, claudeResult] = await Promise.allSettled([
    invokeCommand<OpenAiChatGptStatusPayload>('login_openai_chatgpt'),
    invokeCommand<ClaudeSubscriptionStatusPayload>('claude_subscription_status'),
  ]);

  return {
    openAi: unwrapSettled(
      openAiResult,
      errors,
      'OpenAI login',
      unavailableOpenAiStatus(),
    ),
    claude: unwrapSettled(
      claudeResult,
      errors,
      'Claude auth status',
      unavailableClaudeStatus(),
    ),
    errors,
  };
}

async function connectTimeline(
  runtimeBaseUrl: string,
  onEvent: (event: TimelineEvent) => void,
  onStateChange: (state: TimelineConnectionState) => void,
): Promise<() => void | Promise<void>> {
  const url = buildWebSocketUrl(runtimeBaseUrl, '/api/v1/events/ws');
  onStateChange('connecting');

  try {
    const websocketModule = await import('@tauri-apps/plugin-websocket');
    const socket = await websocketModule.default.connect(url);
    const unsubscribe = socket.addListener((message) => {
      if (message.type === 'Text') {
        const parsed = parseTimelineMessage(message.data);
        if (parsed) {
          onEvent(parsed);
        }
      }
      if (message.type === 'Close') {
        onStateChange('disconnected');
      }
    });

    onStateChange('connected');
    return async () => {
      unsubscribe();
      await socket.disconnect();
    };
  } catch (tauriError) {
    if (typeof window.WebSocket === 'undefined') {
      onStateChange('disconnected');
      throw tauriError;
    }

    return new Promise((resolve, reject) => {
      const socket = new window.WebSocket(url);

      socket.addEventListener('open', () => {
        onStateChange('connected');
        resolve(() => {
          socket.close();
        });
      });

      socket.addEventListener('message', (event) => {
        const parsed = parseTimelineMessage(String(event.data));
        if (parsed) {
          onEvent(parsed);
        }
      });

      socket.addEventListener('close', () => {
        onStateChange('disconnected');
      });

      socket.addEventListener('error', () => {
        socket.close();
        onStateChange('disconnected');
        reject(new Error('websocket connection failed'));
      });
    });
  }
}

async function createTask(
  runtimeBaseUrl: string,
  payload: { description: string; priority: string },
): Promise<CreateTaskResponse> {
  return requestJson<CreateTaskResponse>(buildUrl(runtimeBaseUrl, '/api/v1/tasks'), {
    method: 'POST',
    body: JSON.stringify(payload),
    headers: {
      'Content-Type': 'application/json',
    },
  });
}

async function createSession(
  runtimeBaseUrl: string,
  payload: { message: string; priority: string },
): Promise<SessionTurnAcceptedResponse> {
  return requestJson<SessionTurnAcceptedResponse>(
    buildUrl(runtimeBaseUrl, '/api/v1/sessions'),
    {
      method: 'POST',
      body: JSON.stringify(payload),
      headers: {
        'Content-Type': 'application/json',
      },
    },
  );
}

async function continueSession(
  runtimeBaseUrl: string,
  payload: { sessionId: string; message: string; priority: string },
): Promise<SessionTurnAcceptedResponse> {
  return requestJson<SessionTurnAcceptedResponse>(
    buildUrl(runtimeBaseUrl, `/api/v1/sessions/${payload.sessionId}/turns`),
    {
      method: 'POST',
      body: JSON.stringify({
        message: payload.message,
        priority: payload.priority,
      }),
      headers: {
        'Content-Type': 'application/json',
      },
    },
  );
}

async function endSession(
  runtimeBaseUrl: string,
  sessionId: string,
): Promise<EndSessionResponse> {
  return requestJson<EndSessionResponse>(
    buildUrl(runtimeBaseUrl, `/api/v1/sessions/${sessionId}/end`),
    {
      method: 'POST',
    },
  );
}

async function fetchNatsSnapshot(rawBaseUrl: string): Promise<NatsMonitorSnapshot> {
  const baseUrl = normalizeUrl(rawBaseUrl, 'http://127.0.0.1:8222');
  const errors: string[] = [];

  const [varzResult, connzResult, jszResult] = await Promise.allSettled([
    requestJson<Record<string, unknown>>(buildUrl(baseUrl, '/varz')),
    requestJson<Record<string, unknown>>(buildUrl(baseUrl, '/connz')),
    requestJson<Record<string, unknown>>(buildUrl(baseUrl, '/jsz')),
  ]);

  const varz = unwrapSettled(varzResult, errors, 'NATS /varz');
  const connz = unwrapSettled(connzResult, errors, 'NATS /connz');
  const jsz = unwrapSettled(jszResult, errors, 'NATS /jsz');
  const available = Boolean(varz || connz || jsz);
  const degraded = available
    ? Boolean(errors.length)
    : true;

  return {
    available,
    degraded,
    summary: available
      ? degraded
        ? 'NATS monitor partially reachable'
        : 'NATS monitor healthy'
      : 'NATS monitor unavailable',
    varz,
    connz,
    jsz,
    errors,
  };
}

async function requestJson<T>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  const response = await request(url, init);
  const text = await response.text();
  const payload = parseTextPayload(text);

  if (!response.ok) {
    throw new Error(renderHttpError(response, payload));
  }

  return payload as T;
}

async function requestStatusOk(url: string): Promise<boolean> {
  const response = await request(url, { method: 'GET' });
  return response.ok;
}

async function request(url: string, init?: RequestInit): Promise<Response> {
  try {
    const httpModule = await import('@tauri-apps/plugin-http');
    return await httpModule.fetch(url, init);
  } catch {
    return fetch(url, init);
  }
}

async function invokeCommand<T>(command: string): Promise<T> {
  const coreModule = await import('@tauri-apps/api/core');
  return coreModule.invoke<T>(command);
}

function unwrapSettled<T>(
  result: PromiseSettledResult<T>,
  errors: string[],
  label: string,
  fallback?: T,
): T {
  if (result.status === 'fulfilled') {
    return result.value;
  }

  errors.push(`${label}: ${renderUnknownError(result.reason)}`);
  return fallback as T;
}

function resolveSelection<T>(
  current: string | null,
  rows: T[],
  pickId: (row: T) => string,
): string | null {
  if (current && rows.some((row) => pickId(row) === current)) {
    return current;
  }

  return rows[0] ? pickId(rows[0]) : null;
}

function summarizeRuntime(
  runtimeReachable: boolean,
  health: HealthResponse | null,
  ready: boolean,
): string {
  if (!runtimeReachable) {
    return 'Runtime offline';
  }
  if (!health) {
    return ready ? 'Runtime reachable' : 'Runtime reachable, probes pending';
  }
  if (health.status === 'healthy' && ready) {
    return 'Runtime healthy';
  }
  return `Runtime ${health.status}`;
}

function parseTextPayload(text: string): unknown {
  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

function renderHttpError(response: Response, payload: unknown): string {
  if (typeof payload === 'object' && payload !== null && 'message' in payload) {
    const message = payload.message;
    if (typeof message === 'string' && message) {
      return `${response.status} ${response.statusText}: ${message}`;
    }
  }

  if (typeof payload === 'string' && payload) {
    return `${response.status} ${response.statusText}: ${payload}`;
  }

  return `${response.status} ${response.statusText}`;
}

function renderUnknownError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function buildUrl(baseUrl: string, path: string): string {
  const normalizedBase = normalizeUrl(baseUrl, 'http://127.0.0.1:8080');
  return `${normalizedBase}${path}`;
}

function buildWebSocketUrl(baseUrl: string, path: string): string {
  const normalizedBase = normalizeUrl(baseUrl, 'http://127.0.0.1:8080');
  if (normalizedBase.startsWith('https://')) {
    return `wss://${normalizedBase.slice('https://'.length)}${path}`;
  }
  if (normalizedBase.startsWith('http://')) {
    return `ws://${normalizedBase.slice('http://'.length)}${path}`;
  }
  return `${normalizedBase}${path}`;
}

function parseTimelineMessage(payload: string): TimelineEvent | null {
  const parsed = parseTextPayload(payload);
  if (
    typeof parsed === 'object' &&
    parsed !== null &&
    'event_type' in parsed &&
    'timestamp' in parsed
  ) {
    return parsed as TimelineEvent;
  }

  return null;
}

function unavailableOpenAiStatus(): OpenAiChatGptStatusPayload {
  return {
    authenticated: false,
    account_type: null,
    email: null,
    plan_type: null,
    requires_openai_auth: true,
    summary: 'Desktop OpenAI auth commands are unavailable in this context.',
  };
}

function unavailableClaudeStatus(): ClaudeSubscriptionStatusPayload {
  return {
    authenticated: false,
    expired: false,
    source: null,
    masked_token: null,
    summary: 'Desktop Claude credential status is unavailable in this context.',
  };
}
