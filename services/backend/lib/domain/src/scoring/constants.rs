//! TLS (Total Location Score) constants — weights, thresholds, and mapping tables.
//!
//! All magic numbers for the 5-axis scoring system are centralised here.
//! No scoring function hard-codes a literal; every value references one of the
//! constants below, making the design spec and the implementation stay in sync.
//!
//! See `docs/designs/tls-implementation-design.md` for the full design rationale
//! and the derivation of each threshold.

// ---------------------------------------------------------------------------
// Axis weights — default "Balance" preset (must sum to 1.0)
// ---------------------------------------------------------------------------

/// Weight for the S1 Disaster axis in the [`super::tls::WeightPreset::Balance`] preset.
///
/// Disaster resilience is a primary concern for long-term investment, so it
/// receives an equal share with livability.
pub const AXIS_WEIGHT_DISASTER: f64 = 0.25;

/// Weight for the S2 Terrain axis in the [`super::tls::WeightPreset::Balance`] preset.
///
/// Ground quality is a secondary signal — important but less decisive than
/// disaster resilience or livability for most buyers.
pub const AXIS_WEIGHT_TERRAIN: f64 = 0.15;

/// Weight for the S3 Livability axis in the [`super::tls::WeightPreset::Balance`] preset.
///
/// Transit, education, and medical access heavily influence both rental demand
/// and resale value, earning an equal weight with disaster resilience.
pub const AXIS_WEIGHT_LIVABILITY: f64 = 0.25;

/// Weight for the S4 Future-potential axis in the [`super::tls::WeightPreset::Balance`] preset.
///
/// Population trend, price appreciation, and zoning capacity are weighted
/// equally with terrain as medium-importance forward-looking signals.
pub const AXIS_WEIGHT_FUTURE: f64 = 0.15;

/// Weight for the S5 Profitability axis in the [`super::tls::WeightPreset::Balance`] preset.
///
/// Relative price value and transaction volume provide a market-efficiency
/// signal, intentionally weighted slightly below livability and disaster.
pub const AXIS_WEIGHT_PRICE: f64 = 0.20;

// ---------------------------------------------------------------------------
// S1 Disaster — sub-score weights (must sum to 1.0)
// ---------------------------------------------------------------------------

/// Weight for the flood sub-score within S1 Disaster.
///
/// Flood risk is the most frequent and spatially widespread hazard in Japan,
/// receiving the highest single weight among the five S1 components.
pub const S1_WEIGHT_FLOOD: f64 = 0.30;

/// Weight for the liquefaction sub-score within S1 Disaster.
///
/// Liquefaction potential (PL value from boring surveys) is a leading predictor
/// of structural damage after major earthquakes.
pub const S1_WEIGHT_LIQUEFACTION: f64 = 0.25;

/// Weight for the seismic sub-score within S1 Disaster.
///
/// Based on the National Seismic Hazard Map 30-year exceedance probability
/// (J-SHIS, NIED), covering all of Japan's seismic zones.
pub const S1_WEIGHT_SEISMIC: f64 = 0.25;

/// Weight for the tsunami sub-score within S1 Disaster.
///
/// Lower weight than flood and seismic because tsunami risk is geographically
/// concentrated on coastal areas; inland zones receive [`UNAVAILABLE_DEFAULT`].
pub const S1_WEIGHT_TSUNAMI: f64 = 0.10;

/// Weight for the landslide sub-score within S1 Disaster.
///
/// Steep-slope and landslide hazard zones (土砂災害警戒区域) affect a
/// narrower set of properties than flood or seismic risk.
pub const S1_WEIGHT_LANDSLIDE: f64 = 0.10;

// ---------------------------------------------------------------------------
// S2 Terrain — sub-score weights
// ---------------------------------------------------------------------------

/// Weight for the AVS30 sub-score within S2 Terrain.
///
/// Phase 1 uses AVS30 as the sole terrain signal (weight = 1.0).
/// Future phases will split this as: `0.50×G_avs + 0.30×G_form + 0.20×G_geo`
/// once terrain form and geology data are integrated.
pub const S2_WEIGHT_AVS: f64 = 1.0;

// ---------------------------------------------------------------------------
// S3 Livability — sub-score weights
// ---------------------------------------------------------------------------

/// Weight for transit accessibility within S3 Livability.
///
/// Proximity and frequency of rail/bus stops is the dominant livability
/// factor in the Japanese real-estate market.
pub const S3_WEIGHT_TRANSIT: f64 = 0.45;

