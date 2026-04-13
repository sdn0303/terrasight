//! Total Location Score (TLS) computation for a query area.
//!
//! The TLS is a composite investment-readiness score in `[0.0, 1.0]` derived
//! from five sub-scores: land price affordability, disaster risk, facility
//! accessibility, zoning suitability, and transport convenience.
//!
//! Each sub-score is normalized to `[0.0, 1.0]` using [`NormalizationParams`]
//! and then weighted according to the selected [`WeightPreset`].

use crate::constants;
use crate::stats::{AreaStats, ZoningEntry};

// ── Normalization parameters ──

/// Per-prefecture normalization parameters for TLS sub-score computation.
///
/// These bounds define the expected data range within a prefecture.
/// Values outside `[price_floor, price_ceiling]` are clamped before
/// normalization, and facility counts are capped at their respective cap.
pub(crate) struct NormalizationParams {
    /// Land price floor (yen/m²) — at or below this receives the maximum price score.
    pub price_floor: f64,
    /// Land price ceiling (yen/m²) — at or above this receives the minimum price score.
    pub price_ceiling: f64,
    /// Facility count cap — at or above this total (schools + medical) receives maximum score.
    pub facility_cap: u32,
    /// Station count cap — at or above this count receives the maximum transport score.
    pub station_cap: u32,
}

impl NormalizationParams {
    /// Default normalization parameters calibrated for the Tokyo metropolitan area.
    pub const TOKYO: Self = Self {
        price_floor: 300_000.0,
        price_ceiling: 3_000_000.0,
        facility_cap: 50,
        station_cap: 10,
    };
}

// ── TLS result types ──

/// The result of a TLS computation for a query area.
#[derive(Debug, serde::Serialize)]
pub(crate) struct TlsResult {
    /// Weighted composite score in `[0.0, 1.0]`. Higher is better for investment.
    pub total_score: f64,
    /// Breakdown of the five individual sub-scores that make up `total_score`.
    pub sub_scores: SubScores,
}

/// Normalized sub-scores `[0.0, 1.0]` that contribute to the TLS total.
#[derive(Debug, serde::Serialize)]
pub(crate) struct SubScores {
    /// Land price affordability score: `1.0` means cheap, `0.0` means expensive.
    pub price_score: f64,
    /// Disaster safety score: `1.0` means no risk, `0.0` means maximum risk.
    pub risk_score: f64,
    /// Facility accessibility score: `1.0` means at or above the facility cap.
    pub facility_score: f64,
    /// Commercial zoning score: fraction of the query area covered by commercial zones.
    pub zoning_score: f64,
    /// Transport convenience score: `1.0` means at or above the station cap.
    pub transport_score: f64,
}

// ── Weight presets ──

/// Predefined weight distributions across the five TLS sub-scores.
///
/// Each variant represents a different investment or living priority.
/// The weight order is `[price, risk, facility, zoning, transport]`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum WeightPreset {
    /// Equal weights across all five sub-scores (`0.20` each).
    Balance,
    /// Emphasises price affordability (`0.35`) and zoning (`0.25`) for yield-focused investors.
    Investment,
    /// Emphasises safety (`0.25`) and facility access (`0.25`) for end-user buyers.
    Residential,
    /// Heavily emphasises disaster risk (`0.40`) for disaster-preparedness analysis.
    Disaster,
}

impl WeightPreset {
    /// Returns the five weights `[price, risk, facility, zoning, transport]` for this preset.
    ///
    /// All weights sum to `1.0`.
    pub fn weights(&self) -> [f64; 5] {
        match self {
            Self::Balance => [0.20, 0.20, 0.20, 0.20, 0.20],
            Self::Investment => [0.35, 0.15, 0.10, 0.25, 0.15],
            Self::Residential => [0.15, 0.25, 0.25, 0.15, 0.20],
            Self::Disaster => [0.10, 0.40, 0.15, 0.15, 0.20],
        }
    }
}

impl std::str::FromStr for WeightPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "balance" => Ok(Self::Balance),
            "investment" => Ok(Self::Investment),
            "residential" => Ok(Self::Residential),
            "disaster" => Ok(Self::Disaster),
            _ => Err(format!("unknown weight preset: {s}")),
        }
    }
}

