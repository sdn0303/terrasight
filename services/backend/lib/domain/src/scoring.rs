//! TLS (Total Location Score) — 5-axis scoring system for real estate investment analysis.
//!
//! The TLS condenses multi-dimensional location quality into a single 0–100 score
//! using five orthogonal axes. Each axis is composed from raw data sub-scores,
//! then blended with preset-specific weights to produce the final total.
//!
//! ## Axes
//!
//! | ID | Name | Data sources |
//! |----|------|--------------|
//! | S1 | Disaster resilience | Flood depth, liquefaction PL, seismic probability, tsunami depth, landslide zone |
//! | S2 | Terrain quality | AVS30 shear-wave velocity (Phase 1); terrain form and geology in future phases |
//! | S3 | Livability | Transit accessibility, education (school count, diversity), medical (hospitals, clinics, beds) |
//! | S4 | Future potential | Population trend, land price CAGR, floor area ratio surplus |
//! | S5 | Profitability | Relative price value (z-score), transaction volume |
//!
//! ## Sub-modules
//!
//! | Module | Role |
//! |--------|------|
//! | [`constants`] | All magic numbers — weights, thresholds, lookup tables |
//! | [`sub_scores`] | Pure functions mapping raw values → normalised 0–100 scores |
//! | [`axis`] | Axis composition: combines sub-scores with confidence tracking |
//! | [`tls`] | Final aggregation, [`tls::Grade`] assignment, weight presets, cross-analysis patterns |

pub mod axis;
pub mod constants;
pub mod sub_scores;
pub mod tls;
