import type { Feature, FeatureCollection, Point, Polygon } from "geojson";

// Convert point to small square polygon (approx 30m x 30m at Tokyo latitude ~35.68°)
export const BUFFER_DEG = 0.00015; // ~15m at 35.68° latitude

/**
 * Converts a GeoJSON FeatureCollection of Point features to Polygon features
 * (small squares) suitable for MapLibre fill-extrusion layers.
 * Non-Point features are filtered out.
 *
 * GeoJSON coordinates follow RFC 7946: [longitude, latitude].
 */
export function pointsToPolygons(fc: FeatureCollection): FeatureCollection {
  const features: Feature<Polygon>[] = [];

  for (const f of fc.features) {
    if (f.geometry.type !== "Point") continue;
    const point = f.geometry as Point;
    const [lng, lat] = point.coordinates as [number, number];

    features.push({
      type: "Feature",
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [lng - BUFFER_DEG, lat - BUFFER_DEG],
            [lng + BUFFER_DEG, lat - BUFFER_DEG],
            [lng + BUFFER_DEG, lat + BUFFER_DEG],
            [lng - BUFFER_DEG, lat + BUFFER_DEG],
            [lng - BUFFER_DEG, lat - BUFFER_DEG], // close ring
          ],
        ],
      },
      properties: f.properties,
    });
  }

  return { type: "FeatureCollection", features };
}
