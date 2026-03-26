#!/usr/bin/env python3
"""Run a repeatable live runtime proof smoke harness for Mister Smith.

This harness is intentionally bounded to the currently proven provider-backed
runtime path: `openai_chatgpt` with `gpt-5.4`.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import textwrap
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from typing import Any
from urllib import error as urllib_error
from urllib import request as urllib_request


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_COMPOSE_FILE = REPO_ROOT / "deploy" / "docker-compose.yml"
DEFAULT_ARTIFACT_ROOT = REPO_ROOT / "docs" / "plans" / "artifacts" / "live-runtime-proof-smoke"
DEFAULT_HTTP_PORT = 8080
DEFAULT_NATS_URL = "nats://127.0.0.1:4222"
DEFAULT_PROVIDER_KIND = "openai_chatgpt"
DEFAULT_MODEL_ID = "gpt-5.4"
DEFAULT_PROFILE = "baseline"
DEFAULT_BUDGET_AWARE_PROFILE = "budget_softcap_openai_mock"
DEFAULT_RUNTIME_BUDGET_BUCKET = "runtime_budget"
DEFAULT_RUNTIME_BUDGET_ROOT = "runtime.task_path"
DEFAULT_RUNTIME_BUDGET_POLICY = "soft_cap"
DEFAULT_RUNTIME_BUDGET_LIMIT_TOKENS = 50_000
DEFAULT_RUNTIME_BUDGET_PERIOD = "live_runtime_smoke"
DEFAULT_FALLBACK_PROVIDER_KIND = "mock"
DEFAULT_FALLBACK_MODEL_ID = "mock-budget-fallback"
DEFAULT_BUDGET_SEED_TARGET_DIR = REPO_ROOT / "target" / "live_runtime_budget_seed"
DEFAULT_TIMEOUT_SECONDS = 240.0
DEFAULT_POLL_INTERVAL_SECONDS = 1.0
DEFAULT_TASK_DESCRIPTION = (
    "Create a concise live runtime trace summary using multiple agents. Create exactly two "
    "parallel worker steps: one worker traces bootstrap, readiness, and provider/runtime wiring; "
    "one worker traces task execution, autonomy status, and terminal result markers. Then create "
    "one final worker step that merges both worker outputs into one concise runtime memo with "
    "sections Observed Evidence and Follow-up Needed. Keep the final memo under 180 words and "
    "keep every claim tied to directly observed runtime evidence."
)
REQUIRED_RUNTIME_LOG_MARKERS = (
    "JetStream stream created/updated",
    "Runtime task execution service ready",
    "Mister Smith ready",
)
RUNTIME_SERVICES = ("postgres", "nats")
PLANNER_OUTPUT_TRUST_NOTE = (
    "planner_output is preserved as raw planner output and may carry stale role metadata; "
    "runtime_execution_mode and step_results are the authoritative proof surfaces in this artifact."
)
EMAIL_PATTERN = re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")


class SmokeHarnessError(RuntimeError):
    """Raised when the smoke harness cannot prove the requested runtime path."""


@dataclass(frozen=True)
class HarnessConfig:
    run_id: str
    compose_file: Path
    artifact_dir: Path
    database_name: str
    database_url: str
    http_port: int
    base_url: str
    timeout_seconds: float
    poll_interval_seconds: float
    provider_kind: str
    model_id: str
    profile: str
    runtime_config_path: Path | None
    routing_policy: str
    registered_provider_count: int
    budget_root: str | None
    budget_policy: str | None
    expected_step_action: str | None
    expected_step_tier: str | None
    required_step_checkpoints: tuple[str, ...]


@dataclass(frozen=True)
class HttpJsonResponse:
    status_code: int
    headers: dict[str, str]
    payload: Any
    body_text: str


def utc_run_id(now: datetime | None = None) -> str:
    current = now or datetime.now(timezone.utc)
    return current.strftime("%Y%m%dT%H%M%SZ")


def build_database_name(run_id: str) -> str:
    timestamp = run_id.replace("T", "_").replace("Z", "").lower()
    return f"mistersmith_live_runtime_smoke_{timestamp}"


def build_artifact_dir(root: Path, run_id: str) -> Path:
    artifact_dir = root / run_id
    artifact_dir.mkdir(parents=True, exist_ok=False)
    return artifact_dir


def build_task_request(description: str = DEFAULT_TASK_DESCRIPTION) -> dict[str, Any]:
    return {"description": description, "priority": "high"}


def repo_relative(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def redact_email_addresses(text: str) -> str:
    return EMAIL_PATTERN.sub("<redacted-email>", text)


def redact_database_url(database_url: str) -> str:
    return database_url.replace(":mistersmith_dev@", ":<redacted>@")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def is_budget_aware_profile(profile: str) -> bool:
    return profile == DEFAULT_BUDGET_AWARE_PROFILE


def build_runtime_config_toml(profile: str) -> str | None:
    if not is_budget_aware_profile(profile):
        return None

    return textwrap.dedent(
        f"""
        [llm]
        provider_kind = "{DEFAULT_PROVIDER_KIND}"
        model_id = "{DEFAULT_MODEL_ID}"

        [llm.runtime_routing_profile]
        policy = "cascade"
        budget_root = "{DEFAULT_RUNTIME_BUDGET_ROOT}"

        [[llm.runtime_routing_profile.tiers]]
        label = "primary"
        provider_kind = "{DEFAULT_PROVIDER_KIND}"
        model_id = "{DEFAULT_MODEL_ID}"
        metadata = {{ tier = "primary" }}

        [[llm.runtime_routing_profile.tiers]]
        label = "fallback"
        provider_kind = "{DEFAULT_FALLBACK_PROVIDER_KIND}"
        model_id = "{DEFAULT_FALLBACK_MODEL_ID}"
        metadata = {{ tier = "fallback" }}
        """
    ).strip() + "\n"


def write_runtime_config(artifact_dir: Path, profile: str) -> Path | None:
    config_text = build_runtime_config_toml(profile)
    if config_text is None:
        return None

    config_path = artifact_dir / "runtime-config.toml"
    write_text(config_path, config_text)
    return config_path


def build_budget_seed_request(config: HarnessConfig) -> dict[str, Any]:
    if config.budget_root is None or config.budget_policy is None:
        raise SmokeHarnessError("budget seed requested without a configured budget-aware profile")

    return {
        "nats_url": DEFAULT_NATS_URL,
        "bucket": DEFAULT_RUNTIME_BUDGET_BUCKET,
        "key": config.budget_root,
        "limit_tokens": DEFAULT_RUNTIME_BUDGET_LIMIT_TOKENS,
        "used_tokens": 0,
        "period": DEFAULT_RUNTIME_BUDGET_PERIOD,
        "policy": config.budget_policy,
    }


def budget_seed_helper_manifest() -> str:
    return textwrap.dedent(
        """
        [package]
        name = "ms-live-runtime-budget-seed"
        version = "0.1.0"
        edition = "2021"

        [workspace]

        [dependencies]
        async-nats = { version = "0.46.0", features = ["jetstream", "kv"] }
        serde = { version = "1", features = ["derive"] }
        serde_json = "1"
        tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
        """
    ).strip() + "\n"


def budget_seed_helper_source() -> str:
    return textwrap.dedent(
        """
        use std::env;

        use async_nats::jetstream::{self, kv::{self, Operation as KvOperation}};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum BudgetPolicy {
            HardCap,
            SoftCap,
            Conditioned,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct BudgetNode {
            key: String,
            limit_tokens: u64,
            used_tokens: u64,
            period: String,
            policy: BudgetPolicy,
            #[serde(default)]
            revision: u64,
        }

        fn usage() -> &'static str {
            "usage: seed <nats_url> <bucket> <key> <limit_tokens> <used_tokens> <period> <policy> | fetch <nats_url> <bucket> <key>"
        }

        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let args: Vec<String> = env::args().collect();
            match args.get(1).map(String::as_str) {
                Some("seed") => {
                    if args.len() != 9 {
                        return Err(usage().into());
                    }
                    let nats_url = &args[2];
                    let bucket = &args[3];
                    let key = &args[4];
                    let limit_tokens = args[5].parse::<u64>()?;
                    let used_tokens = args[6].parse::<u64>()?;
                    let period = &args[7];
                    let policy = serde_json::from_value::<BudgetPolicy>(serde_json::Value::String(args[8].clone()))?;

                    let client = async_nats::connect(nats_url).await?;
                    let jetstream = jetstream::new(client);
                    let store = jetstream
                        .create_or_update_key_value(kv::Config {
                            bucket: bucket.to_string(),
                            description: "Runtime budget state for the task-path router".to_string(),
                            history: 1,
                            ..Default::default()
                        })
                        .await?;

                    let node = BudgetNode {
                        key: key.to_string(),
                        limit_tokens,
                        used_tokens,
                        period: period.to_string(),
                        policy,
                        revision: 0,
                    };
                    let payload = serde_json::to_vec(&node)?;
                    store.put(key.as_str(), payload.into()).await?;
                    println!("{}", serde_json::to_string_pretty(&node)?);
                    Ok(())
                }
                Some("fetch") => {
                    if args.len() != 5 {
                        return Err(usage().into());
                    }
                    let nats_url = &args[2];
                    let bucket = &args[3];
                    let key = &args[4];

                    let client = async_nats::connect(nats_url).await?;
                    let jetstream = jetstream::new(client);
                    let store = jetstream
                        .create_or_update_key_value(kv::Config {
                            bucket: bucket.to_string(),
                            description: "Runtime budget state for the task-path router".to_string(),
                            history: 1,
                            ..Default::default()
                        })
                        .await?;

                    let entry = store.entry(key).await?;
                    let Some(entry) = entry else {
                        return Err(format!("budget key '{key}' not found in bucket '{bucket}'").into());
                    };
                    if entry.operation != KvOperation::Put {
                        return Err(format!("budget key '{key}' did not resolve to a put entry").into());
                    }
                    let mut node: BudgetNode = serde_json::from_slice(&entry.value)?;
                    node.key = entry.key;
                    node.revision = entry.revision;
                    println!("{}", serde_json::to_string_pretty(&node)?);
                    Ok(())
                }
                _ => Err(usage().into()),
            }
        }
        """
    ).strip() + "\n"


def ensure_budget_seed_helper(config: HarnessConfig) -> Path:
    helper_dir = config.artifact_dir / ".budget-seed-helper"
    src_dir = helper_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    write_text(helper_dir / "Cargo.toml", budget_seed_helper_manifest())
    write_text(src_dir / "main.rs", budget_seed_helper_source())
    return helper_dir


def run_budget_seed_helper(
    config: HarnessConfig,
    command_name: str,
    *,
    artifact_name: str,
) -> dict[str, Any]:
    helper_dir = ensure_budget_seed_helper(config)
    try:
        if command_name == "seed":
            request = build_budget_seed_request(config)
            helper_args = [
                "seed",
                request["nats_url"],
                request["bucket"],
                request["key"],
                str(request["limit_tokens"]),
                str(request["used_tokens"]),
                request["period"],
                request["policy"],
            ]
            write_json(config.artifact_dir / "budget-seed-request.json", request)
        elif command_name == "fetch":
            if config.budget_root is None:
                raise SmokeHarnessError("budget fetch requested without a configured budget root")
            helper_args = [
                "fetch",
                DEFAULT_NATS_URL,
                DEFAULT_RUNTIME_BUDGET_BUCKET,
                config.budget_root,
            ]
        else:
            raise SmokeHarnessError(f"unknown budget helper command {command_name!r}")

        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(DEFAULT_BUDGET_SEED_TARGET_DIR)
        command = [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(helper_dir / "Cargo.toml"),
            "--",
            *helper_args,
        ]
        result = run_command(command, env=env)
        write_text(config.artifact_dir / f"{artifact_name}.log", result.stdout + result.stderr)
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise SmokeHarnessError(
                f"budget helper returned invalid JSON for {command_name}: {exc}"
            ) from exc
        write_json(config.artifact_dir / f"{artifact_name}.json", payload)
        return payload
    finally:
        shutil.rmtree(helper_dir, ignore_errors=True)


def format_command(command: list[str]) -> str:
    return " ".join(command)


def ensure_required_tools(*tools: str) -> None:
    missing = [tool for tool in tools if shutil.which(tool) is None]
    if missing:
        missing_list = ", ".join(sorted(missing))
        raise SmokeHarnessError(f"required tool(s) not found on PATH: {missing_list}")


def ensure_port_available(port: int) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(0.5)
        if sock.connect_ex(("127.0.0.1", port)) == 0:
            raise SmokeHarnessError(
                f"http port {port} is already in use; rerun with --http-port on a free port"
            )


def compose_command(compose_file: Path, *args: str) -> list[str]:
    return ["docker", "compose", "-f", str(compose_file), *args]


def run_command(
    command: list[str],
    *,
    cwd: Path = REPO_ROOT,
    env: dict[str, str] | None = None,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and result.returncode != 0:
        stderr = result.stderr.strip()
        stdout = result.stdout.strip()
        details = stderr or stdout or f"exit code {result.returncode}"
        raise SmokeHarnessError(f"command failed: {format_command(command)}\n{details}")
    return result


def fetch_json(
    url: str,
    *,
    method: str = "GET",
    payload: Any | None = None,
    timeout_seconds: float = 5.0,
) -> HttpJsonResponse:
    body: bytes | None = None
    headers = {"accept": "application/json"}
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")
        headers["content-type"] = "application/json"
    request = urllib_request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib_request.urlopen(request, timeout=timeout_seconds) as response:
            body_text = response.read().decode("utf-8")
            return HttpJsonResponse(
                status_code=response.status,
                headers=dict(response.headers.items()),
                payload=json.loads(body_text),
                body_text=body_text,
            )
    except urllib_error.HTTPError as exc:
        body_text = exc.read().decode("utf-8", errors="replace")
        raise SmokeHarnessError(
            f"http request failed: {method} {url} -> {exc.code}: {body_text.strip()}"
        ) from exc
    except urllib_error.URLError as exc:
        raise SmokeHarnessError(f"http request failed: {method} {url}: {exc.reason}") from exc
    except json.JSONDecodeError as exc:
        raise SmokeHarnessError(f"invalid JSON from {url}: {exc}") from exc


def wait_for_service_health(
    config: HarnessConfig,
    service: str,
    artifact_name: str,
) -> dict[str, Any]:
    deadline = time.monotonic() + config.timeout_seconds
    last_error = "service container never appeared"
    while time.monotonic() < deadline:
        ps = run_command(
            compose_command(config.compose_file, "ps", "-q", service),
            check=False,
        )
        container_id = ps.stdout.strip()
        if container_id:
            inspect = run_command(
                ["docker", "inspect", "--format", "{{json .State.Health}}", container_id],
                check=False,
            )
            if inspect.returncode == 0 and inspect.stdout.strip():
                payload = json.loads(inspect.stdout)
                write_json(config.artifact_dir / artifact_name, payload)
                status = payload.get("Status")
                if status == "healthy":
                    return payload
                last_error = f"health status {status!r}"
            elif inspect.stderr.strip():
                last_error = inspect.stderr.strip()
        time.sleep(config.poll_interval_seconds)
    raise SmokeHarnessError(f"{service} did not become healthy: {last_error}")


def wait_for_postgres_ready(config: HarnessConfig) -> str:
    deadline = time.monotonic() + config.timeout_seconds
    last_error = "pg_isready never succeeded"
    while time.monotonic() < deadline:
        result = run_command(
            compose_command(
                config.compose_file,
                "exec",
                "-T",
                "postgres",
                "pg_isready",
                "-U",
                "mistersmith",
                "-h",
                "127.0.0.1",
                "-p",
                "5432",
            ),
            check=False,
        )
        output = (result.stdout + result.stderr).strip()
        write_text(config.artifact_dir / "postgres-health.txt", output + "\n")
        if result.returncode == 0:
            return output
        last_error = output or f"exit code {result.returncode}"
        time.sleep(config.poll_interval_seconds)
    raise SmokeHarnessError(f"postgres never became ready: {last_error}")


def validate_nats_varz(payload: dict[str, Any]) -> None:
    if not payload.get("server_id"):
        raise SmokeHarnessError("nats varz payload is missing server_id")
    jetstream = payload.get("jetstream")
    if not jetstream:
        raise SmokeHarnessError("nats varz payload does not report JetStream enabled")


def fetch_nats_varz(config: HarnessConfig) -> dict[str, Any]:
    result = run_command(
        compose_command(
            config.compose_file,
            "exec",
            "-T",
            "nats",
            "wget",
            "-qO-",
            "http://127.0.0.1:8222/varz",
        )
    )
    payload = json.loads(result.stdout)
    validate_nats_varz(payload)
    write_json(config.artifact_dir / "nats-varz.json", payload)
    return payload


def service_running(config: HarnessConfig, service: str) -> bool:
    result = run_command(
        compose_command(config.compose_file, "ps", "-q", service),
        check=False,
    )
    container_id = result.stdout.strip()
    if not container_id:
        return False
    inspect = run_command(
        ["docker", "inspect", "--format", "{{.State.Running}}", container_id],
        check=False,
    )
    return inspect.returncode == 0 and inspect.stdout.strip() == "true"


def format_database_create_artifact(
    config: HarnessConfig,
    command: list[str],
    result: subprocess.CompletedProcess[str],
) -> str:
    return "\n".join(
        (
            f"timestamp: {datetime.now(timezone.utc).isoformat()}",
            f"database_name: {config.database_name}",
            f"command: {format_command(command)}",
            f"sql: CREATE DATABASE {config.database_name};",
            f"returncode: {result.returncode}",
            "stdout:",
            result.stdout.rstrip(),
            "stderr:",
            result.stderr.rstrip(),
            "",
        )
    )


def create_database(config: HarnessConfig) -> None:
    if not re.fullmatch(r"[a-z0-9_]+", config.database_name):
        raise SmokeHarnessError(
            f"database name must be lowercase alphanumeric plus underscore: {config.database_name}"
        )
    command = compose_command(
        config.compose_file,
        "exec",
        "-T",
        "postgres",
        "psql",
        "-U",
        "mistersmith",
        "-d",
        "postgres",
        "-v",
        "ON_ERROR_STOP=1",
        "-c",
        f"CREATE DATABASE {config.database_name};",
    )
    result = run_command(command)
    write_text(
        config.artifact_dir / "database-create.txt",
        format_database_create_artifact(config, command, result),
    )


def check_openai_auth_status(config: HarnessConfig) -> str:
    result = run_command(
        ["cargo", "run", "-q", "-p", "mister-smith-app", "--", "auth", "openai-chatgpt", "status"]
    )
    output = (result.stdout + result.stderr).strip()
    write_text(
        config.artifact_dir / "openai-chatgpt-auth-status.txt",
        redact_email_addresses(output) + "\n",
    )
    if "Authenticated ChatGPT account" not in output:
        raise SmokeHarnessError(
            "openai-chatgpt auth status did not report an authenticated ChatGPT account"
        )
    return output


def start_runtime(config: HarnessConfig) -> tuple[subprocess.Popen[str], Any]:
    runtime_log_path = config.artifact_dir / "runtime.log"
    runtime_log = runtime_log_path.open("w", encoding="utf-8")
    env = os.environ.copy()
    env.update(
        {
            "DATABASE_URL": config.database_url,
            "MISTER_SMITH_TRANSPORT__NATS_URL": DEFAULT_NATS_URL,
            "MISTER_SMITH_TRANSPORT__HTTP_PORT": str(config.http_port),
            "MISTER_SMITH_LLM__PROVIDER_KIND": config.provider_kind,
            "MISTER_SMITH_LLM__MODEL_ID": config.model_id,
        }
    )
    command = ["cargo", "run", "-q", "-p", "mister-smith-app", "--"]
    if config.runtime_config_path is not None:
        command.extend(["--config", str(config.runtime_config_path)])
    command.append("run")
    process = subprocess.Popen(
        command,
        cwd=REPO_ROOT,
        env=env,
        stdout=runtime_log,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    return process, runtime_log


def wait_for_runtime_ready(
    config: HarnessConfig,
    process: subprocess.Popen[str],
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    deadline = time.monotonic() + config.timeout_seconds
    last_error = "runtime did not become ready"
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise SmokeHarnessError(
                f"runtime exited before readiness with code {process.returncode}; "
                f"see {config.artifact_dir / 'runtime.log'}"
            )

        try:
            live = fetch_json(f"{config.base_url}/health/live").payload
            ready = fetch_json(f"{config.base_url}/health/ready").payload
            api_health = fetch_json(f"{config.base_url}/api/v1/health").payload
            if (
                live.get("status") == "alive"
                and ready.get("status") == "ready"
                and api_health.get("status") == "healthy"
            ):
                write_json(config.artifact_dir / "health-live.json", live)
                write_json(config.artifact_dir / "health-ready.json", ready)
                write_json(config.artifact_dir / "api-health.json", api_health)
                return live, ready, api_health
            last_error = (
                f"live={live.get('status')!r} ready={ready.get('status')!r} "
                f"api={api_health.get('status')!r}"
            )
        except SmokeHarnessError as exc:
            last_error = str(exc)
        time.sleep(config.poll_interval_seconds)
    raise SmokeHarnessError(f"runtime never reached ready state: {last_error}")


def assert_runtime_log_markers(log_text: str) -> None:
    missing = [marker for marker in REQUIRED_RUNTIME_LOG_MARKERS if marker not in log_text]
    if missing:
        raise SmokeHarnessError(
            "runtime log is missing required startup markers: " + ", ".join(missing)
        )


def wait_for_runtime_log_markers(
    config: HarnessConfig,
    process: subprocess.Popen[str],
) -> str:
    deadline = time.monotonic() + config.timeout_seconds
    last_error = "runtime log markers never appeared"
    runtime_log_path = config.artifact_dir / "runtime.log"
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise SmokeHarnessError(
                f"runtime exited before startup markers completed with code {process.returncode}; "
                f"see {runtime_log_path}"
            )
        log_text = runtime_log_path.read_text(encoding="utf-8")
        try:
            assert_runtime_log_markers(log_text)
            return log_text
        except SmokeHarnessError as exc:
            last_error = str(exc)
        time.sleep(config.poll_interval_seconds)
    raise SmokeHarnessError(last_error)


def annotate_task_status_artifact(task_status: dict[str, Any]) -> dict[str, Any]:
    result_envelope = task_status.get("result")
    if not isinstance(result_envelope, dict):
        return task_status
    result = result_envelope.get("result", result_envelope)
    if not isinstance(result, dict):
        return task_status
    planner_output = result.get("planner_output")
    if not isinstance(planner_output, dict):
        return task_status
    result["planner_output_trust"] = "raw_untrusted"
    result["planner_output_note"] = PLANNER_OUTPUT_TRUST_NOTE
    return task_status


def summarize_task_status(task_status: dict[str, Any]) -> dict[str, Any]:
    result_envelope = task_status.get("result")
    if not isinstance(result_envelope, dict):
        raise SmokeHarnessError("task status payload is missing a JSON object result")
    result = result_envelope.get("result", result_envelope)
    if not isinstance(result, dict):
        raise SmokeHarnessError("task result envelope did not contain a JSON object payload")
    runtime_execution_mode = result.get("runtime_execution_mode")
    if not isinstance(runtime_execution_mode, dict):
        raise SmokeHarnessError("task result is missing runtime_execution_mode")
    step_results = result.get("step_results")
    if not isinstance(step_results, list):
        raise SmokeHarnessError("task result is missing step_results")

    step_summaries: list[dict[str, Any]] = []
    for step in step_results:
        if not isinstance(step, dict):
            raise SmokeHarnessError("step result entry is not a JSON object")
        nested_result = step.get("result")
        if not isinstance(nested_result, dict):
            raise SmokeHarnessError("nested step result is missing")
        step_summaries.append(
            {
                "task_id": step.get("task_id"),
                "worker_id": step.get("worker_id"),
                "action": step.get("action"),
                "execution_boundary": nested_result.get("execution_boundary"),
                "tool_name": nested_result.get("tool_name"),
            }
        )

    aggregated_result = result.get("aggregated_result")
    aggregated_result_count = len(aggregated_result) if isinstance(aggregated_result, list) else 0

    return {
        "task_id": task_status.get("task_id"),
        "status": task_status.get("status"),
        "provider_kind": result.get("provider_kind"),
        "model_id": result.get("model_id"),
        "proof_outcome": result.get("proof_outcome")
        or result_envelope.get("proof_outcome"),
        "runtime_execution_mode": runtime_execution_mode,
        "routing_policy": runtime_execution_mode.get("routing_policy"),
        "registered_provider_count": runtime_execution_mode.get("registered_provider_count"),
        "budget_root": runtime_execution_mode.get("budget_root"),
        "step_result_count": len(step_results),
        "step_summaries": step_summaries,
        "aggregated_result_count": aggregated_result_count,
    }


def assert_task_summary(
    summary: dict[str, Any],
    *,
    expected_provider_kind: str,
    expected_model_id: str,
    expected_routing_policy: str = "round_robin",
    expected_registered_provider_count: int = 1,
    expected_budget_root: str = "disabled",
) -> None:
    if summary.get("status") != "completed":
        raise SmokeHarnessError(f"task did not complete successfully: {summary.get('status')!r}")
    if summary.get("provider_kind") != expected_provider_kind:
        raise SmokeHarnessError(
            "task result provider_kind did not match expected live proof path"
        )
    if summary.get("model_id") != expected_model_id:
        raise SmokeHarnessError("task result model_id did not match expected live proof path")

    runtime_execution_mode = summary.get("runtime_execution_mode")
    if not isinstance(runtime_execution_mode, dict):
        raise SmokeHarnessError("task summary is missing runtime_execution_mode")

    expected_markers = {
        "workflow_runner": "tokio_task",
        "planner_lifecycle": "supervised_actor",
        "executor_lifecycle": "supervised_actor",
        "execution_boundary": "tool_bus",
        "tool_name": "workflow.execute_step",
        "provider_kind": expected_provider_kind,
        "model_id": expected_model_id,
        "routing_policy": expected_routing_policy,
        "registered_provider_count": expected_registered_provider_count,
        "budget_root": expected_budget_root,
    }
    for key, expected in expected_markers.items():
        actual = runtime_execution_mode.get(key)
        if actual != expected:
            raise SmokeHarnessError(
                f"runtime_execution_mode.{key} expected {expected!r}, observed {actual!r}"
            )

    step_summaries = summary.get("step_summaries")
    if not isinstance(step_summaries, list) or not step_summaries:
        raise SmokeHarnessError("task summary did not include any step results")
    for index, step_summary in enumerate(step_summaries, start=1):
        if step_summary.get("execution_boundary") != "tool_bus":
            raise SmokeHarnessError(
                f"step {index} execution_boundary expected 'tool_bus', observed "
                f"{step_summary.get('execution_boundary')!r}"
            )
        if step_summary.get("tool_name") != "workflow.execute_step":
            raise SmokeHarnessError(
                f"step {index} tool_name expected 'workflow.execute_step', observed "
                f"{step_summary.get('tool_name')!r}"
            )


def assert_workflow_list_contains(workflows_payload: dict[str, Any], workflow_id: str) -> None:
    workflows = workflows_payload.get("workflows")
    if not isinstance(workflows, list):
        raise SmokeHarnessError("autonomy workflows payload is missing workflows[]")
    if workflow_id not in workflows:
        raise SmokeHarnessError(f"workflow {workflow_id} was not present in autonomy workflows")


def assert_autonomy_status(status_payload: dict[str, Any], workflow_id: str) -> None:
    graph = status_payload.get("graph")
    topology = status_payload.get("topology")
    routing_history = status_payload.get("routing_history")
    step_routing_history = status_payload.get("step_routing_history")
    branches = status_payload.get("branches")

    if not isinstance(graph, dict):
        raise SmokeHarnessError("autonomy status payload is missing graph")
    if graph.get("workflow_id") != workflow_id:
        raise SmokeHarnessError("autonomy graph.workflow_id did not match submitted workflow")
    if graph.get("state") != "Completed":
        raise SmokeHarnessError(f"autonomy graph did not complete: {graph.get('state')!r}")
    if int(graph.get("branch_count", 0)) < 1 or int(graph.get("node_count", 0)) < 1:
        raise SmokeHarnessError("autonomy graph reported no branches or nodes")

    if not isinstance(topology, dict) or int(topology.get("parallelism_width", 0)) < 1:
        raise SmokeHarnessError("autonomy topology did not report positive parallelism width")
    if not isinstance(routing_history, list) or not routing_history:
        raise SmokeHarnessError("autonomy routing_history was empty")
    if not isinstance(step_routing_history, list) or not step_routing_history:
        raise SmokeHarnessError("autonomy step_routing_history was empty")
    if not isinstance(branches, list) or len(branches) != int(graph.get("branch_count", 0)):
        raise SmokeHarnessError("autonomy branches did not match graph.branch_count")


def assert_autonomy_step_routing_expectations(
    status_payload: dict[str, Any],
    *,
    expected_action: str | None,
    expected_tier: str | None,
    required_checkpoints: tuple[str, ...],
) -> None:
    step_routing_history = status_payload.get("step_routing_history")
    if not isinstance(step_routing_history, list) or not step_routing_history:
        raise SmokeHarnessError("autonomy step_routing_history was empty")
    latest = step_routing_history[-1]
    if not isinstance(latest, dict):
        raise SmokeHarnessError("latest step_routing_history entry was not a JSON object")

    if expected_action is not None and latest.get("action") != expected_action:
        raise SmokeHarnessError(
            f"latest step action expected {expected_action!r}, observed {latest.get('action')!r}"
        )
    if expected_tier is not None and latest.get("tier") != expected_tier:
        raise SmokeHarnessError(
            f"latest step tier expected {expected_tier!r}, observed {latest.get('tier')!r}"
        )

    checkpoints = latest.get("triggered_checkpoints")
    if not isinstance(checkpoints, list):
        raise SmokeHarnessError("latest step_routing_history entry was missing checkpoints")
    missing = [checkpoint for checkpoint in required_checkpoints if checkpoint not in checkpoints]
    if missing:
        raise SmokeHarnessError(
            "latest step routing entry was missing required checkpoints: "
            + ", ".join(missing)
        )


def wait_for_terminal_task_status(
    config: HarnessConfig,
    task_id: str,
) -> dict[str, Any]:
    deadline = time.monotonic() + config.timeout_seconds
    poll_log_path = config.artifact_dir / "task-poll.log"
    last_payload: dict[str, Any] | None = None
    last_error = "task status never returned a successful response"

    with poll_log_path.open("w", encoding="utf-8") as poll_log:
        while time.monotonic() < deadline:
            timestamp = datetime.now(timezone.utc).isoformat()
            try:
                response = fetch_json(f"{config.base_url}/api/v1/tasks/{task_id}")
                payload = response.payload
                if not isinstance(payload, dict):
                    raise SmokeHarnessError("task status response was not a JSON object")
                last_payload = payload
                status = str(payload.get("status", "unknown"))
                poll_log.write(f"{timestamp} status={status}\n")
                poll_log.flush()
                if status.lower() in {"completed", "failed", "cancelled"}:
                    artifact_payload = annotate_task_status_artifact(payload)
                    write_json(config.artifact_dir / "task-status-latest.json", artifact_payload)
                    return artifact_payload
            except SmokeHarnessError as exc:
                last_error = str(exc)
                poll_log.write(f"{timestamp} error={last_error}\n")
                poll_log.flush()
            time.sleep(config.poll_interval_seconds)

    if last_payload is not None:
        write_json(
            config.artifact_dir / "task-status-latest.json",
            annotate_task_status_artifact(last_payload),
        )
    raise SmokeHarnessError(
        f"task {task_id} did not reach terminal state before timeout: {last_error}"
    )


def shutdown_runtime(process: subprocess.Popen[str], runtime_log: Any) -> None:
    try:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=10)
    finally:
        runtime_log.close()


def build_smoke_summary(
    config: HarnessConfig,
    task_id: str,
    task_summary: dict[str, Any],
    autonomy_status: dict[str, Any],
    budget_state_after: dict[str, Any] | None,
) -> dict[str, Any]:
    graph = autonomy_status.get("graph", {})
    topology = autonomy_status.get("topology", {})
    latest_step = {}
    step_history = autonomy_status.get("step_routing_history")
    if isinstance(step_history, list) and step_history:
        latest = step_history[-1]
        if isinstance(latest, dict):
            latest_step = latest
    return {
        "run_id": config.run_id,
        "profile": config.profile,
        "artifact_dir": repo_relative(config.artifact_dir),
        "database_name": config.database_name,
        "base_url": config.base_url,
        "task_id": task_id,
        "provider_kind": task_summary.get("provider_kind"),
        "model_id": task_summary.get("model_id"),
        "runtime_execution_mode": task_summary.get("runtime_execution_mode"),
        "routing_policy": task_summary.get("routing_policy"),
        "registered_provider_count": task_summary.get("registered_provider_count"),
        "budget_root": task_summary.get("budget_root"),
        "step_result_count": task_summary.get("step_result_count"),
        "aggregated_result_count": task_summary.get("aggregated_result_count"),
        "latest_step_tier": latest_step.get("tier"),
        "latest_step_action": latest_step.get("action"),
        "latest_step_checkpoints": latest_step.get("triggered_checkpoints"),
        "budget_state_after": budget_state_after,
        "graph_state": graph.get("state"),
        "branch_count": graph.get("branch_count"),
        "node_count": graph.get("node_count"),
        "topology_kind": topology.get("topology_kind"),
        "parallelism_width": topology.get("parallelism_width"),
    }


def build_config(args: argparse.Namespace) -> HarnessConfig:
    run_id = utc_run_id()
    artifact_dir = build_artifact_dir(args.artifact_root, run_id)
    database_name = args.database_name or build_database_name(run_id)
    database_url = f"postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/{database_name}"
    base_url = f"http://127.0.0.1:{args.http_port}"
    runtime_config_path = write_runtime_config(artifact_dir, args.profile)
    is_budget_profile = is_budget_aware_profile(args.profile)
    return HarnessConfig(
        run_id=run_id,
        compose_file=args.compose_file,
        artifact_dir=artifact_dir,
        database_name=database_name,
        database_url=database_url,
        http_port=args.http_port,
        base_url=base_url,
        timeout_seconds=args.timeout_seconds,
        poll_interval_seconds=args.poll_interval_seconds,
        provider_kind=DEFAULT_PROVIDER_KIND,
        model_id=DEFAULT_MODEL_ID,
        profile=args.profile,
        runtime_config_path=runtime_config_path,
        routing_policy="cascade" if is_budget_profile else "round_robin",
        registered_provider_count=2 if is_budget_profile else 1,
        budget_root=DEFAULT_RUNTIME_BUDGET_ROOT if is_budget_profile else None,
        budget_policy=DEFAULT_RUNTIME_BUDGET_POLICY if is_budget_profile else None,
        expected_step_action="downgrade" if is_budget_profile else None,
        expected_step_tier="primary" if is_budget_profile else None,
        required_step_checkpoints=("budget_policy",) if is_budget_profile else (),
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--compose-file",
        type=Path,
        default=DEFAULT_COMPOSE_FILE,
        help="Path to deploy/docker-compose.yml.",
    )
    parser.add_argument(
        "--artifact-root",
        type=Path,
        default=DEFAULT_ARTIFACT_ROOT,
        help="Artifact root directory. A timestamped run subdirectory will be created beneath it.",
    )
    parser.add_argument(
        "--database-name",
        help="Optional explicit database name. Defaults to a timestamped smoke database name.",
    )
    parser.add_argument(
        "--http-port",
        type=int,
        default=DEFAULT_HTTP_PORT,
        help="HTTP port used by the temporary runtime process.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Maximum time to wait for each long-running phase.",
    )
    parser.add_argument(
        "--profile",
        choices=(DEFAULT_PROFILE, DEFAULT_BUDGET_AWARE_PROFILE),
        default=DEFAULT_PROFILE,
        help=(
            "Proof profile to exercise. 'baseline' preserves the existing "
            "single-provider proof surface; 'budget_softcap_openai_mock' "
            "boots the config-gated cascade profile with a seeded soft-cap budget root."
        ),
    )
    parser.add_argument(
        "--poll-interval-seconds",
        type=float,
        default=DEFAULT_POLL_INTERVAL_SECONDS,
        help="Polling interval for readiness and task-status checks.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    config = build_config(args)
    write_json(
        config.artifact_dir / "smoke-run-config.json",
        {
            **asdict(config),
            "compose_file": repo_relative(config.compose_file),
            "artifact_dir": repo_relative(config.artifact_dir),
            "database_url": redact_database_url(config.database_url),
            "runtime_config_path": (
                repo_relative(config.runtime_config_path)
                if config.runtime_config_path is not None
                else None
            ),
        },
    )

    process: subprocess.Popen[str] | None = None
    runtime_log: Any | None = None
    started_services: tuple[str, ...] = ()
    budget_state_after: dict[str, Any] | None = None

    try:
        ensure_required_tools("cargo", "docker", "python3")
        ensure_port_available(config.http_port)

        services_running_before = {
            service: service_running(config, service) for service in RUNTIME_SERVICES
        }
        docker_up = run_command(
            compose_command(config.compose_file, "up", "-d", "postgres", "nats")
        )
        write_text(config.artifact_dir / "docker-up.txt", docker_up.stdout + docker_up.stderr)
        started_services = tuple(
            service
            for service in RUNTIME_SERVICES
            if not services_running_before[service] and service_running(config, service)
        )

        docker_ps = run_command(compose_command(config.compose_file, "ps"))
        write_text(config.artifact_dir / "docker-ps.txt", docker_ps.stdout)

        wait_for_service_health(config, "postgres", "postgres-health-inspect.json")
        wait_for_service_health(config, "nats", "nats-health-inspect.json")
        wait_for_postgres_ready(config)
        fetch_nats_varz(config)
        check_openai_auth_status(config)
        if is_budget_aware_profile(config.profile):
            run_budget_seed_helper(
                config,
                "seed",
                artifact_name="budget-state-before",
            )
        create_database(config)

        process, runtime_log = start_runtime(config)
        wait_for_runtime_ready(config, process)
        wait_for_runtime_log_markers(config, process)

        task_request = build_task_request()
        write_json(config.artifact_dir / "task-request.json", task_request)
        submit_response = fetch_json(
            f"{config.base_url}/api/v1/tasks",
            method="POST",
            payload=task_request,
        )
        if submit_response.status_code != 202:
            raise SmokeHarnessError(
                f"task submission expected HTTP 202, observed {submit_response.status_code}"
            )
        if not isinstance(submit_response.payload, dict):
            raise SmokeHarnessError("task submission response was not a JSON object")
        write_json(config.artifact_dir / "task-submit-response.json", submit_response.payload)
        write_json(
            config.artifact_dir / "task-submit-metadata.json",
            {
                "status_code": submit_response.status_code,
                "headers": submit_response.headers,
            },
        )

        task_id = str(submit_response.payload.get("task_id", "")).strip()
        if not task_id:
            raise SmokeHarnessError("task submission response did not include task_id")

        task_status = wait_for_terminal_task_status(config, task_id)
        task_summary = summarize_task_status(task_status)
        assert_task_summary(
            task_summary,
            expected_provider_kind=config.provider_kind,
            expected_model_id=config.model_id,
            expected_routing_policy=config.routing_policy,
            expected_registered_provider_count=config.registered_provider_count,
            expected_budget_root=config.budget_root or "disabled",
        )
        write_json(config.artifact_dir / "task-result-summary.json", task_summary)

        autonomy_workflows = fetch_json(f"{config.base_url}/api/v1/autonomy/workflows").payload
        write_json(config.artifact_dir / "autonomy-workflows.json", autonomy_workflows)
        assert_workflow_list_contains(autonomy_workflows, task_id)

        autonomy_status = fetch_json(
            f"{config.base_url}/api/v1/autonomy/status/{task_id}"
        ).payload
        write_json(config.artifact_dir / "autonomy-status.json", autonomy_status)
        assert_autonomy_status(autonomy_status, task_id)
        assert_autonomy_step_routing_expectations(
            autonomy_status,
            expected_action=config.expected_step_action,
            expected_tier=config.expected_step_tier,
            required_checkpoints=config.required_step_checkpoints,
        )

        if is_budget_aware_profile(config.profile):
            budget_state_after = run_budget_seed_helper(
                config,
                "fetch",
                artifact_name="budget-state-after",
            )

        smoke_summary = build_smoke_summary(
            config,
            task_id,
            task_summary,
            autonomy_status,
            budget_state_after,
        )
        write_json(config.artifact_dir / "smoke-summary.json", smoke_summary)
        print(json.dumps(smoke_summary, indent=2, sort_keys=True))
        return 0
    except SmokeHarnessError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    finally:
        if process is not None and runtime_log is not None:
            shutdown_runtime(process, runtime_log)
        if started_services:
            docker_stop = run_command(
                compose_command(config.compose_file, "stop", *started_services),
                check=False,
            )
            write_text(
                config.artifact_dir / "docker-stop.txt",
                docker_stop.stdout + docker_stop.stderr,
            )


if __name__ == "__main__":
    raise SystemExit(main())
