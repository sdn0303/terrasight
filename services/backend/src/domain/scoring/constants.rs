//! TLS (Total Location Score) constants — weights, thresholds, and mapping tables.
//!
//! All magic numbers for the 5-axis scoring system are centralized here.
//! See `docs/designs/tls-implementation-design.md` for design rationale.

// ---------------------------------------------------------------------------
// Axis weights (default "balance" preset, must sum to 1.0)
// ---------------------------------------------------------------------------

pub const AXIS_WEIGHT_DISASTER: f64 = 0.25;
pub const AXIS_WEIGHT_TERRAIN: f64 = 0.15;
pub const AXIS_WEIGHT_LIVABILITY: f64 = 0.25;
pub const AXIS_WEIGHT_FUTURE: f64 = 0.15;
pub const AXIS_WEIGHT_PRICE: f64 = 0.20;

// ---------------------------------------------------------------------------
// S1 Disaster — sub-score weights (must sum to 1.0)
// ---------------------------------------------------------------------------

pub const S1_WEIGHT_FLOOD: f64 = 0.30;
pub const S1_WEIGHT_LIQUEFACTION: f64 = 0.25;
pub const S1_WEIGHT_SEISMIC: f64 = 0.25;
pub const S1_WEIGHT_TSUNAMI: f64 = 0.10;
pub const S1_WEIGHT_LANDSLIDE: f64 = 0.10;

// ---------------------------------------------------------------------------
// S2 Terrain — sub-score weights
// ---------------------------------------------------------------------------

/// Phase 1: S2 = G_avs only. Future: 0.50×G_avs + 0.30×G_form + 0.20×G_geo
pub const S2_WEIGHT_AVS: f64 = 1.0;

// ---------------------------------------------------------------------------
// S3 Livability — sub-score weights
// ---------------------------------------------------------------------------

pub const S3_WEIGHT_TRANSIT: f64 = 0.45;
pub const S3_WEIGHT_EDUCATION: f64 = 0.25;
pub const S3_WEIGHT_MEDICAL: f64 = 0.30;

/// Phase 1 fallback weights when transit is unavailable.
pub const S3_FALLBACK_WEIGHT_EDUCATION: f64 = 0.45;
pub const S3_FALLBACK_WEIGHT_MEDICAL: f64 = 0.55;

// ---------------------------------------------------------------------------
// S4 Future — sub-score weights
// ---------------------------------------------------------------------------

pub const S4_WEIGHT_POPULATION: f64 = 0.40;
pub const S4_WEIGHT_PRICE_TREND: f64 = 0.35;
pub const S4_WEIGHT_FAR: f64 = 0.25;

// ---------------------------------------------------------------------------
// S5 Price — sub-score weights
// ---------------------------------------------------------------------------

pub const S5_WEIGHT_RELATIVE_VALUE: f64 = 0.65;
pub const S5_WEIGHT_VOLUME: f64 = 0.35;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Flood (depth_rank → score)
// ---------------------------------------------------------------------------

/// `(depth_rank, score)` — no match yields 100.
pub const FLOOD_MAP: &[(i32, f64)] = &[
    (1, 80.0), // < 0.5m
    (2, 50.0), // 0.5-3m
    (3, 20.0), // 3-5m
    (4, 5.0),  // 5-10m
    (5, 0.0),  // 10m+
];
pub const FLOOD_DEFAULT: f64 = 100.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Liquefaction (PL value → score)
// ---------------------------------------------------------------------------

/// `(pl_upper_bound, score)` — evaluated in order, first match wins.
pub const LIQUEFACTION_MAP: &[(f64, f64)] = &[
    (0.0, 100.0), // PL = 0
    (5.0, 80.0),  // 0 < PL ≤ 5
    (15.0, 40.0), // 5 < PL ≤ 15
];
/// PL > 15
pub const LIQUEFACTION_HIGH: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Seismic 30-year probability → score
// ---------------------------------------------------------------------------

/// `(probability_upper_bound, score)` — evaluated in order, first match wins.
pub const SEISMIC_MAP: &[(f64, f64)] = &[
    (0.03, 100.0), // < 3%
    (0.06, 75.0),  // 3-6%
    (0.26, 50.0),  // 6-26%
    (0.50, 25.0),  // 26-50%
];
/// > 50%
pub const SEISMIC_HIGH: f64 = 5.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Tsunami (depth_m → score)
// ---------------------------------------------------------------------------

