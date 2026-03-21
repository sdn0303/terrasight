/// Calculate bounding box area in square degrees (approximate).
///
/// Used for metrics tracking (`spatial.bbox.area_deg2`).
///
/// # Examples
///
/// ```
/// use realestate_geo_math::spatial::bbox_area_deg2;
///
/// let area = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
/// assert!((area - 0.005).abs() < 1e-9);
///
/// assert_eq!(bbox_area_deg2(0.0, 0.0, 1.0, 1.0), 1.0);
/// ```
pub fn bbox_area_deg2(south: f64, west: f64, north: f64, east: f64) -> f64 {
    (north - south).abs() * (east - west).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_tokyo_area() {
        let area = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
        assert!((area - 0.005).abs() < 1e-9, "expected ~0.005, got {area}");
    }

    #[test]
    fn bbox_unit_square() {
        assert_eq!(bbox_area_deg2(0.0, 0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn bbox_inverted_coordinates_still_positive() {
        // south/north and west/east swapped — abs() ensures positive area
        let normal = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
        let inverted = bbox_area_deg2(35.70, 139.80, 35.65, 139.70);
        assert!((normal - inverted).abs() < f64::EPSILON);
    }

    #[test]
    fn bbox_zero_area_point() {
        assert_eq!(bbox_area_deg2(35.0, 139.0, 35.0, 139.0), 0.0);
    }
}
