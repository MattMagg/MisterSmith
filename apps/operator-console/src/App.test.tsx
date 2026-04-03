import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from './App';
import type {
  AgentDetail,
  AgentSummary,
  AuthSnapshot,
  DashboardSnapshot,
  RunSummary,
  SessionInspectResponse,
  SessionSummary,
  StoredSettings,
  TimelineConnectionState,
  TimelineEvent,
} from './types';
import type { OperatorConsoleServices } from './services';

function createSettings(): StoredSettings {
  return {
    runtimeBaseUrl: 'http://127.0.0.1:8080',
    natsMonitorUrl: 'http://127.0.0.1:8222',
    reconnectEnabled: true,
    activeTab: 'runs',
  };
}

function createSnapshot(
  overrides: Partial<DashboardSnapshot> = {},
): DashboardSnapshot {
  const run: RunSummary = {
    task_id: 'run-1',
    status: 'completed',
    priority: 1,
    description: 'Inspect queue health',
    created_at: '2026-03-21T10:00:00Z',
    started_at: '2026-03-21T10:00:10Z',
    completed_at: '2026-03-21T10:01:00Z',
    session_id: null,
    turn_index: null,
    proof_outcome: 'Live',
    result_preview: null,
  };
  const sessionSummary: SessionSummary = {
    session_id: 'session-1',
    status: 'Active',
    coordinator_agent_id: 'agent-1',
    provider_kind: 'openai',
    model_id: 'gpt-5.4',
    active_workflow_id: 'run-2',
    last_completed_workflow_id: 'run-1',
    turn_count: 2,
    updated_at: '2026-03-21T10:02:00Z',
    ended_at: null,
    last_preview: 'Operator summary',
  };
  const sessionDetail: SessionInspectResponse = {
    session_id: 'session-1',
    status: 'Active',
    coordinator_agent_id: 'agent-1',
    provider_kind: 'openai',
    model_id: 'gpt-5.4',
    active_workflow_id: 'run-2',
    last_completed_workflow_id: 'run-1',
    turn_count: 2,
    last_assistant_result: null,
    turns: [
      {
        turn_index: 1,
        workflow_id: 'run-1',
        status: 'completed',
        user_message: 'Give me a summary.',
        assistant_result: null,
        resume_provenance: null,
      },
    ],
    ended_at: null,
  };
  const agentSummary: AgentSummary = {
    agent_id: 'agent-1',
    agent_type: 'Coordinator',
    availability: 'Idle',
    name: 'Coordinator 1',
    status: 'idle',
    last_heartbeat: '2026-03-21T10:03:00Z',
  };
  const agentDetail: AgentDetail = {
    ...agentSummary,
    metadata: { queue_depth: 0 },
  };

  return {
    localRuntime: {
      state: 'managed_ready',
      summary: 'Managed local runtime ready',
      managed_by_app: true,
      dependencies_managed: true,
      runtime_url: 'http://127.0.0.1:8080',
      database_target: 'postgres://127.0.0.1:5432/mistersmith',
      nats_target: 'nats://127.0.0.1:4222',
      last_error: null,
      last_log_line: 'runtime ready',
    },
    runtimeReachable: true,
    runtimeSummary: 'Runtime healthy',
    probes: {
      live: true,
      ready: true,
      health: {
        status: 'healthy',
        components: [{ name: 'http_server', status: 'healthy' }],
      },
      config: {
        version: '0.1.0',
        config: { transport: { http: { bind_address: '0.0.0.0:8080' } } },
      },
    },
    runs: [run],
    selectedRunId: run.task_id,
    runDetail: {
      task_id: run.task_id,
      status: run.status,
      result: {
        workflow_id: run.task_id,
        status: run.status,
        proof_outcome: 'graph_formed_and_completed',
        orchestration_quality: null,
        runtime_truth: {
          evidence_class: 'placeholder_or_simulated_step_completion',
          proof_boundary: {
            graph_execution: 'workflow graph executed successfully',
            semantic_completion: 'semantic completion not yet proven',
            grounded_tool_execution: 'grounded tool execution: none/minimal',
            task_proof: 'result is orchestration proof, not substantive task proof',
          },
          run_trace: {
            trace_root_id: run.task_id,
            workflow_id: run.task_id,
            graph_id: 'graph-7',
            branch_id: 'branch-7',
            node_id: null,
            relationships: ['graph', 'branch', 'tool_boundary', 'supervision'],
          },
          grounded_evidence: [],
        },
        supervision_evidence: {
          target_scope: {
            kind: 'branch',
            branch_id: 'branch-7',
          },
          decision_basis: 'fingerprint_reinforced',
          proof_boundary: 'deterministic-only until a real rerun is captured',
          fingerprint_ref: {
            fingerprint_id: 'fp-1',
            fingerprint_key: 'executor:branch-7',
            confidence: 0.83,
            expires_at: '2026-03-21T11:00:00Z',
          },
          repair_lineage_ref: {
            source: 'packet-020',
            checkpoint_ref: 'checkpoint-retry',
          },
          profile_snapshot: {
            health_state: 'degraded',
            updated_at: '2026-03-21T10:00:45Z',
          },
          intervention_record: {
            rationale: 'kept retry bounded to the preserved branch checkpoint',
            emitted_at: '2026-03-21T10:00:50Z',
          },
        },
        step_policy: {
          difficulty_assessment: {
            workflow_id: run.task_id,
            step_id: 'planner.step.2',
            difficulty_bucket: 'high',
            confidence_label: 'deterministic',
            reason_codes: ['weak_current_evidence', 'unstable_recent_step_history'],
          },
          budget_pressure: {
            workflow_id: run.task_id,
            step_id: 'planner.step.2',
            pressure_level: 'softcap',
            pressure_source: 'runtime.task_path',
            policy_hint: 'prefer_local_correction_before_escalation',
          },
          policy_decision: {
            workflow_id: run.task_id,
            step_id: 'planner.step.2',
            chosen_action: 'downgrade',
            action_reason: 'high_difficulty_plus_softcap_budget_pressure',
            difficulty_ref: 'assessment:planner.step.2',
            budget_ref: 'budget:planner.step.2',
            repair_lineage_ref: 'packet-020:last-stable-checkpoint',
            requires_operator_attention: true,
          },
          input_refs: {
            latest_step_evaluation: 'packet-020:planner.step.2',
            latest_step_routing: 'step-routing:planner.step.2',
            supervision_evidence: 'packet-021:planner.step.2',
            runtime_truth: 'packet-023:placeholder_or_simulated_step_completion',
            boundary_evidence: 'packet-024:clean_quarantine_boundary',
          },
          proof_boundary_ref: {
            owner_packet: '023',
            task_proof: 'result is orchestration proof, not substantive task proof',
          },
          display_note:
            'placeholder orchestration proof only; local correction preferred before escalation',
        },
        result: { ok: true },
      },
    },
    sessions: [sessionSummary],
    selectedSessionId: sessionSummary.session_id,
    sessionDetail,
    agents: [agentSummary],
    selectedAgentId: agentSummary.agent_id,
    agentDetail,
    nats: {
      available: true,
      degraded: false,
      summary: 'NATS monitor healthy',
      varz: { server_id: 'srv-1' },
      connz: { num_connections: 2 },
      jsz: { streams: 1, consumers: 2 },
      errors: [],
    },
    errors: [],
    ...overrides,
  };
}

