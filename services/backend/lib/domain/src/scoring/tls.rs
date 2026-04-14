//! TLS (Total Location Score) aggregation, grading, and cross-analysis.
//!
//! This module is the final step in the scoring pipeline. After axis scores
//! (S1–S5) are computed by [`super::axis`], this module blends them into a
//! single 0–100 total using a [`WeightPreset`], converts it to a [`Grade`],
//! and optionally computes [`CrossAnalysis`] patterns for the investment
//! insight panel.

use serde::Serialize;

use crate::scoring::constants::*;

// ═══════════════════════════════════════════════════════════════════════════
// AxisScores
// ═══════════════════════════════════════════════════════════════════════════

/// Normalized axis scores (0–100 scale) for the 5-axis TLS formula.
///
/// Each field represents one of the five TLS axes. Constructed by the
/// usecase layer after composing sub-scores via [`super::axis`] functions.
#[derive(Debug, Clone, Copy)]
pub struct AxisScores {
    /// S1 — Disaster resilience score.
    pub s1_disaster: f64,
    /// S2 — Terrain quality score.
    pub s2_terrain: f64,
    /// S3 — Livability (transit, education, medical) score.
    pub s3_livability: f64,
    /// S4 — Future potential score.
    pub s4_future: f64,
    /// S5 — Profitability (price competitiveness) score.
    pub s5_profitability: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Grade
// ═══════════════════════════════════════════════════════════════════════════

/// Letter grade derived from a TLS score.
///
/// Grades provide a human-readable quality tier displayed in the frontend
/// sidebar and map popups. Thresholds are defined in
/// [`crate::scoring::constants`] and can be tuned without touching this enum.
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::tls::Grade;
///
/// assert_eq!(Grade::from_score(90.0), Grade::S);
/// assert_eq!(Grade::from_score(50.0), Grade::C);
/// assert_eq!(Grade::from_score(10.0), Grade::E);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Grade {
    /// Excellent — TLS ≥ 85. Exceptional investment quality on all axes.
    S,
    /// Very Good — TLS ≥ 70. Strong location with minor weaknesses.
    A,
    /// Good — TLS ≥ 55. Above-average location; acceptable for most buyers.
    B,
    /// Fair — TLS ≥ 40. Notable trade-offs; requires careful due diligence.
    C,
    /// Below Average — TLS ≥ 25. Significant risks or deficiencies present.
    D,
    /// Poor — TLS < 25. Multiple serious concerns across several axes.
    E,
}

impl Grade {
    /// Derives a [`Grade`] from a raw TLS score in the range `[0.0, 100.0]`.
    ///
    /// Thresholds (in descending order): S ≥ 85, A ≥ 70, B ≥ 55, C ≥ 40, D ≥ 25, E < 25.
    ///
    /// # Examples
    ///
    /// ```
    /// use terrasight_domain::scoring::tls::Grade;
    ///
    /// assert_eq!(Grade::from_score(85.0), Grade::S);
    /// assert_eq!(Grade::from_score(84.9), Grade::A);
    /// ```
    #[must_use]
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

    /// Returns the full English label for this grade, used in UI tooltips.
    ///
    /// # Examples
    ///
    /// ```
    /// use terrasight_domain::scoring::tls::Grade;
    ///
    /// assert_eq!(Grade::S.label(), "Excellent");
    /// assert_eq!(Grade::E.label(), "Poor");
    /// ```
    #[must_use]
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

    /// Returns the single-character grade string (e.g. `"S"`, `"A"`, …, `"E"`).
    ///
    /// Useful for compact display in map labels where full labels do not fit.
    ///
    /// # Examples
    ///
    /// ```
    /// use terrasight_domain::scoring::tls::Grade;
    ///
    /// assert_eq!(Grade::B.as_str(), "B");
    /// ```
    #[must_use]
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

/// Predefined axis weight configurations for the TLS formula.
///
/// Each preset emphasises a different investment objective. The frontend
/// exposes these as a toggle in the score panel; the selected preset is passed
/// through the API query string (`?preset=investment`) and parsed via
/// [`std::str::FromStr`].
///
/// All presets guarantee that the five axis weights sum to exactly `1.0`
/// (verified by the `all_presets_sum_to_one` test in [`super::tls`]).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::tls::WeightPreset;
///
/// let preset: WeightPreset = "investment".parse().unwrap();
/// assert_eq!(preset, WeightPreset::Investment);
///
/// // Unknown strings fall back to Balance.
/// let fallback: WeightPreset = "unknown".parse().unwrap();
/// assert_eq!(fallback, WeightPreset::Balance);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightPreset {
    /// Equal emphasis across all axes — the default for general-purpose analysis.
    ///
    /// Weights: disaster 0.25 · terrain 0.15 · livability 0.25 · future 0.15 · price 0.20.
    #[default]
    Balance,

