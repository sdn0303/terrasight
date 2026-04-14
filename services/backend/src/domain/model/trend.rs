//! Price trend domain types: time-series points, location metadata,
//! trend analysis results, direction classification, and lookback window.

use super::primitives::{Address, PricePerSqm, Year};
use crate::domain::constants::{TREND_DEFAULT_YEARS, TREND_MAX_YEARS, TREND_MIN_YEARS};

/// Single data point in a price trend time series.
#[derive(Debug, Clone)]
pub struct TrendPoint {
    /// Survey year.
    pub year: Year,
    /// Land price in JPY per square metre for that year.
    pub price_per_sqm: PricePerSqm,
}

/// Nearest observation point metadata attached to a trend response.
#[derive(Debug, Clone)]
pub struct TrendLocation {
    /// Human-readable address of the nearest land price point.
    pub address: Address,
    /// Distance in metres from the queried coordinate to the observation point.
    pub distance_m: f64,
}

/// Trend analysis result produced by the `GetTrendUsecase`.
///
/// Contains the nearest observation-point location, the raw time-series data,
/// and the derived CAGR summary. Passed directly to the handler for
/// serialisation into the `/api/v1/trend` response.
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Metadata about the nearest land price observation point.
    pub location: TrendLocation,
    /// Annual price data points over the requested lookback window.
    pub data: Vec<TrendPoint>,
    /// Compound annual growth rate (CAGR) across the lookback window.
    pub cagr: f64,
    /// Whether prices trended upward or downward over the lookback window.
    pub direction: TrendDirection,
}

/// Overall price trend direction over the lookback window.
#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    /// CAGR is positive — prices increased over the period.
    Up,
    /// CAGR is zero or negative — prices were flat or fell over the period.
    Down,
}

impl TrendDirection {
    /// Return the canonical REST API string for this direction.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// Trend lookback window in years.
///
/// Clamped to `[TREND_MIN_YEARS, TREND_MAX_YEARS]` via [`YearsLookback::clamped`].
/// `YearsLookback::DEFAULT` matches `TREND_DEFAULT_YEARS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearsLookback(i32);

impl YearsLookback {
    /// Default lookback window.
    pub const DEFAULT: Self = Self(TREND_DEFAULT_YEARS);

    /// Clamp a raw `i32` year count into `[TREND_MIN_YEARS, TREND_MAX_YEARS]`.
    pub fn clamped(value: i32) -> Self {
        Self(value.clamp(TREND_MIN_YEARS, TREND_MAX_YEARS))
    }

    /// Return the inner `i32` value.
    pub fn value(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn years_lookback_clamps_to_valid_range() {
        assert_eq!(YearsLookback::clamped(0).value(), TREND_MIN_YEARS);
        assert_eq!(YearsLookback::clamped(5).value(), 5);
        assert_eq!(YearsLookback::clamped(100).value(), TREND_MAX_YEARS);
        assert_eq!(YearsLookback::DEFAULT.value(), TREND_DEFAULT_YEARS);
    }
}
