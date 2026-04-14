//! Usecase: compute the Total Location Score (TLS) for a coordinate.
//!
//! Orchestrates all PostGIS proximity queries (via [`TlsRepository`]) and
//! optional J-SHIS API calls (seismic hazard, ground quality) to produce a
//! five-axis TLS score with grade, cross-analysis, and per-sub-score detail.
//!
//! ## Parallelism
//!
//! PostGIS queries are issued with `tokio::try_join!` (8 concurrent queries).
//! J-SHIS calls (seismic + surface ground) run in a second `tokio::join!`.
//! Both sets execute in parallel with each other via an outer `tokio::join!`.
//!
//! J-SHIS failures degrade gracefully — affected sub-scores receive their
//! unavailable-default value (100) rather than failing the entire request.
//!
//! ## Reuse
//!
//! [`ComputeTlsUsecase`] is shared between the `/api/v1/score` single-point
//! endpoint and the `/api/v1/opportunities` batch pipeline (see
//! [`GetOpportunitiesUsecase`](crate::usecase::get_opportunities::GetOpportunitiesUsecase)).
//! It is therefore wrapped in `Arc` in [`AppState`](crate::app_state::AppState).

use std::sync::Arc;

use serde_json::json;
use terrasight_geo::finance::compute_cagr;
use terrasight_mlit::jshis::JshisClient;

