//! Request DTO for `GET /api/v1/area-data`.
//!
//! [`AreaDataQuery`] deserializes the bounding box fields, comma-separated
//! layer list, zoom level, and optional prefecture code from the query string.
//! Validation and conversion to domain types is performed by
//! [`AreaDataQuery::into_domain`].

use serde::Deserialize;

use crate::domain::constants::DEFAULT_ZOOM_LEVEL;
use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LayerType, PrefCode, ZoomLevel};

/// Query parameters for `GET /api/v1/area-data`.
///
/// The bounding box is expressed as four individual latitude/longitude
/// fields. The `layers` field is a comma-separated list of layer names
/// (e.g. `"landprice,flood,zoning"`); unknown layer names are silently
/// skipped with a `WARN` log.
#[derive(Debug, Deserialize)]
pub struct AreaDataQuery {
    /// Southern latitude bound (WGS-84 decimal degrees).
    pub south: f64,
    /// Western longitude bound (WGS-84 decimal degrees).
    pub west: f64,
    /// Northern latitude bound (WGS-84 decimal degrees).
    pub north: f64,
    /// Eastern longitude bound (WGS-84 decimal degrees).
    pub east: f64,
    /// Comma-separated layer names to fetch (e.g. `"landprice,zoning,flood"`).
    /// Must contain at least one recognised layer name after parsing, otherwise
    /// `into_domain` returns [`DomainError::MissingParameter`].
    #[serde(default)]
    pub layers: String,
    /// Map zoom level used to derive per-layer feature limits.
    /// Defaults to `14` (street level) when not provided.
    #[serde(default = "default_zoom")]
    pub zoom: u32,
    /// Optional 2-digit prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
}

impl AreaDataQuery {
    /// Convert to validated domain types.
    ///
    /// Parses and validates the bounding box, splits `layers` by comma,
    /// silently drops unknown layer names, and clamps `zoom` to the
    /// [`ZoomLevel`] range.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::MissingParameter`] when `layers` is empty or
    /// contains only unrecognised names. Propagates [`DomainError::InvalidCoordinate`],
    /// [`DomainError::BBoxTooLarge`], and [`DomainError::InvalidPrefCode`]
    /// from the domain value object constructors.
    pub fn into_domain(
        self,
    ) -> Result<(BBox, Vec<LayerType>, ZoomLevel, Option<PrefCode>), DomainError> {
        let bbox = BBox::new(self.south, self.west, self.north, self.east)?;
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;

        let layers: Vec<LayerType> = self
            .layers
            .split(',')
            .map(str::trim)
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

        Ok((bbox, layers, ZoomLevel::clamped(self.zoom), pref_code))
    }
}

pub(super) fn default_zoom() -> u32 {
    DEFAULT_ZOOM_LEVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_data_query_parses_layers() {
        let q = AreaDataQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
            layers: "landprice,flood,unknown".into(),
            zoom: 14,
            pref_code: None,
        };
        let (_, layers, zoom, pref_code) = q.into_domain().unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(zoom, ZoomLevel::clamped(14));
        assert!(pref_code.is_none());
    }

    #[test]
    fn area_data_query_empty_layers_is_error() {
        let q = AreaDataQuery {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
            layers: String::new(),
            zoom: 14,
            pref_code: None,
        };
        assert!(q.into_domain().is_err());
    }
}
