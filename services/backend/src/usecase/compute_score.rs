use std::sync::Arc;

use mlit_client::jshis::JshisClient;
use realestate_geo_math::{finance::compute_cagr, rounding::round_dp};
use serde_json::json;

use crate::domain::constants::*;
use crate::domain::entity::PriceRecord;
use crate::domain::error::DomainError;
use crate::domain::repository::ScoreRepository;
use crate::domain::value_object::{Coord, InvestmentScore, ScoreComponent};

pub struct ComputeScoreUsecase {
    score_repo: Arc<dyn ScoreRepository>,
    /// J-SHIS seismic hazard client.
    ///
    /// `None` disables live seismic data lookups; the risk component falls back
    /// to zero values for the seismic and ground-amplification factors.
    jshis: Option<Arc<JshisClient>>,
}

impl ComputeScoreUsecase {
    /// Create a new usecase.
    ///
    /// # Parameters
    ///
    /// - `score_repo`: PostGIS-backed repository for spatial queries.
    /// - `jshis`: Optional J-SHIS client. Pass `None` to skip live seismic data
    ///   (useful in tests or when network access is unavailable). The risk score
    ///   degrades gracefully when `None` is provided.
    pub fn new(score_repo: Arc<dyn ScoreRepository>, jshis: Option<Arc<JshisClient>>) -> Self {
        Self { score_repo, jshis }
    }

