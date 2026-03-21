use std::sync::Arc;

use realestate_geo_math::{finance::compute_cagr, rounding::round_dp};
use serde_json::json;

use crate::domain::constants::*;
use crate::domain::entity::PriceRecord;
use crate::domain::error::DomainError;
use crate::domain::repository::ScoreRepository;
use crate::domain::value_object::{Coord, InvestmentScore, ScoreComponent};

pub struct ComputeScoreUsecase {
    score_repo: Arc<dyn ScoreRepository>,
}

impl ComputeScoreUsecase {
    pub fn new(score_repo: Arc<dyn ScoreRepository>) -> Self {
        Self { score_repo }
    }

    /// Compute a composite investment score (0-100) for the given coordinate.
    ///
    /// All 5 repository queries execute in parallel (NFR-1: target < 500ms).
    pub async fn execute(&self, coord: &Coord) -> Result<InvestmentScore, DomainError> {
        let (
            prices,
            flood_overlap,
            steep_nearby,
            (schools_count, nearest_school),
            (medical_count, nearest_medical),
        ) = tokio::try_join!(
            self.score_repo.find_nearest_prices(coord),
            self.score_repo.calc_flood_overlap(coord),
            self.score_repo.has_steep_slope_nearby(coord),
            self.score_repo.count_schools_nearby(coord),
            self.score_repo.count_medical_nearby(coord),
        )?;

        let flood_overlap_fmt = format!("{:.3}", flood_overlap);
        tracing::debug!(
            price_records = prices.len(),
            flood_overlap = %flood_overlap_fmt,
            steep_nearby,
            schools_count,
            medical_count,
            "score inputs fetched"
        );

        let trend = compute_trend(&prices);
        let risk = compute_risk(flood_overlap, steep_nearby);
        let access = compute_access(
            schools_count,
            medical_count,
            nearest_school,
            nearest_medical,
        );
        let yield_potential = compute_yield_potential(&prices);

        let data_freshness = prices
            .last()
            .map(|p| p.year.to_string())
            .unwrap_or_else(|| "N/A".into());

        Ok(InvestmentScore {
            trend,
            risk,
            access,
            yield_potential,
            data_freshness,
        })
    }
}

// ─── Pure scoring functions (no I/O) ───────────────────

/// trend (0-25): CAGR = (latest / oldest)^(1/years) - 1; score = clamp(CAGR * 500, 0, 25)
fn compute_trend(prices: &[PriceRecord]) -> ScoreComponent {
    if prices.len() < 2 {
        return ScoreComponent {
            value: 0.0,
            max: SCORE_COMPONENT_MAX,
            detail: json!({
                "cagr_5y": 0,
                "direction": "unknown",
                "latest_price": null,
                "price_5y_ago": null,
            }),
        };
    }

    let oldest = &prices[0];
    let latest = &prices[prices.len() - 1];
    let years = (latest.year - oldest.year).max(1) as u32;
    let cagr = compute_cagr(
        oldest.price_per_sqm as f64,
        latest.price_per_sqm as f64,
        years,
    );
    let score = (cagr * TREND_CAGR_MULTIPLIER).clamp(0.0, SCORE_COMPONENT_MAX);
    let direction = if cagr > 0.0 { "up" } else { "down" };

    ScoreComponent {
        value: round_dp(score, PRECISION_SCORE),
        max: SCORE_COMPONENT_MAX,
        detail: json!({
            "cagr_5y": round_dp(cagr, PRECISION_RATIO),
            "direction": direction,
            "latest_price": latest.price_per_sqm,
            "price_5y_ago": oldest.price_per_sqm,
        }),
    }
}

/// risk (0-25): composite = flood*0.4 + liquefaction*0.4 + steep*0.2; score = 25*(1-composite)
fn compute_risk(flood_overlap: f64, steep_nearby: bool) -> ScoreComponent {
    let liquefaction_overlap = 0.0; // Phase 1: no liquefaction data yet
    let steep_factor = if steep_nearby { 1.0 } else { 0.0 };
    let composite = flood_overlap * RISK_WEIGHT_FLOOD
        + liquefaction_overlap * RISK_WEIGHT_LIQUEFACTION
        + steep_factor * RISK_WEIGHT_STEEP;
    let score = (SCORE_COMPONENT_MAX * (1.0 - composite)).clamp(0.0, SCORE_COMPONENT_MAX);

    ScoreComponent {
        value: round_dp(score, PRECISION_SCORE),
        max: SCORE_COMPONENT_MAX,
        detail: json!({
            "flood_overlap": round_dp(flood_overlap, PRECISION_RATIO),
            "liquefaction_overlap": liquefaction_overlap,
            "steep_slope_nearby": steep_nearby,
            "composite_risk": round_dp(composite, PRECISION_RATIO),
        }),
    }
}

