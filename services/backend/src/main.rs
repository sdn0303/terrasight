use axum::{Router, routing::get};
use realestate_api_core::middleware::{request_id, response_time};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

mod app_state;
mod config;
mod domain;
mod handler;
mod infra;
mod logging;
mod usecase;

use app_state::AppState;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    logging::init(&config);

    tracing::info!(
        port = config.port,
        db_pool_size = config.db_max_connections,
        reinfolib_key_set = config.reinfolib_api_key.is_some(),
        version = env!("CARGO_PKG_VERSION"),
        "server starting"
    );

    let pool =
        realestate_db::pool::create_pool(&config.database_url, config.db_max_connections).await?;

    let state = AppState::new(pool, config.reinfolib_api_key.is_some());

    let app = Router::new()
        .route("/api/health", get(handler::health::health))
        .with_state(state.health)
        .route("/api/area-data", get(handler::area_data::get_area_data))
        .with_state(state.area_data)
        .route("/api/score", get(handler::score::get_score))
        .with_state(state.score)
        .route("/api/stats", get(handler::stats::get_stats))
        .with_state(state.stats)
        .route("/api/trend", get(handler::trend::get_trend))
        .with_state(state.trend)
        .layer(response_time::response_time_layer())
        .layer(request_id::request_id_layer())
        .layer(realestate_telemetry::http::trace_layer())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(addr = %addr, "server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
