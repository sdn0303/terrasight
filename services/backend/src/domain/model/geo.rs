//! GeoJSON domain types: features, geometry wrappers, layer results, and layer type enum.
//!
//! `serde_json::Value` is an allowed dependency in the domain layer — it is a
//! data-representation library, not an I/O framework.

use serde::Serialize;

/// GeoJSON Feature in domain representation.
///
/// Corresponds to PostGIS `ST_AsGeoJSON` output. Coordinates follow
/// RFC 7946 `[longitude, latitude]` order.
///
/// Note: `serde_json::Value` is an allowed dependency in the domain layer —
/// it is a data-representation library, not an I/O framework.
#[derive(Debug, Clone)]
pub struct GeoFeature {
    /// The GeoJSON geometry (type + coordinates).
    pub geometry: GeoJsonGeometry,
    /// Arbitrary feature properties serialized from the database row.
    pub properties: serde_json::Value,
}

/// GeoJSON geometry type identifier (RFC 7946 §3.1).
///
/// Encodes the set of valid GeoJSON geometry types as an enum so that
/// `GeoJsonGeometry` cannot carry an arbitrary or misspelled type string.
/// The `as_str` method returns the canonical RFC 7946 name; `from_db_str`
/// maps the PostGIS `ST_GeometryType` output to the corresponding variant.
#[derive(Debug, Clone, Serialize)]
pub enum GeoJsonType {
    /// Coordinate pair geometry (RFC 7946 §3.1.2).
    Point,
    /// Closed-ring polygon geometry (RFC 7946 §3.1.6).
    Polygon,
    /// Collection of polygon geometries (RFC 7946 §3.1.7).
    MultiPolygon,
    /// Sequence of positions (RFC 7946 §3.1.4).
    LineString,
}

impl GeoJsonType {
    /// Return the RFC 7946 canonical type string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Polygon => "Polygon",
            Self::MultiPolygon => "MultiPolygon",
            Self::LineString => "LineString",
        }
    }

    /// Map a PostGIS geometry type string to the corresponding variant.
    ///
    /// Unknown strings default to [`GeoJsonType::Point`] as a defensive
    /// fallback; callers that require an exact match should validate
    /// `as_str()` after construction.
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Polygon" => Self::Polygon,
            "MultiPolygon" => Self::MultiPolygon,
            "LineString" => Self::LineString,
            _ => Self::Point,
        }
    }
}

/// GeoJSON geometry (flexible via `serde_json::Value` for coordinates).
///
/// Using `serde_json::Value` for `coordinates` avoids a family of geometry
/// wrapper types (Point, Polygon, MultiPolygon …) without sacrificing
/// correctness — MapLibre GL accepts the raw JSON unchanged.
#[derive(Debug, Clone)]
pub struct GeoJsonGeometry {
    /// GeoJSON geometry type; encodes the RFC 7946 §3.1 type discriminator.
    pub r#type: GeoJsonType,
    /// Raw coordinate array; shape depends on `type`.
    pub coordinates: serde_json::Value,
}

/// Result of a per-layer bbox query with truncation metadata.
///
/// When the database returns more rows than `limit`, the repository fetches
/// `limit + 1` rows (N+1 pattern), sets `truncated = true`, and returns
/// only the first `limit` features. Callers can surface the `truncated` flag
/// and `limit` to MapLibre GL clients so they know to zoom in for full data.
#[derive(Debug, Clone)]
pub struct LayerResult {
    /// GeoJSON features returned (at most `limit` items).
    pub features: Vec<GeoFeature>,
    /// `true` when the result set was capped at `limit`.
    pub truncated: bool,
    /// The limit that was applied for this layer + zoom combination.
    pub limit: i64,
}

/// Map layer type for the `/api/v1/features` endpoint.
///
/// Each variant corresponds to a PostGIS table and a specific feature limit
/// curve as a function of zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    /// 地価公示 / 地価調査 — official land price survey points.
    LandPrice,
    /// 用途地域 — urban planning zone polygons.
    Zoning,
    /// 浸水想定区域 — flood-risk hazard areas.
    Flood,
    /// 急傾斜地崩壊危険区域 — steep-slope hazard areas.
    SteepSlope,
    /// 学校 — elementary, middle, and high schools.
    Schools,
    /// 医療施設 — hospitals and clinics.
    Medical,
}

impl LayerType {
    /// Parse from REST API query string value. Returns `None` for unknown layers.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "landprice" => Some(Self::LandPrice),
            "zoning" => Some(Self::Zoning),
            "flood" => Some(Self::Flood),
            "steep_slope" => Some(Self::SteepSlope),
            "schools" => Some(Self::Schools),
            "medical" => Some(Self::Medical),
            _ => None,
        }
    }

    /// REST API key string for JSON response keys.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LandPrice => "landprice",
            Self::Zoning => "zoning",
            Self::Flood => "flood",
            Self::SteepSlope => "steep_slope",
            Self::Schools => "schools",
            Self::Medical => "medical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_type_roundtrip() {
        for name in [
            "landprice",
            "zoning",
            "flood",
            "steep_slope",
            "schools",
            "medical",
        ] {
            let lt = LayerType::parse(name).unwrap();
            assert_eq!(lt.as_str(), name);
        }
        assert!(LayerType::parse("unknown").is_none());
    }
}
