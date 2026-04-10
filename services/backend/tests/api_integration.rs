//! Integration tests for the Real Estate API.
//!
//! These tests require a running PostGIS database with seed data applied.
//! Set `DATABASE_URL` to point to the test database.
//!
//! Run:
//! ```sh
//! DATABASE_URL=postgres://... cargo test --test api_integration
//! ```
//!
//! When `DATABASE_URL` is not set, all tests are skipped gracefully.

use axum_test::TestServer;
use serde_json::Value;

/// Create a TestServer backed by a real PostGIS database.
/// Returns `None` when `DATABASE_URL` is not set (skip test gracefully).
async fn test_server() -> Option<TestServer> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").ok()?;
    let pool = realestate_db::pool::create_pool(&db_url, 5)
        .await
        .expect("failed to connect to test database");
    // No API key in tests — PostgisFallback is selected automatically.
    let config = realestate_api::config::Config {
        database_url: db_url,
        reinfolib_api_key: None,
        port: 8000,
        db_max_connections: 5,
        rust_log_format: None,
        allowed_origins: None,
        rate_limit_rpm: 120,
        rate_limit_burst: 20,
    };
    let router = realestate_api::build_router(pool, &config);
    Some(TestServer::new(router))
}

/// Skip macro: returns early if DATABASE_URL is not set.
macro_rules! require_db {
    ($server:ident) => {
        let Some($server) = test_server().await else {
            eprintln!("SKIP: DATABASE_URL not set — skipping integration test");
            return;
        };
    };
}

// ============================================================
// /api/health
// ============================================================

#[tokio::test]
async fn health_returns_200_with_db_connected() {
    require_db!(server);

    let resp = server.get("/api/health").await;
    resp.assert_status_ok();

    let body: Value = resp.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["db_connected"], true);
    assert!(body["version"].is_string());
}

// ============================================================
// /api/area-data — bbox covering Tokyo Station seed data
// ============================================================

#[tokio::test]
async fn area_data_returns_landprice_features_in_bbox() {
    require_db!(server);

    // BBox covers Marunouchi/Ginza/Kanda seed data (within 0.5° limit)
    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.66")
        .add_query_param("west", "139.74")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.78")
        .add_query_param("layers", "landprice")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    let fc = &body["landprice"];
    assert_eq!(fc["type"], "FeatureCollection");

    let features = fc["features"].as_array().expect("features should be array");
    // Seed data has 15 land price rows (5 years × 3 locations) in this bbox
    assert!(
        !features.is_empty(),
        "expected land price features in Tokyo Station area"
    );
}

#[tokio::test]
async fn area_data_returns_multiple_layers() {
    require_db!(server);

    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.66")
        .add_query_param("west", "139.74")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.78")
        .add_query_param("layers", "landprice,zoning,schools,medical")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    // Each requested layer should have a FeatureCollection
    for layer in ["landprice", "zoning", "schools", "medical"] {
        assert_eq!(
            body[layer]["type"], "FeatureCollection",
            "missing FeatureCollection for layer: {layer}"
        );
    }
}

#[tokio::test]
async fn area_data_returns_flood_and_steep_slope() {
    require_db!(server);

    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.67")
        .add_query_param("west", "139.76")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.79")
        .add_query_param("layers", "flood,steep_slope")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    assert_eq!(body["flood"]["type"], "FeatureCollection");
    assert_eq!(body["steep_slope"]["type"], "FeatureCollection");
}

// ============================================================
// /api/area-data — validation errors
// ============================================================

