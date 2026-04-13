//! Shared bounding box and coordinate query DTOs.
//!
//! [`BBoxQuery`] is used by the `stats` and `area-data` handlers.
//! [`CoordQuery`] is used by the `score` and `trend` handlers.
//!
//! Both types deserialize from Axum's [`Query`](axum::extract::Query)
//! extractor and provide `into_domain` / `parse_preset` methods that
//! convert raw query strings into validated domain value objects.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, Coord, PrefCode};
use terrasight_domain::scoring::tls::WeightPreset;

/// Bounding box query parameters shared by `GET /api/v1/area-data` and
/// `GET /api/v1/stats`.
///
/// Fields are provided as individual query parameters (not a packed string)
/// and are converted to a validated [`BBox`] domain value object via
/// [`into_domain`](BBoxQuery::into_domain).
#[derive(Debug, Deserialize)]
pub struct BBoxQuery {
    /// Southern latitude bound (WGS-84 decimal degrees, −90 … 90).
    pub south: f64,
    /// Western longitude bound (WGS-84 decimal degrees, −180 … 180).
    pub west: f64,
    /// Northern latitude bound (WGS-84 decimal degrees, −90 … 90).
    pub north: f64,
    /// Eastern longitude bound (WGS-84 decimal degrees, −180 … 180).
    pub east: f64,
    /// Optional 2-digit prefecture code filter (e.g. `"13"` for Tokyo).
    /// When omitted the query is not scoped to any prefecture.
    #[serde(default)]
    pub pref_code: Option<String>,
}

impl BBoxQuery {
    /// Convert to validated domain value objects.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCoordinate`] when any coordinate is out
    /// of range, [`DomainError::BBoxTooLarge`] when the area exceeds the
    /// configured maximum, or [`DomainError::InvalidPrefCode`] when
    /// `pref_code` is present but invalid.
    pub fn into_domain(self) -> Result<(BBox, Option<PrefCode>), DomainError> {
        let bbox = BBox::new(self.south, self.west, self.north, self.east)?;
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;
        Ok((bbox, pref_code))
    }
}

/// Coordinate query parameters for `GET /api/v1/score` and `GET /api/v1/trend`.
#[derive(Debug, Deserialize)]
pub struct CoordQuery {
    /// Latitude in WGS-84 decimal degrees (−90 … 90).
    pub lat: f64,
    /// Longitude in WGS-84 decimal degrees (−180 … 180).
    pub lng: f64,
    /// TLS weight preset key. Accepted values: `"balance"`, `"investment"`,
    /// `"disaster"`, `"livability"`. Defaults to `"balance"` when omitted.
    /// Unknown strings also fall back to `"balance"`.
    #[serde(default = "default_preset")]
    pub preset: String,
}

fn default_preset() -> String {
    "balance".into()
}

impl CoordQuery {
    /// Convert to a validated [`Coord`] domain value object.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCoordinate`] when `lat` or `lng` is
    /// outside the valid WGS-84 range.
    pub fn into_domain(self) -> Result<Coord, DomainError> {
        Coord::new(self.lat, self.lng)
    }

    /// Parse the `preset` query string into a domain [`WeightPreset`].
    ///
    /// Unknown strings fall back to [`WeightPreset::Balance`].
    pub fn parse_preset(&self) -> WeightPreset {
        self.preset.parse().unwrap() // Infallible
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
            pref_code: None,
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
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }
}