/// Weight for education access within S3 Livability.
///
/// School proximity and type diversity (elementary + junior-high) directly
/// affects family-household demand.
pub const S3_WEIGHT_EDUCATION: f64 = 0.25;

/// Weight for medical access within S3 Livability.
pub const S3_WEIGHT_MEDICAL: f64 = 0.30;

/// Fallback education weight used in S3 when transit data is unavailable.
///
/// In Phase 1 the transit layer is not yet integrated, so its 0.45 weight
/// is redistributed proportionally between education (0.45) and medical (0.55).
pub const S3_FALLBACK_WEIGHT_EDUCATION: f64 = 0.45;

/// Fallback medical weight used in S3 when transit data is unavailable.
///
/// See [`S3_FALLBACK_WEIGHT_EDUCATION`] for the full Phase 1 fallback rationale.
pub const S3_FALLBACK_WEIGHT_MEDICAL: f64 = 0.55;

// ---------------------------------------------------------------------------
// S4 Future — sub-score weights
// ---------------------------------------------------------------------------

/// Weight for population trend within S4 Future.
///
/// Population growth is the strongest leading indicator of long-term rental
/// demand and capital appreciation.
pub const S4_WEIGHT_POPULATION: f64 = 0.40;

/// Weight for land price CAGR within S4 Future.
///
/// Compound annual growth rate of official land prices (公示地価 + 基準地価)
/// over the most recent 5-year window.
pub const S4_WEIGHT_PRICE_TREND: f64 = 0.35;

/// Weight for floor area ratio surplus within S4 Future.
///
/// High designated FAR relative to current utilisation signals future
/// redevelopment upside.
pub const S4_WEIGHT_FAR: f64 = 0.25;

// ---------------------------------------------------------------------------
// S5 Price — sub-score weights
// ---------------------------------------------------------------------------

/// Weight for relative price value within S5 Price.
///
/// Relative value compares a parcel's price against the median for the same
/// zoning type within the prefecture; cheaper parcels score higher.
pub const S5_WEIGHT_RELATIVE_VALUE: f64 = 0.65;

/// Weight for transaction volume within S5 Price.
///
/// Higher transaction counts indicate market liquidity and reduce the risk of
/// forced-sale discounts at exit.
pub const S5_WEIGHT_VOLUME: f64 = 0.35;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Flood (depth_rank → score)
// ---------------------------------------------------------------------------

/// Lookup table mapping MLIT flood inundation depth rank to a 0–100 score.
///
/// Each tuple is `(depth_rank, score)`. Rank codes follow the MLIT洪水浸水想定
/// depth classification: 1 = <0.5 m, 2 = 0.5–3 m, 3 = 3–5 m, 4 = 5–10 m,
/// 5 = ≥10 m. An unmatched rank falls back to [`FLOOD_DEFAULT`].
pub const FLOOD_MAP: &[(i32, f64)] = &[
    (1, 80.0), // < 0.5m
    (2, 50.0), // 0.5-3m
    (3, 20.0), // 3-5m
    (4, 5.0),  // 5-10m
    (5, 0.0),  // 10m+
];

/// Score returned when a parcel is outside any designated flood zone.
///
/// A missing or zero `depth_rank` in the DB means "区域外" (outside the zone),
/// which is treated as the safest outcome (100 = no risk).
pub const FLOOD_DEFAULT: f64 = 100.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Liquefaction (PL value → score)
// ---------------------------------------------------------------------------

/// Lookup table mapping liquefaction potential index (PL) to a 0–100 score.
///
/// Each tuple is `(pl_upper_bound, score)`. Evaluated in order; the first
/// entry whose upper bound is ≥ the observed PL value wins.
/// PL > 15 falls back to [`LIQUEFACTION_HIGH`].
pub const LIQUEFACTION_MAP: &[(f64, f64)] = &[
    (0.0, 100.0), // PL = 0
    (5.0, 80.0),  // 0 < PL ≤ 5
    (15.0, 40.0), // 5 < PL ≤ 15
];

/// Score applied when the PL value exceeds 15 (高い liquefaction risk).
pub const LIQUEFACTION_HIGH: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Seismic 30-year probability → score
// ---------------------------------------------------------------------------

