use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

mod models;
mod routes;
mod services;

use models::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let state = AppState {
        db: pool,
        reinfolib_key: std::env::var("REINFOLIB_API_KEY").ok(),
    };

    let app = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/area-data", get(routes::area_data::get_area_data))
        .route("/api/score", get(routes::score::get_score))
        .route("/api/stats", get(routes::stats::get_stats))
        .route("/api/trend", get(routes::trend::get_trend))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
