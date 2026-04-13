//! TLS (Total Location Score) aggregation, grading, and cross-analysis.

use serde::Serialize;

use crate::scoring::constants::*;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightPreset {
    #[default]
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

impl std::str::FromStr for WeightPreset {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "investment" => Self::Investment,
            "residential" => Self::Residential,
            "disaster" | "disaster_focus" => Self::DisasterFocus,
            _ => Self::Balance,
        })
    }
}

impl WeightPreset {
    /// Return the serde snake_case serialization string for this preset.
    ///
    /// Avoids `serde_json::to_value` round-trips at the handler boundary.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balance => "balance",
            Self::Investment => "investment",
            Self::Residential => "residential",
            Self::DisasterFocus => "disaster_focus",
        }
    }

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
                disaster: INVESTMENT_WEIGHT_DISASTER,
                terrain: INVESTMENT_WEIGHT_TERRAIN,
                livability: INVESTMENT_WEIGHT_LIVABILITY,
                future: INVESTMENT_WEIGHT_FUTURE,
                price: INVESTMENT_WEIGHT_PRICE,
            },
            Self::Residential => AxisWeights {
                disaster: RESIDENTIAL_WEIGHT_DISASTER,
                terrain: RESIDENTIAL_WEIGHT_TERRAIN,
                livability: RESIDENTIAL_WEIGHT_LIVABILITY,
                future: RESIDENTIAL_WEIGHT_FUTURE,
                price: RESIDENTIAL_WEIGHT_PRICE,
            },
            Self::DisasterFocus => AxisWeights {
                disaster: DISASTER_FOCUS_WEIGHT_DISASTER,
                terrain: DISASTER_FOCUS_WEIGHT_TERRAIN,
                livability: DISASTER_FOCUS_WEIGHT_LIVABILITY,
                future: DISASTER_FOCUS_WEIGHT_FUTURE,
                price: DISASTER_FOCUS_WEIGHT_PRICE,
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

    #[test]
    fn tls_all_presets_produce_different_scores() {
        // Guards against regression where the preset parameter is dropped
        // somewhere in the request pipeline (wire to handler → usecase → compute).
        // The same 5-axis input with 4 different presets must yield 4 distinct
        // totals (each preset weights axes differently).
        let inputs = (65.0, 60.0, 82.0, 58.0, 71.0);
        let balance = compute_tls(
            inputs.0,
            inputs.1,
            inputs.2,
            inputs.3,
            inputs.4,
            WeightPreset::Balance,
        );
        let investment = compute_tls(
            inputs.0,
            inputs.1,
            inputs.2,
            inputs.3,
            inputs.4,
            WeightPreset::Investment,
        );
        let residential = compute_tls(
            inputs.0,
            inputs.1,
            inputs.2,
            inputs.3,
            inputs.4,
            WeightPreset::Residential,
        );
        let disaster = compute_tls(
            inputs.0,
            inputs.1,
            inputs.2,
            inputs.3,
            inputs.4,
            WeightPreset::DisasterFocus,
        );

        // Every pair must differ by at least 0.1 (noticeable in UI)
        let scores = [
            ("balance", balance),
            ("investment", investment),
            ("residential", residential),
            ("disaster", disaster),
        ];
        for (i, (name_a, a)) in scores.iter().enumerate() {
            for (name_b, b) in scores.iter().skip(i + 1) {
                assert!(
                    (a - b).abs() > 0.1,
                    "{name_a}={a} and {name_b}={b} should differ by >0.1"
                );
            }
        }
    }

    // ── WeightPreset::FromStr ──

    #[test]
    fn weight_preset_from_str_known_variants() {
        assert_eq!(
            "investment".parse::<WeightPreset>().unwrap(),
            WeightPreset::Investment
        );
        assert_eq!(
            "residential".parse::<WeightPreset>().unwrap(),
            WeightPreset::Residential
        );
        assert_eq!(
            "disaster".parse::<WeightPreset>().unwrap(),
            WeightPreset::DisasterFocus
        );
        assert_eq!(
            "disaster_focus".parse::<WeightPreset>().unwrap(),
            WeightPreset::DisasterFocus
        );
    }

    #[test]
    fn weight_preset_from_str_unknown_falls_back_to_balance() {
        assert_eq!(
            "balance".parse::<WeightPreset>().unwrap(),
            WeightPreset::Balance
        );
        assert_eq!(
            "unknown".parse::<WeightPreset>().unwrap(),
            WeightPreset::Balance
        );
        assert_eq!("".parse::<WeightPreset>().unwrap(), WeightPreset::Balance);
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
