//! Client for the J-SHIS (地震ハザードステーション) API.
//!
//! Provides access to seismic hazard, surface ground amplification, and
//! landslide terrain information published by NIED (防災科学技術研究所).
//!
//! No API key is required. All endpoints are publicly accessible.
//!
//! # Example
//!
//! ```ignore
//! use terrasight_mlit::config::MlitConfig;
//! use terrasight_mlit::jshis::JshisClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = MlitConfig::default();
//!     let client = JshisClient::new(&config).expect("client creation should succeed");
//!
//!     // Tokyo Station: longitude 139.7671, latitude 35.6812
//!     let hazard = client
//!         .get_seismic_hazard(139.7671, 35.6812)
//!         .await
//!         .expect("seismic hazard query should succeed");
//!
//!     println!("30-year P(≥5弱): {:?}", hazard.prob_level5_low_30yr);
//! }
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::MlitConfig;
use crate::error::MlitError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_BASE_URL: &str = "https://www.j-shis.bosai.go.jp/map/api";

/// J-SHIS probabilistic seismic hazard map version.
const PSHM_VERSION: &str = "Y2024";
/// J-SHIS hazard case (average scenario).
const PSHM_CASE: &str = "AVR";
/// J-SHIS earthquake code (total of all earthquake sources).
const PSHM_EQCODE: &str = "TTL_MTTL";

/// J-SHIS surface stratigraphy model version.
const SSTRCT_VERSION: &str = "V3";

/// J-SHIS coordinate reference system (WGS-84).
const EPSG_WGS84: &str = "4326";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Seismic hazard information for a 250 m mesh containing the queried point.
///
/// Probabilities express the likelihood that seismic intensity **meets or
/// exceeds** a given threshold within the next 30 years, based on the
/// J-SHIS Y2024 probabilistic model using all earthquake sources (TTL_MTTL).
///
/// # GeoJSON structure
///
/// The API returns a `FeatureCollection`. This struct captures the first
/// feature's `properties` object. Coordinates follow RFC 7946
/// (`[longitude, latitude]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeismicHazardResponse {
    /// Raw GeoJSON features from the API response.
    ///
    /// Each feature represents one 250 m mesh cell. In practice a single
    /// point query returns exactly one feature, but the field is kept as a
    /// `Vec` to faithfully represent the GeoJSON contract.
    pub features: Vec<serde_json::Value>,

    /// 30-year probability of seismic intensity ≥ 5弱 (0.0–1.0).
    ///
    /// Parsed from the first feature's properties; `None` if the response
    /// contained no features or the field was absent.
    pub prob_level5_low_30yr: Option<f64>,

    /// 30-year probability of seismic intensity ≥ 5強 (0.0–1.0).
    pub prob_level5_high_30yr: Option<f64>,

    /// 30-year probability of seismic intensity ≥ 6弱 (0.0–1.0).
    pub prob_level6_low_30yr: Option<f64>,

    /// 30-year probability of seismic intensity ≥ 6強 (0.0–1.0).
    pub prob_level6_high_30yr: Option<f64>,
}

/// Surface ground information for the 250 m mesh containing the queried point.
///
/// Provides S-wave velocity (AVS30) and site amplification factor derived from
/// the J-SHIS V3 surface stratigraphy model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceGroundResponse {
    /// Raw GeoJSON features from the API response.
    pub features: Vec<serde_json::Value>,

    /// Average S-wave velocity in the top 30 m (AVS30, m/s).
    ///
    /// Lower values indicate softer soil; higher values indicate harder rock.
    /// `None` if the point falls outside the coverage area.
    pub avs30: Option<f64>,

    /// Site amplification factor relative to engineering bedrock.
    ///
    /// Values > 1.0 indicate amplification. `None` if outside coverage.
    pub amplification_factor: Option<f64>,
}

/// Landslide terrain check result for the queried point.
///
/// Indicates whether the point falls within a landslide-prone area as
/// identified in the NIED landslide terrain database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandslideResponse {
    /// `true` if the point is within a landslide terrain polygon.
    pub is_containing: bool,

    /// Raw API response body preserved for downstream use.
    pub raw: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Internal deserialization helpers
// ---------------------------------------------------------------------------

/// Deserialization target for the J-SHIS `isContaining` JSON response.
#[derive(Debug, Deserialize)]
struct IsContainingRaw {
    /// 0 = not containing, 1 = containing.
    #[serde(rename = "isContaining")]
    is_containing: u8,
}

