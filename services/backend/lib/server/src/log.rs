//! Structured logging initialization with format selection (JSON / Pretty).
//!
//! This module wraps [`tracing_subscriber`] to provide a single entry point for
//! global subscriber setup.  The same release binary can emit human-readable
//! output in development and newline-delimited JSON in production without
//! recompilation — the format is selected at runtime via [`LogFormat`].
//!
//! ## Usage
//!
//! ```rust,ignore
//! use terrasight_server::log::{LogFormat, init_global_logger};
//!
//! // Read from an environment variable at startup
//! let fmt = std::env::var("LOG_FORMAT")
//!     .map(|s| LogFormat::from_str_lossy(&s))
//!     .unwrap_or(LogFormat::Pretty);
//!
//! init_global_logger(fmt, Some("terrasight_api=debug,info"));
//! ```
//!
//! Call [`init_global_logger`] exactly once before any `tracing` macros are
//! invoked.  Subsequent calls log a `WARN` and return without effect, to
//! allow test harnesses that run multiple `#[tokio::test]` functions in the
//! same process.

mod logger;

pub use logger::{LogFormat, init_global_logger};