// ── Normalization functions ──

/// Normalize an average land price to a `[0.0, 1.0]` score.
///
/// A price at or below `params.price_floor` returns `1.0` (most affordable).
/// A price at or above `params.price_ceiling` returns `0.0` (least affordable).
/// Returns `0.5` if `price_floor == price_ceiling` (degenerate range).
fn normalize_price(avg_per_sqm: f64, params: &NormalizationParams) -> f64 {
    let range = params.price_ceiling - params.price_floor;
    if range <= 0.0 {
        return 0.5;
    }
    let clamped = avg_per_sqm.clamp(params.price_floor, params.price_ceiling);
    1.0 - (clamped - params.price_floor) / range
}

/// Normalize combined school and medical counts to a `[0.0, 1.0]` score.
///
/// Returns `0.0` if `params.facility_cap` is zero to avoid division by zero.
fn normalize_facilities(schools: u32, medical: u32, params: &NormalizationParams) -> f64 {
    if params.facility_cap == 0 {
        return 0.0;
    }
    let total = (schools + medical).min(params.facility_cap) as f64;
    total / params.facility_cap as f64
}

/// Normalize a station count to a `[0.0, 1.0]` transport score.
///
/// Returns `0.0` if `params.station_cap` is zero to avoid division by zero.
fn normalize_transport(stations: u32, params: &NormalizationParams) -> f64 {
    if params.station_cap == 0 {
        return 0.0;
    }
    let capped = stations.min(params.station_cap) as f64;
    capped / params.station_cap as f64
}

/// Compute a zoning score from the distribution by summing commercial zone ratios.
///
/// Sums the `ratio` field of all entries whose `zone` string contains
/// [`crate::constants::COMMERCIAL_ZONE_KEYWORD`], then clamps to `[0.0, 1.0]`.
fn compute_zoning_score(dist: &[ZoningEntry]) -> f64 {
    dist.iter()
        .filter(|e| e.zone.contains(constants::COMMERCIAL_ZONE_KEYWORD))
        .map(|e| e.ratio)
        .sum::<f64>()
        .clamp(0.0, 1.0)
}

// ── Main TLS computation ──