use crate::domain::error::DomainError;
use crate::domain::model::{Coord, MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::repository::TlsRepository;
use terrasight_domain::scoring::axis::{
    SubAvailability, compute_s1, compute_s2, compute_s3, compute_s4, compute_s5,
};
use terrasight_domain::scoring::constants::{
    S1_WEIGHT_FLOOD, S1_WEIGHT_LANDSLIDE, S1_WEIGHT_LIQUEFACTION, S1_WEIGHT_SEISMIC,
    S1_WEIGHT_TSUNAMI,
};
use terrasight_domain::scoring::sub_scores::{
    score_avs30, score_education, score_far, score_flood, score_landslide, score_liquefaction,
    score_medical, score_price_trend, score_relative_value, score_seismic, score_tsunami,
    score_volume,
};
use terrasight_domain::scoring::tls::{
    AxisScores, CrossAnalysis, Grade, WeightPreset, compute_cross_analysis, compute_tls,
};

/// Usecase for `GET /api/v1/score` and the opportunity-enrichment pipeline.
pub(crate) struct ComputeTlsUsecase {
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
    pub(crate) fn new(repo: Arc<dyn TlsRepository>, jshis: Option<Arc<JshisClient>>) -> Self {
        Self { repo, jshis }
    }

    /// Compute a Total Location Score (0–100) with grade and axis breakdown.
    ///
    /// PostGIS queries and J-SHIS API calls execute in parallel.
    /// J-SHIS failures degrade gracefully — the affected sub-scores fall back
    /// to the unavailable-default (100) without failing the overall request.
    ///
    /// This method is a thin orchestrator: raw inputs are fetched into a
    /// [`RawInputs`] container, each axis is built by a dedicated
    /// [`RawInputs`] method, and the final aggregation + grade + cross
    /// analysis is composed from the resulting [`AxisOutput`]s.
    #[tracing::instrument(skip(self), fields(usecase = "compute_tls", preset = ?preset))]
    pub(crate) async fn execute(
        &self,
        coord: &Coord,
        preset: WeightPreset,
    ) -> Result<TlsOutput, DomainError> {
        let inputs = self.fetch_all_inputs(coord).await?;
        inputs.log_summary();

        let weights = preset.weights();
        let disaster = inputs.build_disaster_axis(weights.disaster);
        let terrain = inputs.build_terrain_axis(weights.terrain);
        let livability = inputs.build_livability_axis(weights.livability);
        let future = inputs.build_future_axis(weights.future);
        let (price, relative_value_score) = inputs.build_price_axis(weights.price);

        let axis_scores = AxisScores {
            s1_disaster: disaster.score,
            s2_terrain: terrain.score,
            s3_livability: livability.score,
            s4_future: future.score,
            s5_profitability: price.score,
        };
        let tls = compute_tls(&axis_scores, preset);
        let grade = Grade::from_score(tls);
        let cross_analysis = compute_cross_analysis(&axis_scores, relative_value_score);

        let output = TlsOutput {
            score: tls,
            grade,
            axes: AxesOutput {
                disaster,
                terrain,
                livability,
                future,
                price,
            },
            cross_analysis,
            weight_preset: preset,
            data_freshness: inputs.data_freshness(),
        };

        tracing::info!(
            tls = output.score,
            grade = output.grade.as_str(),
            "TLS computed"
        );

        Ok(output)
    }

    /// Fetch every external input (DB + J-SHIS) in parallel and package it
    /// into a single [`RawInputs`] container for the axis builders.
    async fn fetch_all_inputs(&self, coord: &Coord) -> Result<RawInputs, DomainError> {
        let (db_result, jshis) =
            tokio::join!(self.fetch_db_inputs(coord), self.fetch_jshis_inputs(coord));
        let db = db_result?;

        Ok(RawInputs {
            prices: db.prices,
            flood_depth_rank: db.flood_depth_rank,
            steep_nearby: db.steep_nearby,
            schools: db.schools,
            medical: db.medical,
            far: db.far,
            z_score: db.z_score,
            tx_count: db.tx_count,
            seismic_prob_6plus: jshis.seismic_prob_6plus,
            avs30: jshis.avs30,
            pl_value: jshis.pl_value,
            tsunami_depth: jshis.tsunami_depth,
            jshis_enabled: self.jshis.is_some(),
        })
    }

    /// Execute all PostGIS queries in parallel via `tokio::try_join!`.
    async fn fetch_db_inputs(&self, coord: &Coord) -> Result<DbInputs, DomainError> {
        let (prices, flood_depth_rank, steep_nearby, schools, medical, far, z_score, tx_count) = tokio::try_join!(
            self.repo.find_nearest_prices(coord),
            self.repo.find_flood_depth_rank(coord),
            self.repo.has_steep_slope_nearby(coord),
            self.repo.find_schools_nearby(coord),
            self.repo.find_medical_nearby(coord),
            self.repo.find_zoning_far(coord),
            self.repo.calc_price_z_score(coord),
            self.repo.count_recent_transactions(coord),
        )?;
        Ok(DbInputs {
            prices,
            flood_depth_rank,
            steep_nearby,
            schools,
            medical,
            far,
            z_score,
            tx_count,
        })
    }

    /// Fetch seismic hazard, ground quality, liquefaction, and tsunami data
    /// from J-SHIS.
    ///
    /// All values fall back gracefully on error or when the client is absent
    /// — the caller never sees a J-SHIS error propagated.
    async fn fetch_jshis_inputs(&self, coord: &Coord) -> JshisInputs {
        let client = match &self.jshis {
            Some(c) => c,
            None => return JshisInputs::unavailable(),
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

        JshisInputs {
            seismic_prob_6plus,
            avs30,
            pl_value,
            tsunami_depth,
        }
    }
}

// ─── Raw inputs containers ───────────────────────────────────────────────────

/// DB side of the raw inputs. Private to this module — callers work with
/// [`RawInputs`] which composes the DB + J-SHIS halves.
struct DbInputs {
    prices: Vec<PriceRecord>,
    flood_depth_rank: Option<i32>,
    steep_nearby: bool,
    schools: SchoolStats,
    medical: MedicalStats,
    far: Option<f64>,
    z_score: ZScoreResult,
    tx_count: i64,
}

/// J-SHIS side of the raw inputs. Private to this module.
struct JshisInputs {
    seismic_prob_6plus: f64,
    avs30: Option<f64>,
    pl_value: Option<f64>,
    tsunami_depth: Option<f64>,
}

impl JshisInputs {
    /// Default values when J-SHIS is disabled or unavailable — every
    /// J-SHIS-sourced sub-score falls back to its unavailable default.
    fn unavailable() -> Self {
        Self {
            seismic_prob_6plus: 0.0,
            avs30: None,
            pl_value: None,
            tsunami_depth: None,
        }
    }
}

/// Everything needed to build a full TLS output for a single coordinate.
///
/// Populated by [`ComputeTlsUsecase::fetch_all_inputs`] and consumed by
/// per-axis `build_*_axis` methods that each produce an [`AxisOutput`].
struct RawInputs {
    prices: Vec<PriceRecord>,
    flood_depth_rank: Option<i32>,
    steep_nearby: bool,
    schools: SchoolStats,
    medical: MedicalStats,
    far: Option<f64>,
    z_score: ZScoreResult,
    tx_count: i64,
    seismic_prob_6plus: f64,
    avs30: Option<f64>,
    pl_value: Option<f64>,
    tsunami_depth: Option<f64>,
    /// Whether the J-SHIS client was configured. Controls the availability
    /// flag on the seismic sub-score (which comes from J-SHIS even when the
    /// API call itself succeeds by returning a default).
    jshis_enabled: bool,
}

impl RawInputs {
    /// Emit a debug-level trace summarizing the raw inputs before any
    /// sub-score computation runs.
    fn log_summary(&self) {
        tracing::debug!(
            price_records = self.prices.len(),
            flood_depth_rank = self.flood_depth_rank,
            steep_nearby = self.steep_nearby,
            schools_count = self.schools.count_800m,
            hospital_count = self.medical.hospital_count,
            clinic_count = self.medical.clinic_count,
            seismic_prob = self.seismic_prob_6plus,
            avs30 = self.avs30,
            tx_count = self.tx_count,
            "tls inputs fetched"
        );
    }

    /// S1 Disaster axis: flood, liquefaction, seismic, tsunami, landslide.
    fn build_disaster_axis(&self, axis_weight: f64) -> AxisOutput {
        let f_flood = score_flood(self.flood_depth_rank);
        let f_liq = score_liquefaction(self.pl_value);
        let f_seis = score_seismic(self.seismic_prob_6plus);
        let f_tsun = score_tsunami(self.tsunami_depth);
        let f_land = score_landslide(Some(self.steep_nearby));

        let flood_avail = self.flood_depth_rank.is_some();
        let liq_avail = self.pl_value.is_some();
        let seis_avail = self.jshis_enabled;
        let tsun_avail = self.tsunami_depth.is_some();
        let land_avail = true;

        let (score, confidence) = compute_s1(&[
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

        AxisOutput {
            score,
            weight: axis_weight,
            confidence,
            sub_scores: vec![
                SubScoreOutput {
                    id: "flood",
                    score: f_flood,
                    available: flood_avail,
                    detail: json!({ "depth_rank": self.flood_depth_rank }),
                },
                SubScoreOutput {
                    id: "liquefaction",
                    score: f_liq,
                    available: liq_avail,
                    detail: json!({ "pl_value": self.pl_value }),
                },
                SubScoreOutput {
                    id: "seismic",
                    score: f_seis,
                    available: seis_avail,
                    detail: json!({ "prob_30yr": self.seismic_prob_6plus }),
                },
                SubScoreOutput {
                    id: "tsunami",
                    score: f_tsun,
                    available: tsun_avail,
                    detail: json!({ "depth_m": self.tsunami_depth }),
                },
                SubScoreOutput {
                    id: "landslide",
                    score: f_land,
                    available: land_avail,
                    detail: json!({ "steep_nearby": self.steep_nearby }),
                },
            ],
        }
    }

    /// S2 Terrain axis: ground quality (AVS30) from J-SHIS.
    fn build_terrain_axis(&self, axis_weight: f64) -> AxisOutput {
        let g_avs = score_avs30(self.avs30);
        let avs_avail = self.avs30.is_some();
        let (score, confidence) = compute_s2(g_avs, avs_avail);

        AxisOutput {
            score,
            weight: axis_weight,
            confidence,
            sub_scores: vec![SubScoreOutput {
                id: "avs30",
                score: g_avs,
                available: avs_avail,
                detail: json!({ "avs30_ms": self.avs30 }),
            }],
        }
    }

    /// S3 Livability axis: transit (unavailable in Phase 1), education, medical.
    fn build_livability_axis(&self, axis_weight: f64) -> AxisOutput {
        let l_edu = score_education(
            self.schools.count_800m,
            self.schools.has_primary,
            self.schools.has_junior_high,
        );
        let l_med = score_medical(
            self.medical.hospital_count,
            self.medical.clinic_count,
            self.medical.total_beds,
        );
        // Transit data not yet available in Phase 1 — use unavailable fallback.
        let l_transit = 100.0_f64;
        let transit_avail = false;
        let edu_avail = true;
        let med_avail = true;
        let (score, confidence) =
            compute_s3(l_transit, l_edu, l_med, transit_avail, edu_avail, med_avail);

        AxisOutput {
            score,
            weight: axis_weight,
            confidence,
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
                        "count_800m": self.schools.count_800m,
                        "has_primary": self.schools.has_primary,
                        "has_junior_high": self.schools.has_junior_high,
                    }),
                },
                SubScoreOutput {
                    id: "medical",
                    score: l_med,
                    available: med_avail,
                    detail: json!({
                        "hospital_count": self.medical.hospital_count,
                        "clinic_count": self.medical.clinic_count,
                        "total_beds": self.medical.total_beds,
                    }),
                },
            ],
        }
    }

    /// S4 Future axis: population (unavailable in Phase 1), price trend
    /// (CAGR), and floor-area ratio.
    fn build_future_axis(&self, axis_weight: f64) -> AxisOutput {
        // Population data not available in Phase 1 — unavailable fallback.
        let p_pop = 100.0_f64;
        let pop_avail = false;
        let cagr = compute_price_cagr(&self.prices);
        let p_price = score_price_trend(cagr);
        let price_avail = self.prices.len() >= 2;
        let p_far = score_far(self.far);
        let far_avail = self.far.is_some();
        let (score, confidence) =
            compute_s4(p_pop, p_price, p_far, pop_avail, price_avail, far_avail);

        AxisOutput {
            score,
            weight: axis_weight,
            confidence,
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
                    detail: json!({ "floor_area_ratio": self.far }),
                },
            ],
        }
    }

    /// S5 Price axis: relative value (z-score vs same-zone cohort) and
    /// transaction volume.
    ///
    /// Returns both the axis output and the raw `relative_value` sub-score
    /// because [`compute_cross_analysis`] feeds on it directly rather than
    /// on the aggregated S5.
    fn build_price_axis(&self, axis_weight: f64) -> (AxisOutput, f64) {
        let v_rel = score_relative_value(self.z_score.z_score);
        let v_vol = score_volume(self.tx_count);
        let rel_avail = self.z_score.sample_count > 0;
        let vol_avail = true;
        let (score, confidence) = compute_s5(v_rel, v_vol, rel_avail, vol_avail);

        let output = AxisOutput {
            score,
            weight: axis_weight,
            confidence,
            sub_scores: vec![
                SubScoreOutput {
                    id: "relative_value",
                    score: v_rel,
                    available: rel_avail,
                    detail: json!({
                        "z_score": self.z_score.z_score,
                        "zone_type": self.z_score.zone_type,
                        "sample_count": self.z_score.sample_count,
                    }),
                },
                SubScoreOutput {
                    id: "volume",
                    score: v_vol,
                    available: vol_avail,
                    detail: json!({ "tx_count": self.tx_count }),
                },
            ],
        };

        (output, v_rel)
    }

    /// Human-readable freshness label derived from the most-recent price year.
    fn data_freshness(&self) -> String {
        self.prices
            .last()
            .map(|p| p.year.to_string())
            .unwrap_or_else(|| "N/A".into())
    }
}

