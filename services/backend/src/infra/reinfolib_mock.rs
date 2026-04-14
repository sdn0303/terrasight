//! Reinfolib data-source implementations and factory.
//!
//! This module wires two concrete implementations of [`ReinfolibDataSource`]:
//!
//! - [`PostgisFallback`] — serves every endpoint from the local PostGIS database.
//!   Used automatically when `REINFOLIB_API_KEY` is absent, so the backend
//!   remains fully functional without an API key during development and testing.
//!
//! - [`LiveReinfolib`] — delegates to the real MLIT reinfolib HTTP API.
//!   The response-to-domain conversion is stubbed with `TODO` markers that will
//!   be filled in once the live integration is validated against real API responses.
//!
//! # Factory
//!
//! Call [`create_reinfolib_source`] at the composition root to get the appropriate
//! implementation based on whether an API key is configured:
//!
//! ```rust,ignore
//! let source = create_reinfolib_source(pool.clone(), &config);
//! // source: Arc<dyn ReinfolibDataSource>
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::config::Config;
use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::reinfolib::ReinfolibDataSource;
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType, ZoomLevel};

/// Default zoom level used when `PostgisFallback` delegates to `LayerRepository`.
///
/// The `ReinfolibDataSource` trait does not carry zoom context (it predates
/// the zoom-aware limit feature). Using [`ZoomLevel::DEFAULT`] (street level)
/// ensures a reasonable feature count for the PostGIS fallback path.
const FALLBACK_ZOOM: ZoomLevel = ZoomLevel::DEFAULT;

// ─── PostgisFallback ─────────────────────────────────────────────────────────

/// PostGIS-backed implementation of [`ReinfolibDataSource`].
///
/// Delegates every method to the injected [`LayerRepository`], which queries
/// the local PostGIS database. This is the default when no API key is set.
///
/// # Note on `get_hazard_areas`
///
/// The reinfolib XKT016 endpoint covers all disaster-hazard zone types in a
/// single call. The PostGIS fallback approximates this by merging results from
/// both the `flood_risk` and `steep_slope` tables.
pub(crate) struct PostgisFallback {
    layer_repo: Arc<dyn LayerRepository>,
}

impl PostgisFallback {
    /// Create a new `PostgisFallback` backed by the given repository.
    pub(crate) fn new(layer_repo: Arc<dyn LayerRepository>) -> Self {
        Self { layer_repo }
    }
}

#[async_trait]
impl ReinfolibDataSource for PostgisFallback {
    #[tracing::instrument(skip(self), fields(source = "postgis_fallback"))]
    async fn get_land_prices(
        &self,
        bbox: &BBox,
        _year: u16,
    ) -> Result<Vec<GeoFeature>, DomainError> {
        // The PostGIS `land_prices` table stores all years; the `year` parameter
        // is ignored here. A future enhancement could add a WHERE year = $5 filter.
        self.layer_repo
            .find_layer(LayerType::LandPrice, bbox, FALLBACK_ZOOM, None)
            .await
            .map(|lr| lr.features)
    }

    #[tracing::instrument(skip(self), fields(source = "postgis_fallback"))]
    async fn get_zoning(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        self.layer_repo
            .find_layer(LayerType::Zoning, bbox, FALLBACK_ZOOM, None)
            .await
            .map(|lr| lr.features)
    }

    #[tracing::instrument(skip(self), fields(source = "postgis_fallback"))]
    async fn get_schools(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        self.layer_repo
            .find_layer(LayerType::Schools, bbox, FALLBACK_ZOOM, None)
            .await
            .map(|lr| lr.features)
    }

    #[tracing::instrument(skip(self), fields(source = "postgis_fallback"))]
    async fn get_medical(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        self.layer_repo
            .find_layer(LayerType::Medical, bbox, FALLBACK_ZOOM, None)
            .await
            .map(|lr| lr.features)
    }

    #[tracing::instrument(skip(self), fields(source = "postgis_fallback"))]
    async fn get_hazard_areas(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // Merge flood-risk and steep-slope results to approximate XKT016 coverage.
        let (flood, steep) = tokio::try_join!(
            self.layer_repo
                .find_layer(LayerType::Flood, bbox, FALLBACK_ZOOM, None),
            self.layer_repo
                .find_layer(LayerType::SteepSlope, bbox, FALLBACK_ZOOM, None),
        )?;
        let mut merged = flood.features;
        merged.extend(steep.features);
        Ok(merged)
    }
}

