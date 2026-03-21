use serde::Deserialize;

use crate::domain::constants::TREND_DEFAULT_YEARS;
use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, Coord, LayerType};

/// Bounding box query parameters for `/api/area-data` and `/api/stats`.
///
/// This is a handler-layer DTO that deserializes from query string,
/// then converts to the validated domain `BBox` value object.
#[derive(Debug, Deserialize)]
pub struct BBoxQuery {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl BBoxQuery {
    /// Convert to domain value object (validation happens inside `BBox::new`).
    pub fn into_domain(self) -> Result<BBox, DomainError> {
        BBox::new(self.south, self.west, self.north, self.east)
    }
}

/// Coordinate query parameters for `/api/score` and `/api/trend`.
#[derive(Debug, Deserialize)]
pub struct CoordQuery {
    pub lat: f64,
    pub lng: f64,
}

impl CoordQuery {
    pub fn into_domain(self) -> Result<Coord, DomainError> {
        Coord::new(self.lat, self.lng)
    }
}

/// Area data query with layers parameter.
#[derive(Debug, Deserialize)]
pub struct AreaDataQuery {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    #[serde(default)]
    pub layers: String,
}

impl AreaDataQuery {
    /// Convert to domain types: validated BBox + parsed LayerType list.
    pub fn into_domain(self) -> Result<(BBox, Vec<LayerType>), DomainError> {
        let bbox = BBox::new(self.south, self.west, self.north, self.east)?;

        let layers: Vec<LayerType> = self
            .layers
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|s| {
                let parsed = LayerType::parse(s);
                if parsed.is_none() {
                    tracing::warn!(layer = s, "unknown layer requested, skipping");
                }
                parsed
            })
            .collect();

        if layers.is_empty() {
            return Err(DomainError::MissingParameter("layers".into()));
        }

        Ok((bbox, layers))
    }
}

/// Trend query parameters (includes optional `years`).
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub lat: f64,
    pub lng: f64,
    #[serde(default = "default_years")]
    pub years: i32,
}

fn default_years() -> i32 {
    TREND_DEFAULT_YEARS
}

impl TrendQuery {
    pub fn into_domain(self) -> Result<(Coord, i32), DomainError> {
        let coord = Coord::new(self.lat, self.lng)?;
        Ok((coord, self.years))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_query_into_domain_valid() {
        let q = BBoxQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
        };
        assert!(q.into_domain().is_ok());
    }

    #[test]
    fn bbox_query_into_domain_invalid() {
        let q = BBoxQuery {
            south: 91.0,
            west: 0.0,
            north: 92.0,
            east: 1.0,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn area_data_query_parses_layers() {
        let q = AreaDataQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
            layers: "landprice,flood,unknown".into(),
        };
        let (_, layers) = q.into_domain().unwrap();
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn area_data_query_empty_layers_is_error() {
        let q = AreaDataQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
            layers: "".into(),
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn trend_query_default_years() {
        let q = TrendQuery {
            lat: 35.68,
            lng: 139.76,
            years: default_years(),
        };
        let (_, years) = q.into_domain().unwrap();
        assert_eq!(years, 5);
    }
}
