//! Axum application state and dependency injection container.
//!
//! [`AppState`] is the single composition root for the service. It wires every
//! infra repository implementation to the domain trait it implements, constructs
//! all usecases, and hands them to the Axum router via
//! [`axum::extract::FromRef`] impls.
//!
//! ## Dependency injection pattern
//!
//! Every usecase is wrapped in `Arc<…>` so that Axum can clone the state into
//! each handler task without copying the underlying objects. The `FromRef` impls
//! at the bottom of this file allow handlers to extract only the usecase they
//! need with `State<Arc<FooUsecase>>` rather than extracting the entire
//! `AppState`.
//!
//! ```text
//! AppState::new(pool, config)
//!   ├─ PgAreaRepository ──────────→ GetAreaDataUsecase
//!   ├─ PgLandPriceRepository ─────→ GetLandPricesUsecase (shared)
//!   │                            └→ GetOpportunitiesUsecase
//!   ├─ PgTlsRepository ───────────→ ComputeTlsUsecase (shared)
//!   │                            └→ GetOpportunitiesUsecase
//!   └─ …
//! ```

use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;
use terrasight_mlit::config::MlitConfig;
use terrasight_mlit::jshis::JshisClient;

use crate::config::Config;
use crate::domain::reinfolib::ReinfolibDataSource;
use crate::infra::opportunities_cache::OpportunitiesCache;
use crate::infra::pg_admin_area_stats_repository::PgAdminAreaStatsRepository;
use crate::infra::pg_aggregation_repository::PgAggregationRepository;
use crate::infra::pg_appraisal_repository::PgAppraisalRepository;
use crate::infra::pg_area_repository::PgAreaRepository;
use crate::infra::pg_health_repository::PgHealthRepository;
use crate::infra::pg_land_price_repository::PgLandPriceRepository;
use crate::infra::pg_municipality_repository::PgMunicipalityRepository;
use crate::infra::pg_population_repository::PgPopulationRepository;
use crate::infra::pg_stats_repository::PgStatsRepository;
use crate::infra::pg_tls_repository::PgTlsRepository;
use crate::infra::pg_transaction_repository::PgTransactionRepository;
use crate::infra::pg_trend_repository::PgTrendRepository;
use crate::infra::pg_vacancy_repository::PgVacancyRepository;
use crate::infra::reinfolib_mock::create_reinfolib_source;
use crate::usecase::check_health::CheckHealthUsecase;
use crate::usecase::compute_tls::ComputeTlsUsecase;
use crate::usecase::get_appraisals::GetAppraisalsUsecase;
use crate::usecase::get_area_data::GetAreaDataUsecase;
use crate::usecase::get_area_stats::GetAreaStatsUsecase;
use crate::usecase::get_land_price_aggregation::GetLandPriceAggregationUsecase;
use crate::usecase::get_land_prices::GetLandPricesUsecase;
use crate::usecase::get_land_prices_by_year_range::GetLandPricesByYearRangeUsecase;
use crate::usecase::get_municipalities::GetMunicipalitiesUsecase;
use crate::usecase::get_opportunities::GetOpportunitiesUsecase;
use crate::usecase::get_population::GetPopulationUsecase;
use crate::usecase::get_stats::GetStatsUsecase;
use crate::usecase::get_transaction_aggregation::GetTransactionAggregationUsecase;
use crate::usecase::get_transaction_summary::GetTransactionSummaryUsecase;
use crate::usecase::get_transactions::GetTransactionsUsecase;
use crate::usecase::get_trend::GetTrendUsecase;
use crate::usecase::get_vacancy::GetVacancyUsecase;

