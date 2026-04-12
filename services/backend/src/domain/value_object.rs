use crate::domain::constants::{
    BBOX_MAX_SIDE_DEG, CITY_CODE_LEN, LAT_MAX, LNG_MAX, PREF_CODE_LEN, PREF_CODE_MAX,
    PREF_CODE_MIN, YEAR_MAX, YEAR_MIN,
};
use crate::domain::error::DomainError;

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
#[derive(Debug, Clone, Copy)]
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

/// Calendar year for land price data queries.
///
/// Fields are private; only the validated constructor can create instances.
///
/// # Examples
///
/// ```
/// use realestate_api::domain::value_object::Year;
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

/// Trend lookback window in years.
///
/// Clamped to `[TREND_MIN_YEARS, TREND_MAX_YEARS]` via [`YearsLookback::clamped`].
/// `YearsLookback::DEFAULT` matches [`TREND_DEFAULT_YEARS`](crate::domain::constants::TREND_DEFAULT_YEARS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearsLookback(i32);

impl YearsLookback {
    /// Default lookback window.
    pub const DEFAULT: Self = Self(crate::domain::constants::TREND_DEFAULT_YEARS);

    /// Clamp a raw `i32` year count into `[TREND_MIN_YEARS, TREND_MAX_YEARS]`.
    pub fn clamped(value: i32) -> Self {
        Self(value.clamp(
            crate::domain::constants::TREND_MIN_YEARS,
            crate::domain::constants::TREND_MAX_YEARS,
        ))
    }

    /// Return the inner `i32` value.
    pub fn value(self) -> i32 {
        self.0
    }
}

/// TLS (Total Location Score) clamped to `0..=100` and stored as `u8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TlsScore(u8);

