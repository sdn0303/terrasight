//! Configuration types for the `terrasight-mlit` API clients.
//!
//! [`MlitConfig`] is the single configuration struct shared by all API clients
//! in this crate. Construct it from environment variables using `envy` or
//! `serde_env`, or build it manually for tests.
//!
//! # Environment Variables
//!
//! | Variable | Client | Required |
//! |---|---|---|
//! | `REINFOLIB_API_KEY` | `reinfolib::ReinfolibClient` (feature `reinfolib`) | Yes |
//! | `ESTAT_APP_ID` | `estat::EstatClient` (feature `estat`) | Yes |
//! | `MLIT_REQUEST_TIMEOUT_SECS` | All clients | No (default: 30) |

use serde::Deserialize;

/// Default HTTP request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Configuration for government API clients.
///
/// All fields are optional — only enable the APIs you need.
///
/// # Environment Variables
///
/// | Variable | Description |
/// |---|---|
/// | `REINFOLIB_API_KEY` | reinfolib API key |
/// | `ESTAT_APP_ID` | e-Stat application ID |
/// | `MLIT_REQUEST_TIMEOUT_SECS` | HTTP timeout (default: 30) |
#[derive(Debug, Clone, Deserialize)]
pub struct MlitConfig {
    /// reinfolib API key.
    #[serde(default)]
    pub reinfolib_api_key: Option<String>,

    /// e-Stat application ID.
    #[serde(default)]
    pub estat_app_id: Option<String>,

    /// HTTP request timeout in seconds (default: 30).
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

impl Default for MlitConfig {
    fn default() -> Self {
        Self {
            reinfolib_api_key: None,
            estat_app_id: None,
            request_timeout_secs: default_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_timeout_is_30() {
        let config = MlitConfig::default();
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn default_has_no_api_keys() {
        let config = MlitConfig::default();
        assert!(config.reinfolib_api_key.is_none());
        assert!(config.estat_app_id.is_none());
    }
}
