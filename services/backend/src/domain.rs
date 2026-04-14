//! Domain layer for the Terrasight API.
//!
//! This module is the innermost ring of the Clean Architecture stack. It
//! contains only pure business logic — no framework, no I/O, no database.
//! Every other crate layer (`handler`, `usecase`, `infra`) depends on this
//! one; this module depends on nothing outside the Rust standard library plus
//! a small set of data-representation crates.
//!
//! ## Allowed dependencies
//!
//! `std`, [`serde`], [`thiserror`], [`chrono`], [`async_trait`],
//! `serde_json` (data representation only — not I/O).
//!
//! ## Sub-modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`model`] | All domain model types — entities, value objects, and DTOs (flat re-exports) |
//! | [`error`] | Crate-wide [`DomainError`](error::DomainError) hierarchy |
//! | [`constants`] | Named constants for scoring thresholds, search radii, and API limits |
//! | [`repository`] | Async trait contracts for data access (implemented by the `infra` layer) |
//! | [`reinfolib`] | [`ReinfolibDataSource`](reinfolib::ReinfolibDataSource) trait abstracting the MLIT reinfolib API |

pub mod constants;
pub mod error;
pub mod model;
pub mod reinfolib;
pub mod repository;
