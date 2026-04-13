//! # terrasight-mlit
//!
//! HTTP client for Japanese government real estate and geospatial APIs.
//!
//! ## Supported APIs
//!
//! - **reinfolib** — 不動産情報ライブラリ (Real Estate Information Library)
//! - **ksj** — 国土数値情報 (National Land Numerical Information)
//! - **estat** — e-Stat 政府統計 (Government Statistics)
//! - **jshis** — J-SHIS 地震ハザードステーション (Seismic Hazard Station)

pub mod config;
pub mod error;
pub mod types;

mod retry;

#[cfg(feature = "reinfolib")]
pub mod reinfolib;

#[cfg(feature = "ksj")]
pub mod ksj;

#[cfg(feature = "estat")]
pub mod estat;

#[cfg(feature = "jshis")]
pub mod jshis;
