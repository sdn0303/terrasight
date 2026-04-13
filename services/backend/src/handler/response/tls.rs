//! Response DTOs for `GET /api/score` (the TLS scoring endpoint).

use serde::Serialize;

use crate::domain::constants::SCORE_DISCLAIMER;
use crate::domain::scoring::tls::CrossAnalysis;
use crate::usecase::compute_tls::{AxesOutput, AxisOutput, SubScoreOutput, TlsOutput};

/// Response for `GET /api/score` (TLS system).
#[derive(Debug, Serialize)]
pub struct TlsResponse {
    pub location: LocationDto,
    pub tls: TlsSummaryDto,
    pub axes: AxesDto,
    pub cross_analysis: CrossAnalysisDto,
    pub metadata: TlsMetadataDto,
}

#[derive(Debug, Serialize)]
pub struct LocationDto {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
pub struct TlsSummaryDto {
    pub score: f64,
    pub grade: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AxesDto {
    pub disaster: AxisDto,
    pub terrain: AxisDto,
    pub livability: AxisDto,
    pub future: AxisDto,
    pub price: AxisDto,
}

#[derive(Debug, Serialize)]
pub struct AxisDto {
    pub score: f64,
    pub weight: f64,
    pub confidence: f64,
    pub sub: Vec<SubScoreDto>,
}

#[derive(Debug, Serialize)]
pub struct SubScoreDto {
    pub id: &'static str,
    pub score: f64,
    pub available: bool,
    pub detail: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CrossAnalysisDto {
    pub value_discovery: f64,
    pub demand_signal: f64,
    pub ground_safety: f64,
}

#[derive(Debug, Serialize)]
pub struct TlsMetadataDto {
    pub calculated_at: String,
    pub weight_preset: String,
    pub data_freshness: String,
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
                weight_preset: serde_json::to_value(t.weight_preset)
                    .inspect_err(
                        |e| tracing::warn!(error = %e, "WeightPreset serialization failed"),
                    )
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "balance".to_string()),
                data_freshness: t.data_freshness,
                disclaimer: SCORE_DISCLAIMER.to_string(),
            },
        }
    }
}
