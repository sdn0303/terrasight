//! Financial computation utilities for real estate investment metrics.
//!
//! Currently exposes [`compute_cagr`], which calculates the Compound Annual
//! Growth Rate from a pair of land prices and an elapsed year count. This is
//! the primary metric surfaced to users on the Terrasight map as an indicator
//! of long-term investment performance.

/// Compute Compound Annual Growth Rate (CAGR).
///
/// `CAGR = (latest / oldest)^(1/years) - 1`
///
/// Returns `0.0` if `oldest_price <= 0.0` or `years == 0`.
///
/// # Examples
///
/// ```
/// use terrasight_geo::finance::compute_cagr;
///
/// let cagr = compute_cagr(100_000.0, 120_000.0, 4);
/// assert!((cagr - 0.0466).abs() < 0.001);
///
/// assert_eq!(compute_cagr(100_000.0, 100_000.0, 5), 0.0);
/// assert_eq!(compute_cagr(0.0, 100.0, 5), 0.0);
/// ```
pub fn compute_cagr(oldest_price: f64, latest_price: f64, years: u32) -> f64 {
    if oldest_price <= 0.0 || years == 0 {
        return 0.0;
    }
    (latest_price / oldest_price).powf(1.0 / years as f64) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cagr_positive_growth() {
        let result = compute_cagr(100_000.0, 120_000.0, 4);
        assert!(
            (result - 0.0466).abs() < 0.001,
            "expected ~0.0466, got {result}"
        );
    }

    #[test]
    fn cagr_no_growth() {
        assert_eq!(compute_cagr(100_000.0, 100_000.0, 5), 0.0);
    }

    #[test]
    fn cagr_zero_oldest_price() {
        assert_eq!(compute_cagr(0.0, 100.0, 5), 0.0);
    }

    #[test]
    fn cagr_negative_oldest_price() {
        assert_eq!(compute_cagr(-1.0, 100.0, 5), 0.0);
    }

    #[test]
    fn cagr_zero_years() {
        assert_eq!(compute_cagr(100.0, 200.0, 0), 0.0);
    }

    #[test]
    fn cagr_decline() {
        let result = compute_cagr(100.0, 50.0, 2);
        assert!(
            result < 0.0,
            "declining prices should yield negative CAGR, got {result}"
        );
    }
}