#[tokio::test]
async fn area_data_rejects_bbox_too_large() {
    require_db!(server);

    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.0")
        .add_query_param("west", "139.0")
        .add_query_param("north", "35.8")
        .add_query_param("east", "139.8")
        .add_query_param("layers", "landprice")
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn area_data_rejects_missing_layers() {
    require_db!(server);

    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.65")
        .add_query_param("west", "139.70")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.80")
        .await;

    // Missing `layers` param should fail
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ============================================================
// /api/score — point-based investment score
// ============================================================

#[tokio::test]
async fn score_returns_tls_for_tokyo_station() {
    require_db!(server);

    let resp = server
        .get("/api/score")
        .add_query_param("lat", "35.681")
        .add_query_param("lng", "139.767")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    // Location echo
    assert!(body["location"]["lat"].is_number(), "location.lat");
    assert!(body["location"]["lng"].is_number(), "location.lng");
    // TLS summary
    assert!(body["tls"]["score"].is_number(), "tls.score");
    assert!(body["tls"]["grade"].is_string(), "tls.grade");
    assert!(body["tls"]["label"].is_string(), "tls.label");
    // Axes use `sub` (not `sub_scores`)
    assert!(
        body["axes"]["disaster"]["sub"].is_array(),
        "axes.disaster.sub"
    );
    assert!(
        body["axes"]["disaster"]["confidence"].is_number(),
        "confidence"
    );
    // Metadata
    assert!(
        body["metadata"]["weight_preset"].is_string(),
        "weight_preset in metadata"
    );
    assert!(
        body["metadata"]["calculated_at"].is_string(),
        "calculated_at"
    );
    assert!(body["metadata"]["disclaimer"].is_string(), "disclaimer");
}

// ============================================================
// /api/stats — area statistics
// ============================================================

#[tokio::test]
async fn stats_returns_land_price_stats_in_bbox() {
    require_db!(server);

    let resp = server
        .get("/api/stats")
        .add_query_param("south", "35.66")
        .add_query_param("west", "139.74")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.78")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    assert!(body["land_price"]["count"].is_number());
    assert!(body["risk"]["composite_risk"].is_number());
    assert!(body["facilities"]["schools"].is_number());
    assert!(body["facilities"]["medical"].is_number());
}

// ============================================================
// /api/trend — land price trend analysis
// ============================================================

#[tokio::test]
async fn trend_returns_data_near_marunouchi() {
    require_db!(server);

    // Near Marunouchi seed data (5 years of land prices)
    let resp = server
        .get("/api/trend")
        .add_query_param("lat", "35.681")
        .add_query_param("lng", "139.767")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    assert!(body["location"]["address"].is_string());
    assert!(body["cagr"].is_number());
    assert!(
        body["direction"] == "up" || body["direction"] == "down",
        "direction should be 'up' or 'down'"
    );

    let data = body["data"].as_array().expect("data should be array");
    assert!(!data.is_empty(), "expected trend data points");
}

// ============================================================
// Seed data verification — row counts
// ============================================================

#[tokio::test]
async fn seed_data_has_expected_landprice_rows() {
    require_db!(server);

    // All 3 seed locations are in this bbox
    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.66")
        .add_query_param("west", "139.74")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.78")
        .add_query_param("layers", "landprice")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let features = body["landprice"]["features"]
        .as_array()
        .expect("features array");
    // 15 rows: 5 years × 3 locations (Marunouchi, Ginza, Kanda)
    assert_eq!(
        features.len(),
        15,
        "expected 15 land price features (5y × 3 locations)"
    );
}

// ============================================================
// /api/v1/land-prices/all-years — time machine endpoint
// ============================================================

#[tokio::test]
async fn land_prices_all_years_returns_multi_year_features() {
    require_db!(server);

    // BBox covers all 3 seed locations × 5 years
    let resp = server
        .get("/api/v1/land-prices/all-years")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("from", "2020")
        .add_query_param("to", "2024")
        .add_query_param("zoom", "14")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let features = body["features"]
        .as_array()
        .expect("features array on FeatureCollection");

    // Expect 15 total (3 locations × 5 years), all with year property in range.
    assert_eq!(
        features.len(),
        15,
        "expected 15 features (3 locations × 5 years), got {}",
        features.len()
    );

    // Verify every feature has a year property in the requested range
    // and that all 5 distinct years are present.
    let mut years: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    for f in features {
        let year = f["properties"]["year"]
            .as_i64()
            .expect("year property is number");
        assert!(
            (2020..=2024).contains(&year),
            "year {year} outside requested range"
        );
        years.insert(year);
    }
    assert_eq!(
        years.len(),
        5,
        "expected 5 distinct years in response, got {years:?}"
    );
}

#[tokio::test]
async fn land_prices_all_years_rejects_inverted_range() {
    require_db!(server);

    let resp = server
        .get("/api/v1/land-prices/all-years")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("from", "2024")
        .add_query_param("to", "2020")
        .await;

    // from > to should be rejected as a bad request
    assert_eq!(resp.status_code().as_u16(), 400);
}

#[tokio::test]
async fn land_prices_all_years_uses_default_year_range() {
    require_db!(server);

    // Omit from/to — should default to 2019..=2024
    let resp = server
        .get("/api/v1/land-prices/all-years")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("zoom", "14")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let features = body["features"]
        .as_array()
        .expect("features array on FeatureCollection");
    assert!(
        !features.is_empty(),
        "default year range should include seeded data"
    );
}

#[tokio::test]
async fn seed_data_has_expected_school_rows() {
    require_db!(server);

    // Wide bbox covering all seed schools
    let resp = server
        .get("/api/area-data")
        .add_query_param("south", "35.66")
        .add_query_param("west", "139.74")
        .add_query_param("north", "35.70")
        .add_query_param("east", "139.78")
        .add_query_param("layers", "schools")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let features = body["schools"]["features"]
        .as_array()
        .expect("features array");
    // Some schools (kasei, hibiya) are outside this bbox, so we check >= 4
    assert!(
        features.len() >= 4,
        "expected at least 4 schools in central Tokyo bbox, got {}",
        features.len()
    );
}
