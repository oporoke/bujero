mod config;
mod telemetry;

use axum::{Router, extract::Request, routing::get};
use config::Config;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;

#[tokio::main]
async fn main() {
    // Load .env if present (development). Real environment variables always win.
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error: {e}");
        std::process::exit(1);
    });

    telemetry::init(config.log_format);

    let addr = config.addr();
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        tracing::error!(%addr, error = %e, "failed to bind");
        std::process::exit(1);
    });

    tracing::info!(%addr, env = ?config.app_env, "listening");

    axum::serve(listener, app()).await.expect("server error");
}

/// Builds the application router. Kept separate from `main` so tests can use it (Step 10).
fn app() -> Router {
    Router::new().route("/health", get(health)).layer(
        // ServiceBuilder applies layers top to bottom: the first one sees the request first.
        ServiceBuilder::new()
            // 1. Give every request an x-request-id (keeps one the client already sent).
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            // 2. Open a log span per request, tagged with that id; log status + latency.
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|req: &Request| {
                        let request_id = req
                            .headers()
                            .get("x-request-id")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("-");
                        tracing::info_span!(
                            "http",
                            method = %req.method(),
                            uri = %req.uri(),
                            request_id = %request_id,
                        )
                    })
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Millis),
                    ),
            )
            // 3. Copy x-request-id onto the response so the client can quote it.
            .layer(PropagateRequestIdLayer::x_request_id()),
    )
}

/// Liveness check: returns 200 OK with body "ok".
async fn health() -> &'static str {
    "ok"
}
