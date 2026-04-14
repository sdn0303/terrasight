//! Response DTOs for `GET /api/v1/score` (the TLS scoring endpoint).

use serde::Serialize;

use crate::domain::constants::SCORE_DISCLAIMER;
use crate::usecase::compute_tls::{AxesOutput, AxisOutput, SubScoreOutput, TlsOutput};
use terrasight_domain::scoring::tls::CrossAnalysis;

/// Top-level response for `GET /api/v1/score` (TLS scoring endpoint).
#[derive(Debug, Serialize)]
pub struct TlsResponse {
    /// Geographic coordinate echo-back.
    pub location: LocationDto,
    /// Aggregated TLS score and grade.
    pub tls: TlsSummaryDto,
    /// Per-axis breakdown of the TLS computation.
    pub axes: AxesDto,
    /// Cross-axis composite indicators.
    pub cross_analysis: CrossAnalysisDto,
    /// Scoring metadata: timestamp, weight preset, data freshness, disclaimer.
    pub metadata: TlsMetadataDto,
}

/// Echo-back of the coordinate that was scored.
#[derive(Debug, Serialize)]
pub struct LocationDto {
    /// Latitude supplied in the request (WGS-84 decimal degrees).
    pub lat: f64,
    /// Longitude supplied in the request (WGS-84 decimal degrees).
    pub lng: f64,
}

/// Aggregated TLS score nested inside [`TlsResponse`].
#[derive(Debug, Serialize)]
pub struct TlsSummaryDto {
    /// Total Location Score (0.0 – 100.0).
    pub score: f64,
    /// Letter grade derived from the score: `"S"`, `"A"`, `"B"`, `"C"`, or `"D"`.
    pub grade: &'static str,
    /// Human-readable label for the grade in Japanese.
    pub label: &'static str,
}

/// Five-axis TLS breakdown nested inside [`TlsResponse`].
#[derive(Debug, Serialize)]
pub struct AxesDto {
    /// Disaster risk axis.
    pub disaster: AxisDto,
    /// Terrain quality axis.
    pub terrain: AxisDto,
    /// Livability (amenities, transit) axis.
    pub livability: AxisDto,
    /// Future potential (redevelopment, population trend) axis.
    pub future: AxisDto,
    /// Price value axis.
    pub price: AxisDto,
}

/// Score, weight, and sub-scores for a single TLS axis.
#[derive(Debug, Serialize)]
pub struct AxisDto {
    /// Weighted axis score contribution (0.0 – 100.0).
    pub score: f64,
    /// Normalised weight applied to this axis (sum of all weights = 1.0).
    pub weight: f64,
    /// Data confidence for this axis (0.0 – 1.0; lower means sparse data).
    pub confidence: f64,
    /// Individual sub-score components that make up this axis.
    pub sub: Vec<SubScoreDto>,
}

/// One sub-score component inside an [`AxisDto`].
#[derive(Debug, Serialize)]
pub struct SubScoreDto {
    /// Stable identifier for this sub-score (e.g. `"flood_risk"`).
    pub id: &'static str,
    /// Sub-score value (0.0 – 100.0).
    pub score: f64,
    /// `false` when the underlying data source had no records for this point.
    pub available: bool,
    /// Arbitrary JSON detail blob; schema varies per sub-score type.
    pub detail: serde_json::Value,
}

/// Cross-axis composite indicators nested inside [`TlsResponse`].
#[derive(Debug, Serialize)]
pub struct CrossAnalysisDto {
    /// Value-discovery score: high TLS at a low price signals undervaluation.
    pub value_discovery: f64,
    /// Demand signal derived from price trend and livability.
    pub demand_signal: f64,
    /// Ground safety composite from terrain and disaster axes.
    pub ground_safety: f64,
}

/// Scoring metadata nested inside [`TlsResponse`].
#[derive(Debug, Serialize)]
pub struct TlsMetadataDto {
    /// RFC 3339 timestamp of when this score was computed.
    pub calculated_at: String,
    /// Weight preset used for this computation (e.g. `"balance"`, `"investment"`).
    pub weight_preset: String,
    /// Human-readable note on the age of the underlying data.
    pub data_freshness: String,
    /// Standard investment disclaimer text.
    pub disclaimer: String,
}

fn axis_to_dto(axis: AxisOutput) -> AxisDto {
    AxisDto {
        score: axis.score,
        weight: axis.weight,
        confidence: axis.confidence,
        sub: axis.sub_scores.into_iter().map(sub_score_to_dto).collect(),
    }
}

fn sub_score_to_dto(s: SubScoreOutput) -> SubScoreDto {
    SubScoreDto {
        id: s.id,
        score: s.score,
        available: s.available,
        detail: s.detail,
    }
}

fn axes_to_dto(axes: AxesOutput) -> AxesDto {
    AxesDto {
        disaster: axis_to_dto(axes.disaster),
        terrain: axis_to_dto(axes.terrain),
        livability: axis_to_dto(axes.livability),
        future: axis_to_dto(axes.future),
        price: axis_to_dto(axes.price),
    }
}

fn cross_analysis_to_dto(ca: CrossAnalysis) -> CrossAnalysisDto {
    CrossAnalysisDto {
        value_discovery: ca.value_discovery,
        demand_signal: ca.demand_signal,
        ground_safety: ca.ground_safety,
    }
}

impl TlsResponse {
    /// Construct a TLS response from handler coordinates and usecase output.
    pub fn new(lat: f64, lng: f64, t: TlsOutput) -> Self {
        Self {
            location: LocationDto { lat, lng },
            tls: TlsSummaryDto {
                score: t.score,
                grade: t.grade.as_str(),
                label: t.grade.label(),
            },
            axes: axes_to_dto(t.axes),
            cross_analysis: cross_analysis_to_dto(t.cross_analysis),
            metadata: TlsMetadataDto {
                calculated_at: chrono::Utc::now().to_rfc3339(),
                weight_preset: t.weight_preset.as_str().to_string(),
                data_freshness: t.data_freshness,
                disclaimer: SCORE_DISCLAIMER.to_string(),
            },
        }
    }
}
