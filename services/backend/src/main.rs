use axum::{Router, routing::get};
use http::HeaderValue;
use realestate_api_core::middleware::{rate_limit, request_id, response_time};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

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

    // CORS: explicit origin whitelist in production, permissive in development.
    let cors_layer = match config.parsed_origins() {
        Some(origins) => {
            let header_values: Vec<HeaderValue> = origins
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect();
            tracing::info!(
                origins = ?origins,
                "CORS restricted to explicit origins"
            );
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(header_values))
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        }
        None => {
            tracing::warn!("ALLOWED_ORIGINS not set — CORS is permissive (dev mode)");
            CorsLayer::permissive()
        }
    };

    // Rate limiting: IP-based token bucket.
    let rate_limit = rate_limit::rate_limit_layer(&rate_limit::RateLimitConfig {
        requests_per_minute: config.rate_limit_rpm,
        burst_size: config.rate_limit_burst,
    });
    tracing::info!(
        rpm = config.rate_limit_rpm,
        burst = config.rate_limit_burst,
        "rate limiting enabled"
    );

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
        // Rate limiting must be AFTER with_state (needs peer address from axum::serve).
        .layer(rate_limit)
        .layer(response_time::response_time_layer())
        .layer(request_id::request_id_layer())
        .layer(realestate_telemetry::http::trace_layer())
        .layer(cors_layer)
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(addr = %addr, "server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
