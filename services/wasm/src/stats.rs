//! Area statistics computation for the WASM spatial engine.
//!
//! Computes risk ratios, facility counts, and zoning distributions entirely
//! within WASM, matching the backend `/api/stats` response shape.

use geo::{Area, BooleanOps, Coord, Polygon, Rect};
use serde::Serialize;
pub(crate) use terrasight_domain::types::{LandPriceStats, RiskStats};

use crate::spatial_index::LayerStatsData;

/// Facility counts for a queried area.
#[derive(Debug, Serialize)]
pub(crate) struct FacilityStats {
    pub(crate) schools: u32,
    pub(crate) medical: u32,
    pub(crate) stations_nearby: u32,
}

/// A single entry in the zoning distribution.
#[derive(Debug, Serialize)]
pub(crate) struct ZoningEntry {
    pub(crate) zone: String,
    pub(crate) ratio: f64,
}

/// Aggregated area statistics returned by [`crate::SpatialEngine::compute_area_stats`].
#[derive(Debug, Serialize)]
pub(crate) struct AreaStats {
    pub(crate) land_price: LandPriceStats,
    pub(crate) risk: RiskStats,
    pub(crate) facilities: FacilityStats,
    pub(crate) zoning_distribution: Vec<ZoningEntry>,
}

/// Compute land price statistics from a `PricePoints` layer for the given indices.
///
/// Filters out zero prices (features with missing `price_per_sqm`). Returns all
/// zeros if no valid prices are found.
pub(crate) fn compute_land_price_stats(
    stats_data: &LayerStatsData,
    indices: &[u32],
) -> LandPriceStats {
    let LayerStatsData::PricePoints(prices) = stats_data else {
        return LandPriceStats::default();
    };

    let mut values: Vec<f64> = indices
        .iter()
        .filter_map(|&idx| {
            let p = *prices.get(idx as usize)?;
            if p > 0.0 { Some(p) } else { None }
        })
        .collect();

    if values.is_empty() {
        return LandPriceStats::default();
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let count = values.len();
    let avg = values.iter().sum::<f64>() / count as f64;
    let median = if count.is_multiple_of(2) {
        (values[count / 2 - 1] + values[count / 2]) / 2.0
    } else {
        values[count / 2]
    };
    // SAFETY: values is non-empty (checked above), so first/last always exist.
    let min = values[0] as i64;
    let max = values[count - 1] as i64;

    LandPriceStats {
        avg_per_sqm: Some(avg),
        median_per_sqm: Some(median),
        min_per_sqm: Some(min),
        max_per_sqm: Some(max),
        count: count as i64,
    }
}

/// Validate a bbox rect and return its area, or `None` if the area is non-positive.
///
/// Both `compute_area_ratio` and `compute_zoning_distribution` use this guard to
/// short-circuit before any polygon intersection work.
fn bbox_area(bbox_rect: &Rect<f64>) -> Option<f64> {
    let area = bbox_rect.unsigned_area();
    if area > 0.0 { Some(area) } else { None }
}

/// Compute the ratio of feature geometry area intersecting `bbox_rect` to the bbox area.
///
/// Returns a value in `[0.0, 1.0]`. Returns `0.0` if `stats_data` is not
/// `AreaPolygons` or if no geometries are present.
pub(crate) fn compute_area_ratio(
    bbox_rect: &Rect<f64>,
    stats_data: &LayerStatsData,
    indices: &[u32],
) -> f64 {
    let LayerStatsData::AreaPolygons(geoms) = stats_data else {
        return 0.0;
    };

    let Some(bbox_area) = bbox_area(bbox_rect) else {
        return 0.0;
    };

    let bbox_polygon = rect_to_polygon(bbox_rect);

    let mut total_intersection_area: f64 = 0.0;
    for &idx in indices {
        let Some(maybe_geom) = geoms.get(idx as usize) else {
            continue;
        };
        let Some(geom) = maybe_geom else {
            continue;
        };

        let intersection_area = geometry_intersection_area(geom, &bbox_polygon);
        total_intersection_area += intersection_area;
    }

    (total_intersection_area / bbox_area).clamp(0.0, 1.0)
}

/// Compute the zoning distribution within `bbox_rect` for the given indices.
///
/// Returns a vector of `(zone_type, ratio)` pairs sorted descending by ratio,
/// where ratio is the fraction of the bbox area covered by that zone type.
///
/// Returns an empty vector if `stats_data` is not `ZoningPolygons`.
pub(crate) fn compute_zoning_distribution(
    bbox_rect: &Rect<f64>,
    stats_data: &LayerStatsData,
    indices: &[u32],
) -> Vec<(String, f64)> {
    let LayerStatsData::ZoningPolygons(pairs) = stats_data else {
        return Vec::new();
    };

    let Some(bbox_area) = bbox_area(bbox_rect) else {
        return Vec::new();
    };

    let bbox_polygon = rect_to_polygon(bbox_rect);

    // Accumulate intersection area per zone type.
    let mut zone_areas: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for &idx in indices {
        let Some((zone_type, maybe_geom)) = pairs.get(idx as usize) else {
            continue;
        };
        if zone_type.is_empty() {
            continue;
        }
        let Some(geom) = maybe_geom else {
            continue;
        };

        let area = geometry_intersection_area(geom, &bbox_polygon);
        *zone_areas.entry(zone_type.clone()).or_insert(0.0) += area;
    }

    let mut result: Vec<(String, f64)> = zone_areas
        .into_iter()
        .map(|(zone, area)| (zone, (area / bbox_area).clamp(0.0, 1.0)))
        .collect();

    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// Convert a `geo::Rect` to a `geo::Polygon` for use with `BooleanOps`.
fn rect_to_polygon(rect: &Rect<f64>) -> Polygon<f64> {
    let min_x = rect.min().x;
    let min_y = rect.min().y;
    let max_x = rect.max().x;
    let max_y = rect.max().y;

    Polygon::new(
        geo::LineString::new(vec![
            Coord { x: min_x, y: min_y },
            Coord { x: max_x, y: min_y },
            Coord { x: max_x, y: max_y },
            Coord { x: min_x, y: max_y },
            Coord { x: min_x, y: min_y },
        ]),
        vec![],
    )
}

/// Compute the unsigned area of the intersection between a `geo::Geometry`
/// and a bounding polygon.
///
/// Returns `0.0` if the geometry type is unsupported or the intersection is empty.
fn geometry_intersection_area(geom: &geo::Geometry<f64>, bbox_polygon: &Polygon<f64>) -> f64 {
    match geom {
        geo::Geometry::Polygon(poly) => poly.intersection(bbox_polygon).unsigned_area(),
        geo::Geometry::MultiPolygon(multi) => {
            multi.unsigned_area().min({
                // Compute intersection for each sub-polygon and sum
                let mut total = 0.0;
                for poly in &multi.0 {
                    total += poly.intersection(bbox_polygon).unsigned_area();
                }
                total
            })
        }
        // Points and lines have zero area; other geometry types are unsupported.
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use geo::{Coord, LineString, Polygon, Rect};

    use super::*;
    use crate::spatial_index::LayerStatsData;

    fn simple_bbox() -> Rect<f64> {
        Rect::new(Coord { x: 139.7, y: 35.6 }, Coord { x: 139.8, y: 35.7 })
    }

    // -----------------------------------------------------------------------
    // Land price stats
    // -----------------------------------------------------------------------

    #[test]
    fn test_land_price_stats_computation_basic() {
        let prices = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let data = LayerStatsData::PricePoints(prices);
        let indices: Vec<u32> = (0..5).collect();
        let stats = compute_land_price_stats(&data, &indices);

        assert_eq!(stats.count, 5);
        let avg = stats
            .avg_per_sqm
            .expect("avg should be Some for non-empty data");
        assert!((avg - 300.0).abs() < 1e-9, "avg should be 300");
        let median = stats
            .median_per_sqm
            .expect("median should be Some for non-empty data");
        assert!((median - 300.0).abs() < 1e-9, "median should be 300");
        assert_eq!(stats.min_per_sqm, Some(100));
        assert_eq!(stats.max_per_sqm, Some(500));
    }

    #[test]
    fn test_land_price_stats_even_count_median() {
        let prices = vec![100.0, 200.0, 300.0, 400.0];
        let data = LayerStatsData::PricePoints(prices);
        let indices: Vec<u32> = (0..4).collect();
        let stats = compute_land_price_stats(&data, &indices);

        assert_eq!(stats.count, 4);
        // Median of [100, 200, 300, 400] = (200 + 300) / 2 = 250
        let median = stats
            .median_per_sqm
            .expect("median should be Some for non-empty data");
        assert!((median - 250.0).abs() < 1e-9, "median should be 250");
    }

    #[test]
    fn test_land_price_stats_filters_zero_prices() {
        // Index 1 has price 0.0 (missing data), should be filtered out
        let prices = vec![100.0, 0.0, 300.0];
        let data = LayerStatsData::PricePoints(prices);
        let indices: Vec<u32> = (0..3).collect();
        let stats = compute_land_price_stats(&data, &indices);

        assert_eq!(stats.count, 2, "zero-price features should be excluded");
        assert_eq!(stats.min_per_sqm, Some(100));
        assert_eq!(stats.max_per_sqm, Some(300));
    }

    #[test]
    fn test_land_price_stats_empty_returns_none() {
        let data = LayerStatsData::PricePoints(vec![]);
        let stats = compute_land_price_stats(&data, &[]);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.avg_per_sqm, None);
        assert_eq!(stats.median_per_sqm, None);
        assert_eq!(stats.min_per_sqm, None);
        assert_eq!(stats.max_per_sqm, None);
    }

    #[test]
    fn test_land_price_stats_non_price_data_returns_none() {
        let data = LayerStatsData::None;
        let stats = compute_land_price_stats(&data, &[0, 1]);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.avg_per_sqm, None);
        assert_eq!(stats.median_per_sqm, None);
        assert_eq!(stats.min_per_sqm, None);
        assert_eq!(stats.max_per_sqm, None);
    }

    // -----------------------------------------------------------------------
    // Area ratio
    // -----------------------------------------------------------------------

    #[test]
    fn test_area_ratio_polygon_covering_half_bbox() {
        // bbox: x=[139.7, 139.8], y=[35.6, 35.7]
        // polygon covers left half: x=[139.7, 139.75]
        let bbox = simple_bbox();
        let half_poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 139.7, y: 35.6 },
                Coord { x: 139.75, y: 35.6 },
                Coord { x: 139.75, y: 35.7 },
                Coord { x: 139.7, y: 35.7 },
                Coord { x: 139.7, y: 35.6 },
            ]),
            vec![],
        );

        let data = LayerStatsData::AreaPolygons(vec![Some(geo::Geometry::Polygon(half_poly))]);
        let ratio = compute_area_ratio(&bbox, &data, &[0]);

        assert!(
            (ratio - 0.5).abs() < 0.01,
            "polygon covering half bbox should give ratio ~0.5, got {ratio}"
        );
    }

    #[test]
    fn test_area_ratio_is_clamped_to_one() {
        // polygon larger than bbox: ratio should be clamped to 1.0
        let bbox = simple_bbox();
        let large_poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 139.0, y: 35.0 },
                Coord { x: 140.0, y: 35.0 },
                Coord { x: 140.0, y: 36.0 },
                Coord { x: 139.0, y: 36.0 },
                Coord { x: 139.0, y: 35.0 },
            ]),
            vec![],
        );

        let data = LayerStatsData::AreaPolygons(vec![Some(geo::Geometry::Polygon(large_poly))]);
        let ratio = compute_area_ratio(&bbox, &data, &[0]);

        assert!(ratio <= 1.0, "ratio should be clamped to [0,1]");
        assert!(ratio >= 0.0);
        assert!(
            (ratio - 1.0).abs() < 0.01,
            "full coverage should give ratio ~1.0, got {ratio}"
        );
    }

    #[test]
    fn test_area_ratio_non_overlapping_polygon_returns_zero() {
        let bbox = simple_bbox();
        // polygon in London — no overlap
        let london_poly = Polygon::new(
            LineString::new(vec![
                Coord { x: -0.2, y: 51.4 },
                Coord { x: 0.0, y: 51.4 },
                Coord { x: 0.0, y: 51.6 },
                Coord { x: -0.2, y: 51.6 },
                Coord { x: -0.2, y: 51.4 },
            ]),
            vec![],
        );

        let data = LayerStatsData::AreaPolygons(vec![Some(geo::Geometry::Polygon(london_poly))]);
        let ratio = compute_area_ratio(&bbox, &data, &[0]);
        assert!(
            ratio < 1e-9,
            "non-overlapping polygon should give ratio 0.0, got {ratio}"
        );
    }

    #[test]
    fn test_area_ratio_no_geometry_returns_zero() {
        let bbox = simple_bbox();
        let data = LayerStatsData::AreaPolygons(vec![None]);
        let ratio = compute_area_ratio(&bbox, &data, &[0]);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_area_ratio_wrong_data_type_returns_zero() {
        let bbox = simple_bbox();
        let data = LayerStatsData::None;
        let ratio = compute_area_ratio(&bbox, &data, &[0]);
        assert_eq!(ratio, 0.0);
    }

    // -----------------------------------------------------------------------
    // Zoning distribution
    // -----------------------------------------------------------------------

    #[test]
    fn test_zoning_distribution_ratios_sum_lte_one() {
        let bbox = simple_bbox();

        // Two non-overlapping polygons each covering 30% of bbox
        let poly_a = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 139.70,
                    y: 35.60,
                },
                Coord {
                    x: 139.73,
                    y: 35.60,
                },
                Coord {
                    x: 139.73,
                    y: 35.70,
                },
                Coord {
                    x: 139.70,
                    y: 35.70,
                },
                Coord {
                    x: 139.70,
                    y: 35.60,
                },
            ]),
            vec![],
        );
        let poly_b = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 139.73,
                    y: 35.60,
                },
                Coord {
                    x: 139.76,
                    y: 35.60,
                },
                Coord {
                    x: 139.76,
                    y: 35.70,
                },
                Coord {
                    x: 139.73,
                    y: 35.70,
                },
                Coord {
                    x: 139.73,
                    y: 35.60,
                },
            ]),
            vec![],
        );

        let data = LayerStatsData::ZoningPolygons(vec![
            ("商業地域".to_string(), Some(geo::Geometry::Polygon(poly_a))),
            ("住居地域".to_string(), Some(geo::Geometry::Polygon(poly_b))),
        ]);

        let dist = compute_zoning_distribution(&bbox, &data, &[0, 1]);

        let total: f64 = dist.iter().map(|(_, r)| r).sum();
        assert!(
            total <= 1.0 + 1e-9,
            "zoning ratios should sum to <= 1.0, got {total}"
        );
        assert!(dist.len() == 2, "should have 2 zone types");
    }

    #[test]
    fn test_zoning_distribution_sorted_descending() {
        let bbox = simple_bbox();

        // poly_a is larger (covers full bbox), poly_b is smaller
        let poly_large = Polygon::new(
            LineString::new(vec![
                Coord { x: 139.7, y: 35.6 },
                Coord { x: 139.8, y: 35.6 },
                Coord { x: 139.8, y: 35.7 },
                Coord { x: 139.7, y: 35.7 },
                Coord { x: 139.7, y: 35.6 },
            ]),
            vec![],
        );
        let poly_small = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 139.70,
                    y: 35.60,
                },
                Coord {
                    x: 139.71,
                    y: 35.60,
                },
                Coord {
                    x: 139.71,
                    y: 35.61,
                },
                Coord {
                    x: 139.70,
                    y: 35.61,
                },
                Coord {
                    x: 139.70,
                    y: 35.60,
                },
            ]),
            vec![],
        );

        let data = LayerStatsData::ZoningPolygons(vec![
            (
                "小さいゾーン".to_string(),
                Some(geo::Geometry::Polygon(poly_small)),
            ),
            (
                "大きいゾーン".to_string(),
                Some(geo::Geometry::Polygon(poly_large)),
            ),
        ]);

        let dist = compute_zoning_distribution(&bbox, &data, &[0, 1]);

        assert!(!dist.is_empty(), "distribution should not be empty");
        // 大きいゾーン should come first (higher ratio)
        assert_eq!(dist[0].0, "大きいゾーン", "larger zone should be first");
    }

    #[test]
    fn test_zoning_distribution_empty_returns_empty() {
        let bbox = simple_bbox();
        let data = LayerStatsData::ZoningPolygons(vec![]);
        let dist = compute_zoning_distribution(&bbox, &data, &[]);
        assert!(dist.is_empty());
    }

    #[test]
    fn test_zoning_distribution_wrong_data_type_returns_empty() {
        let bbox = simple_bbox();
        let data = LayerStatsData::None;
        let dist = compute_zoning_distribution(&bbox, &data, &[0]);
        assert!(dist.is_empty());
    }
}