pub const TSUNAMI_MAP: &[(f64, f64)] = &[
    (0.0, 100.0), // 0m
    (0.3, 85.0),  // < 0.3m
    (1.0, 60.0),  // 0.3-1m
    (2.0, 35.0),  // 1-2m
];
/// > 2m
pub const TSUNAMI_HIGH: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Landslide (zone class → score)
// ---------------------------------------------------------------------------

pub const LANDSLIDE_NONE: f64 = 100.0;
pub const LANDSLIDE_WARNING: f64 = 40.0;
pub const LANDSLIDE_SPECIAL_WARNING: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S2 AVS30 (m/s → score)
// ---------------------------------------------------------------------------

/// `(avs30_lower_bound, score)` — evaluated in descending order.
pub const AVS30_MAP: &[(f64, f64)] = &[
    (400.0, 100.0), // > 400: rock / very firm
    (300.0, 85.0),  // 300-400: gravel / good
    (200.0, 60.0),  // 200-300: moderate
    (150.0, 35.0),  // 150-200: slightly soft
];
/// < 150 m/s
pub const AVS30_SOFT: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S3 Transit
// ---------------------------------------------------------------------------

pub const TRANSIT_DISTANCE_DIVISOR: f64 = 8.0;
pub const TRANSIT_DISTANCE_MAX: f64 = 100.0;
pub const TRANSIT_RIDERSHIP_LOG_MULTIPLIER: f64 = 20.0;
pub const TRANSIT_MULTI_LINE_BONUS: f64 = 10.0;
pub const TRANSIT_WEIGHT_DISTANCE: f64 = 0.5;
pub const TRANSIT_WEIGHT_RIDERSHIP: f64 = 0.3;
pub const TRANSIT_WEIGHT_MULTI_LINE: f64 = 0.2;

// ---------------------------------------------------------------------------
// Sub-score parameters: S3 Education
// ---------------------------------------------------------------------------

pub const EDU_RADIUS_M: f64 = 800.0;
pub const EDU_SCORE_PER_SCHOOL: f64 = 12.0;
pub const EDU_DIVERSITY_BONUS: f64 = 15.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S3 Medical
// ---------------------------------------------------------------------------

pub const MED_HOSPITAL_SCORE: f64 = 20.0;
pub const MED_CLINIC_SCORE: f64 = 5.0;
pub const MED_BED_LOG_MULTIPLIER: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S4 Population change rate → score
// ---------------------------------------------------------------------------

/// `(change_rate_lower_bound, score)` — evaluated in descending order.
pub const POPULATION_MAP: &[(f64, f64)] = &[
    (0.20, 100.0), // +20% or more
    (0.10, 85.0),  // +10% to +20%
    (0.00, 65.0),  // 0% to +10%
    (-0.10, 40.0), // -10% to 0%
    (-0.20, 20.0), // -20% to -10%
];
/// < -20%
pub const POPULATION_DECLINE: f64 = 5.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S4 Price trend (CAGR → score)
// ---------------------------------------------------------------------------

pub const PRICE_TREND_MULTIPLIER: f64 = 500.0;
pub const PRICE_TREND_OFFSET: f64 = 50.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S4 FAR (Floor Area Ratio → score)
// ---------------------------------------------------------------------------

/// P_far = min(100, designated_far / FAR_DIVISOR)
/// Design spec: P_far = min(100, designated_far / 8)
/// designated_far is in percent (e.g. 800 for 800%), so divisor = 8.
pub const FAR_DIVISOR: f64 = 8.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S5 Relative value (z-score → score)
// ---------------------------------------------------------------------------

pub const RELATIVE_VALUE_OFFSET: f64 = 50.0;
pub const RELATIVE_VALUE_MULTIPLIER: f64 = 20.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S5 Transaction volume
// ---------------------------------------------------------------------------

/// V_vol = min(100, tx_count × VOLUME_MULTIPLIER)
pub const VOLUME_MULTIPLIER: f64 = 5.0;

// ---------------------------------------------------------------------------
// Grading thresholds
// ---------------------------------------------------------------------------

pub const GRADE_S_MIN: f64 = 85.0;
pub const GRADE_A_MIN: f64 = 70.0;
pub const GRADE_B_MIN: f64 = 55.0;
pub const GRADE_C_MIN: f64 = 40.0;
pub const GRADE_D_MIN: f64 = 25.0;

// ---------------------------------------------------------------------------
// Score bounds
// ---------------------------------------------------------------------------

pub const SCORE_MIN: f64 = 0.0;
pub const SCORE_MAX: f64 = 100.0;

/// Default score for unavailable sub-scores (no risk / best case).
pub const UNAVAILABLE_DEFAULT: f64 = 100.0;