    /// Optimised for return-on-investment analysis.
    ///
    /// Up-weights price (0.30) and future potential (0.25); down-weights
    /// disaster resilience (0.15) and terrain quality (0.10).
    /// Use this preset when evaluating parcels primarily for rental yield or
    /// capital appreciation.
    Investment,

    /// Optimised for long-term owner-occupier decisions.
    ///
    /// Up-weights livability (0.35) for school and transit access; down-weights
    /// price and future potential.
    /// Use this preset when a buyer prioritises everyday quality of life over
    /// investment metrics.
    Residential,

    /// Maximises weight on disaster resilience and terrain quality.
    ///
    /// Disaster (0.40) + terrain (0.25) together account for 65% of the total,
    /// making this the safest-first filter for post-disaster risk screening.
    DisasterFocus,
}

/// Resolved axis weights for a specific [`WeightPreset`].
///
/// Created by [`WeightPreset::weights`] and consumed directly by [`compute_tls`].
pub struct AxisWeights {
    /// Weight applied to the S1 Disaster axis score.
    pub disaster: f64,
    /// Weight applied to the S2 Terrain axis score.
    pub terrain: f64,
    /// Weight applied to the S3 Livability axis score.
    pub livability: f64,
    /// Weight applied to the S4 Future-potential axis score.
    pub future: f64,
    /// Weight applied to the S5 Profitability axis score.
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
    /// Returns the serde `snake_case` serialisation string for this preset.
    ///
    /// Avoids a `serde_json::to_value` round-trip at the handler boundary when
    /// the preset needs to be embedded in a response DTO.
    ///
    /// # Examples
    ///
    /// ```
    /// use terrasight_domain::scoring::tls::WeightPreset;
    ///
    /// assert_eq!(WeightPreset::DisasterFocus.as_str(), "disaster_focus");
    /// ```
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balance => "balance",
            Self::Investment => "investment",
            Self::Residential => "residential",
            Self::DisasterFocus => "disaster_focus",
        }
    }

    /// Returns the resolved [`AxisWeights`] for this preset.
    ///
    /// All constants are sourced from [`crate::scoring::constants`]; no
    /// literals appear in this function.
    #[must_use]
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

