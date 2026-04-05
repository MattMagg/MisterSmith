use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use mister_smith_mcp::{build_smith_compatibility_server, SmithCompatibilityOptions};

fn parse_args(mut options: SmithCompatibilityOptions) -> Result<SmithCompatibilityOptions, String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--repo-root requires a value".to_string())?;
                options.repo_root = PathBuf::from(value);
                options.env_file_path = options.repo_root.join(".env");
            }
            "--codex-config-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--codex-config-path requires a value".to_string())?;
                options.codex_config_path = PathBuf::from(value);
            }
            "--linear-endpoint" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--linear-endpoint requires a value".to_string())?;
                options.linear_endpoint = value;
            }
            "--server-name" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--server-name requires a value".to_string())?;
                options.server_name = value;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: smith-mcp [--repo-root PATH] [--codex-config-path PATH] [--linear-endpoint URL] [--server-name NAME]"
                        .to_string(),
                );
            }
            other => {
                return Err(format!("unrecognized argument: {other}"));
            }
        }
    }

    Ok(options)
}

#[tokio::main]
async fn main() {
    let options = match parse_args(SmithCompatibilityOptions::from_env()) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    let server = match build_smith_compatibility_server(options).await {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to build smith MCP server: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = Arc::clone(&server).serve_stdio().await {
        eprintln!("smith MCP server exited with error: {err}");
        std::process::exit(1);
    }
}
