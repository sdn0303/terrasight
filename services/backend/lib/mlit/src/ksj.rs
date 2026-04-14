//! Client for the 国土数値情報 (KSJ) national land numerical information API.
//!
//! KSJ (国土数値情報ダウンロードサービス) is the MLIT portal for downloading
//! standardised geospatial datasets covering all of Japan. Available datasets
//! include administrative boundaries, railways, roads, land use, and many
//! more. Each dataset is identified by a short code (e.g., `N03` for
//! administrative boundaries, `L01` for land prices).
//!
//! This client is a **Phase 3 stub**. [`KsjClient`] constructs the HTTP client,
//! but no endpoint methods are implemented yet. They will be added when KSJ
//! dataset downloads are integrated into the data pipeline.
//!
//! # Authentication
//!
//! No API key is required. All KSJ download endpoints are publicly accessible.
//!
//! # Planned Endpoints
//!
//! - `get_dataset_list(identifier)` — list available files for a dataset code.
//! - `download_dataset(url)` — stream a KSJ ZIP archive.

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
