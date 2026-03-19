pub mod error;
pub mod geojson;

use sqlx::PgPool;

/// Shared application state injected into all Axum route handlers via [`axum::extract::State`].
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool (PostGIS-enabled).
    pub db: PgPool,
    /// Optional MLIT reinfolib API key. `None` when `REINFOLIB_API_KEY` env var is not set.
    pub reinfolib_key: Option<String>,
}
