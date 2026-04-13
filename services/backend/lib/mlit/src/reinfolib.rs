//! Client for the 不動産情報ライブラリ (Reinfolib) API.
//!
//! Reinfolib is the MLIT real estate information platform. It provides:
//!
//! - **Transaction price data** — actual sale prices for land, condominiums,
//!   and detached houses, reported per quarter at zoom-14 tile granularity
//!   (endpoints `XPT001`, `XIT001`).
//! - **Official land appraisal points** — 地価公示 and 地価調査 survey data,
//!   reported annually (endpoint `XPT002`).
//! - **Urban planning layers** — 用途地域 zoning polygons (`XKT002`), school
//!   facilities (`XKT006`), medical facilities (`XKT010`), and disaster-hazard
//!   areas (`XKT016`).
//!
//! All tile-based endpoints return GeoJSON `FeatureCollection` responses.
//! Use [`ReinfolibClient`] to query any of these endpoints.
//!
//! # Authentication
//!
//! An API key (`Ocp-Apim-Subscription-Key` header) is required. Obtain one
//! from the [MLIT developer portal](https://www.reinfolib.mlit.go.jp/) and
//! set `REINFOLIB_API_KEY` in the environment, then pass it via
//! [`crate::config::MlitConfig`].
//!
//! # Examples
//!
//! ```no_run
//! use terrasight_mlit::config::MlitConfig;
//! use terrasight_mlit::reinfolib::ReinfolibClient;
//!
//! # async fn example() -> Result<(), terrasight_mlit::error::MlitError> {
//! let config = MlitConfig {
//!     reinfolib_api_key: Some("your-api-key".into()),
//!     ..MlitConfig::default()
//! };
//! let client = ReinfolibClient::new(&config)?;
//!
//! // Fetch official land price points around Tokyo Station (2024)
//! let features = client
//!     .get_land_prices(139.766, 35.680, 139.768, 35.682, 2024)
//!     .await?;
//! println!("{} land price features", features.len());
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use crate::config::MlitConfig;
use crate::error::MlitError;
use crate::types::GeoJsonResponse;

/// Client for the 不動産情報ライブラリ (Real Estate Information Library) API.
///
/// Base URL: `https://www.reinfolib.mlit.go.jp/ex-api/external`
///
/// Requires an API key obtained from the MLIT developer portal.
/// All tile-based endpoints use zoom level 14 by default.
pub struct ReinfolibClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

const DEFAULT_BASE_URL: &str = "https://www.reinfolib.mlit.go.jp/ex-api/external";

/// MLIT API subscription key header name.
const API_KEY_HEADER: &str = "Ocp-Apim-Subscription-Key";

/// Query parameter: response format key.
const PARAM_RESPONSE_FORMAT: &str = "response_format";
/// Query parameter: GeoJSON response format value.
const PARAM_VALUE_GEOJSON: &str = "geojson";

impl ReinfolibClient {
    /// Create a new client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError::Config`] if the API key is not configured.
    pub fn new(config: &MlitConfig) -> Result<Self, MlitError> {
        let api_key = config
            .reinfolib_api_key
            .clone()
            .ok_or_else(|| MlitError::Config("REINFOLIB_API_KEY is required".into()))?;

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(MlitError::Http)?;

        Ok(Self {
            http,
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
        })
    }

    /// Override the base URL (useful for testing with a mock server).
    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

impl ReinfolibClient {
    /// Default zoom level for tile-based queries.
    const DEFAULT_ZOOM: u8 = 14;

