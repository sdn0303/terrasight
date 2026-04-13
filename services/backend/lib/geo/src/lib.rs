//! Pure computation utilities for geospatial math, finance, and tiling.
//!
//! `terrasight-geo` is a zero-dependency library that centralises the pure
//! numeric and geometric functions shared across the Terrasight backend.
//! Because it carries no I/O or framework coupling, every function can be
//! unit-tested without database fixtures or network access.
//!
//! ## Modules
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | [`spatial`] | Bounding-box area, feature-count limits, point-to-polygon |
//! | [`tile`] | Web Mercator XYZ tile coordinate conversion |
//! | [`finance`] | Compound Annual Growth Rate (CAGR) |
//! | [`rounding`] | Decimal-place rounding for display values |

pub mod finance;
pub mod rounding;
pub mod spatial;
pub mod tile;
