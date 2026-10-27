use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = app();

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind 0.0.0.0:3000");

    println!("listening on {}", listener.local_addr().expect("local addr"));

    axum::serve(listener, app).await.expect("server error");
}

/// Builds the application router. Kept separate from `main` so tests can use it (Step 10).
fn app() -> Router {
    Router::new().route("/health", get(health))
}

/// Liveness check: returns 200 OK with body "ok".
async fn health() -> &'static str {
    "ok"
}
