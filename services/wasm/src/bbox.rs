//! Axis-aligned bounding box type with validated construction.
//!
//! [`BBox`] enforces WGS84 coordinate range constraints and requires
//! `south < north`, `west < east` at construction time, making invalid
//! bounding boxes unrepresentable after the constructor succeeds.

use crate::constants::{LAT_MAX, LAT_MIN, LAT_RANGE, LNG_MAX, LNG_MIN, LNG_RANGE};
use crate::error::WasmError;

/// Axis-aligned bounding box for spatial queries.
///
/// Invariants:
/// - `south < north`
/// - `west < east`
/// - Latitude ∈ [`LAT_MIN`, `LAT_MAX`]
/// - Longitude ∈ [`LNG_MIN`, `LNG_MAX`]
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    south: f64,
    west: f64,
    north: f64,
    east: f64,
}

impl BBox {
    /// Construct a validated [`BBox`] from four coordinate components.
    ///
    /// All four arguments are validated against WGS84 limits and the
    /// ordering invariants before the struct is created.
    ///
    /// # Examples
    ///
    /// Valid construction for a bbox over central Tokyo:
    ///
    /// ```
    /// # use terrasight_wasm::BBox;
    /// let bbox = BBox::new(35.5, 139.5, 35.8, 139.9)?;
    /// assert!((bbox.south() - 35.5).abs() < f64::EPSILON);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Invalid construction returns an error — south must be strictly less than north:
    ///
    /// ```
    /// # use terrasight_wasm::BBox;
    /// let result = BBox::new(35.8, 139.5, 35.5, 139.9); // south > north
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WasmError::InvalidBBox`] if:
    /// - `south >= north` or `west >= east`
    /// - Any latitude is outside `[-90.0, 90.0]`
    /// - Any longitude is outside `[-180.0, 180.0]`
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Result<Self, WasmError> {
        if south >= north || west >= east {
            return Err(WasmError::InvalidBBox(
                "south must be < north and west must be < east".into(),
            ));
        }
        if !LAT_RANGE.contains(&south) || !LAT_RANGE.contains(&north) {
            return Err(WasmError::InvalidBBox(format!(
                "latitude out of range [{LAT_MIN}, {LAT_MAX}]"
            )));
        }
        if !LNG_RANGE.contains(&west) || !LNG_RANGE.contains(&east) {
            return Err(WasmError::InvalidBBox(format!(
                "longitude out of range [{LNG_MIN}, {LNG_MAX}]"
            )));
        }
        Ok(Self {
            south,
            west,
            north,
            east,
        })
    }

    /// Returns the southern boundary (minimum latitude) in degrees.
    pub fn south(&self) -> f64 {
        self.south
    }

    /// Returns the western boundary (minimum longitude) in degrees.
    pub fn west(&self) -> f64 {
        self.west
    }

    /// Returns the northern boundary (maximum latitude) in degrees.
    pub fn north(&self) -> f64 {
        self.north
    }

    /// Returns the eastern boundary (maximum longitude) in degrees.
    pub fn east(&self) -> f64 {
        self.east
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bbox() {
        let bbox = BBox::new(35.5, 139.5, 35.8, 139.9);
        assert!(bbox.is_ok());
        let bbox = bbox.unwrap();
        assert!((bbox.south() - 35.5).abs() < f64::EPSILON);
    }

    #[test]
    fn south_gte_north_is_err() {
        assert!(BBox::new(35.8, 139.5, 35.5, 139.9).is_err());
    }

    #[test]
    fn west_gte_east_is_err() {
        assert!(BBox::new(35.5, 139.9, 35.8, 139.5).is_err());
    }

    #[test]
    fn latitude_out_of_range_is_err() {
        assert!(BBox::new(-91.0, 0.0, -90.0, 1.0).is_err());
        assert!(BBox::new(89.0, 0.0, 91.0, 1.0).is_err());
    }

    #[test]
    fn longitude_out_of_range_is_err() {
        assert!(BBox::new(0.0, -181.0, 1.0, 0.0).is_err());
        assert!(BBox::new(0.0, 0.0, 1.0, 181.0).is_err());
    }

    #[test]
    fn bbox_accessors_return_correct_values() {
        let bbox = BBox::new(35.5, 139.5, 35.8, 139.9).unwrap();
        assert!((bbox.south() - 35.5).abs() < f64::EPSILON);
        assert!((bbox.west() - 139.5).abs() < f64::EPSILON);
        assert!((bbox.north() - 35.8).abs() < f64::EPSILON);
        assert!((bbox.east() - 139.9).abs() < f64::EPSILON);
    }

    #[test]
    fn bbox_boundary_values_accepted() {
        assert!(BBox::new(-90.0, -180.0, 90.0, 180.0).is_ok());
        assert!(BBox::new(0.0, 0.0, 0.001, 0.001).is_ok());
    }

    #[test]
    fn bbox_equal_south_north_rejected() {
        assert!(BBox::new(35.5, 139.5, 35.5, 139.9).is_err());
    }
}