    /// Compute a composite investment score (0-100) for the given coordinate.
    ///
    /// PostGIS queries and J-SHIS API calls execute in parallel.
    /// J-SHIS failures are logged as warnings and do not fail the request.
    pub async fn execute(&self, coord: &Coord) -> Result<InvestmentScore, DomainError> {
        // Kick off PostGIS queries and J-SHIS API calls concurrently.
        // J-SHIS is fetched independently; its failure must not propagate.
        let (db_results, jshis_data) =
            tokio::join!(self.fetch_db_inputs(coord), self.fetch_jshis_inputs(coord),);

        let (
            prices,
            flood_overlap,
            steep_nearby,
            (schools_count, nearest_school),
            (medical_count, nearest_medical),
        ) = db_results?;

        let (seismic_prob_6plus, avs30) = jshis_data;

        let flood_overlap_fmt = format!("{:.3}", flood_overlap);
        tracing::debug!(
            price_records = prices.len(),
            flood_overlap = %flood_overlap_fmt,
            steep_nearby,
            schools_count,
            medical_count,
            seismic_prob_6plus,
            avs30,
            "score inputs fetched"
        );

        let trend = compute_trend(&prices);
        let risk = compute_risk(flood_overlap, steep_nearby, seismic_prob_6plus, avs30);
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

    /// Execute all PostGIS queries in parallel via `tokio::try_join!`.
    async fn fetch_db_inputs(
        &self,
        coord: &Coord,
    ) -> Result<(Vec<PriceRecord>, f64, bool, (i64, f64), (i64, f64)), DomainError> {
        let (prices, flood_overlap, steep_nearby, schools, medical) = tokio::try_join!(
            self.score_repo.find_nearest_prices(coord),
            self.score_repo.calc_flood_overlap(coord),
            self.score_repo.has_steep_slope_nearby(coord),
            self.score_repo.count_schools_nearby(coord),
            self.score_repo.count_medical_nearby(coord),
        )?;
        Ok((prices, flood_overlap, steep_nearby, schools, medical))
    }

    /// Fetch seismic hazard and surface ground data from J-SHIS in parallel.
    ///
    /// Returns `(seismic_prob_6plus, avs30)`. Both values fall back to
    /// `(0.0, None)` when the client is absent or a call fails, logging a
    /// `WARN` for each failure so operators can detect degraded state.
    async fn fetch_jshis_inputs(&self, coord: &Coord) -> (f64, Option<f64>) {
        let client = match &self.jshis {
            Some(c) => c,
            None => return (0.0, None),
        };

        let lng = coord.lng();
        let lat = coord.lat();

        let (hazard_result, ground_result) = tokio::join!(
            client.get_seismic_hazard(lng, lat),
            client.get_surface_ground(lng, lat),
        );

        let seismic_prob_6plus = match hazard_result {
            Ok(h) => h.prob_level6_low_30yr.unwrap_or(0.0),
            Err(e) => {
                tracing::warn!(
                    lng,
                    lat,
                    error = %e,
                    "J-SHIS seismic hazard call failed; falling back to 0.0"
                );
                0.0
            }
        };

        let avs30 = match ground_result {
            Ok(g) => g.avs30,
            Err(e) => {
                tracing::warn!(
                    lng,
                    lat,
                    error = %e,
                    "J-SHIS surface ground call failed; AVS30 unavailable"
                );
                None
            }
        };

        (seismic_prob_6plus, avs30)
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

/// risk (0-25): composite = flood*0.25 + seismic*0.30 + steep*0.15 + ground_amp*0.30
///
/// # Parameters
///
/// - `flood_overlap`: Fraction of the query buffer covered by flood-risk zones (0.0–1.0).
/// - `steep_nearby`: Whether a steep-slope area exists within the search radius.
/// - `seismic_prob_6plus`: 30-year exceedance probability for seismic intensity ≥ 6弱 (0.0–1.0).
/// - `avs30`: Average S-wave velocity in the top 30 m (m/s). `None` uses a neutral factor of 0.0.
///
/// Ground amplification is derived from AVS30 thresholds:
/// - > 400 m/s → very firm → factor 0.0
/// - 200–400 m/s → moderate → factor [`RISK_GROUND_AMP_MODERATE`]
/// - < 200 m/s → soft → factor [`RISK_GROUND_AMP_SOFT`]
fn compute_risk(
    flood_overlap: f64,
    steep_nearby: bool,
    seismic_prob_6plus: f64,
    avs30: Option<f64>,
) -> ScoreComponent {
    let steep_factor = if steep_nearby { 1.0 } else { 0.0 };

    let ground_amplification = match avs30 {
        Some(v) if v > RISK_AVS30_FIRM => 0.0,
        Some(v) if v < RISK_AVS30_SOFT => RISK_GROUND_AMP_SOFT,
        Some(_) => RISK_GROUND_AMP_MODERATE,
        // AVS30 unavailable: use neutral factor to avoid penalising for missing data
        None => 0.0,
    };

    let composite = flood_overlap * RISK_WEIGHT_FLOOD
        + seismic_prob_6plus * RISK_WEIGHT_SEISMIC
        + steep_factor * RISK_WEIGHT_STEEP
        + ground_amplification * RISK_WEIGHT_GROUND_AMP;

    let score = (SCORE_COMPONENT_MAX * (1.0 - composite)).clamp(0.0, SCORE_COMPONENT_MAX);

    ScoreComponent {
        value: round_dp(score, PRECISION_SCORE),
        max: SCORE_COMPONENT_MAX,
        detail: json!({
            "flood_overlap": round_dp(flood_overlap, PRECISION_RATIO),
            "seismic_prob_6plus_30yr": round_dp(seismic_prob_6plus, PRECISION_RATIO),
            "avs30": avs30,
            "ground_amplification_factor": ground_amplification,
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
        // No flood, no steep slope, zero seismic probability, very firm ground.
        let result = compute_risk(0.0, false, 0.0, Some(500.0));
        assert!(
            (result.value - 25.0).abs() < f64::EPSILON,
            "expected 25.0, got {}",
            result.value
        );
    }

    #[test]
    fn risk_full_flood_reduces_score() {
        let result = compute_risk(1.0, true, 0.0, Some(500.0));
        assert!(result.value < 25.0);
    }

    #[test]
    fn risk_high_seismic_reduces_score() {
        // 80% probability of intensity ≥ 6弱 in 30 years should significantly lower the score.
        let result_high = compute_risk(0.0, false, 0.8, Some(500.0));
        let result_zero = compute_risk(0.0, false, 0.0, Some(500.0));
        assert!(
            result_high.value < result_zero.value,
            "high seismic probability should produce lower score: {} vs {}",
            result_high.value,
            result_zero.value
        );
        // Detail must expose the seismic probability.
        assert_eq!(
            result_high.detail["seismic_prob_6plus_30yr"],
            serde_json::json!(0.8)
        );
    }

    #[test]
    fn risk_soft_ground_reduces_score() {
        // Soft ground (AVS30 < 200) should score lower than very firm ground (AVS30 > 400).
        let soft = compute_risk(0.0, false, 0.0, Some(150.0));
        let firm = compute_risk(0.0, false, 0.0, Some(500.0));
        assert!(
            soft.value < firm.value,
            "soft ground should produce lower score: {} vs {}",
            soft.value,
            firm.value
        );
        // Moderate ground (200–400) should be between the two.
        let moderate = compute_risk(0.0, false, 0.0, Some(300.0));
        assert!(moderate.value < firm.value);
        assert!(moderate.value > soft.value);
    }

    #[test]
    fn risk_avs30_none_uses_neutral_factor() {
        // Missing AVS30 should not penalise the score relative to very firm ground.
        let no_avs30 = compute_risk(0.0, false, 0.0, None);
        let firm = compute_risk(0.0, false, 0.0, Some(500.0));
        assert!(
            (no_avs30.value - firm.value).abs() < f64::EPSILON,
            "None AVS30 should yield the same score as very firm ground: {} vs {}",
            no_avs30.value,
            firm.value
        );
        assert!(no_avs30.detail["avs30"].is_null());
    }

    #[test]
    fn risk_detail_contains_all_fields() {
        let result = compute_risk(0.3, true, 0.1, Some(250.0));
        assert!(result.detail["flood_overlap"].is_number());
        assert!(result.detail["seismic_prob_6plus_30yr"].is_number());
        assert!(result.detail["avs30"].is_number());
        assert!(result.detail["ground_amplification_factor"].is_number());
        assert!(result.detail["steep_slope_nearby"].is_boolean());
        assert!(result.detail["composite_risk"].is_number());
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
