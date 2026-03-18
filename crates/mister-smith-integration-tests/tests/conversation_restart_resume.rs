//! Manual live proof for the session restart-resume HTTP path.
//!
//! This stays env-gated and ignored by default because it requires:
//! - local PostgreSQL and NATS/JetStream infrastructure
//! - an authenticated ChatGPT session for the runtime provider path
//! - a prebuilt `target/debug/mister-smith` binary

use std::env;
use std::fs::{self, OpenOptions};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use uuid::Uuid;

const INFRA_TIMEOUT: Duration = Duration::from_secs(60);
const READY_TIMEOUT: Duration = Duration::from_secs(45);
const SESSION_TIMEOUT: Duration = Duration::from_secs(180);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "manual live runtime restart-resume proof requires running postgres+nats, authenticated ChatGPT auth, and a prebuilt mister-smith binary"]
async fn live_restart_resume_http_roundtrip_recovers_idle_session_and_resumed_lineage() {
    let admin_database_url = env::var("MS67_TEST_ADMIN_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/postgres".to_string()
    });
    let nats_url =
        env::var("MS67_TEST_NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
    let http_port = reserve_free_port();
    let grpc_port = reserve_free_port();
    let database_name = format!("mistersmith_ms67_{}", Uuid::new_v4().simple());
    let database_url = database_url_for(&admin_database_url, &database_name);
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("HTTP client should build");

    wait_for_postgres(&admin_database_url)
        .await
        .expect("postgres should become reachable before the live proof starts");
    wait_for_nats(&nats_url)
        .await
        .expect("nats should become reachable before the live proof starts");
    create_database(&admin_database_url, &database_name)
        .await
        .expect("temporary database should be created");

    let first_log_path = workspace_root().join("target/ms67-first-runtime.log");
    let second_log_path = workspace_root().join("target/ms67-second-runtime.log");
    let binary_path = app_binary_path();
    assert!(
        binary_path.exists(),
        "missing app binary at {}. Run `cargo build -p mister-smith-app --bin mister-smith` first.",
        binary_path.display()
    );

    let base_url = format!("http://127.0.0.1:{http_port}");
    let mut first_runtime = LiveRuntime::spawn(
        &binary_path,
        &database_url,
        &nats_url,
        http_port,
        grpc_port,
        &first_log_path,
    )
    .await
    .expect("first runtime should start");
    first_runtime
        .wait_ready(&client, &base_url)
        .await
        .expect("first runtime should become ready");

    let create_response = post_json(
        &client,
        &format!("{base_url}/api/v1/sessions"),
        json!({
            "message": "Create a restart-resume proof memo by splitting the work into two parallel tracks, one for runtime recovery and one for operator-visible evidence, then synthesize the result into a concise summary.",
            "priority": "high"
        }),
    )
    .await
    .expect("session create should succeed");
    assert_eq!(create_response.status(), StatusCode::ACCEPTED);
    let accepted: Value = create_response
        .json()
        .await
        .expect("session create response should be JSON");
    let session_id = json_string(&accepted, "session_id");
    let first_workflow_id = json_string(&accepted, "workflow_id");
    let coordinator_agent_id = json_string(&accepted, "coordinator_agent_id");

    first_runtime
        .kill()
        .await
        .expect("first runtime should stop cleanly enough for restart proof");

    let mut second_runtime = LiveRuntime::spawn(
        &binary_path,
        &database_url,
        &nats_url,
        http_port,
        grpc_port,
        &second_log_path,
    )
    .await
    .expect("second runtime should start");
    second_runtime
        .wait_ready(&client, &base_url)
        .await
        .expect("second runtime should become ready");

    let recovered_session = poll_session_until(&client, &base_url, &session_id, |session| {
        session
            .get("active_workflow_id")
            .map_or(true, Value::is_null)
            && session
                .get("last_completed_workflow_id")
                .and_then(Value::as_str)
                == Some(first_workflow_id.as_str())
            && session
                .get("turns")
                .and_then(Value::as_array)
                .and_then(|turns| turns.first())
                .and_then(|turn| turn.get("resume_provenance"))
                .and_then(|value| value.get("recovered_after_restart"))
                .and_then(Value::as_bool)
                == Some(true)
    })
    .await
    .expect("session inspect should recover the interrupted first turn");

    assert_eq!(
        recovered_session
            .get("coordinator_agent_id")
            .and_then(Value::as_str),
        Some(coordinator_agent_id.as_str())
    );
    assert_eq!(
        recovered_session.get("turn_count").and_then(Value::as_u64),
        Some(1)
    );

    let continue_response = post_json(
        &client,
        &format!("{base_url}/api/v1/sessions/{session_id}/turns"),
        json!({
            "message": "Turn the recovered summary into a two-item checklist and keep the same session context.",
            "priority": "high"
        }),
    )
    .await
    .expect("session continue should succeed after restart recovery");
    assert_eq!(continue_response.status(), StatusCode::ACCEPTED);
    let continued: Value = continue_response
        .json()
        .await
        .expect("session continue response should be JSON");
    let second_workflow_id = json_string(&continued, "workflow_id");
    assert_ne!(first_workflow_id, second_workflow_id);
    assert_eq!(json_string(&continued, "session_id"), session_id);
    assert_eq!(
        json_string(&continued, "coordinator_agent_id"),
        coordinator_agent_id
    );

    let final_session = poll_session_until(&client, &base_url, &session_id, |session| {
        session
            .get("active_workflow_id")
            .map_or(true, Value::is_null)
            && session.get("turn_count").and_then(Value::as_u64) == Some(2)
            && session
                .get("turns")
                .and_then(Value::as_array)
                .map(|turns| {
                    turns.len() == 2
                        && is_terminal_turn(&turns[1])
                        && turns[1]
                            .get("resume_provenance")
                            .and_then(|value| value.get("resumed_after_restart"))
                            .and_then(Value::as_bool)
                            == Some(true)
                        && turns[1]
                            .get("resume_provenance")
                            .and_then(|value| value.get("resumed_from_turn_index"))
                            .and_then(Value::as_u64)
                            == Some(1)
                        && turns[1]
                            .get("resume_provenance")
                            .and_then(|value| value.get("resumed_from_workflow_id"))
                            .and_then(Value::as_str)
                            == Some(first_workflow_id.as_str())
                })
                .unwrap_or(false)
    })
    .await
    .expect("second turn should complete and preserve resumed lineage");

    assert_eq!(
        final_session
            .get("coordinator_agent_id")
            .and_then(Value::as_str),
        Some(coordinator_agent_id.as_str())
    );
    assert_eq!(
        final_session
            .get("last_completed_workflow_id")
            .and_then(Value::as_str),
        Some(second_workflow_id.as_str())
    );

    second_runtime
        .kill()
        .await
        .expect("second runtime should stop during cleanup");
    drop_database(&admin_database_url, &database_name)
        .await
        .expect("temporary database should be dropped");
}

