use std::sync::Arc;

use sqlx::PgPool;

use crate::infra::pg_area_repository::PgAreaRepository;
use crate::infra::pg_health_repository::PgHealthRepository;
use crate::infra::pg_score_repository::PgScoreRepository;
use crate::infra::pg_stats_repository::PgStatsRepository;
use crate::infra::pg_trend_repository::PgTrendRepository;
use crate::usecase::check_health::CheckHealthUsecase;
use crate::usecase::compute_score::ComputeScoreUsecase;
use crate::usecase::get_area_data::GetAreaDataUsecase;
use crate::usecase::get_stats::GetStatsUsecase;
use crate::usecase::get_trend::GetTrendUsecase;

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
}

impl AppState {
    /// Build the full dependency graph from a database pool and config flags.
    pub fn new(pool: PgPool, reinfolib_key_set: bool) -> Self {
        Self {
            health: Arc::new(CheckHealthUsecase::new(
                Arc::new(PgHealthRepository::new(pool.clone())),
                reinfolib_key_set,
            )),
            area_data: Arc::new(GetAreaDataUsecase::new(Arc::new(PgAreaRepository::new(
                pool.clone(),
            )))),
            score: Arc::new(ComputeScoreUsecase::new(Arc::new(PgScoreRepository::new(
                pool.clone(),
            )))),
            stats: Arc::new(GetStatsUsecase::new(Arc::new(PgStatsRepository::new(
                pool.clone(),
            )))),
            trend: Arc::new(GetTrendUsecase::new(Arc::new(PgTrendRepository::new(pool)))),
        }
    }
}