function createAuthSnapshot(): AuthSnapshot {
  return {
    openAi: {
      authenticated: true,
      account_type: 'chatgpt',
      email: 'operator@example.com',
      plan_type: 'pro',
      requires_openai_auth: false,
      summary: 'ChatGPT session available',
    },
    claude: {
      authenticated: true,
      expired: false,
      source: 'keychain',
      masked_token: 'claude-***',
      summary: 'Claude credentials loaded',
    },
    errors: [],
  };
}

function createServices(options?: {
  dashboard?: DashboardSnapshot;
  auth?: AuthSnapshot;
  onConnectTimeline?: (
    onEvent: (event: TimelineEvent) => void,
    onStateChange: (state: TimelineConnectionState) => void,
  ) => void;
  onCreateTask?: () => void;
  onCreateSession?: () => void;
  onContinueSession?: () => void;
  onEndSession?: () => void;
}): OperatorConsoleServices {
  const dashboard = options?.dashboard ?? createSnapshot();
  const auth = options?.auth ?? createAuthSnapshot();

  return {
    loadSettings: () => createSettings(),
    saveSettings: () => undefined,
    fetchDashboard: async () =>
      dashboard,
    fetchAuthSnapshot: async () => auth,
    loginOpenAiChatGpt: async () => auth,
    connectTimeline: async (runtimeBaseUrl, onEvent, onStateChange) => {
      void runtimeBaseUrl;
      onStateChange('connected');
      options?.onConnectTimeline?.(onEvent, onStateChange);
      return () => undefined;
    },
    createTask: async () => {
      options?.onCreateTask?.();
      return {
        task_id: 'run-new',
        assigned_agent_id: 'agent-1',
        status: 'accepted',
      };
    },
    createSession: async () => {
      options?.onCreateSession?.();
      return {
        session_id: 'session-new',
        workflow_id: 'run-new',
        coordinator_agent_id: 'agent-1',
        turn_index: 1,
        status: 'accepted',
      };
    },
    continueSession: async () => {
      options?.onContinueSession?.();
      return {
        session_id: 'session-1',
        workflow_id: 'run-3',
        coordinator_agent_id: 'agent-1',
        turn_index: 3,
        status: 'accepted',
      };
    },
    endSession: async () => {
      options?.onEndSession?.();
      return {
        session_id: 'session-1',
        status: 'Ended',
        ended_at: '2026-03-21T10:10:00Z',
      };
    },
  };
}

