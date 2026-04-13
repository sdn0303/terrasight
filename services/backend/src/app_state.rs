use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;
use terrasight_mlit::jshis::JshisClient;

use crate::config::Config;
use crate::domain::constants::JSHIS_TIMEOUT_SECS;
use crate::domain::reinfolib::ReinfolibDataSource;
use crate::infra::opportunities_cache::OpportunitiesCache;
use crate::infra::pg_admin_area_stats_repository::PgAdminAreaStatsRepository;
use crate::infra::pg_appraisal_repository::PgAppraisalRepository;
use crate::infra::pg_area_repository::PgAreaRepository;
use crate::infra::pg_health_repository::PgHealthRepository;
use crate::infra::pg_land_price_repository::PgLandPriceRepository;
use crate::infra::pg_municipality_repository::PgMunicipalityRepository;
use crate::infra::pg_stats_repository::PgStatsRepository;
use crate::infra::pg_tls_repository::PgTlsRepository;
use crate::infra::pg_transaction_repository::PgTransactionRepository;
use crate::infra::pg_trend_repository::PgTrendRepository;
use crate::infra::reinfolib_mock::create_reinfolib_source;
use crate::usecase::check_health::CheckHealthUsecase;
use crate::usecase::compute_tls::ComputeTlsUsecase;
use crate::usecase::get_appraisals::GetAppraisalsUsecase;
use crate::usecase::get_area_data::GetAreaDataUsecase;
use crate::usecase::get_area_stats::GetAreaStatsUsecase;
use crate::usecase::get_land_prices::GetLandPricesUsecase;
use crate::usecase::get_land_prices_by_year_range::GetLandPricesByYearRangeUsecase;
use crate::usecase::get_municipalities::GetMunicipalitiesUsecase;
use crate::usecase::get_opportunities::GetOpportunitiesUsecase;
use crate::usecase::get_stats::GetStatsUsecase;
use crate::usecase::get_transaction_summary::GetTransactionSummaryUsecase;
use crate::usecase::get_transactions::GetTransactionsUsecase;
use crate::usecase::get_trend::GetTrendUsecase;

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
    pub(crate) appraisals: Arc<GetAppraisalsUsecase>,
    pub(crate) health: Arc<CheckHealthUsecase>,
    pub(crate) area_data: Arc<GetAreaDataUsecase>,
    pub(crate) area_stats: Arc<GetAreaStatsUsecase>,
    pub(crate) land_prices: Arc<GetLandPricesUsecase>,
    pub(crate) land_prices_by_year_range: Arc<GetLandPricesByYearRangeUsecase>,
    pub(crate) municipalities: Arc<GetMunicipalitiesUsecase>,
    pub(crate) opportunities: Arc<GetOpportunitiesUsecase>,
    pub(crate) score: Arc<ComputeTlsUsecase>,
    pub(crate) stats: Arc<GetStatsUsecase>,
    pub(crate) transaction_summary: Arc<GetTransactionSummaryUsecase>,
    pub(crate) transactions: Arc<GetTransactionsUsecase>,
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
    /// to instantiate: [`PostgisFallback`] (no API key) or [`LiveReinfolib`]
    /// (API key present).
    pub fn new(pool: PgPool, config: &Config) -> Self {
        let reinfolib_key_set = config.reinfolib_api_key.is_some();
        let reinfolib = create_reinfolib_source(pool.clone(), config);

        let jshis = match JshisClient::new(JSHIS_TIMEOUT_SECS) {
            Ok(client) => {
                tracing::info!(
                    "J-SHIS client initialised (timeout {}s)",
                    JSHIS_TIMEOUT_SECS
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
            transaction_summary: Arc::new(GetTransactionSummaryUsecase::new(Arc::new(
                PgTransactionRepository::new(pool.clone()),
            ))),
            transactions: Arc::new(GetTransactionsUsecase::new(Arc::new(
                PgTransactionRepository::new(pool.clone()),
            ))),
            trend: Arc::new(GetTrendUsecase::new(trend_repo)),
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