struct LiveRuntime {
    child: Child,
    log_path: PathBuf,
}

impl LiveRuntime {
    async fn spawn(
        binary_path: &Path,
        database_url: &str,
        nats_url: &str,
        http_port: u16,
        grpc_port: u16,
        log_path: &Path,
    ) -> Result<Self, String> {
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create log directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let log = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(log_path)
            .map_err(|error| {
                format!("failed to open runtime log {}: {error}", log_path.display())
            })?;
        let log_err = log
            .try_clone()
            .map_err(|error| format!("failed to clone runtime log handle: {error}"))?;

        let mut command = Command::new(binary_path);
        command
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_err))
            .env("DATABASE_URL", database_url)
            .env("MISTER_SMITH_TRANSPORT__NATS_URL", nats_url)
            .env("MISTER_SMITH_TRANSPORT__HTTP_PORT", http_port.to_string())
            .env("MISTER_SMITH_TRANSPORT__GRPC_PORT", grpc_port.to_string())
            .env("MISTER_SMITH_OBSERVABILITY__OTLP_ENDPOINT", "")
            .kill_on_drop(true);

        let child = command
            .spawn()
            .map_err(|error| format!("failed to spawn {}: {error}", binary_path.display()))?;
        Ok(Self {
            child,
            log_path: log_path.to_path_buf(),
        })
    }

    async fn wait_ready(&mut self, client: &Client, base_url: &str) -> Result<(), String> {
        let deadline = Instant::now() + READY_TIMEOUT;
        let ready_url = format!("{base_url}/health/ready");
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| format!("failed to poll child status: {error}"))?
            {
                return Err(format!(
                    "runtime exited before readiness with status {status}. logs:\n{}",
                    self.logs()
                ));
            }

            match client.get(&ready_url).send().await {
                Ok(response) if response.status() == StatusCode::OK => return Ok(()),
                Ok(_) | Err(_) => {}
            }

            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out waiting for readiness at {ready_url}. logs:\n{}",
                    self.logs()
                ));
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn kill(&mut self) -> Result<(), String> {
        if self
            .child
            .try_wait()
            .map_err(|error| format!("failed to poll child status: {error}"))?
            .is_some()
        {
            return Ok(());
        }

        self.child
            .start_kill()
            .map_err(|error| format!("failed to send kill signal: {error}"))?;
        let status = self
            .child
            .wait()
            .await
            .map_err(|error| format!("failed waiting for runtime exit: {error}"))?;
        if !status.success() {
            return Ok(());
        }
        Ok(())
    }

    fn logs(&self) -> String {
        fs::read_to_string(&self.log_path)
            .unwrap_or_else(|_| "<runtime logs unavailable>".to_string())
    }
}

