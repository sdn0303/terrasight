//! Decimal-place rounding for display and serialisation.
//!
//! [`round_dp`] is a thin wrapper around `f64::round` that avoids repeated
//! inline `10_f64.powi(…)` expressions throughout the codebase. Use it
//! whenever a computed value (price, CAGR, area) must be rounded before
//! being written into a response DTO or a GeoJSON property.

/// Round a floating-point value to the specified number of decimal places.
///
/// # Examples
///
/// ```
/// use terrasight_geo::rounding::round_dp;
///
/// assert_eq!(round_dp(3.14159, 2), 3.14);
/// assert_eq!(round_dp(25.55, 1), 25.6);
/// assert_eq!(round_dp(100.0, 0), 100.0);
/// ```
pub fn round_dp(value: f64, decimal_places: u32) -> f64 {
    let factor = 10_f64.powi(decimal_places as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_two_decimal_places() {
        assert_eq!(round_dp(3.14159, 2), 3.14);
    }

    #[test]
    fn round_one_decimal_place() {
        assert_eq!(round_dp(25.55, 1), 25.6);
    }

    #[test]
    fn round_zero_value() {
        assert_eq!(round_dp(0.0, 3), 0.0);
    }

    #[test]
    fn round_negative_value() {
        assert_eq!(round_dp(-1.555, 2), -1.56);
    }

    #[test]
    fn round_zero_decimal_places() {
        assert_eq!(round_dp(100.0, 0), 100.0);
    }
}