impl TlsScore {
    /// Construct a `TlsScore` from a raw `f64`, clamping to `[0, 100]`.
    ///
    /// `NaN` is mapped to `0` (infallible fallback for defensive callers).
    pub fn from_f64_clamped(value: f64) -> Self {
        if value.is_nan() {
            return Self(0);
        }
        Self(value.clamp(0.0, 100.0) as u8)
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

/// Risk level bucket derived from the S1 Disaster sub-score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Low,
    Mid,
    High,
}

impl RiskLevel {
    /// Parse the REST API query string value (`"low" | "mid" | "high"`).
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        match s {
            "low" => Ok(Self::Low),
            "mid" => Ok(Self::Mid),
            "high" => Ok(Self::High),
            other => Err(DomainError::Validation(format!(
                "risk_max must be one of low|mid|high, got {other:?}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Mid => "mid",
            Self::High => "high",
        }
    }

    /// Derive a `RiskLevel` from a raw S1 disaster sub-score (higher = safer).
    pub fn from_disaster_score(score: f64) -> Self {
        use crate::domain::scoring::constants::{
            DISASTER_SCORE_LOW_THRESHOLD, DISASTER_SCORE_MID_THRESHOLD,
        };
        if score >= DISASTER_SCORE_LOW_THRESHOLD {
            Self::Low
        } else if score >= DISASTER_SCORE_MID_THRESHOLD {
            Self::Mid
        } else {
            Self::High
        }
    }
}

/// Opportunity signal bucket: `Hot | Warm | Neutral | Cold`.
///
/// Derived from the combination of TLS score and risk level. See
/// [`OpportunitySignal::derive`] for the exact mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpportunitySignal {
    Hot,
    Warm,
    Neutral,
    Cold,
}

impl OpportunitySignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Warm => "warm",
            Self::Neutral => "neutral",
            Self::Cold => "cold",
        }
    }

    /// Derive a signal from a TLS score and risk level.
    ///
    /// High-risk locations are never classified as hotter than `Neutral`,
    /// regardless of TLS score.
    pub fn derive(tls: TlsScore, risk: RiskLevel) -> Self {
        use crate::domain::scoring::constants::{
            SIGNAL_HOT_MIN_TLS, SIGNAL_NEUTRAL_MIN_TLS, SIGNAL_WARM_MIN_TLS,
        };
        let score = tls.value();
        match (score, risk) {
            (s, RiskLevel::Low) if s >= SIGNAL_HOT_MIN_TLS => Self::Hot,
            (s, RiskLevel::Low | RiskLevel::Mid) if s >= SIGNAL_WARM_MIN_TLS => Self::Warm,
            (s, _) if s >= SIGNAL_NEUTRAL_MIN_TLS => Self::Neutral,
            _ => Self::Cold,
        }
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
    Prefecture,
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

/// Prefecture code: 2-digit zero-padded string ("01"–"47").
///
/// Validated at construction to guarantee the code represents one of
/// Japan's 47 prefectures.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefCode(String);

impl PrefCode {
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

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 上位2桁の都道府県コードを返す。
    pub fn pref_code(&self) -> &str {
        &self.0[..2]
    }
}

/// Clamped page-size parameter for the opportunities endpoint.
///
/// Enforces `1 <= value <= MAX_OPPORTUNITY_LIMIT` by clamping; construction is
/// infallible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpportunityLimit(u32);

impl OpportunityLimit {
    pub const MAX: u32 = crate::domain::constants::MAX_OPPORTUNITY_LIMIT;
    pub const DEFAULT: Self = Self(crate::domain::constants::DEFAULT_OPPORTUNITY_LIMIT);

    pub fn clamped(value: u32) -> Self {
        Self(value.clamp(1, Self::MAX))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

/// Offset parameter for paginated opportunities responses.
///
/// No upper bound — callers paginate until the response is shorter than
/// `OpportunityLimit::get()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpportunityOffset(u32);

impl OpportunityOffset {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
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
        // Out of range lat/lng surfaces as InvalidCoordinate
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
    fn years_lookback_clamps_to_valid_range() {
        use crate::domain::constants::{TREND_DEFAULT_YEARS, TREND_MAX_YEARS, TREND_MIN_YEARS};
        assert_eq!(YearsLookback::clamped(0).value(), TREND_MIN_YEARS);
        assert_eq!(YearsLookback::clamped(5).value(), 5);
        assert_eq!(YearsLookback::clamped(100).value(), TREND_MAX_YEARS);
        assert_eq!(YearsLookback::DEFAULT.value(), TREND_DEFAULT_YEARS);
    }

    #[test]
    fn opportunity_limit_clamps() {
        assert_eq!(OpportunityLimit::clamped(0).get(), 1);
        assert_eq!(OpportunityLimit::clamped(1).get(), 1);
        assert_eq!(OpportunityLimit::clamped(50).get(), 50);
        assert_eq!(OpportunityLimit::clamped(200).get(), 50);
        assert_eq!(OpportunityLimit::DEFAULT.get(), 50);
    }

    #[test]
    fn opportunity_offset_preserved() {
        assert_eq!(OpportunityOffset::new(0).get(), 0);
        assert_eq!(OpportunityOffset::new(100).get(), 100);
    }

    #[test]
    fn tls_score_clamps_and_handles_nan() {
        assert_eq!(TlsScore::from_f64_clamped(-10.0).value(), 0);
        assert_eq!(TlsScore::from_f64_clamped(0.0).value(), 0);
        assert_eq!(TlsScore::from_f64_clamped(50.7).value(), 50);
        assert_eq!(TlsScore::from_f64_clamped(100.0).value(), 100);
        assert_eq!(TlsScore::from_f64_clamped(150.0).value(), 100);
        assert_eq!(TlsScore::from_f64_clamped(f64::NAN).value(), 0);
    }

    #[test]
    fn risk_level_parse_and_display() {
        assert_eq!(RiskLevel::parse("low").unwrap(), RiskLevel::Low);
        assert_eq!(RiskLevel::parse("mid").unwrap(), RiskLevel::Mid);
        assert_eq!(RiskLevel::parse("high").unwrap(), RiskLevel::High);
        assert!(RiskLevel::parse("bad").is_err());
        assert_eq!(RiskLevel::Low.as_str(), "low");
        assert_eq!(RiskLevel::High.as_str(), "high");
    }

    #[test]
    fn risk_level_from_disaster_score() {
        assert_eq!(RiskLevel::from_disaster_score(80.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_disaster_score(75.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_disaster_score(60.0), RiskLevel::Mid);
        assert_eq!(RiskLevel::from_disaster_score(50.0), RiskLevel::Mid);
        assert_eq!(RiskLevel::from_disaster_score(30.0), RiskLevel::High);
    }

    #[test]
    fn opportunity_signal_derive_table() {
        let tls = |v: u8| TlsScore(v);
        // Hot: low risk + TLS ≥ 80
        assert_eq!(
            OpportunitySignal::derive(tls(85), RiskLevel::Low),
            OpportunitySignal::Hot
        );
        // High TLS but risk is high → Neutral
        assert_eq!(
            OpportunitySignal::derive(tls(90), RiskLevel::High),
            OpportunitySignal::Neutral
        );
        // Warm: low/mid risk + TLS ≥ 65
        assert_eq!(
            OpportunitySignal::derive(tls(70), RiskLevel::Low),
            OpportunitySignal::Warm
        );
        assert_eq!(
            OpportunitySignal::derive(tls(65), RiskLevel::Mid),
            OpportunitySignal::Warm
        );
        // Neutral: TLS ≥ 50 (any risk)
        assert_eq!(
            OpportunitySignal::derive(tls(55), RiskLevel::High),
            OpportunitySignal::Neutral
        );
        // Cold: TLS < 50
        assert_eq!(
            OpportunitySignal::derive(tls(40), RiskLevel::Low),
            OpportunitySignal::Cold
        );
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