async fn create_database(admin_database_url: &str, database_name: &str) -> Result<(), String> {
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(admin_database_url)
        .await
        .map_err(|error| format!("failed to connect to admin database: {error}"))?;
    let query = format!("CREATE DATABASE \"{database_name}\"");
    admin_pool
        .execute(query.as_str())
        .await
        .map_err(|error| format!("failed to create database {database_name}: {error}"))?;
    admin_pool.close().await;
    Ok(())
}

async fn drop_database(admin_database_url: &str, database_name: &str) -> Result<(), String> {
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(admin_database_url)
        .await
        .map_err(|error| format!("failed to reconnect to admin database: {error}"))?;
    let query = format!("DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)");
    admin_pool
        .execute(query.as_str())
        .await
        .map_err(|error| format!("failed to drop database {database_name}: {error}"))?;
    admin_pool.close().await;
    Ok(())
}

async fn post_json(client: &Client, url: &str, body: Value) -> Result<reqwest::Response, String> {
    client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("POST {url} failed: {error}"))
}

async fn get_json(client: &Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("GET {url} failed: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("GET {url} body read failed: {error}"))?;
    if !status.is_success() {
        return Err(format!("GET {url} returned {status}: {body}"));
    }
    serde_json::from_str(&body).map_err(|error| format!("GET {url} returned invalid JSON: {error}"))
}

async fn poll_session_until<F>(
    client: &Client,
    base_url: &str,
    session_id: &str,
    predicate: F,
) -> Result<Value, String>
where
    F: Fn(&Value) -> bool,
{
    let deadline = Instant::now() + SESSION_TIMEOUT;
    let url = format!("{base_url}/api/v1/sessions/{session_id}");
    loop {
        let session = get_json(client, &url).await?;
        if predicate(&session) {
            return Ok(session);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for session condition at {url}: {}",
                serde_json::to_string_pretty(&session)
                    .unwrap_or_else(|_| "<session json unavailable>".to_string())
            ));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn wait_for_postgres(admin_database_url: &str) -> Result<(), String> {
    let deadline = Instant::now() + INFRA_TIMEOUT;
    loop {
        match PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(2))
            .connect(admin_database_url)
            .await
        {
            Ok(pool) => {
                pool.close().await;
                return Ok(());
            }
            Err(_) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Err(error) => {
                return Err(format!(
                    "postgres at {admin_database_url} did not become ready: {error}"
                ));
            }
        }
    }
}

async fn wait_for_nats(nats_url: &str) -> Result<(), String> {
    let target = nats_host_port(nats_url)?;
    let deadline = Instant::now() + INFRA_TIMEOUT;
    loop {
        match TcpStream::connect(&target).await {
            Ok(_) => return Ok(()),
            Err(_) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Err(error) => {
                return Err(format!("nats at {target} did not become ready: {error}"));
            }
        }
    }
}

fn nats_host_port(nats_url: &str) -> Result<String, String> {
    let without_scheme = nats_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(nats_url);
    let host_port = without_scheme
        .split(',')
        .next()
        .ok_or_else(|| format!("invalid NATS URL: {nats_url}"))?;
    Ok(host_port.to_string())
}

fn json_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {key} in {}", value))
        .to_string()
}

fn is_terminal_turn(turn: &Value) -> bool {
    matches!(
        turn.get("status").and_then(Value::as_str),
        Some("completed" | "failed" | "cancelled")
    )
}

fn reserve_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("ephemeral port bind should succeed")
        .local_addr()
        .expect("ephemeral listener should have a local address")
        .port()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("integration-tests crate should live two levels below the workspace root")
        .to_path_buf()
}

fn database_url_for(admin_database_url: &str, database_name: &str) -> String {
    let (base, query) = match admin_database_url.split_once('?') {
        Some((base, query)) => (base, Some(query)),
        None => (admin_database_url, None),
    };
    let (prefix, _) = base
        .rsplit_once('/')
        .expect("admin database URL should include a database path");
    let mut url = format!("{prefix}/{database_name}");
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    url
}

fn app_binary_path() -> PathBuf {
    if let Some(override_path) = env::var_os("MISTER_SMITH_APP_BINARY") {
        return PathBuf::from(override_path);
    }

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"));
    target_dir.join("debug").join(if cfg!(windows) {
        "mister-smith.exe"
    } else {
        "mister-smith"
    })
}