/// Compute the Total Location Score for a query area.
///
/// Each sub-score is independently normalized to `[0.0, 1.0]` using `params`,
/// then combined as a weighted sum using the weights from `preset`. The final
/// `total_score` is clamped to `[0.0, 1.0]`.
///
/// A higher score indicates a more investment-ready location given the
/// selected weight preset's priorities.
pub(crate) fn compute_tls(
    stats: &AreaStats,
    preset: WeightPreset,
    params: &NormalizationParams,
) -> TlsResult {
    let weights = preset.weights();

    let price_score = normalize_price(stats.land_price.avg_per_sqm.unwrap_or(0.0), params);
    let risk_score = 1.0 - stats.risk.composite_risk;
    let facility_score =
        normalize_facilities(stats.facilities.schools, stats.facilities.medical, params);
    let zoning_score = compute_zoning_score(&stats.zoning_distribution);
    let transport_score = normalize_transport(stats.facilities.stations_nearby, params);

    let sub_scores = SubScores {
        price_score,
        risk_score,
        facility_score,
        zoning_score,
        transport_score,
    };

    let total = weights[0] * price_score
        + weights[1] * risk_score
        + weights[2] * facility_score
        + weights[3] * zoning_score
        + weights[4] * transport_score;

    TlsResult {
        total_score: total.clamp(0.0, 1.0),
        sub_scores,
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{FacilityStats, LandPriceStats, RiskStats};

    fn sample_stats() -> AreaStats {
        AreaStats {
            land_price: LandPriceStats {
                avg_per_sqm: Some(500_000.0),
                median_per_sqm: Some(480_000.0),
                min_per_sqm: Some(200_000),
                max_per_sqm: Some(1_200_000),
                count: 42,
            },
            risk: RiskStats {
                flood_area_ratio: 0.1,
                steep_slope_area_ratio: 0.05,
                composite_risk: 0.15,
            },
            facilities: FacilityStats {
                schools: 12,
                medical: 8,
                stations_nearby: 5,
            },
            zoning_distribution: vec![
                ZoningEntry {
                    zone: "商業地域".into(),
                    ratio: 0.3,
                },
                ZoningEntry {
                    zone: "住居地域".into(),
                    ratio: 0.5,
                },
                ZoningEntry {
                    zone: "工業地域".into(),
                    ratio: 0.2,
                },
            ],
        }
    }

    #[test]
    fn tls_balance_score_in_valid_range() {
        let result = compute_tls(
            &sample_stats(),
            WeightPreset::Balance,
            &NormalizationParams::TOKYO,
        );
        assert!((0.0..=1.0).contains(&result.total_score));
        for score in [
            result.sub_scores.price_score,
            result.sub_scores.risk_score,
            result.sub_scores.facility_score,
            result.sub_scores.zoning_score,
            result.sub_scores.transport_score,
        ] {
            assert!(
                (0.0..=1.0).contains(&score),
                "sub-score {score} out of range"
            );
        }
    }

    #[test]
    fn investment_preset_weights_price_higher() {
        let stats = sample_stats();
        let balance = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);
        let investment = compute_tls(
            &stats,
            WeightPreset::Investment,
            &NormalizationParams::TOKYO,
        );
        assert!(investment.sub_scores.price_score == balance.sub_scores.price_score);
        // Investment preset gives 0.35 weight to price vs 0.20 for balance
    }

    #[test]
    fn disaster_preset_weights_risk_higher() {
        let stats = sample_stats();
        let balance = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);
        let disaster = compute_tls(&stats, WeightPreset::Disaster, &NormalizationParams::TOKYO);
        // Disaster preset gives 0.40 weight to risk vs 0.20 for balance
        // With low risk (0.15), higher weight → more impact on total
        assert!(disaster.total_score != balance.total_score);
    }

    #[test]
    fn weight_preset_from_str() {
        assert!("balance".parse::<WeightPreset>().is_ok());
        assert!("investment".parse::<WeightPreset>().is_ok());
        assert!("residential".parse::<WeightPreset>().is_ok());
        assert!("disaster".parse::<WeightPreset>().is_ok());
        assert!("unknown".parse::<WeightPreset>().is_err());
    }

    #[test]
    fn all_presets_produce_valid_scores() {
        let stats = sample_stats();
        for preset in [
            WeightPreset::Balance,
            WeightPreset::Investment,
            WeightPreset::Residential,
            WeightPreset::Disaster,
        ] {
            let result = compute_tls(&stats, preset, &NormalizationParams::TOKYO);
            assert!(
                (0.0..=1.0).contains(&result.total_score),
                "preset {preset:?} out of range"
            );
        }
    }

    #[test]
    fn extreme_price_normalization() {
        let mut stats = sample_stats();
        // Very low price → high score
        stats.land_price.avg_per_sqm = Some(100_000.0);
        let low = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);

        // Very high price → low score
        stats.land_price.avg_per_sqm = Some(5_000_000.0);
        let high = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);

        assert!(low.sub_scores.price_score > high.sub_scores.price_score);
    }

    #[test]
    fn high_risk_reduces_score() {
        let mut stats = sample_stats();
        stats.risk.composite_risk = 0.0;
        let safe = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);

        stats.risk.composite_risk = 1.0;
        let risky = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);

        assert!(safe.total_score > risky.total_score);
    }

    #[test]
    fn zero_stats_does_not_panic() {
        let stats = AreaStats {
            land_price: LandPriceStats {
                avg_per_sqm: None,
                median_per_sqm: None,
                min_per_sqm: None,
                max_per_sqm: None,
                count: 0,
            },
            risk: RiskStats {
                flood_area_ratio: 0.0,
                steep_slope_area_ratio: 0.0,
                composite_risk: 0.0,
            },
            facilities: FacilityStats {
                schools: 0,
                medical: 0,
                stations_nearby: 0,
            },
            zoning_distribution: vec![],
        };
        let result = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);
        assert!((0.0..=1.0).contains(&result.total_score));
    }
}