/// Composition root: wires Infra → Domain traits → Usecases.
///
/// All dependency injection happens here. Each usecase is wrapped in `Arc`
/// for shared ownership across Axum handler tasks.
///
/// `AppState` is `Clone` (every field is `Arc<…>`) so axum's `FromRef`
/// machinery can produce a per-handler `State<Arc<FooUsecase>>` slice
/// from a single `.with_state(AppState::new(…))` call on the router.
#[derive(Clone)]
pub struct AppState {
    /// Handles `GET /api/v1/appraisals`.
    pub(crate) appraisals: Arc<GetAppraisalsUsecase>,
    /// Handles `GET /api/v1/health`.
    pub(crate) health: Arc<CheckHealthUsecase>,
    /// Handles `GET /api/v1/population`.
    pub(crate) population: Arc<GetPopulationUsecase>,
    /// Handles `GET /api/v1/vacancy`.
    pub(crate) vacancy: Arc<GetVacancyUsecase>,
    /// Handles `GET /api/v1/area-data`.
    pub(crate) area_data: Arc<GetAreaDataUsecase>,
    /// Handles `GET /api/v1/area-stats`.
    pub(crate) area_stats: Arc<GetAreaStatsUsecase>,
    /// Handles `GET /api/v1/land-prices`.
    pub(crate) land_prices: Arc<GetLandPricesUsecase>,
    /// Handles `GET /api/v1/land-prices/all-years`.
    pub(crate) land_prices_by_year_range: Arc<GetLandPricesByYearRangeUsecase>,
    /// Handles `GET /api/v1/municipalities`.
    pub(crate) municipalities: Arc<GetMunicipalitiesUsecase>,
    /// Handles `GET /api/v1/opportunities`. Reuses `score` internally.
    pub(crate) opportunities: Arc<GetOpportunitiesUsecase>,
    /// Handles `GET /api/v1/score`. Shared with `opportunities`.
    pub(crate) score: Arc<ComputeTlsUsecase>,
    /// Handles `GET /api/v1/stats`.
    pub(crate) stats: Arc<GetStatsUsecase>,
    /// Handles `GET /api/v1/transactions/summary`.
    pub(crate) transaction_summary: Arc<GetTransactionSummaryUsecase>,
    /// Handles `GET /api/v1/transactions`.
    pub(crate) transactions: Arc<GetTransactionsUsecase>,
    /// Handles `GET /api/v1/land-prices/aggregation`.
    pub(crate) land_price_aggregation: Arc<GetLandPriceAggregationUsecase>,
    /// Handles `GET /api/v1/transactions/aggregation`.
    pub(crate) transaction_aggregation: Arc<GetTransactionAggregationUsecase>,
    /// Handles `GET /api/v1/trend`.
    pub(crate) trend: Arc<GetTrendUsecase>,
    /// Reinfolib geospatial data source.
    ///
    /// Backed by [`PostgisFallback`] when `REINFOLIB_API_KEY` is absent, or
    /// [`LiveReinfolib`] when the key is present. Handlers that expose the
    /// reinfolib layers inject this field directly.
    pub(crate) reinfolib: Arc<dyn ReinfolibDataSource>,
}

impl AppState {
    /// Build the full dependency graph from a database pool and application config.
    ///
    /// The `config` reference is used to determine which reinfolib data source
    /// to instantiate: `PostgisFallback` (no API key) or `LiveReinfolib`
    /// (API key present).
    pub fn new(pool: PgPool, config: &Config) -> Self {
        let reinfolib_key_set = config.reinfolib_api_key.is_some();
        let reinfolib = create_reinfolib_source(pool.clone(), config);

        let mlit_config = MlitConfig {
            reinfolib_api_key: config.reinfolib_api_key.clone(),
            ..MlitConfig::default()
        };
        let jshis = match JshisClient::new(&mlit_config) {
            Ok(client) => {
                tracing::info!(
                    "J-SHIS client initialised (timeout {}s)",
                    mlit_config.request_timeout_secs
                );
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "J-SHIS client failed to build; seismic data disabled"
                );
                None
            }
        };

        // Shared across usecases: the TLS usecase is reused for both the
        // `/api/score` single-point endpoint and the `/api/v1/opportunities`
        // batch pipeline.
        let score = Arc::new(ComputeTlsUsecase::new(
            Arc::new(PgTlsRepository::new(pool.clone())),
            jshis,
        ));
        let land_price_repo = Arc::new(PgLandPriceRepository::new(pool.clone()));
        let trend_repo = Arc::new(PgTrendRepository::new(pool.clone()));
        let opportunities_cache = Arc::new(OpportunitiesCache::new());

        let opportunities = Arc::new(GetOpportunitiesUsecase::new(
            land_price_repo.clone(),
            score.clone(),
            opportunities_cache,
        ));
        let tx_repo = Arc::new(PgTransactionRepository::new(pool.clone()));
        let aggregation_repo = Arc::new(PgAggregationRepository::new(pool.clone()));

