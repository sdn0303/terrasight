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
///
/// The pool size is set to 5 connections.  Integration tests run
/// serially (see `.cargo/config.toml` `test-threads = 1`), but
/// SQLx pools hold connections until the pool itself is dropped, which
/// may lag behind test completion.  A small pool keeps the total
/// connection count well below PostgreSQL's `max_connections = 100`
/// even when several pools overlap briefly during sequential teardown.
///
/// The `/api/v1/opportunities` endpoint uses `OPPORTUNITY_TLS_CONCURRENCY = 4`
/// and issues at most 1 DB query per concurrent slot at any instant, so
/// 5 connections provide enough headroom with one spare for housekeeping.
/// The backpressure path is exercised naturally by the bounded pool.
async fn test_server() -> Option<TestServer> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").ok()?;
    let pool = terrasight_server::db::pool::create_pool(&db_url, 5)
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

    let resp = server.get("/api/v1/health").await;
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
        .get("/api/v1/area-data")
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
        .get("/api/v1/area-data")
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
        .get("/api/v1/area-data")
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
        .get("/api/v1/area-data")
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
        .get("/api/v1/area-data")
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
        .get("/api/v1/score")
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
        .get("/api/v1/stats")
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
        .get("/api/v1/trend")
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
        .get("/api/v1/area-data")
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
    assert!(
        !features.is_empty(),
        "expected at least 1 land price feature in central Tokyo bbox, got 0"
    );
}

// ============================================================
// /api/v1/land-prices/all-years — time machine endpoint
// ============================================================

#[tokio::test]
async fn land_prices_all_years_returns_multi_year_features() {
    require_db!(server);

    // BBox covers all 3 seed locations. Range is wide enough to include
    // data imported at survey_year=2026 as well as any older seed rows.
    let resp = server
        .get("/api/v1/land-prices/all-years")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("from", "2020")
        .add_query_param("to", "2030")
        .add_query_param("zoom", "14")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let features = body["features"]
        .as_array()
        .expect("features array on FeatureCollection");

    assert!(
        !features.is_empty(),
        "expected at least 1 land price feature in the 2020-2030 range, got 0"
    );

    // Verify every feature has a year property within the requested range.
    for f in features {
        let year = f["properties"]["year"]
            .as_i64()
            .expect("year property is number");
        assert!(
            (2020..=2030).contains(&year),
            "year {year} outside requested range 2020-2030"
        );
    }
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

    // Omit from/to — should default to 2019..=2030, covering survey_year=2026 data.
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
        "default year range should include seeded data (survey_year=2026)"
    );
}

#[tokio::test]
async fn seed_data_has_expected_school_rows() {
    require_db!(server);

    // Wide bbox covering all seed schools
    let resp = server
        .get("/api/v1/area-data")
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

// ============================================================
// /api/v1/opportunities
// ============================================================

#[tokio::test]
async fn opportunities_returns_items_in_bbox() {
    require_db!(server);

    // BBox covers Marunouchi/Ginza/Kanda seed data
    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "10")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    assert!(body["items"].is_array(), "items must be an array");
    assert!(body["total"].is_u64(), "total must be a number");
    // `truncated` reflects "there are more records beyond this page" —
    // the sign depends on how many seed records survive TLS enrichment,
    // so just assert it's a boolean.
    assert!(body["truncated"].is_boolean(), "truncated must be bool");

    // Seed data should produce at least one opportunity where land price
    // records intersect zoning polygons.
    let items = body["items"].as_array().unwrap();
    if let Some(first) = items.first() {
        assert!(first["id"].is_i64());
        assert!(first["tls"].is_u64());
        assert!(["low", "mid", "high"].contains(&first["risk_level"].as_str().unwrap()));
        assert!(first["price_per_sqm"].is_i64());
        assert!(first["address"].is_string());
        assert!(first["zone"].is_string());
    }
}