    /// Fetch GeoJSON features from a tile-based endpoint, merging results across
    /// all tiles that cover the given bounding box.
    ///
    /// `extra_params` are appended as additional query parameters alongside the
    /// standard `response_format`, `z`, `x`, `y` parameters.
    async fn fetch_tile_features(
        &self,
        endpoint: &str,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        extra_params: &[(&str, &str)],
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        let tiles =
            terrasight_geo::tile::bbox_to_tiles(west, south, east, north, Self::DEFAULT_ZOOM);
        tracing::debug!(
            endpoint = endpoint,
            tile_count = tiles.len(),
            zoom = Self::DEFAULT_ZOOM,
            "fetching tile features"
        );
        let mut all_features = Vec::new();

        for tile in &tiles {
            let url = format!("{}/{}", self.base_url, endpoint);
            let mut params = vec![
                (
                    PARAM_RESPONSE_FORMAT.to_string(),
                    PARAM_VALUE_GEOJSON.to_string(),
                ),
                ("z".to_string(), tile.z.to_string()),
                ("x".to_string(), tile.x.to_string()),
                ("y".to_string(), tile.y.to_string()),
            ];
            for (k, v) in extra_params {
                params.push(((*k).to_string(), (*v).to_string()));
            }

            let response = crate::retry::request_with_retry(
                &self.http,
                &url,
                &params,
                Some((API_KEY_HEADER, &self.api_key)),
                "reinfolib",
            )
            .await?;
            let geojson: GeoJsonResponse = response
                .json()
                .await
                .map_err(|e| MlitError::Parse(format!("Failed to parse GeoJSON response: {e}")))?;
            tracing::debug!(
                endpoint = endpoint,
                z = tile.z,
                x = tile.x,
                y = tile.y,
                feature_count = geojson.features.len(),
                "tile response received"
            );

            for feature in geojson.features {
                all_features.push(
                    serde_json::to_value(&feature).map_err(|e| {
                        MlitError::Parse(format!("Failed to serialize feature: {e}"))
                    })?,
                );
            }
        }

        tracing::debug!(
            endpoint = endpoint,
            total_features = all_features.len(),
            "tile features merged"
        );
        Ok(all_features)
    }

    // -----------------------------------------------------------------------
    // Public endpoint methods
    // -----------------------------------------------------------------------

    /// XPT001: Get real estate transaction price points within a bounding box.
    ///
    /// Returns GeoJSON features for all transactions within the given bbox for
    /// the specified time range.
    ///
    /// # Parameters
    ///
    /// - `from` / `to`: Five-character period strings in `YYYYN` format, where
    ///   `YYYY` is the year and `N` is the quarter (1–4). Example: `"20241"` for
    ///   2024 Q1.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_transaction_prices(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        from: &str,
        to: &str,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        self.fetch_tile_features(
            "XPT001",
            west,
            south,
            east,
            north,
            &[("from", from), ("to", to)],
        )
        .await
    }

