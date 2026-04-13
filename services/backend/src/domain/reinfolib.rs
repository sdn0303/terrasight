//! Domain abstraction over the MLIT reinfolib (不動産情報ライブラリ) data source.
//!
//! The [`ReinfolibDataSource`] trait decouples the usecase and handler layers
//! from the "do we have a live API key?" question. Two implementations exist
//! in the `infra` layer:
//!
//! - `PostgisFallback` — reads from local PostGIS tables. Used in development
//!   and CI when no `REINFOLIB_API_KEY` environment variable is set.
//! - `LiveReinfolib` — calls the real MLIT HTTP API. Used in production.
//!
//! All methods return `Vec<GeoFeature>` using RFC 7946 `[longitude, latitude]`
//! coordinate order, regardless of the underlying source.

/// Trait abstracting the reinfolib (不動産情報ライブラリ) data source.
///
/// Callers receive `GeoFeature` collections regardless of whether the data
/// originates from the live MLIT API or a local PostGIS fallback. This keeps
/// usecases and handlers insulated from the "do we have an API key?" decision.
///
/// # Endpoint mapping
///
/// | Method | reinfolib endpoint | PostGIS fallback table |
/// |---|---|---|
/// | [`get_land_prices`] | XPT002 | `land_prices` |
/// | [`get_zoning`] | XKT002 | `zoning` |
/// | [`get_schools`] | XKT006 | `schools` |
/// | [`get_medical`] | XKT010 | `medical_facilities` |
/// | [`get_hazard_areas`] | XKT016 | `flood_risk` + `steep_slope` |
///
/// [`get_land_prices`]: ReinfolibDataSource::get_land_prices
/// [`get_zoning`]: ReinfolibDataSource::get_zoning
/// [`get_schools`]: ReinfolibDataSource::get_schools
/// [`get_medical`]: ReinfolibDataSource::get_medical
/// [`get_hazard_areas`]: ReinfolibDataSource::get_hazard_areas
use async_trait::async_trait;

use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::value_object::BBox;

/// Data provider for reinfolib-compatible geospatial layers.
///
/// Implementations:
/// - `PostgisFallback` — delegates to the local PostGIS database (used when no
///   API key is configured).
/// - `LiveReinfolib` — delegates to the real MLIT reinfolib HTTP API (used when
///   `REINFOLIB_API_KEY` is set).
///
/// # Example
///
/// ```rust,ignore
/// let source: Arc<dyn ReinfolibDataSource> = create_reinfolib_source(pool, &config);
/// let features = source.get_land_prices(&bbox, 2024).await?;
/// ```
#[async_trait]
pub trait ReinfolibDataSource: Send + Sync {
    /// Fetch official land price survey points (地価公示 / 地価調査) within `bbox`.
    ///
    /// `year` is the survey year (e.g. `2024`). The live API endpoint is XPT002.
    async fn get_land_prices(&self, bbox: &BBox, year: u16)
    -> Result<Vec<GeoFeature>, DomainError>;

    /// Fetch urban-planning zoning polygons (用途地域) within `bbox`.
    ///
    /// The live API endpoint is XKT002.
    async fn get_zoning(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;

    /// Fetch school facility points within `bbox`.
    ///
    /// Covers elementary, middle, and high schools.
    /// The live API endpoint is XKT006.
    async fn get_schools(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;

    /// Fetch medical facility points (hospitals, clinics) within `bbox`.
    ///
    /// The live API endpoint is XKT010.
    async fn get_medical(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;

    /// Fetch disaster-hazard area polygons within `bbox`.
    ///
    /// Covers flood-risk zones and steep-slope hazard areas.
    /// The live API endpoint is XKT016; the PostGIS fallback merges
    /// `flood_risk` and `steep_slope` tables.
    async fn get_hazard_areas(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
}