/// Lookup table mapping 30-year earthquake exceedance probability to a 0–100 score.
///
/// Each tuple is `(probability_upper_bound, score)`. Evaluated in order;
/// the first entry whose upper bound exceeds the observed probability wins.
/// Probabilities above 0.50 fall back to [`SEISMIC_HIGH`].
pub const SEISMIC_MAP: &[(f64, f64)] = &[
    (0.03, 100.0), // < 3%
    (0.06, 75.0),  // 3-6%
    (0.26, 50.0),  // 6-26%
    (0.50, 25.0),  // 26-50%
];

/// Score applied when the 30-year exceedance probability exceeds 50%.
pub const SEISMIC_HIGH: f64 = 5.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Tsunami (depth_m → score)
// ---------------------------------------------------------------------------

/// Lookup table mapping expected tsunami inundation depth (metres) to a 0–100 score.
///
/// Each tuple is `(depth_upper_bound_m, score)`. Evaluated in order; the first
/// entry whose upper bound is ≥ the observed depth wins.
/// Depths above 2 m fall back to [`TSUNAMI_HIGH`].
pub const TSUNAMI_MAP: &[(f64, f64)] = &[
    (0.0, 100.0), // 0m
    (0.3, 85.0),  // < 0.3m
    (1.0, 60.0),  // 0.3-1m
    (2.0, 35.0),  // 1-2m
];

/// Score applied when expected tsunami depth exceeds 2 m.
pub const TSUNAMI_HIGH: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S1 Landslide (zone class → score)
// ---------------------------------------------------------------------------

/// Score for a parcel with no nearby steep-slope or landslide hazard zone.
pub const LANDSLIDE_NONE: f64 = 100.0;

/// Score for a parcel within or adjacent to a 土砂災害警戒区域 (landslide warning zone).
///
/// Future data integration will distinguish 警戒区域 (warning, this value)
/// from 特別警戒区域 (special warning, planned score ≈ 10).
pub const LANDSLIDE_WARNING: f64 = 40.0;

// ---------------------------------------------------------------------------
// Sub-score mapping: S2 AVS30 (m/s → score)
// ---------------------------------------------------------------------------

/// Lookup table mapping AVS30 shear-wave velocity (m/s) to a 0–100 score.
///
/// Each tuple is `(avs30_lower_bound_m_per_s, score)`. Evaluated in descending
/// order of the lower bound; the first entry whose lower bound is ≤ the
/// observed velocity wins. Values below 150 m/s fall back to [`AVS30_SOFT`].
pub const AVS30_MAP: &[(f64, f64)] = &[
    (400.0, 100.0), // > 400: rock / very firm
    (300.0, 85.0),  // 300-400: gravel / good
    (200.0, 60.0),  // 200-300: moderate
    (150.0, 35.0),  // 150-200: slightly soft
];

/// Score applied when AVS30 is below 150 m/s (very soft ground).
///
/// Soft ground amplifies seismic shaking and increases structural damage risk.
pub const AVS30_SOFT: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S3 Education
// ---------------------------------------------------------------------------

/// Points added to the education score per school within the bounding box.
///
/// Formula: `L_edu = min(100, school_count × EDU_SCORE_PER_SCHOOL + diversity_bonus)`.
pub const EDU_SCORE_PER_SCHOOL: f64 = 12.0;

/// Bonus points added to the education score for each distinct school type present.
///
/// A bounding box that contains both an elementary school and a junior-high
/// school earns `2 × EDU_DIVERSITY_BONUS` extra points, rewarding areas that
/// cover the full compulsory education range without long commutes.
pub const EDU_DIVERSITY_BONUS: f64 = 15.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S3 Medical
// ---------------------------------------------------------------------------

/// Points added to the medical score per hospital (`病院`) within the bounding box.
pub const MED_HOSPITAL_SCORE: f64 = 20.0;

/// Points added to the medical score per clinic (`診療所`) within the bounding box.
pub const MED_CLINIC_SCORE: f64 = 5.0;

/// Multiplier applied to `log10(total_beds + 1)` in the medical score formula.
///
/// Formula: `L_med = min(100, hospital×20 + clinic×5 + log10(beds+1)×10)`.
/// The logarithmic scaling prevents a single large hospital from completely
/// dominating the score.
pub const MED_BED_LOG_MULTIPLIER: f64 = 10.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S4 Price trend (CAGR → score)
// ---------------------------------------------------------------------------

