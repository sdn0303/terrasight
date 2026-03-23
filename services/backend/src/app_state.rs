use std::sync::Arc;

use mlit_client::jshis::JshisClient;
use sqlx::PgPool;

use crate::config::Config;
use crate::domain::reinfolib::ReinfolibDataSource;
use crate::infra::pg_area_repository::PgAreaRepository;
use crate::infra::pg_health_repository::PgHealthRepository;
use crate::infra::pg_score_repository::PgScoreRepository;
use crate::infra::pg_stats_repository::PgStatsRepository;
use crate::infra::pg_trend_repository::PgTrendRepository;
use crate::infra::reinfolib_mock::create_reinfolib_source;
use crate::usecase::check_health::CheckHealthUsecase;
use crate::usecase::compute_score::ComputeScoreUsecase;
use crate::usecase::get_area_data::GetAreaDataUsecase;
use crate::usecase::get_stats::GetStatsUsecase;
use crate::usecase::get_trend::GetTrendUsecase;

/// Timeout (seconds) for J-SHIS API requests.
const JSHIS_TIMEOUT_SECS: u64 = 30;

/// Composition root: wires Infra → Domain traits → Usecases.
///
/// All dependency injection happens here. Each usecase is wrapped in `Arc`
/// for shared ownership across Axum handler tasks.
pub struct AppState {
    pub health: Arc<CheckHealthUsecase>,
    pub area_data: Arc<GetAreaDataUsecase>,
    pub score: Arc<ComputeScoreUsecase>,
    pub stats: Arc<GetStatsUsecase>,
    pub trend: Arc<GetTrendUsecase>,
    /// Reinfolib geospatial data source.
    ///
    /// Backed by [`PostgisFallback`] when `REINFOLIB_API_KEY` is absent, or
    /// [`LiveReinfolib`] when the key is present. Handlers that expose the
    /// reinfolib layers inject this field directly.
    pub reinfolib: Arc<dyn ReinfolibDataSource>,
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

        Self {
            health: Arc::new(CheckHealthUsecase::new(
                Arc::new(PgHealthRepository::new(pool.clone())),
                reinfolib_key_set,
            )),
            area_data: Arc::new(GetAreaDataUsecase::new(Arc::new(PgAreaRepository::new(
                pool.clone(),
            )))),
            score: Arc::new(ComputeScoreUsecase::new(
                Arc::new(PgScoreRepository::new(pool.clone())),
                jshis,
            )),
            stats: Arc::new(GetStatsUsecase::new(Arc::new(PgStatsRepository::new(
                pool.clone(),
            )))),
            trend: Arc::new(GetTrendUsecase::new(Arc::new(PgTrendRepository::new(pool)))),
            reinfolib,
        }
    }
}
