use serde::Deserialize;

use crate::domain::constants::TREND_DEFAULT_YEARS;
use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, Coord, LayerType, Year};

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
    /// Map zoom level used to compute per-layer feature limits.
    /// Defaults to 14 (street level) when not provided.
    #[serde(default = "default_zoom")]
    pub zoom: u32,
}

impl AreaDataQuery {
    /// Convert to domain types: validated BBox + parsed LayerType list + zoom.
    pub fn into_domain(self) -> Result<(BBox, Vec<LayerType>, u32), DomainError> {
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

        Ok((bbox, layers, self.zoom))
    }
}

fn default_zoom() -> u32 {
    14
}

/// Land price query parameters for `GET /api/v1/land-prices`.
///
/// Expects `year` (integer) and `bbox` as a comma-separated string
/// `"sw_lng,sw_lat,ne_lng,ne_lat"` (longitude-first, RFC 7946 order).
///
/// # Example query string
///
/// ```text
/// ?year=2023&bbox=139.70,35.65,139.80,35.70
/// ?year=2023&bbox=139.70,35.65,139.80,35.70&zoom=16
/// ```
#[derive(Debug, Deserialize)]
pub struct LandPriceQuery {
    pub year: i32,
    /// Comma-separated bounding box: `sw_lng,sw_lat,ne_lng,ne_lat`.
    pub bbox: String,
    /// Map zoom level used to compute the feature limit.
    /// Defaults to 14 (street level) when not provided.
    #[serde(default = "default_zoom")]
    pub zoom: u32,
}

impl LandPriceQuery {
    /// Parse and validate into domain value objects `(Year, BBox, zoom)`.
    ///
    /// The bbox string must contain exactly four comma-separated `f64` values
    /// in the order `sw_lng, sw_lat, ne_lng, ne_lat` (longitude before latitude,
    /// consistent with RFC 7946 coordinate order).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::MissingParameter`] when the bbox string cannot be
    /// parsed, and propagates [`DomainError::InvalidYear`] /
    /// [`DomainError::InvalidCoordinate`] / [`DomainError::BBoxTooLarge`] from
    /// the domain value object constructors.
    pub fn into_domain(self) -> Result<(Year, BBox, u32), DomainError> {
        let year = Year::new(self.year)?;

        let parts: Vec<f64> = self
            .bbox
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<f64>()
                    .map_err(|_| DomainError::MissingParameter("bbox".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if parts.len() != 4 {
            return Err(DomainError::MissingParameter(
                "bbox must have exactly 4 values: sw_lng,sw_lat,ne_lng,ne_lat".into(),
            ));
        }

        // bbox format: sw_lng, sw_lat, ne_lng, ne_lat  (longitude first — RFC 7946)
        let (sw_lng, sw_lat, ne_lng, ne_lat) = (parts[0], parts[1], parts[2], parts[3]);

        // BBox::new expects (south, west, north, east)
        let bbox = BBox::new(sw_lat, sw_lng, ne_lat, ne_lng)?;

        Ok((year, bbox, self.zoom))
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
            zoom: 14,
        };
        let (_, layers, zoom) = q.into_domain().unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(zoom, 14);
    }

    #[test]
    fn area_data_query_empty_layers_is_error() {
        let q = AreaDataQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
            layers: "".into(),
            zoom: 14,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_query_valid() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "139.70,35.65,139.80,35.70".into(),
            zoom: 14,
        };
        let (year, bbox, zoom) = q.into_domain().unwrap();
        assert_eq!(year.value(), 2023);
        assert!((bbox.west() - 139.70).abs() < f64::EPSILON);
        assert!((bbox.south() - 35.65).abs() < f64::EPSILON);
        assert!((bbox.east() - 139.80).abs() < f64::EPSILON);
        assert!((bbox.north() - 35.70).abs() < f64::EPSILON);
        assert_eq!(zoom, 14);
    }

    #[test]
    fn land_price_query_invalid_bbox_string() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "not,valid,bbox".into(),
            zoom: 14,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_query_invalid_year() {
        let q = LandPriceQuery {
            year: 1999,
            bbox: "139.70,35.65,139.80,35.70".into(),
            zoom: 14,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_query_bbox_wrong_field_count() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "139.70,35.65,139.80".into(),
            zoom: 14,
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