/// Multiplier applied to the land price CAGR when computing the price-trend score.
///
/// Formula: `P_price = clamp(PRICE_TREND_OFFSET + cagr × PRICE_TREND_MULTIPLIER, 0, 100)`.
/// A CAGR of +0.10 (10%) maps to `50 + 50 = 100`; −0.10 maps to `50 − 50 = 0`.
pub const PRICE_TREND_MULTIPLIER: f64 = 500.0;

/// Baseline score for a land price CAGR of exactly 0% (flat appreciation).
pub const PRICE_TREND_OFFSET: f64 = 50.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S4 FAR (Floor Area Ratio → score)
// ---------------------------------------------------------------------------

/// Divisor used to normalise the designated floor area ratio (FAR) to a 0–100 score.
///
/// Formula: `P_far = min(100, designated_far / FAR_DIVISOR)`.
/// `designated_far` is expressed in percent (e.g. 800 for 800%), so a FAR of
/// 800% maps to `800 / 8 = 100`. This reflects that Japanese high-density
/// commercial zones typically cap at 800%.
pub const FAR_DIVISOR: f64 = 8.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S5 Relative value (z-score → score)
// ---------------------------------------------------------------------------

/// Baseline score for a parcel priced exactly at the zoning-type median (z-score = 0).
pub const RELATIVE_VALUE_OFFSET: f64 = 50.0;

/// Scaling factor applied to the z-score in the relative-value formula.
///
/// Formula: `V_rel = clamp(RELATIVE_VALUE_OFFSET − z_score × RELATIVE_VALUE_MULTIPLIER, 0, 100)`.
/// A z-score of −1 (one standard deviation cheaper than median) yields 70;
/// a z-score of +1 (one SD more expensive) yields 30.
pub const RELATIVE_VALUE_MULTIPLIER: f64 = 20.0;

// ---------------------------------------------------------------------------
// Sub-score parameters: S5 Transaction volume
// ---------------------------------------------------------------------------

/// Multiplier applied to transaction count in the volume score formula.
///
/// Formula: `V_vol = min(100, tx_count × VOLUME_MULTIPLIER)`.
/// Twenty or more transactions in the viewport reaches the maximum score of 100,
/// indicating a liquid, actively traded market.
pub const VOLUME_MULTIPLIER: f64 = 5.0;

// ---------------------------------------------------------------------------
// Grading thresholds
// ---------------------------------------------------------------------------

/// Minimum TLS score to receive a [`super::tls::Grade::S`] rating (≥ 85).
pub const GRADE_S_MIN: f64 = 85.0;

/// Minimum TLS score to receive a [`super::tls::Grade::A`] rating (≥ 70).
pub const GRADE_A_MIN: f64 = 70.0;

/// Minimum TLS score to receive a [`super::tls::Grade::B`] rating (≥ 55).
pub const GRADE_B_MIN: f64 = 55.0;

/// Minimum TLS score to receive a [`super::tls::Grade::C`] rating (≥ 40).
pub const GRADE_C_MIN: f64 = 40.0;

/// Minimum TLS score to receive a [`super::tls::Grade::D`] rating (≥ 25).
///
/// Scores below this threshold receive [`super::tls::Grade::E`] (Poor).
pub const GRADE_D_MIN: f64 = 25.0;

// ---------------------------------------------------------------------------
// Score bounds
// ---------------------------------------------------------------------------

/// Minimum valid score value across all axes and sub-scores.
pub const SCORE_MIN: f64 = 0.0;

/// Maximum valid score value across all axes and sub-scores.
pub const SCORE_MAX: f64 = 100.0;

/// Default score assigned to a sub-score when source data is unavailable.
///
/// Returning 100 (best case) for missing data avoids penalising areas where
/// a particular hazard simply does not exist or has not been surveyed, while
/// the accompanying confidence value signals to consumers that the score is
/// unconfirmed.
pub const UNAVAILABLE_DEFAULT: f64 = 100.0;

// ---------------------------------------------------------------------------
// Risk level thresholds (derived from S1 Disaster sub-score)
// ---------------------------------------------------------------------------

/// S1 Disaster score threshold above which a parcel is classified as low risk.
///
/// Used by the opportunity pipeline to set `risk_level = 'Low'` in the
/// `opportunities` table.
pub const DISASTER_SCORE_LOW_THRESHOLD: f64 = 75.0;

/// S1 Disaster score threshold above which a parcel is classified as medium risk.
///
/// Scores below this value are classified as high risk. Used together with
/// [`DISASTER_SCORE_LOW_THRESHOLD`] to produce the three-tier risk label.
pub const DISASTER_SCORE_MID_THRESHOLD: f64 = 50.0;