/// access (0-25): school_score + medical_score + distance_score
fn compute_access(
    schools_1km: i64,
    medical_1km: i64,
    nearest_school_m: f64,
    nearest_medical_m: f64,
) -> ScoreComponent {
    let school_score =
        (schools_1km as f64 / ACCESS_SCHOOL_SATURATION).min(1.0) * ACCESS_SCHOOL_MAX_SCORE;
    let medical_score =
        (medical_1km as f64 / ACCESS_MEDICAL_SATURATION).min(1.0) * ACCESS_MEDICAL_MAX_SCORE;
    let distance_score =
        (ACCESS_DISTANCE_MAX_BONUS - nearest_school_m / ACCESS_DISTANCE_DIVISOR).max(0.0);
    let score = (school_score + medical_score + distance_score).clamp(0.0, SCORE_COMPONENT_MAX);

    ScoreComponent {
        value: round_dp(score, PRECISION_SCORE),
        max: SCORE_COMPONENT_MAX,
        detail: json!({
            "schools_1km": schools_1km,
            "medical_1km": medical_1km,
            "nearest_school_m": round_dp(nearest_school_m, PRECISION_DISTANCE),
            "nearest_medical_m": round_dp(nearest_medical_m, PRECISION_DISTANCE),
        }),
    }
}

/// yield_potential (0-25): Phase 1 estimate assumes transaction ≈ 80% of land price
fn compute_yield_potential(prices: &[PriceRecord]) -> ScoreComponent {
    let latest_price = prices.last().map(|p| p.price_per_sqm).unwrap_or(0);
    if latest_price == 0 {
        return ScoreComponent {
            value: 0.0,
            max: SCORE_COMPONENT_MAX,
            detail: json!({
                "avg_transaction_price": null,
                "land_price": null,
                "estimated_yield": 0,
            }),
        };
    }

    let avg_transaction = (latest_price as f64 * YIELD_TRANSACTION_RATIO) as i64;
    let estimated_yield = avg_transaction as f64 / latest_price as f64;
    let score = (estimated_yield * YIELD_SCORE_MULTIPLIER).clamp(0.0, SCORE_COMPONENT_MAX);

    ScoreComponent {
        value: round_dp(score, PRECISION_SCORE),
        max: SCORE_COMPONENT_MAX,
        detail: json!({
            "avg_transaction_price": avg_transaction,
            "land_price": latest_price,
            "estimated_yield": round_dp(estimated_yield, PRECISION_RATIO),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trend_with_insufficient_data_returns_zero() {
        let prices = vec![PriceRecord {
            year: 2023,
            price_per_sqm: 100_000,
            address: "test".into(),
            distance_m: 0.0,
        }];
        let result = compute_trend(&prices);
        assert!((result.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trend_positive_cagr() {
        let prices = vec![
            PriceRecord {
                year: 2019,
                price_per_sqm: 100_000,
                address: "a".into(),
                distance_m: 0.0,
            },
            PriceRecord {
                year: 2023,
                price_per_sqm: 120_000,
                address: "a".into(),
                distance_m: 0.0,
            },
        ];
        let result = compute_trend(&prices);
        assert!(result.value > 0.0);
    }

    #[test]
    fn risk_no_hazards_returns_max() {
        let result = compute_risk(0.0, false);
        assert!((result.value - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn risk_full_flood_reduces_score() {
        let result = compute_risk(1.0, true);
        assert!(result.value < 25.0);
    }

    #[test]
    fn access_zero_facilities() {
        let result = compute_access(0, 0, 1000.0, 1000.0);
        assert!((result.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn yield_no_price_data() {
        let result = compute_yield_potential(&[]);
        assert!((result.value - 0.0).abs() < f64::EPSILON);
    }
}
