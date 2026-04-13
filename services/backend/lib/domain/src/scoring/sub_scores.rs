//! Pure sub-score mapping functions.
//!
//! Each function converts a raw data value into a normalised 0–100 score.
//! Functions for unavailable data sources return [`constants::UNAVAILABLE_DEFAULT`].

use crate::scoring::constants::*;

// ═══════════════════════════════════════════════════════════════════════════
// S1 Disaster sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Flood inundation score from `depth_rank` (0-5).
///
/// - `None` or `Some(0)` = outside flood zone → 100 (safest).
/// - `Some(1..=5)` = rank 1 (<0.5m) to rank 5 (≥10m) → mapped via `FLOOD_MAP`.
/// - Any other value falls back to `FLOOD_DEFAULT` (100).
///
/// The DB column `flood_risk.depth_rank` is constrained to `[0, 5]`
/// (see migration `20260326000001_schema_redesign.sql`).
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

/// Liquefaction score from PL value. `None` = data unavailable = 100.
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

/// Seismic hazard score from 30-year exceedance probability (0.0–1.0).
pub fn score_seismic(prob_30yr: f64) -> f64 {
    for &(upper, score) in SEISMIC_MAP {
        if prob_30yr < upper {
            return score;
        }
    }
    SEISMIC_HIGH
}

/// Tsunami inundation score from depth in metres. `None` = data unavailable = 100.
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

/// Landslide / steep-slope score.
///
/// - `None` = no data → 100 (unavailable default)
/// - `Some(true)` = steep slope nearby → `LANDSLIDE_WARNING` (40)
/// - `Some(false)` = no hazard → `LANDSLIDE_NONE` (100)
///
/// Future: distinguish 警戒区域 vs 特別警戒区域 when zone_class data is available.
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

/// AVS30 ground quality score. `None` = data unavailable = 100.
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

/// Education accessibility score.
///
/// L_edu = min(100, school_count × 12 + diversity_bonus)
/// diversity_bonus = (has_primary + has_junior_high) × 15
pub fn score_education(school_count: i64, has_primary: bool, has_junior_high: bool) -> f64 {
    let diversity_bonus =
        (has_primary as i64 + has_junior_high as i64) as f64 * EDU_DIVERSITY_BONUS;
    (school_count as f64 * EDU_SCORE_PER_SCHOOL + diversity_bonus).min(SCORE_MAX)
}

/// Medical accessibility score.
///
/// L_med = min(100, hospital×20 + clinic×5 + log10(beds+1)×10)
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

/// Land price trend score from CAGR.
///
/// P_price = clamp(50 + cagr × 500, 0, 100)
pub fn score_price_trend(cagr: f64) -> f64 {
    (PRICE_TREND_OFFSET + cagr * PRICE_TREND_MULTIPLIER).clamp(SCORE_MIN, SCORE_MAX)
}

/// Floor area ratio surplus score.
///
/// P_far = min(100, designated_far / 8)
/// `designated_far` is in percent (e.g. 800 = 800%).
pub fn score_far(designated_far: Option<f64>) -> f64 {
    match designated_far {
        None => UNAVAILABLE_DEFAULT,
        Some(far) => (far / FAR_DIVISOR).min(SCORE_MAX),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// S5 Price sub-scores
// ═══════════════════════════════════════════════════════════════════════════

/// Relative value score from z-score within same zoning type.
///
/// V_rel = clamp(50 - z_score × 20, 0, 100)
/// Negative z = below median = cheaper = higher score.
pub fn score_relative_value(z_score: f64) -> f64 {
    (RELATIVE_VALUE_OFFSET - z_score * RELATIVE_VALUE_MULTIPLIER).clamp(SCORE_MIN, SCORE_MAX)
}

/// Transaction volume score.
///
/// V_vol = min(100, tx_count × 5)
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