// ---------------------------------------------------------------------------
// Opportunity signal thresholds (derived from TLS score + risk level)
// ---------------------------------------------------------------------------

/// Minimum TLS score (0–100 integer) required for a `Hot` opportunity signal.
pub const SIGNAL_HOT_MIN_TLS: u8 = 80;

/// Minimum TLS score required for a `Warm` opportunity signal.
pub const SIGNAL_WARM_MIN_TLS: u8 = 65;

/// Minimum TLS score required for a `Neutral` opportunity signal.
///
/// Parcels below this threshold receive a `Cold` signal regardless of risk level.
pub const SIGNAL_NEUTRAL_MIN_TLS: u8 = 50;

// ---------------------------------------------------------------------------
// Weight presets
// ---------------------------------------------------------------------------

// ── Investment preset ────────────────────────────────────────────────────────

/// S1 Disaster weight for the [`super::tls::WeightPreset::Investment`] preset.
///
/// The investment preset down-weights disaster resilience relative to Balance,
/// prioritising price appreciation and future potential instead.
pub const INVESTMENT_WEIGHT_DISASTER: f64 = 0.15;

/// S2 Terrain weight for the [`super::tls::WeightPreset::Investment`] preset.
pub const INVESTMENT_WEIGHT_TERRAIN: f64 = 0.10;

/// S3 Livability weight for the [`super::tls::WeightPreset::Investment`] preset.
pub const INVESTMENT_WEIGHT_LIVABILITY: f64 = 0.20;

/// S4 Future weight for the [`super::tls::WeightPreset::Investment`] preset.
///
/// Future potential receives the second-highest weight in this preset,
/// reflecting growth-oriented investment strategy.
pub const INVESTMENT_WEIGHT_FUTURE: f64 = 0.25;

/// S5 Price weight for the [`super::tls::WeightPreset::Investment`] preset.
///
/// Relative value and transaction volume are the top priority for
/// investment-focused analysis.
pub const INVESTMENT_WEIGHT_PRICE: f64 = 0.30;

// ── Residential preset ───────────────────────────────────────────────────────

/// S1 Disaster weight for the [`super::tls::WeightPreset::Residential`] preset.
pub const RESIDENTIAL_WEIGHT_DISASTER: f64 = 0.25;

/// S2 Terrain weight for the [`super::tls::WeightPreset::Residential`] preset.
pub const RESIDENTIAL_WEIGHT_TERRAIN: f64 = 0.15;

/// S3 Livability weight for the [`super::tls::WeightPreset::Residential`] preset.
///
/// Livability receives the highest single weight in this preset — families
/// choosing a long-term home prioritise schools, transit, and medical access
/// above investment metrics.
pub const RESIDENTIAL_WEIGHT_LIVABILITY: f64 = 0.35;

/// S4 Future weight for the [`super::tls::WeightPreset::Residential`] preset.
pub const RESIDENTIAL_WEIGHT_FUTURE: f64 = 0.10;

/// S5 Price weight for the [`super::tls::WeightPreset::Residential`] preset.
pub const RESIDENTIAL_WEIGHT_PRICE: f64 = 0.15;

// ── DisasterFocus preset ─────────────────────────────────────────────────────

/// S1 Disaster weight for the [`super::tls::WeightPreset::DisasterFocus`] preset.
///
/// Disaster resilience dominates this preset — intended for users who
/// explicitly prioritise safety above all other location factors.
pub const DISASTER_FOCUS_WEIGHT_DISASTER: f64 = 0.40;

/// S2 Terrain weight for the [`super::tls::WeightPreset::DisasterFocus`] preset.
///
/// Ground quality is elevated to the second-highest weight because soil
/// conditions directly amplify earthquake and liquefaction damage.
pub const DISASTER_FOCUS_WEIGHT_TERRAIN: f64 = 0.25;

/// S3 Livability weight for the [`super::tls::WeightPreset::DisasterFocus`] preset.
pub const DISASTER_FOCUS_WEIGHT_LIVABILITY: f64 = 0.20;

/// S4 Future weight for the [`super::tls::WeightPreset::DisasterFocus`] preset.
pub const DISASTER_FOCUS_WEIGHT_FUTURE: f64 = 0.05;

/// S5 Price weight for the [`super::tls::WeightPreset::DisasterFocus`] preset.
pub const DISASTER_FOCUS_WEIGHT_PRICE: f64 = 0.10;
