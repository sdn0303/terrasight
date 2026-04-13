//! Shared bounding box / coordinate query DTOs and their domain
//! conversions, used by the stats, score, and trend handlers.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::scoring::tls::WeightPreset;
use crate::domain::value_object::{BBox, Coord, PrefCode};

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
    /// Optional prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
}

impl BBoxQuery {
    /// Convert to domain value object (validation happens inside `BBox::new`).
    pub fn into_domain(self) -> Result<(BBox, Option<PrefCode>), DomainError> {
        let bbox = BBox::new(self.south, self.west, self.north, self.east)?;
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;
        Ok((bbox, pref_code))
    }
}

/// Coordinate query parameters for `/api/score` and `/api/trend`.
#[derive(Debug, Deserialize)]
pub struct CoordQuery {
    pub lat: f64,
    pub lng: f64,
    /// Weight preset key. Defaults to `"balance"` when omitted.
    #[serde(default = "default_preset")]
    pub preset: String,
}

fn default_preset() -> String {
    "balance".into()
}

impl CoordQuery {
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
