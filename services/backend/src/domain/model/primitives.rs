//! Primitive domain value types: string newtypes, numeric newtypes, and
//! geographic/temporal scalars.
//!
//! Every type in this module enforces its invariants at construction time so
//! that callers holding one of these values never need to re-validate it.
//! See the individual constructors for the exact rules.

use crate::domain::constants::{
    BBOX_MAX_SIDE_DEG, BCR_MAX, BCR_MIN, CITY_CODE_LEN, FAR_MAX, FAR_MIN, LAT_MAX, LNG_MAX,
    PREF_CODE_LEN, PREF_CODE_MAX, PREF_CODE_MIN, YEAR_MAX, YEAR_MIN,
};
use crate::domain::error::DomainError;

/// Human-readable area name (e.g. "新宿区", "Shinjuku").
///
/// Whitespace-trimmed and non-empty by construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AreaName(String);

impl AreaName {
    /// Parse a string into a validated, whitespace-trimmed area name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if the input is empty after trimming.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "area name must be non-empty".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrows the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AreaName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Postal or street address, trimmed and non-empty by construction.
///
/// Used as the human-readable label for land price observation points.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
    /// Parse a string into a validated, whitespace-trimmed address.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if the input is empty after trimming.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation("address must be non-empty".into()));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrows the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Urban-planning zone code (用途地域コード), e.g. `"商業地域"`.
///
/// Trimmed and non-empty by construction. Code set defined by MLIT.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneCode(String);

impl ZoneCode {
    /// Parse a string into a validated, whitespace-trimmed zone code.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if the input is empty after trimming.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "zone code must be non-empty".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrows the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ZoneCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Land price per square meter, stored in JPY (integer yen).
///
/// Rejects negative values via [`DomainError::Validation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PricePerSqm(i64);

impl PricePerSqm {
    /// Construct a [`PricePerSqm`] from a raw integer yen value.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::PricePerSqm;
    /// let price = PricePerSqm::new(1_500_000)?;
    /// assert_eq!(price.value(), 1_500_000);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::Validation(format!(
                "price_per_sqm must be non-negative, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer yen value.
    pub fn value(self) -> i64 {
        self.0
    }
}

/// Building Coverage Ratio (建蔽率) as an integer percentage in `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingCoverageRatio(i32);

impl BuildingCoverageRatio {
    /// Construct a [`BuildingCoverageRatio`] from a percentage integer.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is outside `[BCR_MIN, BCR_MAX]`
    /// (currently `0..=100`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::BuildingCoverageRatio;
    /// let bcr = BuildingCoverageRatio::new(60)?;
    /// assert_eq!(bcr.value(), 60);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(BCR_MIN..=BCR_MAX).contains(&value) {
            return Err(DomainError::Validation(format!(
                "building coverage ratio must be in {BCR_MIN}..={BCR_MAX}, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer percentage.
    pub fn value(self) -> i32 {
        self.0
    }
}

/// Floor Area Ratio (容積率) as an integer percentage in `0..=2000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloorAreaRatio(i32);

impl FloorAreaRatio {
    /// Construct a [`FloorAreaRatio`] from a percentage integer.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is outside `[FAR_MIN, FAR_MAX]`
    /// (currently `0..=2000`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::FloorAreaRatio;
    /// let far = FloorAreaRatio::new(400)?;
    /// assert_eq!(far.value(), 400);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(FAR_MIN..=FAR_MAX).contains(&value) {
            return Err(DomainError::Validation(format!(
                "floor area ratio must be in {FAR_MIN}..={FAR_MAX}, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer percentage.
    pub fn value(self) -> i32 {
        self.0
    }
}

/// Distance in meters (non-negative by construction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Meters(u32);

impl Meters {
    /// Wrap a non-negative meter distance.
    ///
    /// Construction is infallible because `u32` cannot represent negative values.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the raw meter count.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// Percentage value as an `f64` (domain convention: not clamped to `0..=100`;
/// negative values represent e.g. year-over-year decreases).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(f64);

impl Percent {
    /// The zero percentage (no change).
    pub const fn zero() -> Self {
        Self(0.0)
    }

    /// Wrap any `f64` as a percentage.
    ///
    /// Negative values represent decreases (e.g. `-3.5` means −3.5% year-over-year).
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Return the raw `f64` percentage value.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Record count, clamped to `>= 0` at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordCount(i64);

impl RecordCount {
    /// Wrap a count, clamping negative inputs to zero.
    ///
    /// Database `COUNT(*)` calls may return `NULL` which SQLx maps to `0i64`,
    /// but defensive callers can also pass negative raw values safely.
    pub fn new(value: i64) -> Self {
        Self(value.max(0))
    }

    /// Return the non-negative record count.
    pub fn value(self) -> i64 {
        self.0
    }
}

/// Bounding box with enforced invariants:
/// - `south < north`, `west < east`
/// - Each side ≤ 0.5°
/// - Latitude ∈ [-90, 90], Longitude ∈ [-180, 180]
///
/// Fields are private; only the validated constructor can create instances.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    south: f64,
    west: f64,
    north: f64,
    east: f64,
}

impl BBox {
    /// Construct a validated [`BBox`] from the four boundary coordinates.
    ///
    /// Coordinate order matches the PostGIS `ST_MakeEnvelope(west, south, east, north)`
    /// convention used internally.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCoordinate`] if any latitude is outside
    /// `[-90, 90]` or any longitude is outside `[-180, 180]`, or if
    /// `south >= north` or `west >= east`.
    ///
    /// Returns [`DomainError::BBoxTooLarge`] if either side exceeds
    /// `BBOX_MAX_SIDE_DEG`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::BBox;
    /// let bbox = BBox::new(35.65, 139.70, 35.70, 139.80)?;
    /// assert!((bbox.south() - 35.65).abs() < f64::EPSILON);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
            return Err(DomainError::BBoxTooLarge {
                max_deg: BBOX_MAX_SIDE_DEG,
            });
        }
        Ok(Self {
            south,
            west,
            north,
            east,
        })
    }

    /// Parse a bounding box from a comma-separated `sw_lng,sw_lat,ne_lng,ne_lat`
    /// query string (longitude-first per RFC 7946).
    ///
    /// Validates the component count, parses each as `f64`, and delegates to
    /// [`BBox::new`] for invariant checks.
    pub fn parse_sw_ne_str(s: &str) -> Result<Self, DomainError> {
        let parts: Vec<f64> = s
            .split(',')
            .map(|p| {
                p.trim().parse::<f64>().map_err(|_| {
                    DomainError::Validation("bbox contains a non-numeric component".into())
                })
            })
            .collect::<Result<_, _>>()?;
        match parts.as_slice() {
            [sw_lng, sw_lat, ne_lng, ne_lat] => Self::new(*sw_lat, *sw_lng, *ne_lat, *ne_lng),
            _ => Err(DomainError::Validation(
                "bbox must have 4 comma-separated values: sw_lng,sw_lat,ne_lng,ne_lat".into(),
            )),
        }
    }

    /// Southern latitude boundary (WGS-84 degrees).
    pub fn south(&self) -> f64 {
        self.south
    }
    /// Western longitude boundary (WGS-84 degrees).
    pub fn west(&self) -> f64 {
        self.west
    }
    /// Northern latitude boundary (WGS-84 degrees).
    pub fn north(&self) -> f64 {
        self.north
    }
    /// Eastern longitude boundary (WGS-84 degrees).
    pub fn east(&self) -> f64 {
        self.east
    }
}

/// Geographic coordinate with enforced invariants:
/// - Latitude ∈ [-90, 90], Longitude ∈ [-180, 180]
#[derive(Debug, Clone, Copy)]
pub struct Coord {
    lat: f64,
    lng: f64,
}

impl Coord {
    /// Construct a validated coordinate from latitude and longitude.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCoordinate`] if `lat` is outside `[-90, 90]`
    /// or `lng` is outside `[-180, 180]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::Coord;
    /// let coord = Coord::new(35.689_487, 139.691_706)?; // Tokyo Station
    /// assert!((coord.lat() - 35.689_487).abs() < 1e-6);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// WGS-84 latitude in decimal degrees.
    pub fn lat(&self) -> f64 {
        self.lat
    }
    /// WGS-84 longitude in decimal degrees.
    pub fn lng(&self) -> f64 {
        self.lng
    }
}

/// Calendar year for land price data queries.
///
/// Fields are private; only the validated constructor can create instances.
///
/// # Examples
///
/// ```
/// use terrasight_api::domain::model::Year;
///
/// let year = Year::new(2023).unwrap();
/// assert_eq!(year.value(), 2023);
/// assert!(Year::new(1999).is_err());
/// assert!(Year::new(2101).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Year {
    value: i32,
}

impl Year {
    /// Construct a validated `Year`.
    ///
    /// Returns [`DomainError::InvalidYear`] if `value` is outside
    /// `[YEAR_MIN, YEAR_MAX]` (currently 2000–2100).
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(YEAR_MIN..=YEAR_MAX).contains(&value) {
            return Err(DomainError::InvalidYear(format!(
                "year must be between {YEAR_MIN} and {YEAR_MAX}, got {value}"
            )));
        }
        Ok(Self { value })
    }

    /// Return the raw year integer.
    pub fn value(self) -> i32 {
        self.value
    }
}

/// Map zoom level clamped to the MapLibre valid range `[0, 22]`.
///
/// `ZoomLevel::DEFAULT` (= 14, street level) is the fallback when the client
/// omits the parameter. Stored as `u8` for compactness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoomLevel(u8);

impl ZoomLevel {
    /// Default zoom level (street level).
    pub const DEFAULT: Self = Self(14);
    /// Minimum valid zoom level (entire world).
    pub const MIN: u8 = 0;
    /// Maximum valid zoom level (maximum MapLibre detail).
    pub const MAX: u8 = 22;

    /// Clamp a raw `u32` zoom value into `[MIN, MAX]`.
    ///
    /// This constructor is infallible — out-of-range values are saturated to
    /// the nearest bound. Callers never see a `Result`.
    pub fn clamped(value: u32) -> Self {
        Self(value.clamp(Self::MIN as u32, Self::MAX as u32) as u8)
    }

    /// Return the zoom level as a `u32` for use with legacy APIs.
    pub fn get(self) -> u32 {
        self.0 as u32
    }
}

/// Prefecture code: 2-digit zero-padded string ("01"–"47").
///
/// Validated at construction to guarantee the code represents one of
/// Japan's 47 prefectures.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefCode(String);

impl PrefCode {
    /// Parse a 2-digit prefecture code string.
    ///
    /// Trims surrounding whitespace before validation so query-string values
    /// like `" 13 "` are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidPrefCode`] if the input is not exactly
    /// 2 ASCII digits representing a prefecture in the range `01`–`47`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::PrefCode;
    /// let code = PrefCode::new("13")?; // Tokyo
    /// assert_eq!(code.as_str(), "13");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: &str) -> Result<Self, DomainError> {
        let code = code.trim();
        if code.len() == PREF_CODE_LEN && code.chars().all(|c| c.is_ascii_digit()) {
            let num: u8 = code
                .parse()
                .map_err(|_| DomainError::InvalidPrefCode(code.to_string()))?;
            if (PREF_CODE_MIN..=PREF_CODE_MAX).contains(&num) {
                return Ok(Self(code.to_string()));
            }
        }
        Err(DomainError::InvalidPrefCode(code.to_string()))
    }

    /// Borrow the inner 2-digit code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Administrative area code.
///
/// Accepts a 2-digit prefecture code (e.g. "13" for Tokyo) or a 5-digit
/// municipality code (e.g. "13104" for Shinjuku). Validated at construction
/// to guarantee digits-only input of the expected length.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AreaCode(String);

/// Granularity of an [`AreaCode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AreaCodeLevel {
    /// 2-digit prefecture code.
    Prefecture,
    /// 5-digit municipality code.
    Municipality,
}

impl AreaCode {
    /// Parse a raw string into a validated `AreaCode`.
    ///
    /// Returns [`DomainError::Validation`] when the input is not a
    /// 2- or 5-digit string of ASCII digits.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        if !matches!(s.len(), PREF_CODE_LEN | CITY_CODE_LEN)
            || !s.chars().all(|c| c.is_ascii_digit())
        {
            return Err(DomainError::Validation(format!(
                "area code must be 2 or 5 ASCII digits, got {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }

    /// Return the granularity of this area code.
    pub fn level(&self) -> AreaCodeLevel {
        match self.0.len() {
            PREF_CODE_LEN => AreaCodeLevel::Prefecture,
            CITY_CODE_LEN => AreaCodeLevel::Municipality,
            // SAFETY: `parse` enforces the length invariant.
            _ => unreachable!("AreaCode length invariant violated"),
        }
    }

    /// Borrow the inner digit string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// JIS X 0402 市区町村コード（5桁）。
///
/// Invariants:
/// - 5桁の ASCII 数字
/// - 上位2桁は有効な都道府県コード (01–47)
#[derive(Debug, Clone)]
pub struct CityCode(String);

impl CityCode {
    /// Parse a 5-digit JIS X 0402 municipality code.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCityCode`] if the input is not exactly 5
    /// ASCII digits or if the leading 2-digit prefecture component is outside
    /// the valid range `01`–`47`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::model::CityCode;
    /// let code = CityCode::new("13101")?; // 千代田区
    /// assert_eq!(code.pref_code(), "13");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: &str) -> Result<Self, DomainError> {
        let code = code.trim();
        if code.len() != CITY_CODE_LEN || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::InvalidCityCode(code.to_string()));
        }
        let pref: u8 = code[..2]
            .parse()
            .map_err(|_| DomainError::InvalidCityCode(code.to_string()))?;
        if !(PREF_CODE_MIN..=PREF_CODE_MAX).contains(&pref) {
            return Err(DomainError::InvalidCityCode(code.to_string()));
        }
        Ok(Self(code.to_string()))
    }

    /// Borrow the inner 5-digit code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the 2-digit prefecture prefix of this municipality code.
    ///
    /// For example, `"13101".pref_code()` returns `"13"` (Tokyo).
    pub fn pref_code(&self) -> &str {
        &self.0[..2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_per_sqm_rejects_negative() {
        assert!(PricePerSqm::new(-1).is_err());
        assert_eq!(PricePerSqm::new(0).unwrap().value(), 0);
        assert_eq!(PricePerSqm::new(1_500_000).unwrap().value(), 1_500_000);
    }

    #[test]
    fn bcr_and_far_bounds() {
        assert!(BuildingCoverageRatio::new(-1).is_err());
        assert!(BuildingCoverageRatio::new(101).is_err());
        assert_eq!(BuildingCoverageRatio::new(60).unwrap().value(), 60);
        assert!(FloorAreaRatio::new(-1).is_err());
        assert!(FloorAreaRatio::new(2001).is_err());
        assert_eq!(FloorAreaRatio::new(400).unwrap().value(), 400);
    }

    #[test]
    fn meters_and_percent_constructors() {
        assert_eq!(Meters::new(500).value(), 500);
        assert_eq!(Percent::zero().value(), 0.0);
        assert_eq!(Percent::new(-5.2).value(), -5.2);
    }

    #[test]
    fn zone_code_rejects_empty() {
        assert!(ZoneCode::parse("").is_err());
        assert!(ZoneCode::parse("   ").is_err());
        assert_eq!(ZoneCode::parse(" Y1 ").unwrap().as_str(), "Y1");
    }

    #[test]
    fn record_count_clamps_negative_to_zero() {
        assert_eq!(RecordCount::new(-5).value(), 0);
        assert_eq!(RecordCount::new(0).value(), 0);
        assert_eq!(RecordCount::new(42).value(), 42);
    }

    #[test]
    fn area_name_accepts_nonempty_and_trims() {
        let n = AreaName::parse("  Shinjuku  ").unwrap();
        assert_eq!(n.as_str(), "Shinjuku");
    }

    #[test]
    fn area_name_rejects_empty_and_whitespace() {
        assert!(AreaName::parse("").is_err());
        assert!(AreaName::parse("   ").is_err());
        assert!(AreaName::parse("\t\n").is_err());
    }

    #[test]
    fn address_accepts_nonempty_and_trims() {
        let a = Address::parse("1-1 Shinjuku").unwrap();
        assert_eq!(a.as_str(), "1-1 Shinjuku");
    }

    #[test]
    fn address_rejects_empty() {
        assert!(Address::parse("").is_err());
        assert!(Address::parse("   ").is_err());
    }

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
    fn bbox_parse_sw_ne_str_happy_path() {
        let bbox = BBox::parse_sw_ne_str("139.70,35.65,139.80,35.70").unwrap();
        assert!((bbox.south() - 35.65).abs() < f64::EPSILON);
        assert!((bbox.west() - 139.70).abs() < f64::EPSILON);
        assert!((bbox.north() - 35.70).abs() < f64::EPSILON);
        assert!((bbox.east() - 139.80).abs() < f64::EPSILON);
    }

    #[test]
    fn bbox_parse_sw_ne_str_errors() {
        assert!(BBox::parse_sw_ne_str("139.70,abc,139.80,35.70").is_err());
        assert!(BBox::parse_sw_ne_str("1,2,3").is_err());
        assert!(BBox::parse_sw_ne_str("1,2,3,4,5").is_err());
        assert!(BBox::parse_sw_ne_str("200.0,35.65,201.0,35.70").is_err());
        assert!(BBox::parse_sw_ne_str("139.70,95.0,139.80,96.0").is_err());
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
    fn year_rejects_too_low() {
        assert!(Year::new(1999).is_err());
        assert!(Year::new(0).is_err());
    }

    #[test]
    fn year_rejects_too_high() {
        assert!(Year::new(2101).is_err());
        assert!(Year::new(9999).is_err());
    }

    #[test]
    fn year_accepts_valid() {
        let y = Year::new(2023).expect("2023 is within valid range");
        assert_eq!(y.value(), 2023);
        assert!(Year::new(2000).is_ok());
        assert!(Year::new(2100).is_ok());
    }

    #[test]
    fn zoom_level_clamps_to_valid_range() {
        assert_eq!(ZoomLevel::clamped(0).get(), 0);
        assert_eq!(ZoomLevel::clamped(14).get(), 14);
        assert_eq!(ZoomLevel::clamped(22).get(), 22);
        assert_eq!(ZoomLevel::clamped(100).get(), 22);
        assert_eq!(ZoomLevel::DEFAULT.get(), 14);
    }

    #[test]
    fn pref_code_accepts_valid() {
        assert_eq!(PrefCode::new("01").unwrap().as_str(), "01");
        assert_eq!(PrefCode::new("13").unwrap().as_str(), "13");
        assert_eq!(PrefCode::new("47").unwrap().as_str(), "47");
    }

    #[test]
    fn pref_code_rejects_invalid() {
        assert!(PrefCode::new("00").is_err());
        assert!(PrefCode::new("48").is_err());
        assert!(PrefCode::new("1").is_err());
        assert!(PrefCode::new("abc").is_err());
        assert!(PrefCode::new("").is_err());
        assert!(PrefCode::new("001").is_err());
    }

    #[test]
    fn pref_code_trims_whitespace() {
        assert_eq!(PrefCode::new(" 13 ").unwrap().as_str(), "13");
    }

    #[test]
    fn city_code_valid() {
        assert!(CityCode::new("13101").is_ok());
        assert_eq!(CityCode::new("13101").unwrap().pref_code(), "13");
    }

    #[test]
    fn city_code_invalid_length() {
        assert!(CityCode::new("1310").is_err());
        assert!(CityCode::new("131010").is_err());
    }

    #[test]
    fn city_code_invalid_pref() {
        assert!(CityCode::new("00101").is_err());
        assert!(CityCode::new("48101").is_err());
        assert!(CityCode::new("99999").is_err());
    }

    #[test]
    fn area_code_accepts_prefecture_and_municipality() {
        let pref = AreaCode::parse("13").expect("2-digit prefecture");
        assert_eq!(pref.as_str(), "13");
        assert_eq!(pref.level(), AreaCodeLevel::Prefecture);

        let muni = AreaCode::parse("13104").expect("5-digit municipality");
        assert_eq!(muni.as_str(), "13104");
        assert_eq!(muni.level(), AreaCodeLevel::Municipality);
    }

    #[test]
    fn area_code_rejects_invalid_lengths_and_non_digits() {
        assert!(AreaCode::parse("1").is_err());
        assert!(AreaCode::parse("131").is_err());
        assert!(AreaCode::parse("131040").is_err());
        assert!(AreaCode::parse("abc").is_err());
        assert!(AreaCode::parse("13a04").is_err());
        assert!(AreaCode::parse("").is_err());
    }
}
