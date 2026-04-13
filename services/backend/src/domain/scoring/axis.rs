//! Axis composition functions (S1–S5).
//!
//! Each function takes sub-scores and returns `(axis_score, confidence)`.
//! Confidence = sum of weights for sub-scores that are actually available.

use super::constants::*;

/// Availability flags for computing confidence.
pub(crate) struct SubAvailability {
    pub(crate) score: f64,
    pub(crate) weight: f64,
    pub(crate) available: bool,
}

/// Compute weighted average and confidence from sub-score availability data.
fn weighted_avg_with_confidence(subs: &[SubAvailability]) -> (f64, f64) {
    let total_weight: f64 = subs.iter().map(|s| s.weight).sum();
    if total_weight == 0.0 {
        return (0.0, 0.0);
    }
    let weighted_sum: f64 = subs.iter().map(|s| s.score * s.weight).sum();
    let confidence: f64 = subs
        .iter()
        .filter(|s| s.available)
        .map(|s| s.weight)
        .sum::<f64>()
        / total_weight;
    (weighted_sum / total_weight, confidence)
}

/// S1 Disaster: min-penalty composition.
///
/// ```text
/// S1 = min(F_flood, F_liq, F_seis, F_tsun, F_land)
///      × (0.30×F_flood + 0.25×F_liq + 0.25×F_seis + 0.10×F_tsun + 0.10×F_land) / 100
/// ```
///
/// `subs` order: flood, liquefaction, seismic, tsunami, landslide.
/// Confidence = sum of available sub-score weights / total weights.
pub(crate) fn compute_s1(subs: &[SubAvailability]) -> (f64, f64) {
    if subs.is_empty() {
        return (0.0, 0.0);
    }
    let min_score = subs.iter().map(|s| s.score).fold(f64::INFINITY, f64::min);
    let weighted_avg: f64 = subs.iter().map(|s| s.weight * s.score).sum();
    let s1 = (min_score * weighted_avg / SCORE_MAX).clamp(SCORE_MIN, SCORE_MAX);

    let total_weight: f64 = subs.iter().map(|s| s.weight).sum();
    let avail_weight: f64 = subs.iter().filter(|s| s.available).map(|s| s.weight).sum();
    let confidence = if total_weight > 0.0 {
        avail_weight / total_weight
    } else {
        0.0
    };

    (s1, confidence)
}

/// S2 Terrain: Phase 1 = AVS30 only.
pub(crate) fn compute_s2(avs: f64, avs_avail: bool) -> (f64, f64) {
    let confidence = if avs_avail { S2_WEIGHT_AVS } else { 0.0 };
    (avs.clamp(SCORE_MIN, SCORE_MAX), confidence)
}

/// S3 Livability: weighted average with Phase 1 fallback.
///
/// When transit is unavailable, uses fallback weights (edu 0.45, med 0.55).
pub(crate) fn compute_s3(
    transit: f64,
    edu: f64,
    med: f64,
    transit_avail: bool,
    edu_avail: bool,
    med_avail: bool,
) -> (f64, f64) {
    if transit_avail {
        let subs = [
            SubAvailability {
                score: transit,
                weight: S3_WEIGHT_TRANSIT,
                available: transit_avail,
            },
            SubAvailability {
                score: edu,
                weight: S3_WEIGHT_EDUCATION,
                available: edu_avail,
            },
            SubAvailability {
                score: med,
                weight: S3_WEIGHT_MEDICAL,
                available: med_avail,
            },
        ];
        weighted_avg_with_confidence(&subs)
    } else {
        // Phase 1 fallback: redistribute transit weight to edu + med
        let score = S3_FALLBACK_WEIGHT_EDUCATION * edu + S3_FALLBACK_WEIGHT_MEDICAL * med;
        let total = S3_WEIGHT_TRANSIT + S3_WEIGHT_EDUCATION + S3_WEIGHT_MEDICAL;
        let avail = (if edu_avail { S3_WEIGHT_EDUCATION } else { 0.0 }
            + if med_avail { S3_WEIGHT_MEDICAL } else { 0.0 })
            / total;
        (score.clamp(SCORE_MIN, SCORE_MAX), avail)
    }
}

/// S4 Future: weighted average.
pub(crate) fn compute_s4(
    pop: f64,
    price: f64,
    far: f64,
    pop_avail: bool,
    price_avail: bool,
    far_avail: bool,
) -> (f64, f64) {
    let subs = [
        SubAvailability {
            score: pop,
            weight: S4_WEIGHT_POPULATION,
            available: pop_avail,
        },
        SubAvailability {
            score: price,
            weight: S4_WEIGHT_PRICE_TREND,
            available: price_avail,
        },
        SubAvailability {
            score: far,
            weight: S4_WEIGHT_FAR,
            available: far_avail,
        },
    ];
    weighted_avg_with_confidence(&subs)
}

