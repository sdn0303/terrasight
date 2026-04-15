//! Domain model types — entities, value objects, and DTOs.
//!
//! This module re-exports all domain types from focused submodules so that
//! consumers can import everything via a single path:
//!
//! ```rust,ignore
//! use crate::domain::model::{BBox, GeoFeature, PrefCode};
//! ```

mod aggregation;
mod appraisal;
mod area;
mod geo;
mod health;
mod municipality;
mod opportunity;
mod price;
pub mod primitives; // pub because repository traits need direct access
mod tls;
mod transaction;
mod trend;

// Re-export everything flat
pub use aggregation::*;
pub use appraisal::*;
pub use area::*;
pub use geo::*;
pub use health::*;
pub use municipality::*;
pub use opportunity::*;
pub use price::*;
pub use primitives::*;
pub use tls::*;
pub use transaction::*;
pub use trend::*;
