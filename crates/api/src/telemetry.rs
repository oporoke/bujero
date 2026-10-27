use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::LogFormat;

/// Installs the global log subscriber. Call once, at startup.
/// Log levels come from RUST_LOG; default is `api=info,tower_http=info`.
pub fn init(format: LogFormat) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("api=info,tower_http=info"));

    let registry = tracing_subscriber::registry().with(filter);

    match format {
        LogFormat::Pretty => registry.with(fmt::layer()).init(),
        LogFormat::Json => registry.with(fmt::layer().json()).init(),
    }
}
