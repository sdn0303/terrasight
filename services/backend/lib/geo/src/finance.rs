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

/// Estimate gross yield from the transaction-to-land-price ratio.
///
/// In the current model, gross yield equals the transaction ratio
/// because we assume annual rental income ≈ average transaction price.
///
/// # Examples
///
/// ```
/// use terrasight_geo::finance::estimate_yield;
///
/// assert_eq!(estimate_yield(1_000_000, 0.8), 0.8);
/// ```
pub fn estimate_yield(_land_price: i64, transaction_ratio: f64) -> f64 {
    transaction_ratio
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

    #[test]
    fn estimate_yield_typical() {
        assert_eq!(estimate_yield(1_000_000, 0.8), 0.8);
    }
}