        Self {
            appraisals: Arc::new(GetAppraisalsUsecase::new(Arc::new(
                PgAppraisalRepository::new(pool.clone()),
            ))),
            health: Arc::new(CheckHealthUsecase::new(
                Arc::new(PgHealthRepository::new(pool.clone())),
                reinfolib_key_set,
            )),
            area_data: Arc::new(GetAreaDataUsecase::new(Arc::new(PgAreaRepository::new(
                pool.clone(),
            )))),
            area_stats: Arc::new(GetAreaStatsUsecase::new(Arc::new(
                PgAdminAreaStatsRepository::new(pool.clone()),
            ))),
            land_prices: Arc::new(GetLandPricesUsecase::new(land_price_repo.clone())),
            land_prices_by_year_range: Arc::new(GetLandPricesByYearRangeUsecase::new(
                land_price_repo,
            )),
            municipalities: Arc::new(GetMunicipalitiesUsecase::new(Arc::new(
                PgMunicipalityRepository::new(pool.clone()),
            ))),
            opportunities,
            score,
            stats: Arc::new(GetStatsUsecase::new(Arc::new(PgStatsRepository::new(
                pool.clone(),
            )))),
            transaction_summary: Arc::new(GetTransactionSummaryUsecase::new(tx_repo.clone())),
            transactions: Arc::new(GetTransactionsUsecase::new(tx_repo)),
            land_price_aggregation: Arc::new(GetLandPriceAggregationUsecase::new(
                aggregation_repo.clone(),
            )),
            transaction_aggregation: Arc::new(GetTransactionAggregationUsecase::new(
                aggregation_repo,
            )),
            trend: Arc::new(GetTrendUsecase::new(trend_repo)),
            population: Arc::new(GetPopulationUsecase::new(Arc::new(
                PgPopulationRepository::new(pool.clone()),
            ))),
            vacancy: Arc::new(GetVacancyUsecase::new(Arc::new(PgVacancyRepository::new(
                pool.clone(),
            )))),
            reinfolib,
        }
    }
}

// ── FromRef impls ────────────────────────────────────────────────────────────
//
// Each per-handler `State<Arc<FooUsecase>>` extractor is derived from the
// single shared `AppState` via `FromRef`. The implementations are trivial
// clones of the matching `Arc` field because every usecase is already wrapped
// in `Arc<…>` for shared ownership across tasks.

impl FromRef<AppState> for Arc<CheckHealthUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.health)
    }
}

impl FromRef<AppState> for Arc<GetAreaDataUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.area_data)
    }
}

impl FromRef<AppState> for Arc<GetAreaStatsUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.area_stats)
    }
}

impl FromRef<AppState> for Arc<GetLandPricesUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.land_prices)
    }
}

impl FromRef<AppState> for Arc<GetLandPricesByYearRangeUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.land_prices_by_year_range)
    }
}

impl FromRef<AppState> for Arc<GetOpportunitiesUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.opportunities)
    }
}

impl FromRef<AppState> for Arc<ComputeTlsUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.score)
    }
}

impl FromRef<AppState> for Arc<GetStatsUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.stats)
    }
}

impl FromRef<AppState> for Arc<GetTrendUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.trend)
    }
}

impl FromRef<AppState> for Arc<dyn ReinfolibDataSource> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.reinfolib)
    }
}

impl FromRef<AppState> for Arc<GetAppraisalsUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.appraisals)
    }
}

impl FromRef<AppState> for Arc<GetMunicipalitiesUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.municipalities)
    }
}

impl FromRef<AppState> for Arc<GetTransactionSummaryUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.transaction_summary)
    }
}

impl FromRef<AppState> for Arc<GetTransactionsUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.transactions)
    }
}

impl FromRef<AppState> for Arc<GetLandPriceAggregationUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.land_price_aggregation)
    }
}

impl FromRef<AppState> for Arc<GetTransactionAggregationUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.transaction_aggregation)
    }
}

impl FromRef<AppState> for Arc<GetPopulationUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.population)
    }
}

impl FromRef<AppState> for Arc<GetVacancyUsecase> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.vacancy)
    }
}