    /// XPT002: Get official land price survey points within a bounding box.
    ///
    /// Returns GeoJSON features for all land price survey points (地価公示 /
    /// 地価調査) within the given bbox for the specified year.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_land_prices(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        year: u16,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        let year_str = year.to_string();
        self.fetch_tile_features("XPT002", west, south, east, north, &[("year", &year_str)])
            .await
    }

    /// XIT001: Get bulk transaction data for a prefecture (non-tile endpoint).
    ///
    /// Unlike the tile-based endpoints, XIT001 returns all transactions for a
    /// given prefecture, year, and quarter as a JSON array (not GeoJSON).
    ///
    /// # Parameters
    ///
    /// - `area`: Two-digit prefecture code, e.g. `"13"` for Tokyo.
    ///   Comma-separated values are accepted by the API.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_transaction_data(
        &self,
        year: u16,
        quarter: u8,
        area: &str,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        let url = format!("{}/XIT001", self.base_url);
        let params = vec![
            ("year".to_string(), year.to_string()),
            ("quarter".to_string(), quarter.to_string()),
            ("area".to_string(), area.to_string()),
        ];
        let resp = crate::retry::request_with_retry(
            &self.http,
            &url,
            &params,
            Some((API_KEY_HEADER, &self.api_key)),
            "reinfolib",
        )
        .await?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MlitError::Parse(format!("Failed to parse XIT001 response: {e}")))?;
        // XIT001 returns a JSON array, not a GeoJSON FeatureCollection.
        match data {
            serde_json::Value::Array(arr) => Ok(arr),
            other => Ok(vec![other]),
        }
    }

    /// XKT002: Get urban-planning zoning polygons within a bounding box.
    ///
    /// Returns GeoJSON polygon features representing 用途地域 (use-area zones).
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_zoning(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        self.fetch_tile_features("XKT002", west, south, east, north, &[])
            .await
    }

    /// XKT006: Get school facility points within a bounding box.
    ///
    /// Returns GeoJSON point features for elementary, middle, and high schools.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_schools(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        self.fetch_tile_features("XKT006", west, south, east, north, &[])
            .await
    }

    /// XKT010: Get medical facility points within a bounding box.
    ///
    /// Returns GeoJSON point features for hospitals and clinics.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_medical(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        self.fetch_tile_features("XKT010", west, south, east, north, &[])
            .await
    }

    /// XKT016: Get disaster-hazard area polygons within a bounding box.
    ///
    /// Returns GeoJSON polygon features for designated disaster-hazard zones
    /// (disaster risk areas covering flood, landslide, tsunami, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP, rate-limit, or parse failures.
    pub async fn get_hazard_areas(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError> {
        self.fetch_tile_features("XKT016", west, south, east, north, &[])
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config() -> MlitConfig {
        MlitConfig {
            reinfolib_api_key: Some("test-key".into()),
            estat_app_id: None,
            request_timeout_secs: 5,
        }
    }

    fn make_geojson_body(features: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "type": "FeatureCollection",
            "features": features
        })
    }

    // -----------------------------------------------------------------------
    // get_land_prices (XPT002) — happy path
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_land_prices_success() {
        let mock_server = MockServer::start().await;

        let body = make_geojson_body(serde_json::json!([{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [139.76, 35.68]},
            "properties": {
                "point_id": "L01-001",
                "u_current_year_price_ja": "1200000"
            }
        }]));

        Mock::given(method("GET"))
            .and(path("/XPT002"))
            .and(query_param("response_format", "geojson"))
            .and(header("Ocp-Apim-Subscription-Key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1..)
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let features = client
            .get_land_prices(139.766, 35.680, 139.768, 35.682, 2024)
            .await
            .expect("get_land_prices should succeed");

        assert!(!features.is_empty(), "should return at least one feature");
    }

    // -----------------------------------------------------------------------
    // get_transaction_prices (XPT001) — happy path
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_transaction_prices_success() {
        let mock_server = MockServer::start().await;

        let body = make_geojson_body(serde_json::json!([{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [139.76, 35.68]},
            "properties": {
                "u_transaction_price_total_ja": "45000000",
                "land_type_name_ja": "中古マンション"
            }
        }]));

        Mock::given(method("GET"))
            .and(path("/XPT001"))
            .and(query_param("response_format", "geojson"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1..)
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let features = client
            .get_transaction_prices(139.766, 35.680, 139.768, 35.682, "20241", "20244")
            .await
            .expect("get_transaction_prices should succeed");

        assert!(!features.is_empty());
    }

    // -----------------------------------------------------------------------
    // Retry on HTTP 429
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_retry_on_429() {
        let mock_server = MockServer::start().await;

        let success_body = make_geojson_body(serde_json::json!([{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [139.76, 35.68]},
            "properties": {}
        }]));

        // First two requests return 429, third returns 200.
        Mock::given(method("GET"))
            .and(path("/XPT002"))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/XPT002"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&success_body))
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let features = client
            .get_land_prices(139.766, 35.680, 139.768, 35.682, 2024)
            .await
            .expect("should succeed after retries");

        assert!(!features.is_empty());
    }

    // -----------------------------------------------------------------------
    // API error 500 → MlitError::Api
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_api_error_500() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/XPT002"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let err = client
            .get_land_prices(139.766, 35.680, 139.768, 35.682, 2024)
            .await
            .expect_err("should return an error on HTTP 500");

        match err {
            MlitError::Api { status, .. } => {
                assert_eq!(status, 500, "error status should be 500");
            }
            other => panic!("expected MlitError::Api, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Empty FeatureCollection (data not found for tile)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_empty_feature_collection() {
        let mock_server = MockServer::start().await;

        let body = make_geojson_body(serde_json::json!([]));

        Mock::given(method("GET"))
            .and(path("/XKT002"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let features = client
            .get_zoning(139.766, 35.680, 139.768, 35.682)
            .await
            .expect("empty response should not be an error");

        assert!(
            features.is_empty(),
            "should return empty vec for empty tile"
        );
    }

    // -----------------------------------------------------------------------
    // XIT001 non-tile endpoint
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_transaction_data_success() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!([
            {"Type": "中古マンション", "TradePrice": "45000000", "Area": "65"},
            {"Type": "土地", "TradePrice": "90000000", "Area": "120"}
        ]);

        Mock::given(method("GET"))
            .and(path("/XIT001"))
            .and(query_param("year", "2024"))
            .and(query_param("quarter", "1"))
            .and(query_param("area", "13"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let client = ReinfolibClient::new(&test_config())
            .expect("client creation should succeed")
            .with_base_url(mock_server.uri());

        let records = client
            .get_transaction_data(2024, 1, "13")
            .await
            .expect("get_transaction_data should succeed");

        assert_eq!(records.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Missing API key
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_fails_without_api_key() {
        let config = MlitConfig {
            reinfolib_api_key: None,
            estat_app_id: None,
            request_timeout_secs: 5,
        };
        let result = ReinfolibClient::new(&config);
        assert!(result.is_err(), "should fail without API key");
    }
}