/// Computes the Total Location Score (TLS) from five axis scores using the
/// given weight preset.
///
/// Each axis score in `scores` must be in the range `[0.0, 100.0]`. The
/// return value is clamped to `[0.0, 100.0]` regardless of input values.
///
/// Using [`AxisScores`] instead of five positional `f64` parameters prevents
/// transposition bugs at the call site (e.g. swapping S3 and S4).
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::tls::{AxisScores, compute_tls, WeightPreset};
///
/// let scores = AxisScores {
///     s1_disaster: 80.0,
///     s2_terrain: 75.0,
///     s3_livability: 90.0,
///     s4_future: 60.0,
///     s5_profitability: 55.0,
/// };
/// let tls = compute_tls(&scores, WeightPreset::Balance);
/// // 0.25×80 + 0.15×75 + 0.25×90 + 0.15×60 + 0.20×55
/// // = 20 + 11.25 + 22.5 + 9 + 11 = 73.75
/// assert!((tls - 73.75).abs() < 0.01);
/// ```
#[must_use]
pub fn compute_tls(scores: &AxisScores, preset: WeightPreset) -> f64 {
    let w = preset.weights();
    let tls = w.disaster * scores.s1_disaster
        + w.terrain * scores.s2_terrain
        + w.livability * scores.s3_livability
        + w.future * scores.s4_future
        + w.price * scores.s5_profitability;
    tls.clamp(SCORE_MIN, SCORE_MAX)
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-analysis patterns
// ═══════════════════════════════════════════════════════════════════════════

/// Investment insight signals derived by combining axis scores.
///
/// These cross-axis composites highlight patterns that a single axis cannot
/// capture alone. They are displayed in the frontend insight panel as
/// secondary indicators beneath the main TLS score.
#[derive(Debug, Clone)]
pub struct CrossAnalysis {
    /// Safe but underpriced — a market blind spot indicator.
    ///
    /// Computed as `S1 × (100 − V_rel) / 100`.
    /// A high score flags parcels with strong disaster resilience that the
    /// market has not yet priced in.
    pub value_discovery: f64,

    /// Strong livability in a growing area — demand concentration signal.
    ///
    /// Computed as `S3 × S4 / 100`.
    /// High scores indicate areas where quality of life and population /
    /// price growth are simultaneously strong, driving rental demand.
    pub demand_signal: f64,

    /// Comprehensive ground safety combining disaster resilience and terrain quality.
    ///
    /// Computed as `S1 × S2 / 100`.
    /// Used as a conservative filter for buyers who prioritise structural and
    /// geological safety above all other location factors.
    pub ground_safety: f64,
}

/// Computes cross-axis investment insight signals from five axis scores.
///
/// - `scores` — The five TLS axis scores bundled as [`AxisScores`].
/// - `v_rel` — Relative value sub-score from S5 (0–100); higher means cheaper
///   than the zoning-type median. Kept separate because it is a raw sub-score,
///   not an aggregated axis output.
///
/// All output fields are clamped to `[0.0, 100.0]`.
///
/// # Examples
///
/// ```
/// use terrasight_domain::scoring::tls::{AxisScores, compute_cross_analysis};
///
/// let scores = AxisScores {
///     s1_disaster: 80.0,
///     s2_terrain: 60.0,
///     s3_livability: 82.0,
///     s4_future: 58.0,
///     s5_profitability: 70.0,
/// };
/// // S1=80, V_rel=30 → value_discovery = 80 × (100−30)/100 = 56
/// let ca = compute_cross_analysis(&scores, 30.0);
/// assert!((ca.value_discovery - 56.0).abs() < 0.01);
/// ```
#[must_use]
pub fn compute_cross_analysis(scores: &AxisScores, v_rel: f64) -> CrossAnalysis {
    CrossAnalysis {
        value_discovery: (scores.s1_disaster * (SCORE_MAX - v_rel) / SCORE_MAX)
            .clamp(SCORE_MIN, SCORE_MAX),
        demand_signal: (scores.s3_livability * scores.s4_future / SCORE_MAX)
            .clamp(SCORE_MIN, SCORE_MAX),
        ground_safety: (scores.s1_disaster * scores.s2_terrain / SCORE_MAX)
            .clamp(SCORE_MIN, SCORE_MAX),
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

    fn axis_scores(s1: f64, s2: f64, s3: f64, s4: f64, s5: f64) -> AxisScores {
        AxisScores {
            s1_disaster: s1,
            s2_terrain: s2,
            s3_livability: s3,
            s4_future: s4,
            s5_profitability: s5,
        }
    }

    #[test]
    fn tls_balance_all_100() {
        let tls = compute_tls(
            &axis_scores(100.0, 100.0, 100.0, 100.0, 100.0),
            WeightPreset::Balance,
        );
        assert!((tls - 100.0).abs() < 0.01);
    }

    #[test]
    fn tls_balance_all_zero() {
        let tls = compute_tls(&axis_scores(0.0, 0.0, 0.0, 0.0, 0.0), WeightPreset::Balance);
        assert!(tls.abs() < 0.01);
    }

    #[test]
    fn tls_balance_mixed() {
        // 0.25*65 + 0.15*60 + 0.25*82 + 0.15*58 + 0.20*71
        // = 16.25 + 9.0 + 20.5 + 8.7 + 14.2 = 68.65
        let tls = compute_tls(
            &axis_scores(65.0, 60.0, 82.0, 58.0, 71.0),
            WeightPreset::Balance,
        );
        assert!((tls - 68.65).abs() < 0.01, "expected 68.65, got {tls}");
    }

    #[test]
    fn tls_investment_preset() {
        // Investment: 0.15*65 + 0.10*60 + 0.20*82 + 0.25*58 + 0.30*71
        // = 9.75 + 6.0 + 16.4 + 14.5 + 21.3 = 67.95
        let tls = compute_tls(
            &axis_scores(65.0, 60.0, 82.0, 58.0, 71.0),
            WeightPreset::Investment,
        );
        assert!((tls - 67.95).abs() < 0.01, "expected 67.95, got {tls}");
    }

    #[test]
    fn tls_all_presets_produce_different_scores() {
        // Guards against regression where the preset parameter is dropped
        // somewhere in the request pipeline (wire to handler → usecase → compute).
        // The same 5-axis input with 4 different presets must yield 4 distinct
        // totals (each preset weights axes differently).
        let input = axis_scores(65.0, 60.0, 82.0, 58.0, 71.0);
        let balance = compute_tls(&input, WeightPreset::Balance);
        let investment = compute_tls(&input, WeightPreset::Investment);
        let residential = compute_tls(&input, WeightPreset::Residential);
        let disaster = compute_tls(&input, WeightPreset::DisasterFocus);

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
        let ca = compute_cross_analysis(&axis_scores(80.0, 60.0, 82.0, 58.0, 0.0), 30.0);
        assert!((ca.value_discovery - 56.0).abs() < 0.01);
    }

    #[test]
    fn cross_analysis_demand() {
        // S3=82, S4=58 → demand = 82*58/100 = 47.56
        let ca = compute_cross_analysis(&axis_scores(80.0, 60.0, 82.0, 58.0, 0.0), 50.0);
        assert!((ca.demand_signal - 47.56).abs() < 0.01);
    }

    #[test]
    fn cross_analysis_ground() {
        // S1=80, S2=60 → ground = 80*60/100 = 48
        let ca = compute_cross_analysis(&axis_scores(80.0, 60.0, 82.0, 58.0, 0.0), 50.0);
        assert!((ca.ground_safety - 48.0).abs() < 0.01);
    }
}