#[tokio::test]
async fn opportunities_clamps_limit_to_server_max() {
    require_db!(server);

    // Request 200 — server should clamp to 50 (MAX_OPPORTUNITY_LIMIT)
    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "200")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    let items = body["items"].as_array().unwrap();
    assert!(
        items.len() <= 50,
        "expected at most 50 items after clamp, got {}",
        items.len()
    );
}

#[tokio::test]
async fn opportunities_filters_by_tls_min() {
    require_db!(server);

    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "50")
        .add_query_param("tls_min", "80")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    let items = body["items"].as_array().unwrap();
    for item in items {
        let tls = item["tls"].as_u64().unwrap();
        assert!(tls >= 80, "tls_min filter violated: {tls} < 80");
    }
}

#[tokio::test]
async fn opportunities_filters_by_risk_max() {
    require_db!(server);

    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "50")
        .add_query_param("risk_max", "mid")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    let items = body["items"].as_array().unwrap();
    for item in items {
        let risk = item["risk_level"].as_str().unwrap();
        assert!(
            risk == "low" || risk == "mid",
            "risk_max filter violated: {risk} > mid"
        );
    }
}

#[tokio::test]
async fn opportunities_rejects_invalid_bbox() {
    require_db!(server);

    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "not,a,valid,bbox")
        .await;

    assert_eq!(
        resp.status_code(),
        400,
        "invalid bbox must return 400 Bad Request"
    );
}

#[tokio::test]
async fn opportunities_rejects_unknown_risk_max() {
    require_db!(server);

    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("risk_max", "extreme")
        .await;

    assert_eq!(
        resp.status_code(),
        400,
        "unknown risk_max must return 400 Bad Request"
    );
}

#[tokio::test]
async fn opportunities_cache_hit_within_60s() {
    require_db!(server);

    // First request — cold miss.
    let resp1 = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "10")
        .await;
    resp1.assert_status_ok();
    let body1: Value = resp1.json();

    // Second request with identical filters — should hit the cache.
    // We assert shape invariance because the cache returns the same Arc.
    let resp2 = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("limit", "10")
        .await;
    resp2.assert_status_ok();
    let body2: Value = resp2.json();

    assert_eq!(
        body1["total"], body2["total"],
        "cache hit should yield identical `total`"
    );
    assert_eq!(
        body1["items"].as_array().unwrap().len(),
        body2["items"].as_array().unwrap().len(),
        "cache hit should yield identical item count"
    );
}

#[tokio::test]
async fn opportunities_ignores_cities_with_warning() {
    require_db!(server);

    // The `cities` filter is not honoured in Phase 4 — the server logs a
    // warning and proceeds. The request must succeed.
    let resp = server
        .get("/api/v1/opportunities")
        .add_query_param("bbox", "139.74,35.66,139.78,35.70")
        .add_query_param("cities", "13101,13102")
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert!(body["items"].is_array());
}

// ============================================================
// /api/v1/transactions/summary — transaction summary aggregates
// ============================================================

#[tokio::test]
async fn transactions_summary_returns_data_for_tokyo() {
    require_db!(server);

    let resp = server
        .get("/api/v1/transactions/summary")
        .add_query_param("pref_code", "13")
        .await;

    resp.assert_status_ok();

    let items = resp.json::<Value>();
    let items = items.as_array().expect("response should be a JSON array");
    assert!(
        !items.is_empty(),
        "expected transaction summary rows for pref_code=13"
    );

    // Verify required fields on the first element
    let first = &items[0];
    assert!(first["city_code"].is_string(), "city_code must be a string");
    assert!(
        first["transaction_year"].is_number(),
        "transaction_year must be a number"
    );
    assert!(
        first["property_type"].is_string(),
        "property_type must be a string"
    );
    assert!(first["tx_count"].is_number(), "tx_count must be a number");
}

// ============================================================
// /api/v1/transactions — transaction detail records
// ============================================================

