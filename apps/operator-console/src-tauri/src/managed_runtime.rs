use serde::Serialize;
use std::{
    env,
    io::{Read, Write},
    net::TcpStream as StdTcpStream,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex as StdMutex,
    },
    time::Duration,
};
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::{
    net::TcpStream,
    sync::Mutex,
    time::{sleep, Instant},
};

const DEFAULT_RUNTIME_URL: &str = "http://127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str =
    "postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith";
const DEFAULT_NATS_URL: &str = "nats://127.0.0.1:4222";
const DEFAULT_GRPC_PORT: &str = "50051";
const DEFAULT_HTTP_PORT: &str = "8080";
const POSTGRES_ADDR: &str = "127.0.0.1:5432";
const NATS_ADDR: &str = "127.0.0.1:4222";

#[derive(Debug, Clone, Serialize)]
pub struct ManagedRuntimeStatusPayload {
    pub state: String,
    pub summary: String,
    pub managed_by_app: bool,
    pub dependencies_managed: bool,
    pub runtime_url: String,
    pub database_target: String,
    pub nats_target: String,
    pub last_error: Option<String>,
    pub last_log_line: Option<String>,
}

#[derive(Clone)]
pub struct ManagedRuntimeManager {
    snapshot: Arc<Mutex<ManagedRuntimeStatusPayload>>,
    child: Arc<StdMutex<Option<CommandChild>>>,
    startup_started: Arc<AtomicBool>,
}

impl ManagedRuntimeManager {
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(default_status())),
            child: Arc::new(StdMutex::new(None)),
            startup_started: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn snapshot(&self) -> ManagedRuntimeStatusPayload {
        self.snapshot.lock().await.clone()
    }

    pub fn ensure_started(&self, app: AppHandle) {
        if self.startup_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            manager.launch(app).await;
        });
    }

    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.take() {
                let _ = child.kill();
            }
        }
    }

    async fn launch(&self, app: AppHandle) {
        if runtime_http_ready().await {
            self.update(|snapshot| {
                snapshot.state = "external_ready".to_string();
                snapshot.summary = "Using existing local runtime".to_string();
                snapshot.managed_by_app = false;
                snapshot.dependencies_managed = false;
                snapshot.last_error = None;
            })
            .await;
            return;
        }

        self.update(|snapshot| {
            snapshot.state = "starting_dependencies".to_string();
            snapshot.summary = "Starting local postgres and nats".to_string();
            snapshot.managed_by_app = true;
            snapshot.last_error = None;
            snapshot.last_log_line = None;
        })
        .await;

        let compose_path = match resolve_compose_path(&app) {
            Ok(path) => path,
            Err(error) => {
                self.fail(error).await;
                return;
            }
        };

        if let Err(error) = docker_compose_up(compose_path).await {
            self.fail(error).await;
            return;
        }

        self.update(|snapshot| {
            snapshot.dependencies_managed = true;
        })
        .await;

        if let Err(error) =
            wait_for_port("PostgreSQL", POSTGRES_ADDR, Duration::from_secs(40)).await
        {
            self.fail(error).await;
            return;
        }

        if let Err(error) = wait_for_port("NATS", NATS_ADDR, Duration::from_secs(40)).await {
            self.fail(error).await;
            return;
        }

        if let Err(error) = wait_for_http_ready(
            "NATS monitor",
            "127.0.0.1",
            8222,
            "/varz",
            Duration::from_secs(40),
        )
        .await
        {
            self.fail(error).await;
            return;
        }

        self.update(|snapshot| {
            snapshot.state = "starting_runtime".to_string();
            snapshot.summary = "Starting bundled runtime".to_string();
        })
        .await;

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let nats_url = env::var("MISTER_SMITH_TRANSPORT__NATS_URL")
            .unwrap_or_else(|_| DEFAULT_NATS_URL.to_string());

        self.update(|snapshot| {
            snapshot.database_target = redact_url_credentials(&database_url);
            snapshot.nats_target = redact_url_credentials(&nats_url);
        })
        .await;

        let sidecar_command = match app.shell().sidecar("mister-smith-runtime") {
            Ok(command) => command
                .args(["run"])
                .env("DATABASE_URL", database_url)
                .env("MISTER_SMITH_TRANSPORT__NATS_URL", nats_url)
                .env("MISTER_SMITH_TRANSPORT__HTTP_PORT", DEFAULT_HTTP_PORT)
                .env("MISTER_SMITH_TRANSPORT__GRPC_PORT", DEFAULT_GRPC_PORT),
            Err(error) => {
                self.fail(format!("failed to prepare bundled runtime: {error}"))
                    .await;
                return;
            }
        };

        let (mut rx, child) = match sidecar_command.spawn() {
            Ok(child) => child,
            Err(error) => {
                self.fail(format!("failed to spawn bundled runtime: {error}"))
                    .await;
                return;
            }
        };

        {
            let mut guard = self
                .child
                .lock()
                .expect("managed runtime child mutex should not be poisoned");
            *guard = Some(child);
        }

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                manager.record_command_event(event).await;
            }
        });

        if let Err(error) = wait_for_runtime_ready(Duration::from_secs(60)).await {
            self.fail(error).await;
            return;
        }

        self.update(|snapshot| {
            snapshot.state = "managed_ready".to_string();
            snapshot.summary = "Managed local runtime ready".to_string();
            snapshot.last_error = None;
        })
        .await;
    }

    async fn record_command_event(&self, event: CommandEvent) {
        match event {
            CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes).trim().to_string();
                if line.is_empty() {
                    return;
                }

                self.update(|snapshot| {
                    snapshot.last_log_line = Some(line.clone());
                })
                .await;
            }
            _ => {}
        }
    }

    async fn fail(&self, error: String) {
        self.update(|snapshot| {
            snapshot.state = "failed".to_string();
            snapshot.summary = "Managed runtime start failed".to_string();
            snapshot.managed_by_app = true;
            snapshot.last_error = Some(error.clone());
        })
        .await;
    }

    async fn update<F>(&self, mut mutate: F)
    where
        F: FnMut(&mut ManagedRuntimeStatusPayload),
    {
        let mut snapshot = self.snapshot.lock().await;
        mutate(&mut snapshot);
    }
}

