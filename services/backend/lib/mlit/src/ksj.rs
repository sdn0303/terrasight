use std::time::Duration;

use crate::config::MlitConfig;
use crate::error::MlitError;

/// Client for the 国土数値情報 (National Land Numerical Information) download API.
///
/// Base URL: `https://nlftp.mlit.go.jp/ksj/api/1.0b/index.php/app`
///
/// No authentication required.
pub struct KsjClient {
    _http: reqwest::Client,
    _base_url: String,
}

const DEFAULT_BASE_URL: &str = "https://nlftp.mlit.go.jp/ksj/api/1.0b/index.php/app";

impl KsjClient {
    /// Create a new client.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError::Http`] if the HTTP client fails to build.
    pub fn new(config: &MlitConfig) -> Result<Self, MlitError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(MlitError::Http)?;

        Ok(Self {
            _http: http,
            _base_url: DEFAULT_BASE_URL.to_string(),
        })
    }
}
