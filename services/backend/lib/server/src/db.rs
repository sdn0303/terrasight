//! Database infrastructure: connection pooling, error mapping, spatial query helpers, and GeoJSON conversion.
//!
//! This module consolidates the common PostgreSQL + PostGIS plumbing shared across
//! all repository implementations in the Terrasight backend.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`error`] | [`DbError`] newtype wrapping `sqlx::Error`; [`map_db_err`] convenience adapter |
//! | [`geo`] | [`RawGeoFeature`] DTO and [`to_raw_geo_feature`] parser for `ST_AsGeoJSON` output |
//! | [`pool`] | [`create_pool`] — centralised `PgPool` constructor |
//! | [`spatial`] | [`bind_bbox`] / [`bind_coord`] — typed PostGIS parameter binders |
//!
//! ## Typical usage
//!
//! ```rust,ignore
//! use terrasight_server::db::{create_pool, map_db_err, bind_bbox, to_raw_geo_feature};
//!
//! let pool = create_pool(&database_url, 10).await?;
//!
//! let rows = sqlx::query_as::<_, MyRow>(SQL)
//!     .fetch_all(&pool)
//!     .await
//!     .map_err(map_db_err)?;
//! ```

pub mod error;
pub mod geo;
pub mod pool;
/// PostGIS parameter binders for geometry types and bounding boxes.
pub mod spatial;

pub use error::{DbError, map_db_err};
pub use geo::{RawGeoFeature, to_raw_geo_feature};
pub use pool::create_pool;
pub use spatial::{bind_bbox, bind_coord};
