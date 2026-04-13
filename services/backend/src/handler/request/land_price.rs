//! Request DTOs for the land-price family of endpoints.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, PrefCode, Year, ZoomLevel};
use crate::handler::request::area_data::default_zoom;

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
    /// Optional prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
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
    pub fn into_domain(self) -> Result<(Year, BBox, ZoomLevel, Option<PrefCode>), DomainError> {
        let year = Year::new(self.year)?;
        let bbox = BBox::parse_sw_ne_str(&self.bbox)?;
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;
        Ok((year, bbox, ZoomLevel::clamped(self.zoom), pref_code))
    }
}

/// Land price year-range query parameters for `GET /api/v1/land-prices/all-years`.
///
/// Expects `bbox` as a comma-separated string `"sw_lng,sw_lat,ne_lng,ne_lat"`
/// and an optional year range `from`/`to` (defaults to `2019..=2030`).
///
/// # Example query string
///
/// ```text
/// ?bbox=139.70,35.65,139.80,35.70
/// ?bbox=139.70,35.65,139.80,35.70&from=2020&to=2024&zoom=15
/// ```
#[derive(Debug, Deserialize)]
pub struct LandPriceByYearRangeQuery {
    /// Comma-separated bounding box: `sw_lng,sw_lat,ne_lng,ne_lat`.
    pub bbox: String,
    #[serde(default = "default_from_year")]
    pub from: i32,
    #[serde(default = "default_to_year")]
    pub to: i32,
    #[serde(default = "default_zoom")]
    pub zoom: u32,
    /// Optional prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
}

fn default_from_year() -> i32 {
    2019
}

fn default_to_year() -> i32 {
    2030
}

impl LandPriceByYearRangeQuery {
    /// Parse and validate into domain value objects `(from_year, to_year, BBox, zoom)`.
    ///
    /// The bbox string must contain exactly four comma-separated `f64` values
    /// in the order `sw_lng, sw_lat, ne_lng, ne_lat`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] when `from > to`, and propagates
    /// year/coordinate validation errors from the domain value objects.
    pub fn into_domain(
        self,
    ) -> Result<(Year, Year, BBox, ZoomLevel, Option<PrefCode>), DomainError> {
        if self.from > self.to {
            return Err(DomainError::Validation(
                "from year must be <= to year".into(),
            ));
        }

        let from_year = Year::new(self.from)?;
        let to_year = Year::new(self.to)?;
        let bbox = BBox::parse_sw_ne_str(&self.bbox)?;
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;

        Ok((
            from_year,
            to_year,
            bbox,
            ZoomLevel::clamped(self.zoom),
            pref_code,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn land_price_query_valid() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "139.70,35.65,139.80,35.70".into(),
            zoom: 14,
            pref_code: None,
        };
        let (year, bbox, zoom, _pref_code) = q.into_domain().unwrap();
        assert_eq!(year.value(), 2023);
        assert!((bbox.west() - 139.70).abs() < f64::EPSILON);
        assert!((bbox.south() - 35.65).abs() < f64::EPSILON);
        assert!((bbox.east() - 139.80).abs() < f64::EPSILON);
        assert!((bbox.north() - 35.70).abs() < f64::EPSILON);
        assert_eq!(zoom, ZoomLevel::clamped(14));
    }

    #[test]
    fn land_price_query_invalid_bbox_string() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "not,valid,bbox".into(),
            zoom: 14,
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_query_invalid_year() {
        let q = LandPriceQuery {
            year: 1999,
            bbox: "139.70,35.65,139.80,35.70".into(),
            zoom: 14,
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_query_bbox_wrong_field_count() {
        let q = LandPriceQuery {
            year: 2023,
            bbox: "139.70,35.65,139.80".into(),
            zoom: 14,
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn land_price_by_year_range_query_from_gt_to_is_error() {
        let q = LandPriceByYearRangeQuery {
            bbox: "139.70,35.65,139.80,35.70".into(),
            from: 2024,
            to: 2020,
            zoom: 14,
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }
}