describe('App', () => {
  it('renders disconnected runtime state', async () => {
    const onConnectTimeline = vi.fn();
    const services = createServices({
      dashboard: createSnapshot({
        localRuntime: {
          state: 'starting_dependencies',
          summary: 'Starting local postgres and nats',
          managed_by_app: true,
          dependencies_managed: false,
          runtime_url: 'http://127.0.0.1:8080',
          database_target: 'postgres://127.0.0.1:5432/mistersmith',
          nats_target: 'nats://127.0.0.1:4222',
          last_error: null,
          last_log_line: null,
        },
        runtimeReachable: false,
        runtimeSummary: 'Starting local postgres and nats',
        runs: [],
        selectedRunId: null,
        runDetail: null,
        sessions: [],
        selectedSessionId: null,
        sessionDetail: null,
        agents: [],
        selectedAgentId: null,
        agentDetail: null,
        nats: {
          available: false,
          degraded: true,
          summary: 'NATS monitor unavailable',
          errors: ['NATS /varz: 503 Service Unavailable'],
        },
      }),
      onConnectTimeline,
    });

    render(<App services={services} initialSettings={createSettings()} />);

    await waitFor(() =>
      expect(screen.getByTestId('runtime-status')).toHaveTextContent(
        'Starting local postgres and nats',
      ),
    );
    expect(onConnectTimeline).not.toHaveBeenCalled();
    expect(screen.getAllByText('waiting on runtime').length).toBeGreaterThan(0);
    expect(screen.getByText('Launcher')).toBeInTheDocument();
    expect(screen.getByText('No runs available')).toBeInTheDocument();
  });

  it('submits a task from the runs view', async () => {
    const onCreateTask = vi.fn();
    const user = userEvent.setup();

    render(
      <App
        services={createServices({ onCreateTask })}
        initialSettings={createSettings()}
      />,
    );

    await screen.findByText('Root workflow queue');
    await user.type(
      screen.getByPlaceholderText('Ask Mister Smith to do something concrete.'),
      'Investigate operator websocket lag',
    );
    await user.click(screen.getByRole('button', { name: 'Submit task' }));

    await waitFor(() => expect(onCreateTask).toHaveBeenCalledTimes(1));
  });

  it('renders bounded predictive supervision from the selected run detail payload', async () => {
    render(<App services={createServices()} initialSettings={createSettings()} />);

    await screen.findByText('Runtime truth');
    expect(
      screen.getByText('workflow graph executed successfully'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('result is orchestration proof, not substantive task proof'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('graph, branch, tool_boundary, supervision'),
    ).toBeInTheDocument();

    await screen.findByText('Predictive supervision');
    expect(screen.getByText('branch: branch-7')).toBeInTheDocument();
    expect(screen.getByText('fingerprint_reinforced')).toBeInTheDocument();
    expect(screen.getByText('executor:branch-7 (83%)')).toBeInTheDocument();
    expect(screen.getByText('packet-020 via checkpoint-retry')).toBeInTheDocument();
    expect(
      screen.getByText('deterministic-only until a real rerun is captured'),
    ).toBeInTheDocument();
  });

  it('renders the packet-owned step policy summary in the selected run detail', async () => {
    render(<App services={createServices()} initialSettings={createSettings()} />);

    await screen.findByText('Step policy');
    expect(screen.getByText('planner.step.2')).toBeInTheDocument();
    expect(screen.getByText('downgrade')).toBeInTheDocument();
    expect(
      screen.getByText('softcap via runtime.task_path (prefer_local_correction_before_escalation)'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('packet-023:placeholder_or_simulated_step_completion'),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        'placeholder orchestration proof only; local correction preferred before escalation',
      ),
    ).toBeInTheDocument();
  });

  it('creates, continues, and ends a session from the sessions view', async () => {
    const onCreateSession = vi.fn();
    const onContinueSession = vi.fn();
    const onEndSession = vi.fn();
    const user = userEvent.setup();

    render(
      <App
        services={createServices({
          onCreateSession,
          onContinueSession,
          onEndSession,
        })}
        initialSettings={{ ...createSettings(), activeTab: 'sessions' }}
      />,
    );

    await screen.findByText('Persistent conversations');

    await user.type(
      screen.getByPlaceholderText(
        'Seed the first turn for a retained conversation.',
      ),
      'Start a retained operator session.',
    );
    await user.click(screen.getByRole('button', { name: 'Create session' }));
    await waitFor(() => expect(onCreateSession).toHaveBeenCalledTimes(1));

    await user.type(
      screen.getByPlaceholderText('Add the next prompt to the selected session.'),
      'Continue the thread with the next operator message.',
    );
    await user.click(screen.getByRole('button', { name: 'Continue session' }));
    await waitFor(() => expect(onContinueSession).toHaveBeenCalledTimes(1));

    await user.click(screen.getByRole('button', { name: 'End session' }));
    await waitFor(() => expect(onEndSession).toHaveBeenCalledTimes(1));
  });

  it('appends timeline events from the websocket connector', async () => {
    const services = createServices({
      onConnectTimeline: (onEvent) => {
        onEvent({
          event_type: 'workflow.completed',
          payload: { workflow_id: 'run-1' },
          timestamp: '2026-03-21T10:04:00Z',
        });
      },
    });

    render(<App services={services} initialSettings={createSettings()} />);

    expect(await screen.findByText('workflow.completed')).toBeInTheDocument();
    expect(screen.getByTestId('timeline-list')).toHaveTextContent('run-1');
  });
});
