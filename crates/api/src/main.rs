mod config;

use axum::{Router, routing::get};
use config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load .env if present (development). Real environment variables always win.
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error: {e}");
        std::process::exit(1);
    });

    let addr = config.addr();
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!("failed to bind {addr}: {e}");
        std::process::exit(1);
    });

    println!("listening on {addr} ({:?})", config.app_env);

    axum::serve(listener, app()).await.expect("server error");
}

/// Builds the application router. Kept separate from `main` so tests can use it (Step 10).
fn app() -> Router {
    Router::new().route("/health", get(health))
}

/// Liveness check: returns 200 OK with body "ok".
async fn health() -> &'static str {
    "ok"
}
