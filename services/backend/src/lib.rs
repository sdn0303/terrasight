//! Real Estate Investment API — library entry point.
//!
//! Exposes [`build_router`] so that both `main.rs` and integration tests
//! can construct the same Axum router against a real database pool.

pub mod app_state;
pub mod config;
pub mod domain;
pub mod handler;
pub mod infra;
pub mod logging;
pub mod usecase;

use axum::{Router, routing::get};
use realestate_api_core::middleware::{request_id, response_time};
use sqlx::PgPool;

use app_state::AppState;

/// Build the Axum router with all routes and middleware (except CORS / rate limiting / compression).
///
/// CORS, rate limiting, and compression are added in `main.rs` because they
/// depend on runtime configuration and are not needed for integration tests.
///
/// `config` is forwarded to [`AppState::new`] to select the correct reinfolib
/// data source (`PostgisFallback` vs `LiveReinfolib`).
pub fn build_router(pool: PgPool, config: &config::Config) -> Router {
    let state = AppState::new(pool, config);

    Router::new()
        .route("/api/health", get(handler::health::health))
        .route("/api/area-data", get(handler::area_data::get_area_data))
        .route("/api/area-stats", get(handler::area_stats::get_area_stats))
        .route(
            "/api/v1/land-prices",
            get(handler::land_price::get_land_prices),
        )
        .route(
            "/api/v1/land-prices/all-years",
            get(handler::land_price_by_year_range::get_land_prices_by_year_range),
        )
        .route(
            "/api/v1/opportunities",
            get(handler::opportunities::get_opportunities),
        )
        .route("/api/score", get(handler::score::get_score))
        .route("/api/stats", get(handler::stats::get_stats))
        .route("/api/trend", get(handler::trend::get_trend))
        .layer(response_time::response_time_layer())
        .layer(request_id::request_id_layer())
        .with_state(state)
}
