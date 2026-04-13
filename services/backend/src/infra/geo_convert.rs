//! Shared PostGIS → domain [`GeoFeature`] conversion helper.

use crate::domain::entity::{GeoFeature, GeoJsonGeometry};

/// Parse PostGIS `ST_AsGeoJSON` output into a domain [`GeoFeature`].
pub(crate) fn to_geo_feature(
    geojson: serde_json::Value,
    properties: serde_json::Value,
) -> GeoFeature {
    let raw = realestate_db::geo::to_raw_geo_feature(geojson, properties);
    GeoFeature {
        geometry: GeoJsonGeometry {
            r#type: raw.geo_type,
            coordinates: raw.coordinates,
        },
        properties: raw.properties,
    }
}