fn default_status() -> ManagedRuntimeStatusPayload {
    ManagedRuntimeStatusPayload {
        state: "checking".to_string(),
        summary: "Checking local runtime".to_string(),
        managed_by_app: false,
        dependencies_managed: false,
        runtime_url: DEFAULT_RUNTIME_URL.to_string(),
        database_target: redact_url_credentials(DEFAULT_DATABASE_URL),
        nats_target: redact_url_credentials(DEFAULT_NATS_URL),
        last_error: None,
        last_log_line: None,
    }
}

async fn docker_compose_up(compose_path: PathBuf) -> Result<(), String> {
    let compose_path_for_error = compose_path.display().to_string();
    let output = tauri::async_runtime::spawn_blocking(move || {
        let mut command = Command::new("docker");
        command.args(docker_compose_args(&compose_path));
        command.output()
    })
    .await
    .map_err(|error| format!("docker compose join error: {error}"))?
    .map_err(|error| format!("failed to execute docker compose: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        "docker compose returned a non-zero exit status".to_string()
    };

    Err(format!(
        "docker compose bootstrap failed for {compose_path_for_error}: {detail}"
    ))
}

fn docker_compose_args(compose_path: &std::path::Path) -> Vec<String> {
    vec![
        "compose".to_string(),
        "-f".to_string(),
        compose_path.display().to_string(),
        "up".to_string(),
        "-d".to_string(),
        "--force-recreate".to_string(),
        "postgres".to_string(),
        "nats".to_string(),
    ]
}

fn resolve_compose_path(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_path) = app.path().resolve(
        "_up_/_up_/_up_/deploy/docker-compose.yml",
        BaseDirectory::Resource,
    ) {
        if resource_path.exists() {
            return Ok(resource_path);
        }
    }

    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../deploy/docker-compose.yml")
        .canonicalize()
        .map_err(|error| format!("failed to resolve bundled compose fallback: {error}"))?;

    if fallback.exists() {
        Ok(fallback)
    } else {
        Err("docker compose resource was not bundled into the app".to_string())
    }
}

