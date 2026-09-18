pub mod health;
pub mod metrics;

use alvio_core::AlvioResult;
use tracing_subscriber::{fmt, EnvFilter};

pub use health::{
    get_uptime_secs, health_handler, init_server_start_time, is_draining, metrics_handler,
    ready_handler, set_draining,
};
pub use metrics::{get_metrics, AlvioMetrics};

/// Initializes structured logging for AlvioRelay using `tracing` and `tracing-subscriber`.
pub fn init_telemetry(default_level: &str) -> AlvioResult<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "alvio={default_level},alvio_relay={default_level},info"
        ))
    });

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    init_server_start_time();

    Ok(())
}
