use std::sync::Arc;

use mlit_client::jshis::JshisClient;
use realestate_geo_math::finance::compute_cagr;
use serde_json::json;

use crate::domain::entity::{MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::error::DomainError;
use crate::domain::repository::TlsRepository;
use crate::domain::scoring::axis::{
    SubAvailability, compute_s1, compute_s2, compute_s3, compute_s4, compute_s5,
};
use crate::domain::scoring::constants::{
    S1_WEIGHT_FLOOD, S1_WEIGHT_LANDSLIDE, S1_WEIGHT_LIQUEFACTION, S1_WEIGHT_SEISMIC,
    S1_WEIGHT_TSUNAMI,
};
use crate::domain::scoring::sub_scores::{
    score_avs30, score_education, score_far, score_flood, score_landslide, score_liquefaction,
    score_medical, score_price_trend, score_relative_value, score_seismic, score_tsunami,
    score_volume,
};
use crate::domain::scoring::tls::{
    CrossAnalysis, Grade, WeightPreset, compute_cross_analysis, compute_tls,
};
use crate::domain::value_object::Coord;

pub struct ComputeTlsUsecase {
    repo: Arc<dyn TlsRepository>,
    /// J-SHIS seismic hazard client.
    ///
    /// `None` disables live seismic data lookups; the disaster and terrain
    /// axes fall back to unavailable-defaults for J-SHIS-sourced sub-scores.
    jshis: Option<Arc<JshisClient>>,
}

impl ComputeTlsUsecase {
    /// Create a new TLS usecase.
    ///
    /// # Parameters
    ///
    /// - `repo`: PostGIS-backed TLS repository for spatial queries.
    /// - `jshis`: Optional J-SHIS client. Pass `None` to skip live seismic
    ///   and ground data (useful in tests or when network access is unavailable).
    pub fn new(repo: Arc<dyn TlsRepository>, jshis: Option<Arc<JshisClient>>) -> Self {
        Self { repo, jshis }
    }

    /// Compute a Total Location Score (0–100) with grade and axis breakdown.
    ///
    /// PostGIS queries and J-SHIS API calls execute in parallel.
    /// J-SHIS failures degrade gracefully — the affected sub-scores fall back
    /// to the unavailable-default (100) without failing the overall request.
    pub async fn execute(&self, coord: &Coord) -> Result<TlsOutput, DomainError> {
        let (db_result, jshis_data) =
            tokio::join!(self.fetch_db_inputs(coord), self.fetch_jshis_inputs(coord));

        let (prices, flood_depth_rank, steep_nearby, schools, medical, far, z_score, tx_count) =
            db_result?;

        let (seismic_prob_6plus, avs30, pl_value, tsunami_depth) = jshis_data;

        tracing::debug!(
            price_records = prices.len(),
            flood_depth_rank,
            steep_nearby,
            schools_count = schools.count_800m,
            hospital_count = medical.hospital_count,
            clinic_count = medical.clinic_count,
            seismic_prob = seismic_prob_6plus,
            avs30,
            tx_count,
            "tls inputs fetched"
        );

        // ── Sub-scores ──────────────────────────────────────────────────────

        // S1 Disaster
        let f_flood = score_flood(flood_depth_rank);
        let f_liq = score_liquefaction(pl_value);
        let f_seis = score_seismic(seismic_prob_6plus);
        let f_tsun = score_tsunami(tsunami_depth);
        let f_land = score_landslide(Some(steep_nearby));

        let flood_avail = flood_depth_rank.is_some();
        let liq_avail = pl_value.is_some();
        let seis_avail = self.jshis.is_some();
        let tsun_avail = tsunami_depth.is_some();
        let land_avail = true;

        let (s1, s1_conf) = compute_s1(&[
            SubAvailability {
                score: f_flood,
                weight: S1_WEIGHT_FLOOD,
                available: flood_avail,
            },
            SubAvailability {
                score: f_liq,
                weight: S1_WEIGHT_LIQUEFACTION,
                available: liq_avail,
            },
            SubAvailability {
                score: f_seis,
                weight: S1_WEIGHT_SEISMIC,
                available: seis_avail,
            },
            SubAvailability {
                score: f_tsun,
                weight: S1_WEIGHT_TSUNAMI,
                available: tsun_avail,
            },
            SubAvailability {
                score: f_land,
                weight: S1_WEIGHT_LANDSLIDE,
                available: land_avail,
            },
        ]);

        // S2 Terrain
        let g_avs = score_avs30(avs30);
        let avs_avail = avs30.is_some();
        let (s2, s2_conf) = compute_s2(g_avs, avs_avail);

        // S3 Livability
        let l_edu = score_education(
            schools.count_800m,
            schools.has_primary,
            schools.has_junior_high,
        );
        let l_med = score_medical(
            medical.hospital_count,
            medical.clinic_count,
            medical.total_beds,
        );
        // Transit data not yet available in Phase 1 — use unavailable fallback
        let l_transit = 100.0_f64;
        let transit_avail = false;
        let edu_avail = true;
        let med_avail = true;
        let (s3, s3_conf) =
            compute_s3(l_transit, l_edu, l_med, transit_avail, edu_avail, med_avail);

        // S4 Future
        let p_pop = 100.0_f64; // population data not available in Phase 1
        let pop_avail = false;
        let cagr = compute_price_cagr(&prices);
        let p_price = score_price_trend(cagr);
        let price_avail = prices.len() >= 2;
        let p_far = score_far(far);
        let far_avail = far.is_some();
        let (s4, s4_conf) = compute_s4(p_pop, p_price, p_far, pop_avail, price_avail, far_avail);

        // S5 Price
        let v_rel = score_relative_value(z_score.z_score);
        let v_vol = score_volume(tx_count);
        let rel_avail = z_score.sample_count > 0;
        let vol_avail = true;
        let (s5, s5_conf) = compute_s5(v_rel, v_vol, rel_avail, vol_avail);

        // ── Axis aggregation ─────────────────────────────────────────────────

        let preset = WeightPreset::Balance;
        let weights = preset.weights();
        let tls = compute_tls(s1, s2, s3, s4, s5, preset);
        let grade = Grade::from_score(tls);
        let cross = compute_cross_analysis(s1, s2, s3, s4, v_rel);

        let data_freshness = prices
            .last()
            .map(|p| p.year.to_string())
            .unwrap_or_else(|| "N/A".into());

        let output = TlsOutput {
            score: tls,
            grade,
            axes: AxesOutput {
                disaster: AxisOutput {
                    score: s1,
                    weight: weights.disaster,
                    confidence: s1_conf,
                    sub_scores: vec![
                        SubScoreOutput {
                            id: "flood",
                            score: f_flood,
                            available: flood_avail,
                            detail: json!({ "depth_rank": flood_depth_rank }),
                        },
                        SubScoreOutput {
                            id: "liquefaction",
                            score: f_liq,
                            available: liq_avail,
                            detail: json!({ "pl_value": pl_value }),
                        },
                        SubScoreOutput {
                            id: "seismic",
                            score: f_seis,
                            available: seis_avail,
                            detail: json!({ "prob_30yr": seismic_prob_6plus }),
                        },
                        SubScoreOutput {
                            id: "tsunami",
                            score: f_tsun,
                            available: tsun_avail,
                            detail: json!({ "depth_m": tsunami_depth }),
                        },
                        SubScoreOutput {
                            id: "landslide",
                            score: f_land,
                            available: land_avail,
                            detail: json!({ "steep_nearby": steep_nearby }),
                        },
                    ],
                },
                terrain: AxisOutput {
                    score: s2,
                    weight: weights.terrain,
                    confidence: s2_conf,
                    sub_scores: vec![SubScoreOutput {
                        id: "avs30",
                        score: g_avs,
                        available: avs_avail,
                        detail: json!({ "avs30_ms": avs30 }),
                    }],
                },
                livability: AxisOutput {
                    score: s3,
                    weight: weights.livability,
                    confidence: s3_conf,
                    sub_scores: vec![
                        SubScoreOutput {
                            id: "transit",
                            score: l_transit,
                            available: transit_avail,
                            detail: json!({}),
                        },
                        SubScoreOutput {
                            id: "education",
                            score: l_edu,
                            available: edu_avail,
                            detail: json!({
                                "count_800m": schools.count_800m,
                                "has_primary": schools.has_primary,
                                "has_junior_high": schools.has_junior_high,
                            }),
                        },
                        SubScoreOutput {
                            id: "medical",
                            score: l_med,
                            available: med_avail,
                            detail: json!({
                                "hospital_count": medical.hospital_count,
                                "clinic_count": medical.clinic_count,
                                "total_beds": medical.total_beds,
                            }),
                        },
                    ],
                },
                future: AxisOutput {
                    score: s4,
                    weight: weights.future,
                    confidence: s4_conf,
                    sub_scores: vec![
                        SubScoreOutput {
                            id: "population",
                            score: p_pop,
                            available: pop_avail,
                            detail: json!({}),
                        },
                        SubScoreOutput {
                            id: "price_trend",
                            score: p_price,
                            available: price_avail,
                            detail: json!({ "cagr": cagr }),
                        },
                        SubScoreOutput {
                            id: "far",
                            score: p_far,
                            available: far_avail,
                            detail: json!({ "floor_area_ratio": far }),
                        },
                    ],
                },
                price: AxisOutput {
                    score: s5,
                    weight: weights.price,
                    confidence: s5_conf,
                    sub_scores: vec![
                        SubScoreOutput {
                            id: "relative_value",
                            score: v_rel,
                            available: rel_avail,
                            detail: json!({
                                "z_score": z_score.z_score,
                                "zone_type": z_score.zone_type,
                                "sample_count": z_score.sample_count,
                            }),
                        },
                        SubScoreOutput {
                            id: "volume",
                            score: v_vol,
                            available: vol_avail,
                            detail: json!({ "tx_count": tx_count }),
                        },
                    ],
                },
            },
            cross_analysis: cross,
            weight_preset: preset,
            data_freshness,
        };

        tracing::info!(
            tls = output.score,
            grade = output.grade.as_str(),
            "TLS computed"
        );

        Ok(output)
    }

    /// Execute all PostGIS queries in parallel via `tokio::try_join!`.
    #[allow(clippy::type_complexity)]
    async fn fetch_db_inputs(
        &self,
        coord: &Coord,
    ) -> Result<
        (
            Vec<PriceRecord>,
            Option<i32>,
            bool,
            SchoolStats,
            MedicalStats,
            Option<f64>,
            ZScoreResult,
            i64,
        ),
        DomainError,
    > {
        let (prices, flood_rank, steep, schools, medical, far, z_score, tx_count) = tokio::try_join!(
            self.repo.find_nearest_prices(coord),
            self.repo.find_flood_depth_rank(coord),
            self.repo.has_steep_slope_nearby(coord),
            self.repo.find_schools_nearby(coord),
            self.repo.find_medical_nearby(coord),
            self.repo.find_zoning_far(coord),
            self.repo.calc_price_z_score(coord),
            self.repo.count_recent_transactions(coord),
        )?;
        Ok((
            prices, flood_rank, steep, schools, medical, far, z_score, tx_count,
        ))
    }

    /// Fetch seismic hazard, ground quality, liquefaction, and tsunami data from J-SHIS.
    ///
    /// Returns `(seismic_prob_6plus, avs30, pl_value, tsunami_depth_m)`.
    /// All values fall back gracefully on error or when the client is absent.
    async fn fetch_jshis_inputs(
        &self,
        coord: &Coord,
    ) -> (f64, Option<f64>, Option<f64>, Option<f64>) {
        let client = match &self.jshis {
            Some(c) => c,
            None => return (0.0, None, None, None),
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

        let (avs30, pl_value, tsunami_depth) = match ground_result {
            Ok(g) => (g.avs30, None, None),
            Err(e) => {
                tracing::warn!(
                    lng,
                    lat,
                    error = %e,
                    "J-SHIS surface ground call failed; AVS30/liquefaction/tsunami unavailable"
                );
                (None, None, None)
            }
        };

        (seismic_prob_6plus, avs30, pl_value, tsunami_depth)
    }
}

// ─── Output types ────────────────────────────────────────────────────────────

/// Full TLS result returned by [`ComputeTlsUsecase::execute`].
pub struct TlsOutput {
    pub score: f64,
    pub grade: Grade,
    pub axes: AxesOutput,
    pub cross_analysis: CrossAnalysis,
    pub weight_preset: WeightPreset,
    pub data_freshness: String,
}

/// Five-axis breakdown of the TLS.
pub struct AxesOutput {
    pub disaster: AxisOutput,
    pub terrain: AxisOutput,
    pub livability: AxisOutput,
    pub future: AxisOutput,
    pub price: AxisOutput,
}

/// Single axis with its score, weight, confidence, and sub-score detail.
pub struct AxisOutput {
    pub score: f64,
    pub weight: f64,
    pub confidence: f64,
    pub sub_scores: Vec<SubScoreOutput>,
}

/// One sub-score within an axis.
pub struct SubScoreOutput {
    pub id: &'static str,
    pub score: f64,
    pub available: bool,
    pub detail: serde_json::Value,
}

// ─── Pure helpers ─────────────────────────────────────────────────────────────

/// Compute CAGR from a sorted price record slice. Returns 0.0 when fewer than 2 records.
fn compute_price_cagr(prices: &[PriceRecord]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    let oldest = &prices[0];
    let latest = &prices[prices.len() - 1];
    let years = (latest.year - oldest.year).max(1) as u32;
    compute_cagr(
        oldest.price_per_sqm as f64,
        latest.price_per_sqm as f64,
        years,
    )
}
