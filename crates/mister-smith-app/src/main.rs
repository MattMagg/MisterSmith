//! Mister Smith — Multi-agent orchestration framework binary entry point.
//!
//! Orchestrates process lifecycle: deterministic startup, graceful shutdown,
//! signal handling, observability initialization, and cross-phase integration wiring.

mod autonomy;
mod auth;
mod bootstrap;
#[allow(dead_code)]
mod bridges;
mod config;
mod observability;
mod shutdown;

use clap::{Parser, Subcommand};
use mister_smith_config::LogFormat;
use mister_smith_core::ProcessLifecycle;
use std::error::Error;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::{error, info};

/// Mister Smith — Multi-agent orchestration framework.
#[derive(Parser, Debug)]
#[command(name = "mister-smith", version, about)]
struct Cli {
    /// Subcommand to execute. Omit to run the framework runtime.
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to the configuration file.
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Override log level (trace, debug, info, warn, error).
    #[arg(long, global = true)]
    log_level: Option<String>,

    /// Override log format (json, pretty).
    #[arg(long, global = true)]
    log_format: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the Mister Smith framework runtime.
    Run,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        None | Some(Command::Run) => run_runtime(cli).await,
        Some(Command::Autonomy { command }) => execute_autonomy_command(command, &cli).await,
        Some(Command::Auth { command }) => execute_auth_command(command).await,
    };

    if let Err(error) = result {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

async fn run_runtime(cli: Cli) -> Result<(), Box<dyn Error>> {
    let Cli {
        command: _,
        config,
        log_level,
        log_format,
    } = cli;

    // Build CLI overrides
    let overrides = config::CliOverrides {
        config_path: config,
        log_level,
        log_format: log_format.as_deref().and_then(parse_log_format),
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
}
