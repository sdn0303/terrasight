//! Total Location Score (TLS) sub-score types: school stats, medical stats,
//! z-score results, composite score, and risk level classification.

use crate::domain::error::DomainError;

/// School accessibility details used in TLS S2 (education sub-score).
///
/// Collected within an 800 m radius of the scored coordinate by
/// [`TlsRepository::find_schools_nearby`](crate::domain::repository::TlsRepository::find_schools_nearby).
#[derive(Debug, Clone)]
pub struct SchoolStats {
    /// Total number of school facilities within 800 m.
    pub count_800m: i64,
    /// `true` when at least one elementary school (小学校) is present.
    pub has_primary: bool,
    /// `true` when at least one junior-high school (中学校) is present.
    pub has_junior_high: bool,
}

/// Medical facility details used in TLS S3 (medical sub-score).
///
/// Collected within a 1 000 m radius of the scored coordinate by
/// [`TlsRepository::find_medical_nearby`](crate::domain::repository::TlsRepository::find_medical_nearby).
#[derive(Debug, Clone)]
pub struct MedicalStats {
    /// Number of hospitals (病院, ≥ 20 beds) within 1 000 m.
    pub hospital_count: i64,
    /// Number of clinics (診療所, < 20 beds) within 1 000 m.
    pub clinic_count: i64,
    /// Sum of licensed bed counts across all hospitals within 1 000 m.
    pub total_beds: i64,
}

/// Z-score of a land price observation relative to all prices in the same
/// zoning type.
///
/// Used in TLS P2 (price attractiveness) to flag statistically cheap
/// locations within their zoning class.
#[derive(Debug, Clone)]
pub struct ZScoreResult {
    /// Standardised score. Negative values are below the zoning-type mean.
    pub z_score: f64,
    /// JIS zoning type code used as the comparison population.
    pub zone_type: String,
    /// Number of records in the comparison population.
    pub sample_count: i64,
}

/// TLS (Total Location Score) clamped to `0..=100` and stored as `u8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TlsScore(u8);

impl TlsScore {
    /// Construct a `TlsScore` from a raw `f64`, clamping to `[0, 100]`.
    ///
    /// `NaN` is mapped to `0` (infallible fallback for defensive callers).
    pub fn from_f64_clamped(value: f64) -> Self {
        if value.is_nan() {
            return Self(0);
        }
        Self(value.clamp(0.0, 100.0) as u8)
    }

    /// Return the TLS score as a `u8` in `0..=100`.
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Risk level bucket derived from the S1 Disaster sub-score.
///
/// Higher S1 scores (safer locations) map to [`Low`](RiskLevel::Low).
/// The mapping thresholds are defined in `terrasight-domain` scoring constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    /// Disaster score is above the low-risk threshold — safe to invest.
    Low,
    /// Disaster score is between the mid and low thresholds — some caution warranted.
    Mid,
    /// Disaster score is below the mid-risk threshold — significant hazard exposure.
    High,
}

impl RiskLevel {
    /// Parse a risk level from a REST API query-string value.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] for any value other than `"low"`,
    /// `"mid"`, or `"high"`.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        match s {
            "low" => Ok(Self::Low),
            "mid" => Ok(Self::Mid),
            "high" => Ok(Self::High),
            other => Err(DomainError::Validation(format!(
                "risk_max must be one of low|mid|high, got {other:?}"
            ))),
        }
    }

    /// Return the canonical REST API string for this risk level.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Mid => "mid",
            Self::High => "high",
        }
    }

    /// Derive a [`RiskLevel`] from a raw S1 disaster sub-score (higher = safer).
    pub fn from_disaster_score(score: f64) -> Self {
        use terrasight_domain::scoring::constants::{
            DISASTER_SCORE_LOW_THRESHOLD, DISASTER_SCORE_MID_THRESHOLD,
        };
        if score >= DISASTER_SCORE_LOW_THRESHOLD {
            Self::Low
        } else if score >= DISASTER_SCORE_MID_THRESHOLD {
            Self::Mid
        } else {
            Self::High
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_score_clamps_and_handles_nan() {
        assert_eq!(TlsScore::from_f64_clamped(-10.0).value(), 0);
        assert_eq!(TlsScore::from_f64_clamped(0.0).value(), 0);
        assert_eq!(TlsScore::from_f64_clamped(50.7).value(), 50);
        assert_eq!(TlsScore::from_f64_clamped(100.0).value(), 100);
        assert_eq!(TlsScore::from_f64_clamped(150.0).value(), 100);
        assert_eq!(TlsScore::from_f64_clamped(f64::NAN).value(), 0);
    }

    #[test]
    fn risk_level_parse_and_display() {
        assert_eq!(RiskLevel::parse("low").unwrap(), RiskLevel::Low);
        assert_eq!(RiskLevel::parse("mid").unwrap(), RiskLevel::Mid);
        assert_eq!(RiskLevel::parse("high").unwrap(), RiskLevel::High);
        assert!(RiskLevel::parse("bad").is_err());
        assert_eq!(RiskLevel::Low.as_str(), "low");
        assert_eq!(RiskLevel::High.as_str(), "high");
    }

    #[test]
    fn risk_level_from_disaster_score() {
        assert_eq!(RiskLevel::from_disaster_score(80.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_disaster_score(75.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_disaster_score(60.0), RiskLevel::Mid);
        assert_eq!(RiskLevel::from_disaster_score(50.0), RiskLevel::Mid);
        assert_eq!(RiskLevel::from_disaster_score(30.0), RiskLevel::High);
    }
}