/// Deserialization target for a J-SHIS GeoJSON `FeatureCollection`.
#[derive(Debug, Deserialize)]
struct JshisFeatureCollection {
    #[serde(default)]
    features: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for the J-SHIS public API.
///
/// No API key is required. Construct with [`JshisClient::new`] and call the
/// query methods directly.
///
/// # Retry behaviour
///
/// All methods retry up to 3 times with exponential backoff (0 s, 2 s, 4 s)
/// on HTTP 429 or transport errors. Non-2xx responses other than 429 are
/// returned immediately as [`MlitError::Api`].
pub struct JshisClient {
    http: reqwest::Client,
    base_url: String,
}

impl JshisClient {
    /// Create a new client from a shared [`MlitConfig`].
    ///
    /// J-SHIS requires no API key; only `config.request_timeout_secs` is used.
    /// This constructor signature is consistent with [`crate::reinfolib::ReinfolibClient::new`].
    ///
    /// # Errors
    ///
    /// Returns [`MlitError::Http`] if the underlying `reqwest` client cannot
    /// be built (e.g., invalid TLS configuration).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terrasight_mlit::config::MlitConfig;
    /// use terrasight_mlit::jshis::JshisClient;
    ///
    /// let config = MlitConfig::default();
    /// let client = JshisClient::new(&config).expect("client should build");
    /// ```
    pub fn new(config: &MlitConfig) -> Result<Self, MlitError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(MlitError::Http)?;

        Ok(Self {
            http,
            base_url: DEFAULT_BASE_URL.to_string(),
        })
    }

    /// Override the base URL. Only compiled in `#[cfg(test)]` to allow
    /// pointing the client at a wiremock server.
    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    // -----------------------------------------------------------------------
    // Public endpoint methods
    // -----------------------------------------------------------------------

    /// Query probabilistic seismic hazard for a point (pshm endpoint).
    ///
    /// Returns 30-year exceedance probabilities for seismic intensity levels
    /// 5弱, 5強, 6弱, and 6強 for the 250 m mesh cell containing `(lng, lat)`.
    ///
    /// Uses J-SHIS model version `Y2024`, average scenario (`AVR`), with all
    /// earthquake sources (`TTL_MTTL`).
    ///
    /// # Parameters
    ///
    /// - `lng`: Longitude in decimal degrees (WGS-84), e.g. `139.7671`.
    /// - `lat`: Latitude in decimal degrees (WGS-84), e.g. `35.6812`.
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP transport, rate-limit, or parse failures.
    pub async fn get_seismic_hazard(
        &self,
        lng: f64,
        lat: f64,
    ) -> Result<SeismicHazardResponse, MlitError> {
        let url = format!(
            "{}/pshm/{}/{}/{}/meshinfo.geojson",
            self.base_url, PSHM_VERSION, PSHM_CASE, PSHM_EQCODE,
        );
        let position = format!("{lng},{lat}");
        let params = [("position", position.as_str()), ("epsg", EPSG_WGS84)];

        tracing::debug!(lng, lat, "querying J-SHIS seismic hazard");

        let resp =
            crate::retry::request_with_retry(&self.http, &url, &params, None, "jshis").await?;
        let collection: JshisFeatureCollection = resp
            .json()
            .await
            .map_err(|e| MlitError::Parse(format!("Failed to parse pshm GeoJSON: {e}")))?;

        let (prob5l, prob5h, prob6l, prob6h) = extract_seismic_probs(collection.features.first());

        tracing::debug!(
            lng,
            lat,
            feature_count = collection.features.len(),
            "J-SHIS seismic hazard response received"
        );

        Ok(SeismicHazardResponse {
            features: collection.features,
            prob_level5_low_30yr: prob5l,
            prob_level5_high_30yr: prob5h,
            prob_level6_low_30yr: prob6l,
            prob_level6_high_30yr: prob6h,
        })
    }

    /// Query surface ground amplification for a point (sstrct endpoint).
    ///
    /// Returns the AVS30 S-wave velocity and site amplification factor for
    /// the 250 m mesh cell containing `(lng, lat)`, based on the J-SHIS V3
    /// surface stratigraphy model.
    ///
    /// # Parameters
    ///
    /// - `lng`: Longitude in decimal degrees (WGS-84).
    /// - `lat`: Latitude in decimal degrees (WGS-84).
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP transport, rate-limit, or parse failures.
    pub async fn get_surface_ground(
        &self,
        lng: f64,
        lat: f64,
    ) -> Result<SurfaceGroundResponse, MlitError> {
        let url = format!(
            "{}/sstrct/{}/meshinfo.geojson",
            self.base_url, SSTRCT_VERSION,
        );
        let position = format!("{lng},{lat}");
        let params = [("position", position.as_str()), ("epsg", EPSG_WGS84)];

        tracing::debug!(lng, lat, "querying J-SHIS surface ground");

        let resp =
            crate::retry::request_with_retry(&self.http, &url, &params, None, "jshis").await?;
        let collection: JshisFeatureCollection = resp
            .json()
            .await
            .map_err(|e| MlitError::Parse(format!("Failed to parse sstrct GeoJSON: {e}")))?;

        let (avs30, amplification_factor) =
            extract_surface_ground_props(collection.features.first());

        tracing::debug!(
            lng,
            lat,
            feature_count = collection.features.len(),
            "J-SHIS surface ground response received"
        );

        Ok(SurfaceGroundResponse {
            features: collection.features,
            avs30,
            amplification_factor,
        })
    }

