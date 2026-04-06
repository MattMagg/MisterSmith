//! Mister Smith — Multi-agent orchestration framework binary entry point.
//!
//! Orchestrates process lifecycle: deterministic startup, graceful shutdown,
//! signal handling, observability initialization, and cross-phase integration wiring.

mod agent_inspection;
mod auth;
mod autonomy;
mod bootstrap;
#[allow(dead_code)]
mod bridges;
mod config;
mod conversation;
mod execution;
mod observability;
mod shutdown;

use clap::{Parser, Subcommand};
use mister_smith_config::LogFormat;
use mister_smith_core::ProcessLifecycle;
use std::error::Error;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::{error, info};

/// Mister Smith — Multi-agent orchestration framework.
#[derive(Parser, Debug)]
#[command(name = "mister-smith", version, about)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Override the runtime API base URL for session-shell flows.
    #[arg(long, global = true)]
    base_url: Option<String>,

    /// Override log level (trace, debug, info, warn, error).
    #[arg(long, global = true)]
    log_level: Option<String>,

    /// Override log format (json, pretty).
    #[arg(long, global = true)]
    log_format: Option<String>,

    /// Subcommand to execute. Omit to open the session-first CLI shell.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the Mister Smith framework runtime.
    Run,
    /// Resume the most recent session or a specific session.
    Resume {
        /// Resume the most recently updated retained session.
        #[arg(long, conflicts_with = "session_id")]
        last: bool,
        /// Session UUID to resume directly.
        session_id: Option<String>,
    },
    /// Browse retained sessions and open one directly.
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    /// Create, continue, inspect, and end durable conversations.
    Conversation {
        #[command(subcommand)]
        command: ConversationCommand,
    },
    /// Inspect the operator-facing autonomy control plane.
    Autonomy {
        #[command(subcommand)]
        command: AutonomyCommand,
    },
    /// Authentication helpers for provider-backed integrations.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
}

