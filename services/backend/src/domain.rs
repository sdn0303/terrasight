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
//! | [`entity`] | Core domain entities, value types, and the `nonempty_string_type!` macro |
//! | [`value_object`] | Validated newtypes that enforce domain invariants at construction |
//! | [`error`] | Crate-wide [`DomainError`](error::DomainError) hierarchy |
//! | [`constants`] | Named constants for scoring thresholds, search radii, and API limits |
//! | [`repository`] | Async trait contracts for data access (implemented by the `infra` layer) |
//! | [`appraisal`] | Official land appraisal record type (MLIT 鑑定評価) |
//! | [`municipality`] | Municipality lookup type (JIS X 0402 市区町村) |
//! | [`transaction`] | Real-estate transaction summary and detail types |
//! | [`reinfolib`] | [`ReinfolibDataSource`](reinfolib::ReinfolibDataSource) trait abstracting the MLIT reinfolib API |

pub mod appraisal;
pub mod constants;
pub mod entity;
pub mod error;
pub mod municipality;
pub mod reinfolib;
pub mod repository;
pub mod transaction;
pub mod value_object;