/// S5 Price: weighted average.
pub(crate) fn compute_s5(rel: f64, vol: f64, rel_avail: bool, vol_avail: bool) -> (f64, f64) {
    let subs = [
        SubAvailability {
            score: rel,
            weight: S5_WEIGHT_RELATIVE_VALUE,
            available: rel_avail,
        },
        SubAvailability {
            score: vol,
            weight: S5_WEIGHT_VOLUME,
            available: vol_avail,
        },
    ];
    weighted_avg_with_confidence(&subs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── S1 ──

    fn s1_subs(scores: [f64; 5], avails: [bool; 5]) -> Vec<SubAvailability> {
        let weights = [
            S1_WEIGHT_FLOOD,
            S1_WEIGHT_LIQUEFACTION,
            S1_WEIGHT_SEISMIC,
            S1_WEIGHT_TSUNAMI,
            S1_WEIGHT_LANDSLIDE,
        ];
        scores
            .iter()
            .zip(weights.iter())
            .zip(avails.iter())
            .map(|((&s, &w), &a)| SubAvailability {
                score: s,
                weight: w,
                available: a,
            })
            .collect()
    }

    #[test]
    fn s1_all_safe() {
        let (score, conf) = compute_s1(&s1_subs([100.0; 5], [true; 5]));
        assert!((score - 100.0).abs() < 0.01);
        assert!((conf - 1.0).abs() < 0.01);
    }

    #[test]
    fn s1_one_critical_risk() {
        let (score, _) = compute_s1(&s1_subs([0.0, 100.0, 100.0, 100.0, 100.0], [true; 5]));
        assert!(
            score.abs() < 0.01,
            "min penalty should drive score to 0: {score}"
        );
    }

    #[test]
    fn s1_moderate_risks() {
        let (score, _) = compute_s1(&s1_subs([50.0, 80.0, 50.0, 100.0, 100.0], [true; 5]));
        assert!((score - 33.75).abs() < 0.01, "expected 33.75, got {score}");
    }

    #[test]
    fn s1_confidence_with_missing() {
        let (_, conf) = compute_s1(&s1_subs(
            [80.0, 100.0, 50.0, 100.0, 40.0],
            [true, false, true, false, true],
        ));
        assert!((conf - 0.65).abs() < 0.01, "expected 0.65, got {conf}");
    }

    // ── S2 ──

    #[test]
    fn s2_with_avs() {
        let (score, conf) = compute_s2(85.0, true);
        assert_eq!(score, 85.0);
        assert!((conf - 1.0).abs() < 0.01);
    }

    #[test]
    fn s2_unavailable() {
        let (score, conf) = compute_s2(100.0, false);
        assert_eq!(score, 100.0);
        assert!((conf - 0.0).abs() < 0.01);
    }

    // ── S3 ──

    #[test]
    fn s3_with_transit() {
        // transit=90, edu=65, med=88
        // 0.45*90 + 0.25*65 + 0.30*88 = 40.5 + 16.25 + 26.4 = 83.15
        let (score, conf) = compute_s3(90.0, 65.0, 88.0, true, true, true);
        assert!((score - 83.15).abs() < 0.01, "expected 83.15, got {score}");
        assert!((conf - 1.0).abs() < 0.01);
    }

    #[test]
    fn s3_fallback_no_transit() {
        // edu=65, med=88 → 0.45*65 + 0.55*88 = 29.25 + 48.4 = 77.65
        let (score, conf) = compute_s3(100.0, 65.0, 88.0, false, true, true);
        assert!((score - 77.65).abs() < 0.01, "expected 77.65, got {score}");
        // confidence = (0.25 + 0.30) / 1.0 = 0.55
        assert!((conf - 0.55).abs() < 0.01, "expected 0.55, got {conf}");
    }

    // ── S4 ──

    #[test]
    fn s4_all_available() {
        // pop=65, price=75, far=50
        // 0.40*65 + 0.35*75 + 0.25*50 = 26 + 26.25 + 12.5 = 64.75
        let (score, conf) = compute_s4(65.0, 75.0, 50.0, true, true, true);
        assert!((score - 64.75).abs() < 0.01, "expected 64.75, got {score}");
        assert!((conf - 1.0).abs() < 0.01);
    }

    #[test]
    fn s4_population_missing() {
        // pop=100(unavail), price=75, far=50
        // avg = (0.40*100 + 0.35*75 + 0.25*50) / 1.0 = 78.75
        // conf = (0.35 + 0.25) / 1.0 = 0.60
        let (score, conf) = compute_s4(100.0, 75.0, 50.0, false, true, true);
        assert!((score - 78.75).abs() < 0.01, "expected 78.75, got {score}");
        assert!((conf - 0.60).abs() < 0.01, "expected 0.60, got {conf}");
    }

    // ── S5 ──

    #[test]
    fn s5_both_available() {
        // rel=70, vol=50 → 0.65*70 + 0.35*50 = 45.5 + 17.5 = 63
        let (score, conf) = compute_s5(70.0, 50.0, true, true);
        assert!((score - 63.0).abs() < 0.01);
        assert!((conf - 1.0).abs() < 0.01);
    }
}
