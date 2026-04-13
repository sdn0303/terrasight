//! Request DTO for `GET /api/area-data`.

use serde::Deserialize;

use crate::domain::constants::DEFAULT_ZOOM_LEVEL;
use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, LayerType, PrefCode, ZoomLevel};

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
    /// Optional prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
}

impl AreaDataQuery {
    /// Convert to domain types: validated BBox + parsed LayerType list + ZoomLevel + optional PrefCode.
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