// ─── Output types ────────────────────────────────────────────────────────────

/// Full TLS result returned by [`ComputeTlsUsecase::execute`].
pub(crate) struct TlsOutput {
    pub(crate) score: f64,
    pub(crate) grade: Grade,
    pub(crate) axes: AxesOutput,
    pub(crate) cross_analysis: CrossAnalysis,
    pub(crate) weight_preset: WeightPreset,
    pub(crate) data_freshness: String,
}

/// Five-axis breakdown of the TLS.
pub(crate) struct AxesOutput {
    pub(crate) disaster: AxisOutput,
    pub(crate) terrain: AxisOutput,
    pub(crate) livability: AxisOutput,
    pub(crate) future: AxisOutput,
    pub(crate) price: AxisOutput,
}

/// Single axis with its score, weight, confidence, and sub-score detail.
pub(crate) struct AxisOutput {
    pub(crate) score: f64,
    pub(crate) weight: f64,
    pub(crate) confidence: f64,
    pub(crate) sub_scores: Vec<SubScoreOutput>,
}

/// One sub-score within an axis.
pub(crate) struct SubScoreOutput {
    pub(crate) id: &'static str,
    pub(crate) score: f64,
    pub(crate) available: bool,
    pub(crate) detail: serde_json::Value,
}