async fn wait_for_port(name: &str, address: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;

    loop {
        if TcpStream::connect(address).await.is_ok() {
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(format!("{name} did not become reachable at {address}"));
        }

        sleep(Duration::from_millis(500)).await;
    }
}

async fn wait_for_runtime_ready(timeout: Duration) -> Result<(), String> {
    wait_for_http_ready("runtime", "127.0.0.1", 8080, "/health/live", timeout).await
}

async fn wait_for_http_ready(
    name: &str,
    host: &str,
    port: u16,
    path: &str,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;

    loop {
        let host_owned = host.to_string();
        let path_owned = path.to_string();
        let probe_host = host_owned.clone();
        let probe_path = path_owned.clone();
        if tauri::async_runtime::spawn_blocking(move || {
            blocking_health_probe(&probe_host, port, &probe_path)
        })
        .await
        .unwrap_or(false)
        {
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "{name} did not become reachable on http://{host_owned}:{port}{path_owned}"
            ));
        }

        sleep(Duration::from_millis(500)).await;
    }
}

async fn runtime_http_ready() -> bool {
    tauri::async_runtime::spawn_blocking(|| {
        blocking_health_probe("127.0.0.1", 8080, "/health/live")
    })
    .await
    .unwrap_or(false)
}

fn blocking_health_probe(host: &str, port: u16, path: &str) -> bool {
    let mut stream = match StdTcpStream::connect((host, port)) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let request =
        format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut buffer = String::new();
    if stream.read_to_string(&mut buffer).is_err() {
        return false;
    }

    buffer.starts_with("HTTP/1.1 200") || buffer.starts_with("HTTP/1.0 200")
}

fn redact_url_credentials(value: &str) -> String {
    if let Some((scheme, remainder)) = value.split_once("://") {
        if let Some((_, suffix)) = remainder.rsplit_once('@') {
            return format!("{scheme}://{suffix}");
        }
    }

    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{default_status, docker_compose_args, redact_url_credentials};
    use std::path::Path;

    #[test]
    fn redacts_url_credentials_when_present() {
        assert_eq!(
            redact_url_credentials("postgres://user:secret@127.0.0.1:5432/mistersmith"),
            "postgres://127.0.0.1:5432/mistersmith"
        );
        assert_eq!(
            redact_url_credentials("nats://operator:secret@127.0.0.1:4222"),
            "nats://127.0.0.1:4222"
        );
    }

    #[test]
    fn leaves_plain_urls_unchanged() {
        assert_eq!(
            redact_url_credentials("http://127.0.0.1:8080"),
            "http://127.0.0.1:8080"
        );
    }

    #[test]
    fn default_status_reflects_local_bootstrap_defaults() {
        let status = default_status();
        assert_eq!(status.state, "checking");
        assert_eq!(status.runtime_url, "http://127.0.0.1:8080");
        assert_eq!(
            status.database_target,
            "postgres://127.0.0.1:5432/mistersmith"
        );
        assert_eq!(status.nats_target, "nats://127.0.0.1:4222");
    }

    #[test]
    fn docker_compose_args_force_recreate_local_dependencies() {
        let args = docker_compose_args(Path::new("/tmp/docker-compose.yml"));
        assert_eq!(
            args,
            vec![
                "compose",
                "-f",
                "/tmp/docker-compose.yml",
                "up",
                "-d",
                "--force-recreate",
                "postgres",
                "nats",
            ]
        );
    }
}
