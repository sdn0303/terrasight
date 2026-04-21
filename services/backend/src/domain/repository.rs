//! Repository trait contracts for domain data access.
//!
//! Each submodule defines one trait — the interface between the `usecase` layer
//! and the `infra` layer. Usecases depend on these traits via `Arc<dyn Trait>`;
//! the `infra` layer provides concrete PostgreSQL implementations.
//!
//! ## Traits
//!
//! | Trait | Infra impl |
//! |-------|------------|
//! | [`AggregationRepository`] | `PgAggregationRepository` |
//! | [`AdminAreaStatsRepository`] | `PgAdminAreaStatsRepository` |
//! | [`AppraisalRepository`] | `PgAppraisalRepository` |
//! | [`HealthRepository`] | `PgHealthRepository` |
//! | [`LandPriceRepository`] | `PgLandPriceRepository` |
//! | [`LayerRepository`] | `PgLayerRepository` |
//! | [`MunicipalityRepository`] | `PgMunicipalityRepository` |
//! | [`PopulationRepository`] | `PgPopulationRepository` |
//! | [`StatsRepository`] | `PgStatsRepository` |
//! | [`TlsRepository`] | `PgTlsRepository` |
//! | [`TransactionRepository`] | `PgTransactionRepository` |
//! | [`TrendRepository`] | `PgTrendRepository` |
//! | [`VacancyRepository`] | `PgVacancyRepository` |
//!
//! ## Design principles
//!
//! - Traits are scoped to a single aggregate root (one table family per trait).
//! - All methods are `async` (via `async_trait`) and return `Result<_, DomainError>`.
//! - The infra layer must never let `sqlx::Error` or other framework errors
//!   escape — they are converted to `DomainError::Database` at the boundary.
//! - `mock` (test-only) provides in-process test doubles for every trait in
//!   this module, gated behind `#[cfg(test)]`.

mod admin_area;
mod aggregation;
mod appraisal;
mod health;
mod land_price;
mod layer;
mod municipality;
mod population;
mod stats;
mod tls;
mod transaction;
mod trend;
mod vacancy;

pub use admin_area::AdminAreaStatsRepository;
pub use aggregation::AggregationRepository;
pub use appraisal::AppraisalRepository;
pub use health::HealthRepository;
pub use land_price::LandPriceRepository;
pub use layer::LayerRepository;
pub use municipality::MunicipalityRepository;
pub use population::PopulationRepository;
pub use stats::StatsRepository;
pub use tls::TlsRepository;
pub use transaction::TransactionRepository;
pub use trend::TrendRepository;
pub use vacancy::VacancyRepository;

#[cfg(test)]
pub mod mock;
