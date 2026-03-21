use crate::domain::constants::{BBOX_MAX_SIDE_DEG, LAT_MAX, LNG_MAX};
use crate::domain::error::DomainError;

/// Bounding box with enforced invariants:
/// - `south < north`, `west < east`
/// - Each side ≤ 0.5°
/// - Latitude ∈ [-90, 90], Longitude ∈ [-180, 180]
///
/// Fields are private; only the validated constructor can create instances.
#[derive(Debug, Clone)]
pub struct BBox {
    south: f64,
    west: f64,
    north: f64,
    east: f64,
}

impl BBox {
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Result<Self, DomainError> {
        if !(-LAT_MAX..=LAT_MAX).contains(&south) || !(-LAT_MAX..=LAT_MAX).contains(&north) {
            return Err(DomainError::InvalidCoordinate(
                "latitude must be between -90 and 90".into(),
            ));
        }
        if !(-LNG_MAX..=LNG_MAX).contains(&west) || !(-LNG_MAX..=LNG_MAX).contains(&east) {
            return Err(DomainError::InvalidCoordinate(
                "longitude must be between -180 and 180".into(),
            ));
        }
        if south >= north {
            return Err(DomainError::InvalidCoordinate(
                "south must be less than north".into(),
            ));
        }
        if west >= east {
            return Err(DomainError::InvalidCoordinate(
                "west must be less than east".into(),
            ));
        }
        if (north - south) > BBOX_MAX_SIDE_DEG || (east - west) > BBOX_MAX_SIDE_DEG {
            return Err(DomainError::BBoxTooLarge);
        }
        Ok(Self {
            south,
            west,
            north,
            east,
        })
    }

    pub fn south(&self) -> f64 {
        self.south
    }
    pub fn west(&self) -> f64 {
        self.west
    }
    pub fn north(&self) -> f64 {
        self.north
    }
    pub fn east(&self) -> f64 {
        self.east
    }
}

/// Geographic coordinate with enforced invariants:
/// - Latitude ∈ [-90, 90], Longitude ∈ [-180, 180]
#[derive(Debug, Clone)]
pub struct Coord {
    lat: f64,
    lng: f64,
}

impl Coord {
    pub fn new(lat: f64, lng: f64) -> Result<Self, DomainError> {
        if !(-LAT_MAX..=LAT_MAX).contains(&lat) {
            return Err(DomainError::InvalidCoordinate(
                "latitude must be between -90 and 90".into(),
            ));
        }
        if !(-LNG_MAX..=LNG_MAX).contains(&lng) {
            return Err(DomainError::InvalidCoordinate(
                "longitude must be between -180 and 180".into(),
            ));
        }
        Ok(Self { lat, lng })
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }
    pub fn lng(&self) -> f64 {
        self.lng
    }
}

/// Map layer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    LandPrice,
    Zoning,
    Flood,
    SteepSlope,
    Schools,
    Medical,
}

impl LayerType {
    /// Parse from REST API query string value. Returns `None` for unknown layers.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "landprice" => Some(Self::LandPrice),
            "zoning" => Some(Self::Zoning),
            "flood" => Some(Self::Flood),
            "steep_slope" => Some(Self::SteepSlope),
            "schools" => Some(Self::Schools),
            "medical" => Some(Self::Medical),
            _ => None,
        }
    }

    /// REST API key string for JSON response keys.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LandPrice => "landprice",
            Self::Zoning => "zoning",
            Self::Flood => "flood",
            Self::SteepSlope => "steep_slope",
            Self::Schools => "schools",
            Self::Medical => "medical",
        }
    }
}

/// Investment score: 4 components each 0–25, total 0–100.
#[derive(Debug, Clone)]
pub struct InvestmentScore {
    pub trend: ScoreComponent,
    pub risk: ScoreComponent,
    pub access: ScoreComponent,
    pub yield_potential: ScoreComponent,
    pub data_freshness: String,
}

impl InvestmentScore {
    pub fn total(&self) -> f64 {
        self.trend.value + self.risk.value + self.access.value + self.yield_potential.value
    }
}

#[derive(Debug, Clone)]
pub struct ScoreComponent {
    pub value: f64,
    pub max: f64,
    pub detail: serde_json::Value,
}

/// Trend analysis result produced by the usecase layer.
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub location: crate::domain::entity::TrendLocation,
    pub data: Vec<crate::domain::entity::TrendPoint>,
    pub cagr: f64,
    pub direction: TrendDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Up,
    Down,
}

impl TrendDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_rejects_out_of_range_latitude() {
        assert!(BBox::new(91.0, 0.0, 92.0, 1.0).is_err());
    }

    #[test]
    fn bbox_rejects_south_gte_north() {
        assert!(BBox::new(35.7, 139.7, 35.6, 139.8).is_err());
    }

    #[test]
    fn bbox_rejects_too_large() {
        assert!(BBox::new(35.0, 139.0, 35.6, 139.6).is_err());
    }

    #[test]
    fn bbox_accepts_valid() {
        let bbox = BBox::new(35.65, 139.70, 35.70, 139.80).unwrap();
        assert!((bbox.south() - 35.65).abs() < f64::EPSILON);
    }

    #[test]
    fn coord_rejects_invalid() {
        assert!(Coord::new(91.0, 0.0).is_err());
        assert!(Coord::new(0.0, 181.0).is_err());
    }

    #[test]
    fn coord_accepts_valid() {
        let c = Coord::new(35.68, 139.76).unwrap();
        assert!((c.lat() - 35.68).abs() < f64::EPSILON);
    }

    #[test]
    fn layer_type_roundtrip() {
        for name in [
            "landprice",
            "zoning",
            "flood",
            "steep_slope",
            "schools",
            "medical",
        ] {
            let lt = LayerType::parse(name).unwrap();
            assert_eq!(lt.as_str(), name);
        }
        assert!(LayerType::parse("unknown").is_none());
    }
}
