//! PostGIS `ST_AsGeoJSON` output → domain [`GeoFeature`] conversion.
//!
//! Every repository that projects `ST_AsGeoJSON(geom)::jsonb AS geometry`
//! calls [`to_geo_feature`] to unpack the raw JSON into the typed
//! [`GeoFeature`] domain struct. The actual geometry parsing is delegated
//! to `terrasight_server::db::geo::to_raw_geo_feature` so that low-level
//! JSON field extraction lives in one place.

use crate::domain::entity::{GeoFeature, GeoJsonGeometry};

/// Convert PostGIS `ST_AsGeoJSON(geom)::jsonb` output and a properties object
/// into a domain [`GeoFeature`].
///
/// `geojson` must be the raw JSON value produced by `ST_AsGeoJSON(geom)::jsonb`
/// — a GeoJSON geometry object with `"type"` and `"coordinates"` keys.
/// `properties` is any JSON object to attach as the feature's property bag.
pub(crate) fn to_geo_feature(
    geojson: serde_json::Value,
    properties: serde_json::Value,
) -> GeoFeature {
    let raw = terrasight_server::db::geo::to_raw_geo_feature(geojson, properties);
    GeoFeature {
        geometry: GeoJsonGeometry {
            r#type: raw.geo_type,
            coordinates: raw.coordinates,
        },
        properties: raw.properties,
    }
}