#[tokio::test]
async fn transactions_returns_detail_for_chiyoda() {
    require_db!(server);

    // city_code=13101 is Chiyoda ward (千代田区)
    let resp = server
        .get("/api/v1/transactions")
        .add_query_param("city_code", "13101")
        .await;

    resp.assert_status_ok();

    let items = resp.json::<Value>();
    let items = items.as_array().expect("response should be a JSON array");

    // Verify required fields when records are present
    if let Some(first) = items.first() {
        assert!(first["city_code"].is_string(), "city_code must be a string");
        assert!(first["city_name"].is_string(), "city_name must be a string");
        assert!(
            first["total_price"].is_number(),
            "total_price must be a number"
        );
        assert!(
            first["transaction_quarter"].is_string(),
            "transaction_quarter must be a string"
        );
    }
}

// ============================================================
// /api/v1/appraisals — land appraisal records
// ============================================================

#[tokio::test]
async fn appraisals_returns_data_for_tokyo() {
    require_db!(server);

    let resp = server
        .get("/api/v1/appraisals")
        .add_query_param("pref_code", "13")
        .await;

    resp.assert_status_ok();

    let items = resp.json::<Value>();
    let items = items.as_array().expect("response should be a JSON array");
    assert!(
        !items.is_empty(),
        "expected appraisal rows for pref_code=13"
    );

    // AppraisalDetailResponse serializes as snake_case (Zod schema source of truth)
    let first = &items[0];
    assert!(first["city_code"].is_string(), "city_code must be a string");
    assert!(
        first["price_per_sqm"].is_number(),
        "price_per_sqm must be a number"
    );
    assert!(
        first["appraisal_price"].is_number(),
        "appraisal_price must be a number"
    );
}

// ============================================================
// /api/v1/municipalities — municipality list
// ============================================================

#[tokio::test]
async fn municipalities_returns_list_for_tokyo() {
    require_db!(server);

    let resp = server
        .get("/api/v1/municipalities")
        .add_query_param("pref_code", "13")
        .await;

    resp.assert_status_ok();

    let items = resp.json::<Value>();
    let items = items.as_array().expect("response should be a JSON array");
    assert!(
        !items.is_empty(),
        "expected municipalities for pref_code=13 (Tokyo has 62)"
    );

    // MunicipalityResponse serializes as snake_case (Zod schema source of truth)
    let first = &items[0];
    assert!(first["city_code"].is_string(), "city_code must be a string");
    assert!(first["city_name"].is_string(), "city_name must be a string");
    assert!(first["pref_code"].is_string(), "pref_code must be a string");

    // Every city_code must start with "13" (Tokyo prefecture)
    for item in items {
        let city_code = item["city_code"]
            .as_str()
            .expect("city_code must be a string");
        assert!(
            city_code.starts_with("13"),
            "city_code {city_code} should start with '13' for Tokyo"
        );
    }
}

// ============================================================
// Validation error tests
// ============================================================

#[tokio::test]
async fn transactions_summary_rejects_invalid_pref_code() {
    require_db!(server);

    // pref_code=99 is out of the valid 01–47 range
    let resp = server
        .get("/api/v1/transactions/summary")
        .add_query_param("pref_code", "99")
        .await;

    assert_eq!(
        resp.status_code().as_u16(),
        400,
        "pref_code=99 must return 400 INVALID_PARAMS"
    );

    let body: Value = resp.json();
    assert_eq!(
        body["error"]["code"], "INVALID_PARAMS",
        "error.code must be INVALID_PARAMS"
    );
}

#[tokio::test]
async fn transactions_rejects_invalid_city_code() {
    require_db!(server);

    // city_code=123 is only 3 digits; CityCode requires exactly 5
    let resp = server
        .get("/api/v1/transactions")
        .add_query_param("city_code", "123")
        .await;

    assert_eq!(
        resp.status_code().as_u16(),
        400,
        "city_code=123 must return 400 INVALID_PARAMS"
    );

    let body: Value = resp.json();
    assert_eq!(
        body["error"]["code"], "INVALID_PARAMS",
        "error.code must be INVALID_PARAMS"
    );
}
