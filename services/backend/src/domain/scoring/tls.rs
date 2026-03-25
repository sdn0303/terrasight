//! TLS (Total Location Score) aggregation, grading, and cross-analysis.

use serde::Serialize;

use super::constants::*;

// ═══════════════════════════════════════════════════════════════════════════
// Grade
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    E,
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        if score >= GRADE_S_MIN {
            Self::S
        } else if score >= GRADE_A_MIN {
            Self::A
        } else if score >= GRADE_B_MIN {
            Self::B
        } else if score >= GRADE_C_MIN {
            Self::C
        } else if score >= GRADE_D_MIN {
            Self::D
        } else {
            Self::E
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::S => "Excellent",
            Self::A => "Very Good",
            Self::B => "Good",
            Self::C => "Fair",
            Self::D => "Below Average",
            Self::E => "Poor",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::S => "S",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Weight Presets
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightPreset {
    Balance,
    Investment,
    Residential,
    DisasterFocus,
}

/// Axis weights: (disaster, terrain, livability, future, price)
pub struct AxisWeights {
    pub disaster: f64,
    pub terrain: f64,
    pub livability: f64,
    pub future: f64,
    pub price: f64,
}

impl WeightPreset {
    pub fn weights(self) -> AxisWeights {
        match self {
            Self::Balance => AxisWeights {
                disaster: AXIS_WEIGHT_DISASTER,
                terrain: AXIS_WEIGHT_TERRAIN,
                livability: AXIS_WEIGHT_LIVABILITY,
                future: AXIS_WEIGHT_FUTURE,
                price: AXIS_WEIGHT_PRICE,
            },
            Self::Investment => AxisWeights {
                disaster: 0.15,
                terrain: 0.10,
                livability: 0.20,
                future: 0.25,
                price: 0.30,
            },
            Self::Residential => AxisWeights {
                disaster: 0.25,
                terrain: 0.15,
                livability: 0.35,
                future: 0.10,
                price: 0.15,
            },
            Self::DisasterFocus => AxisWeights {
                disaster: 0.40,
                terrain: 0.25,
                livability: 0.20,
                future: 0.05,
                price: 0.10,
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TLS computation
// ═══════════════════════════════════════════════════════════════════════════

/// Compute TLS from 5 axis scores using the given weight preset.
pub fn compute_tls(s1: f64, s2: f64, s3: f64, s4: f64, s5: f64, preset: WeightPreset) -> f64 {
    let w = preset.weights();
    let tls = w.disaster * s1 + w.terrain * s2 + w.livability * s3 + w.future * s4 + w.price * s5;
    tls.clamp(SCORE_MIN, SCORE_MAX)
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-analysis patterns
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CrossAnalysis {
    /// Safe but cheap = market blind spot. `S1 × (100 - V_rel) / 100`
    pub value_discovery: f64,
    /// High livability × growing area = strong demand. `S3 × S4 / 100`
    pub demand_signal: f64,
    /// Comprehensive ground safety. `S1 × S2 / 100`
    pub ground_safety: f64,
}

pub fn compute_cross_analysis(s1: f64, s2: f64, s3: f64, s4: f64, v_rel: f64) -> CrossAnalysis {
    CrossAnalysis {
        value_discovery: (s1 * (SCORE_MAX - v_rel) / SCORE_MAX).clamp(SCORE_MIN, SCORE_MAX),
        demand_signal: (s3 * s4 / SCORE_MAX).clamp(SCORE_MIN, SCORE_MAX),
        ground_safety: (s1 * s2 / SCORE_MAX).clamp(SCORE_MIN, SCORE_MAX),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Grade ──

    #[test]
    fn grade_boundaries() {
        assert_eq!(Grade::from_score(100.0), Grade::S);
        assert_eq!(Grade::from_score(85.0), Grade::S);
        assert_eq!(Grade::from_score(84.9), Grade::A);
        assert_eq!(Grade::from_score(70.0), Grade::A);
        assert_eq!(Grade::from_score(69.9), Grade::B);
        assert_eq!(Grade::from_score(55.0), Grade::B);
        assert_eq!(Grade::from_score(54.9), Grade::C);
        assert_eq!(Grade::from_score(40.0), Grade::C);
        assert_eq!(Grade::from_score(39.9), Grade::D);
        assert_eq!(Grade::from_score(25.0), Grade::D);
        assert_eq!(Grade::from_score(24.9), Grade::E);
        assert_eq!(Grade::from_score(0.0), Grade::E);
    }

    #[test]
    fn grade_labels() {
        assert_eq!(Grade::S.label(), "Excellent");
        assert_eq!(Grade::A.label(), "Very Good");
        assert_eq!(Grade::E.label(), "Poor");
    }

    // ── TLS ──

    #[test]
    fn tls_balance_all_100() {
        let tls = compute_tls(100.0, 100.0, 100.0, 100.0, 100.0, WeightPreset::Balance);
        assert!((tls - 100.0).abs() < 0.01);
    }

    #[test]
    fn tls_balance_all_zero() {
        let tls = compute_tls(0.0, 0.0, 0.0, 0.0, 0.0, WeightPreset::Balance);
        assert!(tls.abs() < 0.01);
    }

    #[test]
    fn tls_balance_mixed() {
        // 0.25*65 + 0.15*60 + 0.25*82 + 0.15*58 + 0.20*71
        // = 16.25 + 9.0 + 20.5 + 8.7 + 14.2 = 68.65
        let tls = compute_tls(65.0, 60.0, 82.0, 58.0, 71.0, WeightPreset::Balance);
        assert!((tls - 68.65).abs() < 0.01, "expected 68.65, got {tls}");
    }

    #[test]
    fn tls_investment_preset() {
        // Investment: 0.15*65 + 0.10*60 + 0.20*82 + 0.25*58 + 0.30*71
        // = 9.75 + 6.0 + 16.4 + 14.5 + 21.3 = 67.95
        let tls = compute_tls(65.0, 60.0, 82.0, 58.0, 71.0, WeightPreset::Investment);
        assert!((tls - 67.95).abs() < 0.01, "expected 67.95, got {tls}");
    }

    // ── Weight presets sum to 1.0 ──

    #[test]
    fn all_presets_sum_to_one() {
        for preset in [
            WeightPreset::Balance,
            WeightPreset::Investment,
            WeightPreset::Residential,
            WeightPreset::DisasterFocus,
        ] {
            let w = preset.weights();
            let sum = w.disaster + w.terrain + w.livability + w.future + w.price;
            assert!(
                (sum - 1.0).abs() < 0.001,
                "{preset:?} weights sum to {sum}, expected 1.0"
            );
        }
    }

    // ── Cross-analysis ──

    #[test]
    fn cross_analysis_safe_cheap() {
        // S1=80, V_rel=30 → value_discovery = 80 * (100-30)/100 = 56
        let ca = compute_cross_analysis(80.0, 60.0, 82.0, 58.0, 30.0);
        assert!((ca.value_discovery - 56.0).abs() < 0.01);
    }

    #[test]
    fn cross_analysis_demand() {
        // S3=82, S4=58 → demand = 82*58/100 = 47.56
        let ca = compute_cross_analysis(80.0, 60.0, 82.0, 58.0, 50.0);
        assert!((ca.demand_signal - 47.56).abs() < 0.01);
    }

    #[test]
    fn cross_analysis_ground() {
        // S1=80, S2=60 → ground = 80*60/100 = 48
        let ca = compute_cross_analysis(80.0, 60.0, 82.0, 58.0, 50.0);
        assert!((ca.ground_safety - 48.0).abs() < 0.01);
    }
}
