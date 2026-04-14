//! Pure sub-score mapping functions.
//!
//! Each function converts a raw data value into a normalised 0–100 score.
//! Functions whose data source may be absent accept `Option` inputs and return
//! [`crate::scoring::constants::UNAVAILABLE_DEFAULT`] (100) when the value is
//! `None` — the best-case assumption used during Phase 1 data onboarding.
//!
//! All output values are in the range `[0.0, 100.0]` unless stated otherwise.
//! Lookup tables and threshold constants are defined in
//! [`crate::scoring::constants`].

use crate::scoring::constants::*;

// ═══════════════════════════════════════════════════════════════════════════
// S1 Disaster sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Maps a flood inundation depth rank to a 0–100 score.
///
/// Input `depth_rank` follows the MLIT洪水浸水想定 depth classification:
///
/// | `depth_rank` | Inundation depth | Score |
/// |---|---|---|
/// | `None` or `Some(0)` | Outside flood zone (区域外) | 100 |
/// | `Some(1)` | < 0.5 m | 80 |
/// | `Some(2)` | 0.5–3 m | 50 |
/// | `Some(3)` | 3–5 m | 20 |
/// | `Some(4)` | 5–10 m | 5 |
/// | `Some(5)` | ≥ 10 m | 0 |
///
/// Any rank not in the table falls back to [`FLOOD_DEFAULT`] (100). The
/// DB column `flood_risk.depth_rank` is constrained to `[0, 5]`
/// (migration `20260326000001_schema_redesign.sql`).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_flood;
///
/// assert_eq!(score_flood(None), 100.0);    // outside flood zone
/// assert_eq!(score_flood(Some(1)), 80.0);  // shallow inundation
/// assert_eq!(score_flood(Some(5)), 0.0);   // catastrophic depth
/// ```
#[must_use]
pub fn score_flood(depth_rank: Option<i32>) -> f64 {
    match depth_rank {
        None => FLOOD_DEFAULT,
        Some(rank) => FLOOD_MAP
            .iter()
            .find(|(r, _)| *r == rank)
            .map(|(_, s)| *s)
            .unwrap_or(FLOOD_DEFAULT),
    }
}

/// Maps a liquefaction potential index (PL) to a 0–100 score.
///
/// Input `pl_value` is the PL index from boring survey data.
/// `None` means survey data is unavailable and returns
/// [`UNAVAILABLE_DEFAULT`] (100).
///
/// | PL range | Score |
/// |---|---|
/// | `None` | 100 |
/// | PL = 0 | 100 |
/// | 0 < PL ≤ 5 | 80 |
/// | 5 < PL ≤ 15 | 40 |
/// | PL > 15 | 10 (see [`LIQUEFACTION_HIGH`]) |
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_liquefaction;
///
/// assert_eq!(score_liquefaction(None), 100.0);
/// assert_eq!(score_liquefaction(Some(0.0)), 100.0);
/// assert_eq!(score_liquefaction(Some(20.0)), 10.0);
/// ```
#[must_use]
pub fn score_liquefaction(pl_value: Option<f64>) -> f64 {
    match pl_value {
        None => UNAVAILABLE_DEFAULT,
        Some(pl) => {
            if pl <= 0.0 {
                return LIQUEFACTION_MAP[0].1; // PL = 0 → 100
            }
            for &(upper, score) in &LIQUEFACTION_MAP[1..] {
                if pl <= upper {
                    return score;
                }
            }
            LIQUEFACTION_HIGH
        }
    }
}

/// Maps a 30-year earthquake exceedance probability (0.0–1.0) to a 0–100 score.
///
/// Source: National Seismic Hazard Map (J-SHIS, NIED). Higher probabilities
/// produce lower scores because they indicate greater seismic risk.
///
/// | Probability | Score |
/// |---|---|
/// | < 0.03 (< 3%) | 100 |
/// | 0.03–0.06 | 75 |
/// | 0.06–0.26 | 50 |
/// | 0.26–0.50 | 25 |
/// | > 0.50 | 5 (see [`SEISMIC_HIGH`]) |
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_seismic;
///
/// assert_eq!(score_seismic(0.02), 100.0);
/// assert_eq!(score_seismic(0.50), 5.0);
/// ```
#[must_use]
pub fn score_seismic(prob_30yr: f64) -> f64 {
    for &(upper, score) in SEISMIC_MAP {
        if prob_30yr < upper {
            return score;
        }
    }
    SEISMIC_HIGH
}

