use std::path::PathBuf;

use axum::Router;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub fn binary_path() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_mister-smith")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_mister-smith should be set for integration tests")
}

pub async fn spawn_mock_server(app: Router) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock server should bind");
    let address = listener.local_addr().expect("mock server should have an address");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("mock server should run");
    });
    (format!("http://{address}"), handle)
}
