#![warn(missing_docs)]

//! # terrasight-domain
//!
//! Shared domain types, scoring logic, and constants for the Terrasight platform.
//!
//! This crate is a pure domain library with no I/O dependencies. It is consumed
//! by two separate runtimes:
//!
//! - **`terrasight-api`** — the Axum backend, which calls the scoring functions
//!   from within use-case handlers after fetching raw risk data from PostGIS.
//! - **`terrasight-wasm`** — the WASM spatial engine compiled for the browser,
//!   which uses the same scoring logic for client-side analysis without a
//!   network round-trip.
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`constants`] | Layer IDs, GeoJSON property keys, coordinate bounds, and risk weights shared by both runtimes |
//! | [`scoring`] | 5-axis TLS (Total Location Score) system — axis composition, sub-score mapping, grade thresholds |
//! | [`types`] | Aggregated statistics DTOs ([`types::LandPriceStats`], [`types::RiskStats`]) serialised to JSON |
//!
//! ## Quick Start
//!
//! ```rust
//! use terrasight_domain::scoring::tls::{compute_tls, WeightPreset};
//!
//! // Compute a balanced TLS from 5 axis scores (each 0–100).
//! let tls = compute_tls(80.0, 75.0, 90.0, 60.0, 55.0, WeightPreset::Balance);
//! assert!(tls > 0.0 && tls <= 100.0);
//! ```

pub mod constants;
pub mod scoring;
pub mod types;