    /// Check whether a point falls within a landslide terrain polygon.
    ///
    /// Returns `true` when the point is inside a NIED landslide terrain area,
    /// `false` otherwise.
    ///
    /// # Parameters
    ///
    /// - `lng`: Longitude in decimal degrees (WGS-84).
    /// - `lat`: Latitude in decimal degrees (WGS-84).
    ///
    /// # Errors
    ///
    /// Returns [`MlitError`] on HTTP transport, rate-limit, or parse failures.
    pub async fn is_landslide_terrain(
        &self,
        lng: f64,
        lat: f64,
    ) -> Result<LandslideResponse, MlitError> {
        let url = format!("{}/landslide/isContaining.json", self.base_url);
        let position = format!("{lng},{lat}");
        let params = [("position", position.as_str()), ("epsg", EPSG_WGS84)];

        tracing::debug!(lng, lat, "querying J-SHIS landslide terrain");

        let resp =
            crate::retry::request_with_retry(&self.http, &url, &params, None, "jshis").await?;

        // Parse raw JSON first so we can preserve it and also decode the flag.
        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MlitError::Parse(format!("Failed to parse landslide response: {e}")))?;

        let is_containing_raw: IsContainingRaw =
            serde_json::from_value(raw.clone()).map_err(|e| {
                MlitError::Parse(format!(
                    "Missing 'isContaining' field in landslide response: {e}"
                ))
            })?;

        tracing::debug!(
            lng,
            lat,
            is_containing = is_containing_raw.is_containing,
            "J-SHIS landslide response received"
        );

        Ok(LandslideResponse {
            is_containing: is_containing_raw.is_containing != 0,
            raw,
        })
    }
}

// ---------------------------------------------------------------------------
// Property extraction helpers (pure functions, easy to unit-test)
// ---------------------------------------------------------------------------

/// Extract the four 30-year exceedance probabilities from the first GeoJSON
/// feature returned by the pshm endpoint.
///
/// J-SHIS field names for the AVR/TTL_MTTL scenario:
/// - `AVR_TTL_MTTL_I50_30`: P(≥ 5弱) in 30 yr
/// - `AVR_TTL_MTTL_I55_30`: P(≥ 5強) in 30 yr
/// - `AVR_TTL_MTTL_I60_30`: P(≥ 6弱) in 30 yr
/// - `AVR_TTL_MTTL_I65_30`: P(≥ 6強) in 30 yr
fn extract_seismic_probs(
    feature: Option<&serde_json::Value>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let props = match feature.and_then(|f| f.get("properties")) {
        Some(p) => p,
        None => return (None, None, None, None),
    };

    let get_prob =
        |key: &str| -> Option<f64> { props.get(key).and_then(serde_json::Value::as_f64) };

    (
        get_prob("AVR_TTL_MTTL_I50_30"),
        get_prob("AVR_TTL_MTTL_I55_30"),
        get_prob("AVR_TTL_MTTL_I60_30"),
        get_prob("AVR_TTL_MTTL_I65_30"),
    )
}

