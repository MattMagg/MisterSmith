use std::{env, fs, path::PathBuf, process::Command};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn main() {
    build_runtime_sidecar();
    tauri_build::build()
}

fn build_runtime_sidecar() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let repo_root = manifest_dir
        .join("../../..")
        .canonicalize()
        .expect("failed to resolve repo root");
    let target = env::var("TARGET").expect("missing target triple");
    let profile = env::var("PROFILE").expect("missing build profile");
    let binaries_dir = manifest_dir.join("binaries");
    let built_binary_name = if cfg!(windows) {
        "mister-smith.exe"
    } else {
        "mister-smith"
    };
    let bundled_binary_name = if cfg!(windows) {
        format!("mister-smith-runtime-{target}.exe")
    } else {
        format!("mister-smith-runtime-{target}")
    };
    let built_binary_path = repo_root
        .join("target")
        .join(&target)
        .join(&profile)
        .join(built_binary_name);
    let bundled_binary_path = binaries_dir.join(bundled_binary_name);

    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join("Cargo.lock").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root
            .join("crates/mister-smith-app/Cargo.toml")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join("crates/mister-smith-app/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join("deploy/docker-compose.yml").display()
    );

    let mut command = Command::new("cargo");
    command
        .current_dir(&repo_root)
        .arg("build")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(repo_root.join("Cargo.toml"))
        .arg("--package")
        .arg("mister-smith-app")
        .arg("--bin")
        .arg("mister-smith")
        .arg("--target")
        .arg(&target);

    if profile == "release" {
        command.arg("--release");
    }

    let status = command
        .status()
        .expect("failed to build mister-smith runtime sidecar");
    if !status.success() {
        panic!("mister-smith runtime sidecar build failed with status {status}");
    }

    fs::create_dir_all(&binaries_dir).expect("failed to create sidecar binaries dir");
    fs::copy(&built_binary_path, &bundled_binary_path).unwrap_or_else(|error| {
        panic!(
            "failed to copy runtime sidecar from {} to {}: {error}",
            built_binary_path.display(),
            bundled_binary_path.display()
        )
    });

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&bundled_binary_path)
            .expect("failed to stat bundled runtime sidecar")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&bundled_binary_path, permissions)
            .expect("failed to mark bundled runtime sidecar executable");
    }
}
