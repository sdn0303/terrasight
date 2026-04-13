//! Real Estate Investment API — library entry point.
//!
//! Exposes [`build_router`] so that both `main.rs` and integration tests
//! can construct the same Axum router against a real database pool.

pub mod app_state;
pub mod config;
pub mod domain;
pub(crate) mod handler;
pub(crate) mod infra;
pub mod logging;
pub(crate) mod usecase;

use axum::{Router, routing::get};
use sqlx::PgPool;
use terrasight_server::http::middleware::{request_id, response_time};

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
        .route("/api/v1/health", get(handler::health::health))
        .route("/api/v1/area-data", get(handler::area_data::get_area_data))
        .route(
            "/api/v1/area-stats",
            get(handler::area_stats::get_area_stats),
        )
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
        .route("/api/v1/score", get(handler::score::get_score))
        .route("/api/v1/stats", get(handler::stats::get_stats))
        .route("/api/v1/trend", get(handler::trend::get_trend))
        .route(
            "/api/v1/transactions/summary",
            get(handler::transaction_summary::get_transaction_summary),
        )
        .route(
            "/api/v1/transactions",
            get(handler::transactions::get_transactions),
        )
        .route(
            "/api/v1/appraisals",
            get(handler::appraisals::get_appraisals),
        )
        .route(
            "/api/v1/municipalities",
            get(handler::municipalities::get_municipalities),
        )
        .layer(response_time::response_time_layer())
        .layer(request_id::request_id_layer())
        .with_state(state)
}