#[derive(Subcommand, Debug)]
enum SessionsCommand {
    /// List retained sessions in recent-first order.
    List {
        /// Optional max rows to return.
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Open one retained session by id.
    Open {
        /// Session UUID to inspect or reopen.
        session_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum AutonomyCommand {
    /// Show the typed autonomy status for one workflow from the running runtime.
    Status {
        /// Workflow UUID to inspect.
        #[arg(long)]
        workflow_id: String,
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
    /// List workflow IDs that currently have typed autonomy status in the runtime.
    List {
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ConversationCommand {
    /// Create a durable session and accept the first turn.
    Start {
        /// Operator message for the first turn.
        #[arg(long)]
        message: String,
        /// Optional priority label for the first turn.
        #[arg(long)]
        priority: Option<String>,
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Continue an existing session with one more turn.
    Continue {
        /// Session UUID to continue.
        #[arg(long)]
        session_id: String,
        /// Operator message for the next turn.
        #[arg(long)]
        message: String,
        /// Optional priority label for the turn workflow.
        #[arg(long)]
        priority: Option<String>,
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Inspect one durable session.
    Inspect {
        /// Session UUID to inspect.
        #[arg(long)]
        session_id: String,
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Logically end one idle session.
    End {
        /// Session UUID to end.
        #[arg(long)]
        session_id: String,
        /// Base URL of the running local runtime. Defaults to the configured HTTP port.
        #[arg(long)]
        base_url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    /// Manage Claude subscription authentication.
    Claude {
        #[command(subcommand)]
        command: ClaudeAuthCommand,
    },
    /// Manage ChatGPT-backed OpenAI authentication.
    OpenaiChatgpt {
        #[command(subcommand)]
        command: OpenaiChatgptAuthCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ClaudeAuthCommand {
    /// Show the current Claude subscription authentication state.
    Status,
}

#[derive(Subcommand, Debug)]
enum OpenaiChatgptAuthCommand {
    /// Start the ChatGPT browser-login flow.
    Login,
    /// Show the current ChatGPT authentication state.
    Status,
}

/// Thread-safe process state tracker.
///
/// Uses `AtomicU8` for lock-free reads from health probe endpoints.
#[derive(Clone)]
pub struct ProcessStateTracker {
    state: Arc<AtomicU8>,
}

impl Default for ProcessStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessStateTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(ProcessLifecycle::Starting as u8)),
        }
    }

    pub fn set(&self, lifecycle: ProcessLifecycle) {
        self.state.store(lifecycle as u8, Ordering::SeqCst);
    }

    pub fn get(&self) -> ProcessLifecycle {
        match self.state.load(Ordering::SeqCst) {
            0 => ProcessLifecycle::Starting,
            1 => ProcessLifecycle::Ready,
            2 => ProcessLifecycle::Draining,
            3 => ProcessLifecycle::Stopped,
            _ => ProcessLifecycle::Failed,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.get() == ProcessLifecycle::Ready
    }
}

fn parse_log_format(s: &str) -> Option<LogFormat> {
    match s.to_lowercase().as_str() {
        "json" => Some(LogFormat::Json),
        "pretty" => Some(LogFormat::Pretty),
        _ => None,
    }
}

fn is_top_level_subcommand(token: &str) -> bool {
    matches!(
        token,
        "run" | "resume" | "sessions" | "conversation" | "autonomy" | "auth"
    )
}

fn is_global_option_with_inline_value(token: &str) -> bool {
    token.starts_with("--config=")
        || token.starts_with("--base-url=")
        || token.starts_with("--log-level=")
        || token.starts_with("--log-format=")
}

fn is_global_option_requiring_value(token: &str) -> bool {
    matches!(token, "-c" | "--config" | "--base-url" | "--log-level" | "--log-format")
}

fn split_prompt_words(raw_args: &[String]) -> (Vec<String>, Option<Vec<String>>) {
    if raw_args.is_empty() {
        return (Vec::new(), None);
    }

    let mut cli_args = vec![raw_args[0].clone()];
    let mut index = 1;
    while index < raw_args.len() {
        let token = &raw_args[index];
        if matches!(token.as_str(), "-h" | "--help" | "-V" | "--version") {
            return (raw_args.to_vec(), None);
        }
        if token == "--" {
            let prompt_words = raw_args[(index + 1)..].to_vec();
            return (cli_args, (!prompt_words.is_empty()).then_some(prompt_words));
        }
        if is_global_option_with_inline_value(token) {
            cli_args.push(token.clone());
            index += 1;
            continue;
        }
        if is_global_option_requiring_value(token) {
            if let Some(value) = raw_args.get(index + 1) {
                cli_args.push(token.clone());
                cli_args.push(value.clone());
                index += 2;
            } else {
                return (raw_args.to_vec(), None);
            }
            continue;
        }
        if token.starts_with('-') || is_top_level_subcommand(token) {
            return (raw_args.to_vec(), None);
        }

        return (cli_args, Some(raw_args[index..].to_vec()));
    }

    (raw_args.to_vec(), None)
}

struct LoadedCliContext {
    config: mister_smith_config::FrameworkConfig,
    base_url: String,
    config_action: String,
}

fn load_cli_context(cli: &Cli) -> Result<LoadedCliContext, Box<dyn Error>> {
    let overrides = config::CliOverrides {
        config_path: cli.config.clone(),
        log_level: cli.log_level.clone(),
        log_format: cli.log_format.as_deref().and_then(parse_log_format),
    };
    let config = config::load_framework_config(&overrides)
        .map_err(|error| format!("Failed to load configuration: {error}"))?;
    let base_url = cli
        .base_url
        .clone()
        .unwrap_or_else(|| conversation::default_base_url(&config));
    let config_action = cli
        .config
        .clone()
        .unwrap_or_else(|| "auto-discovered config".to_string());

    Ok(LoadedCliContext {
        config,
        base_url,
        config_action,
    })
}

fn join_prompt(words: &[String]) -> Option<String> {
    let joined = words.join(" ");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn read_shell_line(prompt: &str) -> Result<Option<String>, Box<dyn Error>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok(None);
    }

    Ok(Some(line.trim().to_string()))
}

fn print_shell_error(error: &dyn std::fmt::Display) {
    println!("action_blocked: {error}");
}

#[tokio::main]
async fn main() {
    let raw_args = std::env::args().collect::<Vec<_>>();
    let (cli_args, prompt_words) = split_prompt_words(&raw_args);
    let cli = Cli::parse_from(cli_args);

    let result = match (&cli.command, prompt_words.as_ref()) {
        (Some(Command::Run), _) => run_runtime(&cli).await,
        (Some(Command::Resume { last, session_id }), _) => {
            execute_resume_command(session_id.as_deref(), *last, &cli).await
        }
        (Some(Command::Sessions { command }), _) => execute_sessions_command(command, &cli).await,
        (Some(Command::Conversation { command }), _) => {
            execute_conversation_command(command, &cli).await
        }
        (Some(Command::Autonomy { command }), _) => execute_autonomy_command(command, &cli).await,
        (Some(Command::Auth { command }), _) => execute_auth_command(command).await,
        (None, Some(prompt_words)) => execute_direct_prompt(prompt_words, &cli).await,
        (None, None) => execute_default_entry(&cli).await,
    };

    if let Err(error) = result {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

async fn run_runtime(cli: &Cli) -> Result<(), Box<dyn Error>> {
    // Build CLI overrides
    let overrides = config::CliOverrides {
        config_path: cli.config.clone(),
        log_level: cli.log_level.clone(),
        log_format: cli.log_format.as_deref().and_then(parse_log_format),
    };

    // Load configuration
    let config = config::load_framework_config(&overrides)
        .map_err(|error| format!("Failed to load configuration: {error}"))?;

    // Initialize the observability pipeline (tracing, metrics, OTel)
    // Must happen before any logging.
    let otel_guard = observability::init_observability(&config.observability)
        .map_err(|error| format!("Failed to initialize observability: {error}"))?;

    // Initialize process state tracker and shutdown coordinator
    let state_tracker = ProcessStateTracker::new();
    let shutdown_coordinator = shutdown::ShutdownCoordinator::new();

    info!(version = env!("CARGO_PKG_VERSION"), "Mister Smith starting");

    // Run the deterministic bootstrap sequence
    let ctx = match bootstrap::bootstrap(&config, &state_tracker, &otel_guard).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!(error = %e, "Bootstrap failed");
            state_tracker.set(ProcessLifecycle::Failed);
            return Err(format!("Bootstrap failed: {e}").into());
        }
    };

    // Wait for termination signal
    let signal_name = shutdown::wait_for_signal().await;
    info!(signal = signal_name, "Received termination signal");

    // Begin graceful shutdown
    shutdown_coordinator.begin_graceful_shutdown();
    state_tracker.set(ProcessLifecycle::Draining);

    // Spawn forced shutdown listener (second signal → immediate exit)
    let forced_flag = shutdown_coordinator.shutdown_in_progress.clone();
    let forced_handle = tokio::spawn(async move {
        shutdown::wait_for_forced_signal(forced_flag).await;
        error!("Forced shutdown — exiting immediately");
        std::process::exit(2);
    });

    // Execute the graceful shutdown sequence
    shutdown::graceful_shutdown(ctx, &state_tracker, &config).await;

    // Flush and shut down observability providers
    observability::shutdown_observability(otel_guard);

    // Cancel the forced shutdown listener (we're done gracefully)
    forced_handle.abort();
    Ok(())
}

async fn execute_default_entry(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let context = load_cli_context(cli)?;
    let home = conversation::build_startup_home(
        &context.base_url,
        &context.config,
        context.config_action.clone(),
    )
    .await;
    println!("{}", conversation::render_startup_home(&home));

    if io::stdin().is_terminal() {
        run_home_loop(&context).await?;
    }

    Ok(())
}

async fn execute_direct_prompt(prompt_words: &[String], cli: &Cli) -> Result<(), Box<dyn Error>> {
    let context = load_cli_context(cli)?;
    let prompt = join_prompt(prompt_words)
        .ok_or_else(|| "direct prompt entry requires a non-empty prompt".to_string())?;
    start_session_and_maybe_attach(&context, &prompt).await
}

async fn execute_resume_command(
    session_id: Option<&str>,
    last: bool,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    let context = load_cli_context(cli)?;
    let resolved_session_id = if last || session_id.is_none() {
        conversation::resolve_last_session_id(&context.base_url, &context.config)
            .await
            .ok_or_else(|| "no retained session is available to resume".to_string())?
    } else {
        conversation::parse_session_id(
            session_id.expect("session id should exist when --last is not set"),
        )?
    };

    open_session_and_maybe_attach(&context, resolved_session_id, None).await
}

async fn execute_sessions_command(
    command: &SessionsCommand,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    let context = load_cli_context(cli)?;
    match command {
        SessionsCommand::List { limit } => {
            let home = conversation::build_startup_home(
                &context.base_url,
                &context.config,
                context.config_action.clone(),
            )
            .await;
            let sessions = if let Some(limit) = limit {
                home.recent_sessions.into_iter().take(*limit).collect::<Vec<_>>()
            } else {
                home.recent_sessions
            };
            println!(
                "warnings:\n{}\n{}",
                conversation::render_support_notices(&home.startup_warnings),
                conversation::render_session_list(&sessions)
            );
        }
        SessionsCommand::Open { session_id } => {
            let session_id = conversation::parse_session_id(session_id)?;
            open_session_and_maybe_attach(&context, session_id, None).await?;
        }
    }

    Ok(())
}

async fn run_home_loop(context: &LoadedCliContext) -> Result<(), Box<dyn Error>> {
    loop {
        let Some(line) = read_shell_line("home> ")? else {
            break;
        };
        if line.is_empty() {
            continue;
        }

        if matches!(line.as_str(), "quit" | "exit") {
            break;
        }
        if line == "help" {
            println!(
                "home commands:\n  new <message>\n  resume last\n  resume <session_id>\n  open <session_id>\n  sessions\n  config\n  quit\nplain text also starts a new session"
            );
            continue;
        }
        if line == "config" {
            println!(
                "config_action: {}\nbase_url: {}\nprovider: {}\nmodel: {}",
                context.config_action,
                context.base_url,
                context.config.llm.provider_kind.as_str(),
                context.config.llm.model_id
            );
            continue;
        }
        if line == "sessions" {
            let home = conversation::build_startup_home(
                &context.base_url,
                &context.config,
                context.config_action.clone(),
            )
            .await;
            println!(
                "warnings:\n{}\n{}",
                conversation::render_support_notices(&home.startup_warnings),
                conversation::render_session_list(&home.recent_sessions)
            );
            continue;
        }

        if let Some(prompt) = line.strip_prefix("new ") {
            if let Err(error) = start_session_and_maybe_attach(context, prompt.trim()).await {
                print_shell_error(error.as_ref());
            }
            continue;
        }
        if line == "resume last" {
            match conversation::resolve_last_session_id(&context.base_url, &context.config).await {
                Some(session_id) => {
                    if let Err(error) = open_session_and_maybe_attach(context, session_id, None).await
                    {
                        print_shell_error(error.as_ref());
                    }
                }
                None => print_shell_error(&"no retained session is available to resume"),
            }
            continue;
        }
        if let Some(raw) = line.strip_prefix("resume ") {
            match conversation::parse_session_id(raw.trim()) {
                Ok(session_id) => {
                    if let Err(error) = open_session_and_maybe_attach(context, session_id, None).await
                    {
                        print_shell_error(error.as_ref());
                    }
                }
                Err(error) => print_shell_error(&error),
            }
            continue;
        }
        if let Some(raw) = line.strip_prefix("open ") {
            match conversation::parse_session_id(raw.trim()) {
                Ok(session_id) => {
                    if let Err(error) = open_session_and_maybe_attach(context, session_id, None).await
                    {
                        print_shell_error(error.as_ref());
                    }
                }
                Err(error) => print_shell_error(&error),
            }
            continue;
        }

        if let Err(error) = start_session_and_maybe_attach(context, &line).await {
            print_shell_error(error.as_ref());
        }
    }

    Ok(())
}

async fn start_session_and_maybe_attach(
    context: &LoadedCliContext,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let accepted = conversation::start_session_http(&context.base_url, message, None).await?;
    println!("{}", conversation::render_turn_accepted(&accepted));

    if io::stdin().is_terminal() {
        let session_id = conversation::parse_session_id(&accepted.session_id)?;
        run_live_session_loop(context, session_id).await?;
    }

    Ok(())
}

async fn open_session_and_maybe_attach(
    context: &LoadedCliContext,
    session_id: mister_smith_core::SessionId,
    prompt: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    if let Some(message) = prompt {
        let accepted =
            conversation::continue_session_http(&context.base_url, session_id, message, None).await?;
        println!("{}", conversation::render_turn_accepted(&accepted));
    }

    let view = conversation::inspect_session_for_cli(&context.base_url, &context.config, session_id).await?;
    println!("{}", conversation::render_session(&view));

    if io::stdin().is_terminal() {
        run_live_session_loop(context, session_id).await?;
    }

    Ok(())
}

async fn run_live_session_loop(
    context: &LoadedCliContext,
    mut session_id: mister_smith_core::SessionId,
) -> Result<(), Box<dyn Error>> {
    loop {
        let Some(line) = read_shell_line("session> ")? else {
            break;
        };
        if line.is_empty() {
            continue;
        }

        if !line.starts_with('/') {
            match conversation::continue_session_http(&context.base_url, session_id, &line, None)
                .await
            {
                Ok(accepted) => {
                    println!("{}", conversation::render_turn_accepted(&accepted));
                    match conversation::inspect_session_for_cli(
                        &context.base_url,
                        &context.config,
                        session_id,
                    )
                    .await
                    {
                        Ok(view) => println!("{}", conversation::render_session(&view)),
                        Err(error) => print_shell_error(&error),
                    }
                }
                Err(error) => {
                    println!("send_failed: {error}");
                }
            }
            continue;
        }

        let mut parts = line.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let arg = parts.next().map(str::trim).filter(|value| !value.is_empty());

        match command {
            "/quit" => break,
            "/help" => {
                println!(
                    "session commands:\n  /model [model or provider:model]\n  /permissions [default|review|full]\n  /status [summary|detail]\n  /config [inline|support]\n  /mcp [connected|support_only|detached]\n  /sessions\n  /resume <session_id|last>\n  /new <message>\n  /quit\nplain text sends the next turn"
                );
            }
            "/sessions" => {
                let result = async {
                    let sessions = match conversation::list_sessions_http(&context.base_url, 20).await
                    {
                        Ok(rows) => rows,
                        Err(e) => {
                            tracing::debug!("list_sessions_http failed, falling back to direct: {}", e);
                            conversation::list_sessions_direct(20).await?
                        }
                    };
                    println!("{}", conversation::render_session_list(&sessions));
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/status" => {
                let result = async {
                    if let Some(mode) = arg {
                        let control = conversation::update_session_control_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                            mister_smith_http::server::ConversationSessionControlUpdateRequest {
                                status_view: Some(mode.to_string()),
                                ..Default::default()
                            },
                        )
                        .await?;
                        println!(
                            "status_view: {}\nselected_model: {}\npermission_mode: {}\nmcp_posture: {}",
                            control.status_view,
                            control.selected_model_id.as_deref().unwrap_or("inherit"),
                            control.permission_mode,
                            control.mcp_posture
                        );
                    }
                    let view = conversation::inspect_session_for_cli(
                        &context.base_url,
                        &context.config,
                        session_id,
                    )
                    .await?;
                    println!("{}", conversation::render_session(&view));
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/config" => {
                let result = async {
                    if let Some(posture) = arg {
                        let control = conversation::update_session_control_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                            mister_smith_http::server::ConversationSessionControlUpdateRequest {
                                config_posture: Some(posture.to_string()),
                                ..Default::default()
                            },
                        )
                        .await?;
                        println!("config_posture: {}", control.config_posture);
                    }
                    println!(
                        "config_action: {}\nbase_url: {}\nprovider: {}\nmodel: {}",
                        context.config_action,
                        context.base_url,
                        context.config.llm.provider_kind.as_str(),
                        context.config.llm.model_id
                    );
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/permissions" => {
                let result = async {
                    if let Some(mode) = arg {
                        let control = conversation::update_session_control_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                            mister_smith_http::server::ConversationSessionControlUpdateRequest {
                                permission_mode: Some(mode.to_string()),
                                ..Default::default()
                            },
                        )
                        .await?;
                        println!("permission_mode: {}", control.permission_mode);
                    } else {
                        let view = conversation::inspect_session_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                        )
                        .await?;
                        println!("permission_mode: {}", view.control_state.permission_mode);
                    }
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/mcp" => {
                let result = async {
                    if let Some(posture) = arg {
                        let control = conversation::update_session_control_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                            mister_smith_http::server::ConversationSessionControlUpdateRequest {
                                mcp_posture: Some(posture.to_string()),
                                ..Default::default()
                            },
                        )
                        .await?;
                        println!("mcp_posture: {}", control.mcp_posture);
                    } else {
                        let view = conversation::inspect_session_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                        )
                        .await?;
                        println!("mcp_posture: {}", view.control_state.mcp_posture);
                    }
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/model" => {
                let result = async {
                    if let Some(value) = arg {
                        let (selected_provider_kind, selected_model_id) = value
                            .split_once(':')
                            .map(|(provider, model)| {
                                (Some(provider.to_string()), model.to_string())
                            })
                            .unwrap_or((None, value.to_string()));
                        let control = conversation::update_session_control_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                            mister_smith_http::server::ConversationSessionControlUpdateRequest {
                                selected_provider_kind,
                                selected_model_id: Some(selected_model_id),
                                ..Default::default()
                            },
                        )
                        .await?;
                        println!(
                            "selected_provider: {}\nselected_model: {}",
                            control.selected_provider_kind.as_deref().unwrap_or("inherit"),
                            control.selected_model_id.as_deref().unwrap_or("inherit")
                        );
                    } else {
                        let view = conversation::inspect_session_for_cli(
                            &context.base_url,
                            &context.config,
                            session_id,
                        )
                        .await?;
                        println!(
                            "runtime_provider: {}\nruntime_model: {}\nselected_provider: {}\nselected_model: {}",
                            view.provider_kind,
                            view.model_id,
                            view.control_state
                                .selected_provider_kind
                                .as_deref()
                                .unwrap_or("inherit"),
                            view.control_state
                                .selected_model_id
                                .as_deref()
                                .unwrap_or("inherit")
                        );
                    }
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/resume" => {
                let Some(target) = arg else {
                    println!("usage: /resume <session_id|last>");
                    continue;
                };
                let result = async {
                    let new_session_id = if target == "last" {
                        conversation::resolve_last_session_id(&context.base_url, &context.config)
                            .await
                            .ok_or_else(|| {
                                "no retained session is available to resume".to_string()
                            })?
                    } else {
                        conversation::parse_session_id(target)?
                    };
                    session_id = new_session_id;
                    let view = conversation::inspect_session_for_cli(
                        &context.base_url,
                        &context.config,
                        session_id,
                    )
                    .await?;
                    println!("{}", conversation::render_session(&view));
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            "/new" => {
                let Some(message) = arg else {
                    println!("usage: /new <message>");
                    continue;
                };
                let result = async {
                    let accepted =
                        conversation::start_session_http(&context.base_url, message, None).await?;
                    session_id = conversation::parse_session_id(&accepted.session_id)?;
                    println!("{}", conversation::render_turn_accepted(&accepted));
                    let view = conversation::inspect_session_for_cli(
                        &context.base_url,
                        &context.config,
                        session_id,
                    )
                    .await?;
                    println!("{}", conversation::render_session(&view));
                    Ok::<(), Box<dyn Error>>(())
                }
                .await;
                if let Err(error) = result {
                    print_shell_error(error.as_ref());
                }
            }
            _ => println!("unknown command: {command}"),
        }
    }

    Ok(())
}

async fn execute_auth_command(command: &AuthCommand) -> Result<(), Box<dyn Error>> {
    match command {
        AuthCommand::Claude { command } => execute_claude_auth(command),
        AuthCommand::OpenaiChatgpt { command } => execute_openai_chatgpt_auth(command).await,
    }
}

fn execute_claude_auth(command: &ClaudeAuthCommand) -> Result<(), Box<dyn Error>> {
    match command {
        ClaudeAuthCommand::Status => match auth::claude_subscription_status() {
            Ok(creds) => {
                println!("{}", auth::render_claude_subscription_status(&creds));
            }
            Err(_) => {
                println!("{}", auth::render_claude_subscription_missing());
            }
        },
    }

    Ok(())
}

async fn execute_openai_chatgpt_auth(
    command: &OpenaiChatgptAuthCommand,
) -> Result<(), Box<dyn Error>> {
    match command {
        OpenaiChatgptAuthCommand::Login => {
            let status = auth::login_openai_chatgpt().await?;
            println!("{}", auth::render_openai_chatgpt_status(&status));
        }
        OpenaiChatgptAuthCommand::Status => {
            let status = auth::openai_chatgpt_status().await?;
            println!("{}", auth::render_openai_chatgpt_status(&status));
        }
    }

    Ok(())
}

async fn execute_autonomy_command(
    command: &AutonomyCommand,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    let overrides = config::CliOverrides {
        config_path: cli.config.clone(),
        log_level: cli.log_level.clone(),
        log_format: cli.log_format.as_deref().and_then(parse_log_format),
    };
    let config = config::load_framework_config(&overrides)
        .map_err(|error| format!("Failed to load configuration: {error}"))?;

    match command {
        AutonomyCommand::Status {
            workflow_id,
            base_url,
        } => {
            let workflow_id = autonomy::parse_workflow_id(workflow_id)?;
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| autonomy::default_base_url(&config));
            let view = autonomy::fetch_status(&base_url, workflow_id).await?;
            println!("{}", autonomy::render_status(&view));
        }
        AutonomyCommand::List { base_url } => {
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| autonomy::default_base_url(&config));
            let workflows = autonomy::fetch_workflows(&base_url).await?;
            for workflow_id in workflows.workflows {
                println!("{workflow_id}");
            }
        }
    }

    Ok(())
}

async fn execute_conversation_command(
    command: &ConversationCommand,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    let overrides = config::CliOverrides {
        config_path: cli.config.clone(),
        log_level: cli.log_level.clone(),
        log_format: cli.log_format.as_deref().and_then(parse_log_format),
    };
    let config = config::load_framework_config(&overrides)
        .map_err(|error| format!("Failed to load configuration: {error}"))?;

    match command {
        ConversationCommand::Start {
            message,
            priority,
            base_url,
        } => {
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| conversation::default_base_url(&config));
            let view =
                conversation::start_session_http(&base_url, message, priority.clone()).await?;
            println!("{}", conversation::render_turn_accepted(&view));
        }
        ConversationCommand::Continue {
            session_id,
            message,
            priority,
            base_url,
        } => {
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| conversation::default_base_url(&config));
            let session_id = conversation::parse_session_id(session_id)?;
            let view = conversation::continue_session_http(
                &base_url,
                session_id,
                message,
                priority.clone(),
            )
            .await?;
            println!("{}", conversation::render_turn_accepted(&view));
        }
        ConversationCommand::Inspect {
            session_id,
            base_url,
        } => {
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| conversation::default_base_url(&config));
            let session_id = conversation::parse_session_id(session_id)?;
            let view = conversation::inspect_session_http(&base_url, session_id).await?;
            println!("{}", conversation::render_session(&view));
        }
        ConversationCommand::End {
            session_id,
            base_url,
        } => {
            let base_url = base_url
                .clone()
                .unwrap_or_else(|| conversation::default_base_url(&config));
            let session_id = conversation::parse_session_id(session_id)?;
            let view = conversation::end_session_http(&base_url, session_id).await?;
            println!("{}", conversation::render_end_view(&view));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_chatgpt_auth_login_subcommand() {
        let cli = Cli::try_parse_from(["mister-smith", "auth", "openai-chatgpt", "login"]).unwrap();

        assert!(matches!(
            cli.command,
            Some(Command::Auth {
                command: AuthCommand::OpenaiChatgpt {
                    command: OpenaiChatgptAuthCommand::Login
                }
            })
        ));
        assert_eq!(cli.config, None);
        assert_eq!(cli.base_url, None);
        assert_eq!(cli.log_level, None);
        assert_eq!(cli.log_format, None);
    }

    #[test]
    fn cli_parses_chatgpt_auth_status_subcommand() {
        let cli =
            Cli::try_parse_from(["mister-smith", "auth", "openai-chatgpt", "status"]).unwrap();

        assert!(matches!(
            cli.command,
            Some(Command::Auth {
                command: AuthCommand::OpenaiChatgpt {
                    command: OpenaiChatgptAuthCommand::Status
                }
            })
        ));
    }

    #[test]
    fn cli_parses_direct_prompt_entry() {
        let raw = vec![
            "mister-smith".to_string(),
            "continue".to_string(),
            "the".to_string(),
            "current".to_string(),
            "session".to_string(),
        ];
        let (cli_args, prompt_words) = split_prompt_words(&raw);
        let cli = Cli::try_parse_from(cli_args).unwrap();

        assert!(cli.command.is_none());
        assert_eq!(
            prompt_words,
            Some(vec![
                "continue".to_string(),
                "the".to_string(),
                "current".to_string(),
                "session".to_string()
            ])
        );
    }

    #[test]
    fn cli_parses_resume_last() {
        let cli = Cli::try_parse_from(["mister-smith", "resume", "--last"]).unwrap();

        assert!(matches!(
            cli.command,
            Some(Command::Resume {
                last: true,
                session_id: None
            })
        ));
    }

    #[test]
    fn cli_parses_resume_last_with_global_base_url_before_subcommand() {
        let raw = vec![
            "mister-smith".to_string(),
            "--base-url".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "resume".to_string(),
            "--last".to_string(),
        ];
        let (cli_args, prompt_words) = split_prompt_words(&raw);
        let cli = Cli::try_parse_from(cli_args).unwrap();

        assert!(matches!(
            cli.command,
            Some(Command::Resume {
                last: true,
                session_id: None
            })
        ));
        assert_eq!(prompt_words, None);
    }

    #[test]
    fn cli_treats_global_base_url_plus_unknown_word_as_direct_prompt() {
        let raw = vec![
            "mister-smith".to_string(),
            "--base-url".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "draft".to_string(),
            "the".to_string(),
            "follow-up".to_string(),
        ];
        let (cli_args, prompt_words) = split_prompt_words(&raw);
        let cli = Cli::try_parse_from(cli_args).unwrap();

        assert!(cli.command.is_none());
        assert_eq!(
            prompt_words,
            Some(vec![
                "draft".to_string(),
                "the".to_string(),
                "follow-up".to_string()
            ])
        );
    }

    #[test]
    fn cli_parses_sessions_open_with_global_base_url_before_subcommand() {
        let raw = vec![
            "mister-smith".to_string(),
            "--base-url".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "sessions".to_string(),
            "open".to_string(),
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
        ];
        let (cli_args, prompt_words) = split_prompt_words(&raw);
        let cli = Cli::try_parse_from(cli_args).unwrap();

        assert!(matches!(
            cli.command,
            Some(Command::Sessions {
                command: SessionsCommand::Open { .. }
            })
        ));
        assert_eq!(prompt_words, None);
    }
}