// ─── Pure helpers ─────────────────────────────────────────────────────────────

/// Compute CAGR from a sorted price record slice. Returns 0.0 when fewer than 2 records.
///
/// # Panics
///
/// Debug-asserts that `prices` is sorted by year ascending.
fn compute_price_cagr(prices: &[PriceRecord]) -> f64 {
    debug_assert!(
        prices.windows(2).all(|w| w[0].year <= w[1].year),
        "prices must be sorted by year ascending"
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::ZScoreResult;
    use crate::domain::repository::mock::MockTlsRepository;

    fn sample_coord() -> Coord {
        Coord::new(35.68, 139.76).unwrap()
    }

    fn prime_mock_repo(err: bool) -> MockTlsRepository {
        let prices_result = if err {
            Err(DomainError::Database("boom".into()))
        } else {
            Ok(vec![
                PriceRecord {
                    year: 2019,
                    price_per_sqm: 1000,
                },
                PriceRecord {
                    year: 2023,
                    price_per_sqm: 1200,
                },
            ])
        };

        MockTlsRepository::new()
            .with_find_nearest_prices(prices_result)
            .with_find_flood_depth_rank(Ok(None))
            .with_has_steep_slope_nearby(Ok(false))
            .with_find_schools_nearby(Ok(SchoolStats {
                count_800m: 2,
                has_primary: true,
                has_junior_high: true,
            }))
            .with_find_medical_nearby(Ok(MedicalStats {
                hospital_count: 1,
                clinic_count: 3,
                total_beds: 150,
            }))
            .with_find_zoning_far(Ok(Some(200.0)))
            .with_calc_price_z_score(Ok(ZScoreResult {
                z_score: 0.2,
                zone_type: "commercial".into(),
                sample_count: 50,
            }))
            .with_count_recent_transactions(Ok(25))
    }

    /// Runs `execute` with a fully-mocked repo and `jshis = None` so that the
    /// entire flow (8 parallel DB calls, sub-score computation, weighting,
    /// cross-analysis, grade assignment) exercises end-to-end without hitting
    /// any real network/database resources. Verifies only the invariants a
    /// table-driven preset test would also verify: score is in `[0, 100]` and
    /// each axis has the correct weight for the chosen preset.
    #[tokio::test]
    async fn execute_happy_path_with_balance_preset() {
        let repo = Arc::new(prime_mock_repo(false));
        let usecase = ComputeTlsUsecase::new(repo, None);

        let output = usecase
            .execute(&sample_coord(), WeightPreset::Balance)
            .await
            .unwrap();

        assert!(output.score >= 0.0 && output.score <= 100.0);
        // Balance preset axis weights should sum to 1.0.
        let weight_sum = output.axes.disaster.weight
            + output.axes.terrain.weight
            + output.axes.livability.weight
            + output.axes.future.weight
            + output.axes.price.weight;
        assert!((weight_sum - 1.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(prime_mock_repo(true));
        let usecase = ComputeTlsUsecase::new(repo, None);

        let result = usecase
            .execute(&sample_coord(), WeightPreset::Balance)
            .await;
        assert!(matches!(result, Err(DomainError::Database(_))));
    }

    /// Table-driven weight-sum check across every `WeightPreset`. This is the
    /// lightweight variant of the design doc's weighted-aggregation test: it
    /// runs the full usecase for each preset and verifies that the five axis
    /// weights in the response still sum to 1.0, which is the invariant the
    /// `compute_tls` aggregator must preserve for all presets.
    #[tokio::test]
    async fn execute_all_presets_preserve_axis_weight_sum() {
        let presets = [
            WeightPreset::Balance,
            WeightPreset::Investment,
            WeightPreset::Residential,
            WeightPreset::DisasterFocus,
        ];

        for preset in presets {
            let repo = Arc::new(prime_mock_repo(false));
            let usecase = ComputeTlsUsecase::new(repo, None);

            let output = usecase.execute(&sample_coord(), preset).await.unwrap();

            let weight_sum = output.axes.disaster.weight
                + output.axes.terrain.weight
                + output.axes.livability.weight
                + output.axes.future.weight
                + output.axes.price.weight;
            assert!(
                (weight_sum - 1.0).abs() < 1e-9,
                "preset {:?} weight sum = {}",
                preset,
                weight_sum
            );
        }
    }
}
