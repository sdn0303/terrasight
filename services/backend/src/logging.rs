//! Thin wrapper that delegates to [`realestate_telemetry`].
//!
//! Keeps `main.rs` imports stable while the underlying telemetry
//! implementation evolves independently in the `lib/telemetry` crate.

use crate::config::Config;
use realestate_telemetry::log::LogFormat;

/// Initialize structured logging based on application configuration.
///
/// Delegates to [`realestate_telemetry::log::init_global_logger`].
pub fn init(config: &Config) {
    let format = config
        .rust_log_format
        .as_deref()
        .map(LogFormat::from_str_lossy)
        .unwrap_or(LogFormat::Pretty);

    realestate_telemetry::log::init_global_logger(format);
}