/// Maps an expected tsunami inundation depth (metres) to a 0–100 score.
///
/// `None` means the parcel is not in a designated tsunami hazard zone and
/// returns [`UNAVAILABLE_DEFAULT`] (100).
///
/// | Depth | Score |
/// |---|---|
/// | `None` | 100 |
/// | 0 m | 100 |
/// | < 0.3 m | 85 |
/// | 0.3–1.0 m | 60 |
/// | 1.0–2.0 m | 35 |
/// | > 2.0 m | 10 (see [`TSUNAMI_HIGH`]) |
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_tsunami;
///
/// assert_eq!(score_tsunami(None), 100.0);
/// assert_eq!(score_tsunami(Some(0.0)), 100.0);
/// assert_eq!(score_tsunami(Some(3.0)), 10.0);
/// ```
#[must_use]
pub fn score_tsunami(depth_m: Option<f64>) -> f64 {
    match depth_m {
        None => UNAVAILABLE_DEFAULT,
        Some(d) => {
            for &(upper, score) in TSUNAMI_MAP {
                if d <= upper {
                    return score;
                }
            }
            TSUNAMI_HIGH
        }
    }
}

/// Maps steep-slope / landslide hazard presence to a 0–100 score.
///
/// `None` means survey data is unavailable and returns
/// [`UNAVAILABLE_DEFAULT`] (100). `Some(true)` means the parcel is within or
/// adjacent to a 土砂災害警戒区域 (landslide warning zone).
///
/// | Input | Score |
/// |---|---|
/// | `None` | 100 (unavailable) |
/// | `Some(false)` | 100 (no hazard) — [`LANDSLIDE_NONE`] |
/// | `Some(true)` | 40 (warning zone) — [`LANDSLIDE_WARNING`] |
///
/// Future data integration will distinguish 警戒区域 (warning, score 40) from
/// 特別警戒区域 (special warning, planned score ≈ 10).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_landslide;
///
/// assert_eq!(score_landslide(None), 100.0);
/// assert_eq!(score_landslide(Some(false)), 100.0);
/// assert_eq!(score_landslide(Some(true)), 40.0);
/// ```
#[must_use]
pub fn score_landslide(steep_nearby: Option<bool>) -> f64 {
    match steep_nearby {
        None => UNAVAILABLE_DEFAULT,
        Some(false) => LANDSLIDE_NONE,
        Some(true) => LANDSLIDE_WARNING,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// S2 Terrain sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Maps an AVS30 shear-wave velocity (m/s) to a 0–100 ground-quality score.
///
/// `None` means survey data is unavailable and returns
/// [`UNAVAILABLE_DEFAULT`] (100). The lookup table ([`AVS30_MAP`]) is
/// evaluated in descending order of the lower bound.
///
/// | AVS30 (m/s) | Ground type | Score |
/// |---|---|---|
/// | `None` | Unavailable | 100 |
/// | ≥ 400 | Rock / very firm | 100 |
/// | 300–400 | Gravel / good | 85 |
/// | 200–300 | Moderate | 60 |
/// | 150–200 | Slightly soft | 35 |
/// | < 150 | Very soft | 10 — [`AVS30_SOFT`] |
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_avs30;
///
/// assert_eq!(score_avs30(None), 100.0);
/// assert_eq!(score_avs30(Some(500.0)), 100.0);
/// assert_eq!(score_avs30(Some(100.0)), 10.0);
/// ```
#[must_use]
pub fn score_avs30(avs30: Option<f64>) -> f64 {
    match avs30 {
        None => UNAVAILABLE_DEFAULT,
        Some(v) => {
            for &(lower, score) in AVS30_MAP {
                if v >= lower {
                    return score;
                }
            }
            AVS30_SOFT
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// S3 Livability sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Computes an education accessibility score from school count and type diversity.
///
/// Formula: `L_edu = min(100, school_count × 12 + diversity_bonus)`
/// where `diversity_bonus = (has_primary + has_junior_high) × 15`.
///
/// Constants: [`EDU_SCORE_PER_SCHOOL`] (12), [`EDU_DIVERSITY_BONUS`] (15).
///
/// Output is in `[0.0, 100.0]`.
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_education;
///
/// // 3 schools with both primary and junior-high: 36 + 30 = 66
/// assert_eq!(score_education(3, true, true), 66.0);
/// // Capped at 100
/// assert_eq!(score_education(10, true, true), 100.0);
/// ```
#[must_use]
pub fn score_education(school_count: i64, has_primary: bool, has_junior_high: bool) -> f64 {
    let diversity_bonus =
        (has_primary as i64 + has_junior_high as i64) as f64 * EDU_DIVERSITY_BONUS;
    (school_count as f64 * EDU_SCORE_PER_SCHOOL + diversity_bonus).min(SCORE_MAX)
}

/// Computes a medical accessibility score from facility counts and bed capacity.
///
/// Formula: `L_med = min(100, hospital×20 + clinic×5 + log10(beds+1)×10)`.
///
/// Constants: [`MED_HOSPITAL_SCORE`] (20), [`MED_CLINIC_SCORE`] (5),
/// [`MED_BED_LOG_MULTIPLIER`] (10).
///
/// The logarithmic bed scaling prevents a single large hospital from
/// completely dominating the score. Output is in `[0.0, 100.0]`.
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_medical;
///
/// // 2 hospitals + 5 clinics + 100 beds
/// // = 40 + 25 + log10(101)×10 ≈ 85.04
/// let s = score_medical(2, 5, 100);
/// assert!((s - 85.04).abs() < 0.1);
/// ```
#[must_use]
pub fn score_medical(hospital_count: i64, clinic_count: i64, total_beds: i64) -> f64 {
    let bed_score = ((total_beds as f64 + 1.0).log10()) * MED_BED_LOG_MULTIPLIER;
    (hospital_count as f64 * MED_HOSPITAL_SCORE
        + clinic_count as f64 * MED_CLINIC_SCORE
        + bed_score)
        .min(SCORE_MAX)
}

// ═══════════════════════════════════════════════════════════════════════════
// S4 Future sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Converts a land price Compound Annual Growth Rate (CAGR) to a 0–100 score.
///
/// Formula: `P_price = clamp(50 + cagr × 500, 0, 100)`.
///
/// A CAGR of `+0.10` (+10%/yr) maps to 100; `−0.10` maps to 0; `0.0` maps
/// to 50. Constants: [`PRICE_TREND_OFFSET`] (50), [`PRICE_TREND_MULTIPLIER`] (500).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_price_trend;
///
/// assert_eq!(score_price_trend(0.0), 50.0);   // flat
/// assert_eq!(score_price_trend(0.05), 75.0);  // +5%/yr
/// assert_eq!(score_price_trend(-0.05), 25.0); // −5%/yr
/// assert_eq!(score_price_trend(1.0), 100.0);  // clamped
/// ```
#[must_use]
pub fn score_price_trend(cagr: f64) -> f64 {
    (PRICE_TREND_OFFSET + cagr * PRICE_TREND_MULTIPLIER).clamp(SCORE_MIN, SCORE_MAX)
}

/// Converts a designated floor area ratio (FAR) to a 0–100 development-upside score.
///
/// Formula: `P_far = min(100, designated_far / 8)`.
/// `designated_far` is expressed in percent (e.g. `800` means 800% FAR).
///
/// `None` means the zoning FAR is unavailable and returns
/// [`UNAVAILABLE_DEFAULT`] (100). Constant: [`FAR_DIVISOR`] (8).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_far;
///
/// assert_eq!(score_far(None), 100.0);
/// assert_eq!(score_far(Some(400.0)), 50.0);  // 400% / 8 = 50
/// assert_eq!(score_far(Some(800.0)), 100.0); // capped
/// ```
#[must_use]
pub fn score_far(designated_far: Option<f64>) -> f64 {
    match designated_far {
        None => UNAVAILABLE_DEFAULT,
        Some(far) => (far / FAR_DIVISOR).min(SCORE_MAX),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// S5 Price sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Converts a z-score (relative to the zoning-type median price) to a 0–100
/// relative-value score.
///
/// Formula: `V_rel = clamp(50 − z_score × 20, 0, 100)`.
/// A negative z-score means the parcel is cheaper than the median — a higher
/// investment value — and therefore produces a score above 50.
///
/// Constants: [`RELATIVE_VALUE_OFFSET`] (50), [`RELATIVE_VALUE_MULTIPLIER`] (20).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_relative_value;
///
/// assert_eq!(score_relative_value(0.0), 50.0);   // at median price
/// assert_eq!(score_relative_value(-1.0), 70.0);  // cheaper → higher score
/// assert_eq!(score_relative_value(2.0), 10.0);   // expensive → lower score
/// assert_eq!(score_relative_value(-5.0), 100.0); // clamped
/// ```
#[must_use]
pub fn score_relative_value(z_score: f64) -> f64 {
    (RELATIVE_VALUE_OFFSET - z_score * RELATIVE_VALUE_MULTIPLIER).clamp(SCORE_MIN, SCORE_MAX)
}

/// Converts a transaction count to a 0–100 market-liquidity score.
///
/// Formula: `V_vol = min(100, tx_count × 5)`.
/// Twenty or more transactions in the viewport reach the maximum, indicating
/// an actively traded, liquid market. Constant: [`VOLUME_MULTIPLIER`] (5).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::sub_scores::score_volume;
///
/// assert_eq!(score_volume(0), 0.0);
/// assert_eq!(score_volume(10), 50.0);
/// assert_eq!(score_volume(20), 100.0);
/// assert_eq!(score_volume(30), 100.0); // capped
/// ```
#[must_use]
pub fn score_volume(tx_count: i64) -> f64 {
    (tx_count as f64 * VOLUME_MULTIPLIER).min(SCORE_MAX)
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── S1 Flood ──

    #[test]
    fn flood_no_zone() {
        assert_eq!(score_flood(None), 100.0);
    }

    #[test]
    fn flood_depth_ranks() {
        assert_eq!(score_flood(Some(1)), 80.0);
        assert_eq!(score_flood(Some(2)), 50.0);
        assert_eq!(score_flood(Some(3)), 20.0);
        assert_eq!(score_flood(Some(4)), 5.0);
        assert_eq!(score_flood(Some(5)), 0.0);
    }

    #[test]
    fn flood_unknown_rank_returns_default() {
        assert_eq!(score_flood(Some(99)), 100.0);
    }

    #[test]
    fn flood_rank_zero_is_safe() {
        // depth_rank = 0 in the DB means "outside flood zone" (区域外).
        // The constraint `CHECK (depth_rank >= 0 AND depth_rank <= 5)` allows
        // it, so we must handle it explicitly rather than crashing.
        assert_eq!(score_flood(Some(0)), 100.0);
    }

    // ── S1 Liquefaction ──

    #[test]
    fn liquefaction_unavailable() {
        assert_eq!(score_liquefaction(None), 100.0);
    }

    #[test]
    fn liquefaction_thresholds() {
        assert_eq!(score_liquefaction(Some(0.0)), 100.0);
        assert_eq!(score_liquefaction(Some(3.0)), 80.0);
        assert_eq!(score_liquefaction(Some(5.0)), 80.0);
        assert_eq!(score_liquefaction(Some(10.0)), 40.0);
        assert_eq!(score_liquefaction(Some(15.0)), 40.0);
        assert_eq!(score_liquefaction(Some(20.0)), 10.0);
    }

    // ── S1 Seismic ──

    #[test]
    fn seismic_thresholds() {
        assert_eq!(score_seismic(0.0), 100.0);
        assert_eq!(score_seismic(0.02), 100.0);
        assert_eq!(score_seismic(0.03), 75.0);
        assert_eq!(score_seismic(0.05), 75.0);
        assert_eq!(score_seismic(0.06), 50.0);
        assert_eq!(score_seismic(0.20), 50.0);
        assert_eq!(score_seismic(0.26), 25.0);
        assert_eq!(score_seismic(0.50), 5.0);
        assert_eq!(score_seismic(0.80), 5.0);
    }

    // ── S1 Tsunami ──

    #[test]
    fn tsunami_unavailable() {
        assert_eq!(score_tsunami(None), 100.0);
    }

    #[test]
    fn tsunami_thresholds() {
        assert_eq!(score_tsunami(Some(0.0)), 100.0);
        assert_eq!(score_tsunami(Some(0.2)), 85.0);
        assert_eq!(score_tsunami(Some(0.3)), 85.0);
        assert_eq!(score_tsunami(Some(0.5)), 60.0);
        assert_eq!(score_tsunami(Some(1.0)), 60.0);
        assert_eq!(score_tsunami(Some(1.5)), 35.0);
        assert_eq!(score_tsunami(Some(2.0)), 35.0);
        assert_eq!(score_tsunami(Some(3.0)), 10.0);
    }

    // ── S1 Landslide ──

    #[test]
    fn landslide_variants() {
        assert_eq!(score_landslide(None), 100.0);
        assert_eq!(score_landslide(Some(false)), 100.0);
        assert_eq!(score_landslide(Some(true)), 40.0);
    }

    // ── S2 AVS30 ──

    #[test]
    fn avs30_unavailable() {
        assert_eq!(score_avs30(None), 100.0);
    }

    #[test]
    fn avs30_thresholds() {
        assert_eq!(score_avs30(Some(500.0)), 100.0);
        assert_eq!(score_avs30(Some(400.0)), 100.0);
        assert_eq!(score_avs30(Some(350.0)), 85.0);
        assert_eq!(score_avs30(Some(300.0)), 85.0);
        assert_eq!(score_avs30(Some(250.0)), 60.0);
        assert_eq!(score_avs30(Some(200.0)), 60.0);
        assert_eq!(score_avs30(Some(175.0)), 35.0);
        assert_eq!(score_avs30(Some(150.0)), 35.0);
        assert_eq!(score_avs30(Some(100.0)), 10.0);
    }

    // ── S3 Education ──

    #[test]
    fn education_none() {
        assert_eq!(score_education(0, false, false), 0.0);
    }

    #[test]
    fn education_with_diversity() {
        // 3 schools + primary + junior high = 36 + 30 = 66
        assert_eq!(score_education(3, true, true), 66.0);
    }

    #[test]
    fn education_capped_at_100() {
        assert_eq!(score_education(10, true, true), 100.0);
    }

    // ── S3 Medical ──

    #[test]
    fn medical_none() {
        let s = score_medical(0, 0, 0);
        // log10(0+1) × 10 = 0
        assert!(s.abs() < f64::EPSILON);
    }

    #[test]
    fn medical_mixed() {
        // 2 hospitals + 5 clinics + 100 beds = 40 + 25 + log10(101)*10 ≈ 40+25+20.04 = 85.04
        let s = score_medical(2, 5, 100);
        assert!((s - 85.04).abs() < 0.1, "expected ~85.04, got {s}");
    }

    #[test]
    fn medical_capped_at_100() {
        let s = score_medical(5, 10, 1000);
        assert_eq!(s, 100.0);
    }

    // ── S4 Price Trend ──

    #[test]
    fn price_trend_positive() {
        // cagr = 0.05 → 50 + 25 = 75
        assert_eq!(score_price_trend(0.05), 75.0);
    }

    #[test]
    fn price_trend_negative() {
        // cagr = -0.05 → 50 - 25 = 25
        assert_eq!(score_price_trend(-0.05), 25.0);
    }

    #[test]
    fn price_trend_clamped() {
        assert_eq!(score_price_trend(1.0), 100.0);
        assert_eq!(score_price_trend(-1.0), 0.0);
    }

    // ── S4 FAR ──

    #[test]
    fn far_unavailable() {
        assert_eq!(score_far(None), 100.0);
    }

    #[test]
    fn far_values() {
        assert_eq!(score_far(Some(400.0)), 50.0); // 400% / 8 = 50
        assert_eq!(score_far(Some(800.0)), 100.0); // capped
        assert_eq!(score_far(Some(1000.0)), 100.0);
    }

    // ── S5 Relative Value ──

    #[test]
    fn relative_value_cheap() {
        // z = -1.0 → 50 - (-1.0)*20 = 70 (below median = cheap = high score)
        assert_eq!(score_relative_value(-1.0), 70.0);
    }

    #[test]
    fn relative_value_expensive() {
        // z = 2.0 → 50 - 40 = 10 (above median = expensive = low score)
        assert_eq!(score_relative_value(2.0), 10.0);
    }

    #[test]
    fn relative_value_clamped() {
        assert_eq!(score_relative_value(-5.0), 100.0);
        assert_eq!(score_relative_value(5.0), 0.0);
    }

    // ── S5 Volume ──

    #[test]
    fn volume_values() {
        assert_eq!(score_volume(0), 0.0);
        assert_eq!(score_volume(10), 50.0);
        assert_eq!(score_volume(20), 100.0);
        assert_eq!(score_volume(30), 100.0); // capped
    }
}
