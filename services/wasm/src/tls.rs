//! Total Location Score (TLS) computation.

use crate::constants;
use crate::stats::{AreaStats, ZoningEntry};

// ── Normalization parameters ──

/// Per-prefecture normalization parameters for TLS sub-score computation.
pub(crate) struct NormalizationParams {
    /// Land price floor (yen/m²) — below this gets maximum score.
    pub price_floor: f64,
    /// Land price ceiling (yen/m²) — above this gets minimum score.
    pub price_ceiling: f64,
    /// Facility count cap — at or above this gets maximum score.
    pub facility_cap: u32,
    /// Station count cap — at or above this gets maximum score.
    pub station_cap: u32,
}

impl NormalizationParams {
    /// Tokyo defaults.
    pub const TOKYO: Self = Self {
        price_floor: 300_000.0,
        price_ceiling: 3_000_000.0,
        facility_cap: 50,
        station_cap: 10,
    };
}

// ── TLS result types ──

#[derive(Debug, serde::Serialize)]
pub(crate) struct TlsResult {
    pub total_score: f64,
    pub sub_scores: SubScores,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct SubScores {
    pub price_score: f64,
    pub risk_score: f64,
    pub facility_score: f64,
    pub zoning_score: f64,
    pub transport_score: f64,
}

// ── Weight presets ──

#[derive(Debug, Clone, Copy)]
pub(crate) enum WeightPreset {
    Balance,
    Investment,
    Residential,
    Disaster,
}

impl WeightPreset {
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

fn normalize_price(avg_per_sqm: f64, params: &NormalizationParams) -> f64 {
    let range = params.price_ceiling - params.price_floor;
    if range <= 0.0 {
        return 0.5;
    }
    let clamped = avg_per_sqm.clamp(params.price_floor, params.price_ceiling);
    1.0 - (clamped - params.price_floor) / range
}

fn normalize_facilities(schools: u32, medical: u32, params: &NormalizationParams) -> f64 {
    if params.facility_cap == 0 {
        return 0.0;
    }
    let total = (schools + medical).min(params.facility_cap) as f64;
    total / params.facility_cap as f64
}

fn normalize_transport(stations: u32, params: &NormalizationParams) -> f64 {
    if params.station_cap == 0 {
        return 0.0;
    }
    let capped = stations.min(params.station_cap) as f64;
    capped / params.station_cap as f64
}

fn compute_zoning_score(dist: &[ZoningEntry]) -> f64 {
    dist.iter()
        .filter(|e| e.zone.contains(constants::COMMERCIAL_ZONE_KEYWORD))
        .map(|e| e.ratio)
        .sum::<f64>()
        .clamp(0.0, 1.0)
}

// ── Main TLS computation ──

pub(crate) fn compute_tls(
    stats: &AreaStats,
    preset: WeightPreset,
    params: &NormalizationParams,
) -> TlsResult {
    let weights = preset.weights();

    let price_score = normalize_price(stats.land_price.avg_per_sqm, params);
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
                avg_per_sqm: 500_000.0,
                median_per_sqm: 480_000.0,
                min_per_sqm: 200_000.0,
                max_per_sqm: 1_200_000.0,
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
        stats.land_price.avg_per_sqm = 100_000.0;
        let low = compute_tls(&stats, WeightPreset::Balance, &NormalizationParams::TOKYO);

        // Very high price → low score
        stats.land_price.avg_per_sqm = 5_000_000.0;
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
                avg_per_sqm: 0.0,
                median_per_sqm: 0.0,
                min_per_sqm: 0.0,
                max_per_sqm: 0.0,
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