// ─── LiveReinfolib ───────────────────────────────────────────────────────────

/// Live MLIT API implementation of [`ReinfolibDataSource`].
///
/// Wraps a `ReinfolibClient` and converts raw `serde_json::Value` API responses
/// into domain [`GeoFeature`] objects.
///
/// # Conversion status
///
/// The field-mapping from reinfolib property bags to the domain `GeoFeature`
/// properties object is not yet finalised — the reinfolib API has 50+ fields
/// per endpoint and the exact subset needed by the frontend is TBD. Each method
/// returns a `TODO` error until the conversion logic is implemented.
///
/// Replace the `Err(...)` stubs with real conversion code after validating
/// against live API responses.
///
/// Once the `terrasight-mlit` crate is wired in, add the dependency to `Cargo.toml`
/// and hold the client here:
///
/// ```toml
/// # In [dependencies]:
/// terrasight-mlit = { path = "lib/mlit", features = ["reinfolib"] }
/// ```
pub(crate) struct LiveReinfolib;

impl LiveReinfolib {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReinfolibDataSource for LiveReinfolib {
    async fn get_land_prices(
        &self,
        _bbox: &BBox,
        _year: u16,
    ) -> Result<Vec<GeoFeature>, DomainError> {
        // TODO: call ReinfolibClient::get_land_prices, then convert each
        // serde_json::Value feature into a domain GeoFeature.
        // Example conversion outline:
        //   let raw = self.client.get_land_prices(bbox.west(), bbox.south(),
        //       bbox.east(), bbox.north(), year).await
        //       .map_err(|e| DomainError::Database(e.to_string()))?;
        //   raw.into_iter().map(raw_value_to_geo_feature).collect()
        Err(DomainError::Database(
            "LiveReinfolib::get_land_prices is not yet implemented — \
             add terrasight-mlit dependency and implement response conversion"
                .into(),
        ))
    }

    async fn get_zoning(&self, _bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // TODO: call ReinfolibClient::get_zoning (XKT002) and convert response.
        Err(DomainError::Database(
            "LiveReinfolib::get_zoning is not yet implemented".into(),
        ))
    }

    async fn get_schools(&self, _bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // TODO: call ReinfolibClient::get_schools (XKT006) and convert response.
        Err(DomainError::Database(
            "LiveReinfolib::get_schools is not yet implemented".into(),
        ))
    }

    async fn get_medical(&self, _bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // TODO: call ReinfolibClient::get_medical (XKT010) and convert response.
        Err(DomainError::Database(
            "LiveReinfolib::get_medical is not yet implemented".into(),
        ))
    }

