//! XYZ slippy map tile coordinate conversion utilities.
//!
//! Implements the standard Web Mercator tile math used by OpenStreetMap,
//! Google Maps, and MapLibre GL JS.
//!
//! # References
//!
//! - <https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames>
//! - <https://maps.gsi.go.jp/development/tileCoordCheck.html>

/// XYZ slippy map tile coordinate.
///
/// # Examples
///
/// ```
/// use terrasight_geo::tile::TileCoord;
/// let tile = TileCoord { z: 14, x: 14552, y: 6451 };
/// assert_eq!(tile.z, 14);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoord {
    /// Zoom level (typically 0–22).
    pub z: u8,
    /// Tile X coordinate (column, increases eastward).
    pub x: u32,
    /// Tile Y coordinate (row, increases southward).
    pub y: u32,
}

const WGS84_LNG_RANGE: f64 = 360.0;
const WGS84_LNG_OFFSET: f64 = 180.0;

/// Convert longitude (degrees) to tile X coordinate at zoom level `z`.
///
/// # Examples
///
/// ```
/// use terrasight_geo::tile::lng_to_tile_x;
/// // Tokyo Station longitude
/// let x = lng_to_tile_x(139.7671, 14);
/// assert_eq!(x, 14552);
/// ```
pub fn lng_to_tile_x(lng: f64, z: u8) -> u32 {
    let n = 2_f64.powi(i32::from(z));
    ((lng + WGS84_LNG_OFFSET) / WGS84_LNG_RANGE * n).floor() as u32
}

/// Convert latitude (degrees) to tile Y coordinate at zoom level `z`.
///
/// Y increases southward: north latitudes produce smaller Y values.
///
/// # Examples
///
/// ```
/// use terrasight_geo::tile::lat_to_tile_y;
/// // Tokyo Station latitude
/// let y = lat_to_tile_y(35.6812, 14);
/// assert_eq!(y, 6451);
/// ```
pub fn lat_to_tile_y(lat: f64, z: u8) -> u32 {
    let n = 2_f64.powi(i32::from(z));
    let lat_rad = lat.to_radians();
    ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n).floor()
        as u32
}

