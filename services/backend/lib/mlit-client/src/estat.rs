use std::time::Duration;

use crate::config::MlitConfig;
use crate::error::MlitError;

/// Client for the e-Stat (政府統計の総合窓口) API.
///
/// Base URL: `https://api.e-stat.go.jp/rest/3.0/app`
///
/// Requires an application ID.
// Phase 3 will add endpoint methods that consume these fields.
pub struct EstatClient {
    _http: reqwest::Client,
    _app_id: String,
    _base_url: String,
}

const DEFAULT_BASE_URL: &str = "https://api.e-stat.go.jp/rest/3.0/app";

impl EstatClient {
    /// Create a new client.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError::Config`] if the app ID is not configured.
    pub fn new(config: &MlitConfig) -> Result<Self, MlitError> {
        let app_id = config
            .estat_app_id
            .clone()
            .ok_or_else(|| MlitError::Config("ESTAT_APP_ID is required".into()))?;

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(MlitError::Http)?;

        Ok(Self {
            _http: http,
            _app_id: app_id,
            _base_url: DEFAULT_BASE_URL.to_string(),
        })
    }
}

// Phase 3 methods:
// - get_stats_data(stats_data_id)
