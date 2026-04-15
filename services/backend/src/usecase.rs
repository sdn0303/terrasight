//! Business logic orchestration layer.
//!
//! Each submodule contains a single usecase struct with an `execute()` method.
//! Usecases depend only on domain traits (repository interfaces) — never on
//! concrete infra types — so they can be exercised in unit tests with in-process
//! mock repositories.
//!
//! ## Dependency direction
//!
//! ```text
//! handler → usecase → domain ← infra
//! ```
//!
//! ## Sub-modules
//!
//! | Module | Endpoint |
//! |--------|---------|
//! | [`check_health`] | `GET /api/v1/health` |
//! | [`compute_tls`] | `GET /api/v1/score` + opportunity enrichment |
//! | [`get_appraisals`] | `GET /api/v1/appraisals` |
//! | [`get_area_data`] | `GET /api/v1/area-data` |
//! | [`get_area_stats`] | `GET /api/v1/area-stats` |
//! | [`get_land_prices`] | `GET /api/v1/land-prices` |
//! | [`get_land_prices_by_year_range`] | `GET /api/v1/land-prices/all-years` |
//! | [`get_municipalities`] | `GET /api/v1/municipalities` |
//! | [`get_opportunities`] | `GET /api/v1/opportunities` |
//! | [`get_stats`] | `GET /api/v1/stats` |
//! | [`get_transaction_summary`] | `GET /api/v1/transactions/summary` |
//! | [`get_transactions`] | `GET /api/v1/transactions` |
//! | [`get_land_price_aggregation`] | `GET /api/v1/land-prices/aggregation` |
//! | [`get_transaction_aggregation`] | `GET /api/v1/transactions/aggregation` |
//! | [`get_trend`] | `GET /api/v1/trend` |

pub(crate) mod check_health;
pub(crate) mod compute_tls;
pub(crate) mod get_appraisals;
pub(crate) mod get_area_data;
pub(crate) mod get_area_stats;
pub(crate) mod get_land_price_aggregation;
pub(crate) mod get_land_prices;
pub(crate) mod get_land_prices_by_year_range;
pub(crate) mod get_municipalities;
pub(crate) mod get_opportunities;
pub(crate) mod get_stats;
pub(crate) mod get_transaction_aggregation;
pub(crate) mod get_transaction_summary;
pub(crate) mod get_transactions;
pub(crate) mod get_trend;