/// Return all tile coordinates that cover the given bounding box at zoom level `z`.
///
/// `west`, `south`, `east`, `north` are decimal degrees. Both tile endpoints are
/// inclusive, so a point bbox always returns exactly one tile.
///
/// # Example
///
/// ```
/// use terrasight_geo::tile::bbox_to_tiles;
///
/// // Small area around Tokyo Station — fits in a single tile at z=14
/// let tiles = bbox_to_tiles(139.766, 35.680, 139.768, 35.682, 14);
/// assert_eq!(tiles.len(), 1);
/// assert_eq!(tiles[0].z, 14);
/// ```
pub fn bbox_to_tiles(west: f64, south: f64, east: f64, north: f64, z: u8) -> Vec<TileCoord> {
    let x_min = lng_to_tile_x(west, z);
    let x_max = lng_to_tile_x(east, z);
    let y_min = lat_to_tile_y(north, z); // north has smaller y
    let y_max = lat_to_tile_y(south, z);

    let cols = (x_max.saturating_sub(x_min) + 1) as usize;
    let rows = (y_max.saturating_sub(y_min) + 1) as usize;
    let mut tiles = Vec::with_capacity(cols * rows);
    for x in x_min..=x_max {
        for y in y_min..=y_max {
            tiles.push(TileCoord { z, x, y });
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Single coordinate conversions
    // -----------------------------------------------------------------------

    /// Tokyo Station (35.6812°N, 139.7671°E) at z=14.
    /// Reference values computed via the standard Web Mercator formula.
    #[test]
    fn tokyo_station_tile_z14() {
        let x = lng_to_tile_x(139.7671, 14);
        let y = lat_to_tile_y(35.6812, 14);
        assert_eq!(x, 14552, "Tokyo Station tile X at z=14");
        assert_eq!(y, 6451, "Tokyo Station tile Y at z=14");
    }

    /// Zooming out 3 levels divides tile coords by 2^3 = 8.
    #[test]
    fn tokyo_station_tile_z11() {
        let x = lng_to_tile_x(139.7671, 11);
        let y = lat_to_tile_y(35.6812, 11);
        assert_eq!(x, 14552 / 8, "Tokyo Station tile X at z=11");
        assert_eq!(y, 6451 / 8, "Tokyo Station tile Y at z=11");
    }

    /// Equator / prime meridian at z=1: should be tile (1, 1).
    /// x = floor(180/360 * 2) = 1
    /// y = floor((1 - ln(tan(0) + 1/cos(0)) / π) / 2 * 2)
    ///   = floor((1 - 0) / 2 * 2) = 1
    #[test]
    fn equator_prime_meridian_z1() {
        let x = lng_to_tile_x(0.0, 1);
        let y = lat_to_tile_y(0.0, 1);
        assert_eq!(x, 1);
        assert_eq!(y, 1);
    }

    /// At z=0 there is only one tile (0, 0) covering the whole world.
    #[test]
    fn equator_prime_meridian_z0() {
        let x = lng_to_tile_x(0.0, 0);
        let y = lat_to_tile_y(0.0, 0);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }

    /// Negative longitude — New York City (40.7128°N, 74.0060°W).
    #[test]
    fn negative_longitude_nyc() {
        let x = lng_to_tile_x(-74.006, 14);
        let y = lat_to_tile_y(40.7128, 14);
        assert_eq!(x, 4823);
        // Y should be within valid range for z=14 (0..16384)
        assert!(y < 16384, "tile Y should be within valid range for z=14");
    }

    // -----------------------------------------------------------------------
    // bbox_to_tiles
    // -----------------------------------------------------------------------

    /// A very small bbox entirely within one tile at z=14 returns exactly 1 tile.
    #[test]
    fn bbox_single_tile_returns_one() {
        let tiles = bbox_to_tiles(139.766, 35.680, 139.768, 35.682, 14);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].z, 14);
    }

    /// A bbox whose west and east edges land on different tile columns returns
    /// tiles spanning both columns.
    #[test]
    fn bbox_spanning_two_columns() {
        // 139.760 and 139.780 are in different x tiles at z=14 (14552 and 14553)
        let tiles = bbox_to_tiles(139.760, 35.680, 139.780, 35.682, 14);
        let unique_x: std::collections::HashSet<u32> = tiles.iter().map(|t| t.x).collect();
        assert!(unique_x.len() >= 2, "bbox should span at least 2 x tiles");
        for tile in &tiles {
            assert_eq!(tile.z, 14);
        }
    }

    /// A bbox whose south and north edges land on different tile rows returns
    /// tiles spanning both rows.
    #[test]
    fn bbox_spanning_two_rows() {
        // 35.670 (south) and 35.690 (north) are in different y tiles at z=14
        let tiles = bbox_to_tiles(139.766, 35.670, 139.768, 35.690, 14);
        let unique_y: std::collections::HashSet<u32> = tiles.iter().map(|t| t.y).collect();
        assert!(unique_y.len() >= 2, "bbox should span at least 2 y tiles");
    }

    /// A wider bbox spanning multiple columns and rows at z=14.
    /// The result count equals unique_x * unique_y (rectangular grid).
    #[test]
    fn bbox_multiple_tiles_rectangular_grid() {
        // 139.755–139.790 spans 2 x-columns; 35.670–35.695 spans 3 y-rows at z=14
        let tiles = bbox_to_tiles(139.755, 35.670, 139.790, 35.695, 14);
        let unique_x: std::collections::HashSet<u32> = tiles.iter().map(|t| t.x).collect();
        let unique_y: std::collections::HashSet<u32> = tiles.iter().map(|t| t.y).collect();
        assert!(unique_x.len() >= 2, "should span multiple x tiles");
        assert!(unique_y.len() >= 2, "should span multiple y tiles");
        assert_eq!(
            tiles.len(),
            unique_x.len() * unique_y.len(),
            "tile count should equal columns * rows"
        );
    }

    /// Every tile returned has the zoom level that was requested.
    #[test]
    fn bbox_all_tiles_have_correct_zoom() {
        let z = 13_u8;
        let tiles = bbox_to_tiles(139.6, 35.5, 139.9, 35.8, z);
        assert!(!tiles.is_empty());
        for tile in &tiles {
            assert_eq!(tile.z, z);
        }
    }

    /// A point bbox (west == east, south == north) returns exactly 1 tile.
    #[test]
    fn point_bbox_returns_one_tile() {
        let tiles = bbox_to_tiles(139.7671, 35.6812, 139.7671, 35.6812, 14);
        assert_eq!(tiles.len(), 1);
    }
}
