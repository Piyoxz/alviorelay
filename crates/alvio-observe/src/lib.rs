use alvio_core::AlvioResult;
use tracing_subscriber::{fmt, EnvFilter};

/// Initializes structured logging for AlvioRelay using `tracing` and `tracing-subscriber`.
pub fn init_telemetry(default_level: &str) -> AlvioResult<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("alvio={default_level},alvio_relay={default_level},info")));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    Ok(())
}