/// Extract AVS30 and amplification factor from the first GeoJSON feature
/// returned by the sstrct endpoint.
///
/// J-SHIS field names:
/// - `AVS30`: Average S-wave velocity in top 30 m (m/s)
/// - `Z1400`: Depth to VS=1400 m/s (proxy used for amplification; actual
///   factor field is `AMP` in some API versions)
fn extract_surface_ground_props(feature: Option<&serde_json::Value>) -> (Option<f64>, Option<f64>) {
    let props = match feature.and_then(|f| f.get("properties")) {
        Some(p) => p,
        None => return (None, None),
    };

    let avs30 = props.get("AVS30").and_then(serde_json::Value::as_f64);

    // The J-SHIS sstrct API returns the amplification factor as "AMP".
    // Fall back to None gracefully if the field is absent.
    let amp = props.get("AMP").and_then(serde_json::Value::as_f64);

    (avs30, amp)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_client(base_url: String) -> JshisClient {
        let config = MlitConfig {
            request_timeout_secs: 5,
            ..MlitConfig::default()
        };
        JshisClient::new(&config)
            .expect("JshisClient should build in tests")
            .with_base_url(base_url)
    }

    fn seismic_geojson(prob5l: f64, prob5h: f64, prob6l: f64, prob6h: f64) -> serde_json::Value {
        serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[139.76, 35.68], [139.7625, 35.68],
                                     [139.7625, 35.6825], [139.76, 35.6825],
                                     [139.76, 35.68]]]
                },
                "properties": {
                    "AVR_TTL_MTTL_I50_30": prob5l,
                    "AVR_TTL_MTTL_I55_30": prob5h,
                    "AVR_TTL_MTTL_I60_30": prob6l,
                    "AVR_TTL_MTTL_I65_30": prob6h
                }
            }]
        })
    }

    fn surface_geojson(avs30: f64, amp: f64) -> serde_json::Value {
        serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[139.76, 35.68], [139.7625, 35.68],
                                     [139.7625, 35.6825], [139.76, 35.6825],
                                     [139.76, 35.68]]]
                },
                "properties": {
                    "AVS30": avs30,
                    "AMP": amp
                }
            }]
        })
    }

    fn landslide_json(containing: u8) -> serde_json::Value {
        serde_json::json!({ "isContaining": containing })
    }

    // -----------------------------------------------------------------------
    // get_seismic_hazard — happy path
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_seismic_hazard_success() {
        let mock_server = MockServer::start().await;

        let body = seismic_geojson(0.26, 0.14, 0.06, 0.02);

        Mock::given(method("GET"))
            .and(path(format!(
                "/pshm/{PSHM_VERSION}/{PSHM_CASE}/{PSHM_EQCODE}/meshinfo.geojson"
            )))
            .and(query_param("epsg", "4326"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .get_seismic_hazard(139.7671, 35.6812)
            .await
            .expect("get_seismic_hazard should succeed");

        assert_eq!(result.features.len(), 1, "should return one mesh feature");
        assert_eq!(
            result.prob_level5_low_30yr,
            Some(0.26),
            "P(≥5弱) should match"
        );
        assert_eq!(
            result.prob_level5_high_30yr,
            Some(0.14),
            "P(≥5強) should match"
        );
        assert_eq!(
            result.prob_level6_low_30yr,
            Some(0.06),
            "P(≥6弱) should match"
        );
        assert_eq!(
            result.prob_level6_high_30yr,
            Some(0.02),
            "P(≥6強) should match"
        );
    }

    // -----------------------------------------------------------------------
    // get_seismic_hazard — empty FeatureCollection (point outside coverage)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_seismic_hazard_empty_response() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({ "type": "FeatureCollection", "features": [] });

        Mock::given(method("GET"))
            .and(path(format!(
                "/pshm/{PSHM_VERSION}/{PSHM_CASE}/{PSHM_EQCODE}/meshinfo.geojson"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .get_seismic_hazard(0.0, 0.0)
            .await
            .expect("empty FeatureCollection should not be an error");

        assert!(result.features.is_empty());
        assert!(result.prob_level5_low_30yr.is_none());
        assert!(result.prob_level6_high_30yr.is_none());
    }

    // -----------------------------------------------------------------------
    // get_surface_ground — happy path
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_surface_ground_success() {
        let mock_server = MockServer::start().await;

        let body = surface_geojson(250.0, 1.8);

        Mock::given(method("GET"))
            .and(path(format!("/sstrct/{SSTRCT_VERSION}/meshinfo.geojson")))
            .and(query_param("epsg", "4326"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .get_surface_ground(139.7671, 35.6812)
            .await
            .expect("get_surface_ground should succeed");

        assert_eq!(result.features.len(), 1);
        assert_eq!(result.avs30, Some(250.0), "AVS30 should match");
        assert_eq!(
            result.amplification_factor,
            Some(1.8),
            "amplification factor should match"
        );
    }

    // -----------------------------------------------------------------------
    // get_surface_ground — missing AMP field (graceful None)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_surface_ground_missing_amp() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [139.76, 35.68] },
                "properties": { "AVS30": 300.0 }
            }]
        });

        Mock::given(method("GET"))
            .and(path(format!("/sstrct/{SSTRCT_VERSION}/meshinfo.geojson")))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .get_surface_ground(139.7671, 35.6812)
            .await
            .expect("missing AMP should not be an error");

        assert_eq!(result.avs30, Some(300.0));
        assert!(
            result.amplification_factor.is_none(),
            "AMP absent should yield None"
        );
    }

    // -----------------------------------------------------------------------
    // is_landslide_terrain — point is inside (isContaining = 1)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_landslide_containing_true() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/landslide/isContaining.json"))
            .and(query_param("epsg", "4326"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&landslide_json(1)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .is_landslide_terrain(139.7671, 35.6812)
            .await
            .expect("landslide query should succeed");

        assert!(result.is_containing, "should be inside landslide terrain");
    }

    // -----------------------------------------------------------------------
    // is_landslide_terrain — point is outside (isContaining = 0)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_landslide_containing_false() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/landslide/isContaining.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&landslide_json(0)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .is_landslide_terrain(135.0, 34.0)
            .await
            .expect("landslide query should succeed");

        assert!(!result.is_containing, "should be outside landslide terrain");
    }

    // -----------------------------------------------------------------------
    // Error handling — HTTP 404 → MlitError::Api
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_error_404_returns_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(format!(
                "/pshm/{PSHM_VERSION}/{PSHM_CASE}/{PSHM_EQCODE}/meshinfo.geojson"
            )))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let err = client
            .get_seismic_hazard(139.7671, 35.6812)
            .await
            .expect_err("404 should return an error");

        match err {
            MlitError::Api { status, .. } => {
                assert_eq!(status, 404, "error status should be 404");
            }
            other => panic!("expected MlitError::Api, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Error handling — HTTP 500 → MlitError::Api (no retry)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_error_500_returns_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/landslide/isContaining.json"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let err = client
            .is_landslide_terrain(139.7671, 35.6812)
            .await
            .expect_err("500 should return an error");

        match err {
            MlitError::Api { status, .. } => {
                assert_eq!(status, 500);
            }
            other => panic!("expected MlitError::Api, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Error handling — malformed JSON → MlitError::Parse
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_malformed_json_returns_parse_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/landslide/isContaining.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let err = client
            .is_landslide_terrain(139.7671, 35.6812)
            .await
            .expect_err("malformed JSON should return parse error");

        assert!(
            matches!(err, MlitError::Parse(_)),
            "expected MlitError::Parse, got {err:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Retry on HTTP 429 — succeeds on third attempt
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_retry_on_429_succeeds() {
        let mock_server = MockServer::start().await;

        let body = seismic_geojson(0.1, 0.05, 0.02, 0.01);

        // First two requests return 429; third succeeds.
        Mock::given(method("GET"))
            .and(path(format!(
                "/pshm/{PSHM_VERSION}/{PSHM_CASE}/{PSHM_EQCODE}/meshinfo.geojson"
            )))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!(
                "/pshm/{PSHM_VERSION}/{PSHM_CASE}/{PSHM_EQCODE}/meshinfo.geojson"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let client = make_client(mock_server.uri());
        let result = client
            .get_seismic_hazard(139.7671, 35.6812)
            .await
            .expect("should succeed after two 429s");

        assert_eq!(result.features.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Unit tests for pure extraction helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_seismic_probs_all_present() {
        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": null,
            "properties": {
                "AVR_TTL_MTTL_I50_30": 0.3,
                "AVR_TTL_MTTL_I55_30": 0.15,
                "AVR_TTL_MTTL_I60_30": 0.07,
                "AVR_TTL_MTTL_I65_30": 0.03
            }
        });
        let (p5l, p5h, p6l, p6h) = extract_seismic_probs(Some(&feature));
        assert_eq!(p5l, Some(0.3));
        assert_eq!(p5h, Some(0.15));
        assert_eq!(p6l, Some(0.07));
        assert_eq!(p6h, Some(0.03));
    }

    #[test]
    fn test_extract_seismic_probs_none_on_missing_feature() {
        let (p5l, p5h, p6l, p6h) = extract_seismic_probs(None);
        assert!(p5l.is_none());
        assert!(p5h.is_none());
        assert!(p6l.is_none());
        assert!(p6h.is_none());
    }

    #[test]
    fn test_extract_surface_ground_props_all_present() {
        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": null,
            "properties": { "AVS30": 200.0, "AMP": 2.1 }
        });
        let (avs30, amp) = extract_surface_ground_props(Some(&feature));
        assert_eq!(avs30, Some(200.0));
        assert_eq!(amp, Some(2.1));
    }

    #[test]
    fn test_extract_surface_ground_props_none_on_no_feature() {
        let (avs30, amp) = extract_surface_ground_props(None);
        assert!(avs30.is_none());
        assert!(amp.is_none());
    }
}