    async fn get_hazard_areas(&self, _bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // TODO: call ReinfolibClient::get_hazard_areas (XKT016) and convert response.
        Err(DomainError::Database(
            "LiveReinfolib::get_hazard_areas is not yet implemented".into(),
        ))
    }
}

// ─── Factory ─────────────────────────────────────────────────────────────────

/// Build the appropriate [`ReinfolibDataSource`] based on the application config.
///
/// - When `config.reinfolib_api_key` is `None`, logs a warning and returns a
///   [`PostgisFallback`] backed by a freshly-constructed [`PgAreaRepository`].
/// - When the key is `Some(_)`, logs an info message and returns a
///   [`LiveReinfolib`] stub. Replace the stub body once the live conversion
///   is implemented.
///
/// The returned `Arc<dyn ReinfolibDataSource>` is suitable for injection into
/// any usecase that depends on the trait.
///
/// # Example
///
/// ```rust,ignore
/// let source = create_reinfolib_source(pool.clone(), &config);
/// state.reinfolib = source;
/// ```
pub(crate) fn create_reinfolib_source(
    pool: PgPool,
    config: &Config,
) -> Arc<dyn ReinfolibDataSource> {
    if config.reinfolib_api_key.is_some() {
        tracing::info!(
            mode = "live",
            "reinfolib data source: using live MLIT API (REINFOLIB_API_KEY is set)"
        );
        Arc::new(LiveReinfolib::new())
    } else {
        tracing::warn!(
            mode = "fallback",
            "reinfolib data source: REINFOLIB_API_KEY not set — \
             falling back to local PostGIS database"
        );
        let area_repo = Arc::new(crate::infra::pg_area_repository::PgAreaRepository::new(
            pool,
        ));
        Arc::new(PostgisFallback::new(area_repo))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::PostgisFallback;
    use crate::domain::entity::{GeoFeature, GeoJsonGeometry, GeoJsonType, LayerResult};
    use crate::domain::error::DomainError;
    use crate::domain::reinfolib::ReinfolibDataSource;
    use crate::domain::repository::LayerRepository;
    use crate::domain::value_object::{BBox, Coord, LayerType, PrefCode, ZoomLevel};

    // ── Stub LayerRepository ─────────────────────────────────────────────────

    /// Records which layer variants were requested so tests can assert delegation.
    #[derive(Default)]
    struct StubLayerRepo {
        pub land_prices_calls: std::sync::atomic::AtomicU32,
        pub zoning_calls: std::sync::atomic::AtomicU32,
        pub flood_risk_calls: std::sync::atomic::AtomicU32,
        pub steep_slope_calls: std::sync::atomic::AtomicU32,
        pub schools_calls: std::sync::atomic::AtomicU32,
        pub medical_calls: std::sync::atomic::AtomicU32,
    }

    impl StubLayerRepo {
        fn land_prices_count(&self) -> u32 {
            self.land_prices_calls
                .load(std::sync::atomic::Ordering::SeqCst)
        }
        fn zoning_count(&self) -> u32 {
            self.zoning_calls.load(std::sync::atomic::Ordering::SeqCst)
        }
        fn flood_risk_count(&self) -> u32 {
            self.flood_risk_calls
                .load(std::sync::atomic::Ordering::SeqCst)
        }
        fn steep_slope_count(&self) -> u32 {
            self.steep_slope_calls
                .load(std::sync::atomic::Ordering::SeqCst)
        }
        fn schools_count(&self) -> u32 {
            self.schools_calls.load(std::sync::atomic::Ordering::SeqCst)
        }
        fn medical_count(&self) -> u32 {
            self.medical_calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    fn stub_feature(geo_type: &str) -> GeoFeature {
        GeoFeature {
            geometry: GeoJsonGeometry {
                r#type: GeoJsonType::from_db_str(geo_type),
                coordinates: serde_json::json!([139.76, 35.68]),
            },
            properties: serde_json::json!({}),
        }
    }

    fn stub_layer_result(geo_type: &str) -> LayerResult {
        LayerResult {
            features: vec![stub_feature(geo_type)],
            truncated: false,
            limit: 100,
        }
    }

    #[async_trait]
    impl LayerRepository for StubLayerRepo {
        async fn find_layer(
            &self,
            layer: LayerType,
            _bbox: &BBox,
            _zoom: ZoomLevel,
            _pref_code: Option<&PrefCode>,
        ) -> Result<LayerResult, DomainError> {
            use std::sync::atomic::Ordering::SeqCst;
            match layer {
                LayerType::LandPrice => {
                    self.land_prices_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Point"))
                }
                LayerType::Zoning => {
                    self.zoning_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Polygon"))
                }
                LayerType::Flood => {
                    self.flood_risk_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Polygon"))
                }
                LayerType::SteepSlope => {
                    self.steep_slope_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Polygon"))
                }
                LayerType::Schools => {
                    self.schools_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Point"))
                }
                LayerType::Medical => {
                    self.medical_calls.fetch_add(1, SeqCst);
                    Ok(stub_layer_result("Point"))
                }
            }
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn test_bbox() -> BBox {
        BBox::new(35.65, 139.70, 35.70, 139.80).expect("test bbox coordinates are valid")
    }

    fn make_fallback() -> (Arc<StubLayerRepo>, PostgisFallback) {
        let stub = Arc::new(StubLayerRepo::default());
        let fallback = PostgisFallback::new(Arc::clone(&stub) as Arc<dyn LayerRepository>);
        (stub, fallback)
    }

    // ── Delegation tests ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_land_prices_delegates_to_find_land_prices() {
        let (stub, fallback) = make_fallback();
        let result = fallback.get_land_prices(&test_bbox(), 2024).await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            result.unwrap().len(),
            1,
            "should return the stub's single feature"
        );
        assert_eq!(
            stub.land_prices_count(),
            1,
            "find_land_prices must be called exactly once"
        );
        assert_eq!(
            stub.zoning_count(),
            0,
            "no other repo method should be called"
        );
    }

    #[tokio::test]
    async fn get_zoning_delegates_to_find_zoning() {
        let (stub, fallback) = make_fallback();
        let result = fallback.get_zoning(&test_bbox()).await;
        assert!(result.is_ok());
        assert_eq!(stub.zoning_count(), 1);
        assert_eq!(stub.land_prices_count(), 0);
    }

    #[tokio::test]
    async fn get_schools_delegates_to_find_schools() {
        let (stub, fallback) = make_fallback();
        let result = fallback.get_schools(&test_bbox()).await;
        assert!(result.is_ok());
        assert_eq!(stub.schools_count(), 1);
        assert_eq!(stub.medical_count(), 0);
    }

    #[tokio::test]
    async fn get_medical_delegates_to_find_medical() {
        let (stub, fallback) = make_fallback();
        let result = fallback.get_medical(&test_bbox()).await;
        assert!(result.is_ok());
        assert_eq!(stub.medical_count(), 1);
        assert_eq!(stub.schools_count(), 0);
    }

    #[tokio::test]
    async fn get_hazard_areas_merges_flood_and_steep_slope() {
        let (stub, fallback) = make_fallback();
        let result = fallback.get_hazard_areas(&test_bbox()).await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");

        let features = result.unwrap();
        // The stub returns exactly one feature per method, so merged total = 2.
        assert_eq!(
            features.len(),
            2,
            "should merge flood_risk (1) + steep_slope (1) = 2 features"
        );
        assert_eq!(
            stub.flood_risk_count(),
            1,
            "find_flood_risk must be called once"
        );
        assert_eq!(
            stub.steep_slope_count(),
            1,
            "find_steep_slope must be called once"
        );
        // Unrelated repo methods must not be touched.
        assert_eq!(stub.land_prices_count(), 0);
        assert_eq!(stub.zoning_count(), 0);
        assert_eq!(stub.schools_count(), 0);
        assert_eq!(stub.medical_count(), 0);
    }

    // ── Year parameter is forwarded (no panic on edge values) ────────────────

    #[tokio::test]
    async fn get_land_prices_accepts_edge_year_values() {
        let (_, fallback) = make_fallback();
        // Year is currently passed through without a DB filter; these should not
        // panic or return an error from the fallback.
        assert!(fallback.get_land_prices(&test_bbox(), 1970).await.is_ok());
        assert!(fallback.get_land_prices(&test_bbox(), 2099).await.is_ok());
    }

    // ── Trait object usability ───────────────────────────────────────────────

    #[tokio::test]
    async fn postgis_fallback_is_usable_as_trait_object() {
        let stub = Arc::new(StubLayerRepo::default());
        let source: Arc<dyn ReinfolibDataSource> =
            Arc::new(PostgisFallback::new(stub as Arc<dyn LayerRepository>));
        // Exercise every method through the trait object to confirm vtable wiring.
        let bbox = test_bbox();
        assert!(source.get_land_prices(&bbox, 2024).await.is_ok());
        assert!(source.get_zoning(&bbox).await.is_ok());
        assert!(source.get_schools(&bbox).await.is_ok());
        assert!(source.get_medical(&bbox).await.is_ok());
        assert!(source.get_hazard_areas(&bbox).await.is_ok());
    }

    // ── Coord is used elsewhere; verify BBox invariants hold in tests ────────

    #[test]
    fn test_bbox_invariants() {
        // south >= north is rejected
        assert!(BBox::new(35.70, 139.70, 35.65, 139.80).is_err());
        // Too large (> 0.5 deg per side)
        assert!(BBox::new(35.0, 139.0, 35.6, 139.6).is_err());
        // Valid small bbox
        assert!(BBox::new(35.65, 139.70, 35.70, 139.80).is_ok());
    }

    #[test]
    fn coord_value_object_validates_bounds() {
        assert!(Coord::new(91.0, 0.0).is_err());
        assert!(Coord::new(35.68, 139.76).is_ok());
    }
}
