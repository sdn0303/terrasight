//! Repository trait contracts for domain data access.
//!
//! Each submodule defines one trait — the interface between the `usecase` layer
//! and the `infra` layer. Usecases depend on these traits via `Arc<dyn Trait>`;
//! the `infra` layer provides concrete PostgreSQL implementations.
//!
//! ## Traits
//!
//! | Module | Trait | Infra impl |
//! |--------|-------|------------|
//! | [`admin_area`] | [`AdminAreaStatsRepository`] | `PgAdminAreaStatsRepository` |
//! | [`appraisal`] | [`AppraisalRepository`] | `PgAppraisalRepository` |
//! | [`health`] | [`HealthRepository`] | `PgHealthRepository` |
//! | [`land_price`] | [`LandPriceRepository`] | `PgLandPriceRepository` |
//! | [`layer`] | [`LayerRepository`] | `PgLayerRepository` |
//! | [`municipality`] | [`MunicipalityRepository`] | `PgMunicipalityRepository` |
//! | [`stats`] | [`StatsRepository`] | `PgStatsRepository` |
//! | [`tls`] | [`TlsRepository`] | `PgTlsRepository` |
//! | [`transaction`] | [`TransactionRepository`] | `PgTransactionRepository` |
//! | [`trend`] | [`TrendRepository`] | `PgTrendRepository` |
//!
//! ## Design principles
//!
//! - Traits are scoped to a single aggregate root (one table family per trait).
//! - All methods are `async` (via `async_trait`) and return `Result<_, DomainError>`.
//! - The infra layer must never let `sqlx::Error` or other framework errors
//!   escape — they are converted to [`DomainError::Database`] at the boundary.
//! - `mock` (test-only) provides in-process test doubles for every trait in
//!   this module, gated behind `#[cfg(test)]`.

mod admin_area;
mod appraisal;
mod health;
mod land_price;
mod layer;
mod municipality;
mod stats;
mod tls;
mod transaction;
mod trend;

pub use admin_area::AdminAreaStatsRepository;
pub use appraisal::AppraisalRepository;
pub use health::HealthRepository;
pub use land_price::LandPriceRepository;
pub use layer::LayerRepository;
pub use municipality::MunicipalityRepository;
pub use stats::StatsRepository;
pub use tls::TlsRepository;
pub use transaction::TransactionRepository;
pub use trend::TrendRepository;

#[cfg(test)]
pub mod mock;
