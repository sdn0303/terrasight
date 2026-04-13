//! Request DTO and validated filters for `GET /api/v1/opportunities`.
//!
//! [`OpportunitiesQuery`] is the raw axum `Query` extractor type; it is
//! converted into [`OpportunitiesFilters`] — a strongly-typed filter set
//! that the usecase layer consumes directly — via `into_filters`.
//!
//! All runtime validation (bbox format, price range bounds, risk_max
//! parsing, zone CSV parsing) happens in `into_filters`; downstream code
//! never sees raw strings or ints.

use serde::Deserialize;

use crate::domain::constants::DEFAULT_OPPORTUNITY_LIMIT;
use crate::domain::entity::{Meters, PricePerSqm, ZoneCode};
use crate::domain::error::DomainError;
use crate::domain::scoring::tls::WeightPreset;
use crate::domain::value_object::{
    BBox, OpportunitiesFilters, OpportunityLimit, OpportunityOffset, PrefCode, RiskLevel, TlsScore,
};

/// Raw query string parameters for `GET /api/v1/opportunities`.
///
/// Fields map 1:1 to the frontend filter-store. `cities` and
/// `station_max` are accepted but currently logged as warnings (see
/// `warn_unsupported`) because they are not yet honoured by Phase 4.
#[derive(Debug, Clone, Deserialize)]
pub struct OpportunitiesQuery {
    pub bbox: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub tls_min: Option<u32>,
    #[serde(default)]
    pub risk_max: Option<String>,
    #[serde(default)]
    pub zones: Option<String>,
    #[serde(default)]
    pub station_max: Option<u32>,
    #[serde(default)]
    pub price_min: Option<i64>,
    #[serde(default)]
    pub price_max: Option<i64>,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default)]
    pub cities: Option<String>,
    /// Optional prefecture code filter (e.g. `"13"` for Tokyo).
    #[serde(default)]
    pub pref_code: Option<String>,
}

const fn default_limit() -> u32 {
    DEFAULT_OPPORTUNITY_LIMIT
}

fn default_preset() -> String {
    "balance".into()
}

impl OpportunitiesQuery {
    /// Validate and convert the raw query into [`OpportunitiesFilters`].
    ///
    /// Returns [`DomainError::Validation`] if:
    ///
    /// - `bbox` cannot be parsed,
    /// - `risk_max` is not one of `low|mid|high`,
    /// - `zones` contains an empty entry after trimming (catches
    ///   malformed CSV like `",,商業地域,"`),
    /// - `price_min > price_max` when both bounds are present,
    /// - either price bound does not fit the database's 32-bit
    ///   `integer` column,
    /// - [`PricePerSqm::new`] rejects a negative price.
    pub fn into_filters(self) -> Result<OpportunitiesFilters, DomainError> {
        let bbox = BBox::parse_sw_ne_str(&self.bbox)?;
        let limit = OpportunityLimit::clamped(self.limit);
        let offset = OpportunityOffset::new(self.offset);
        let tls_min = self
            .tls_min
            .map(|s| TlsScore::from_f64_clamped(f64::from(s)));
        let risk_max = self.risk_max.as_deref().map(RiskLevel::parse).transpose()?;
        let zones = parse_zones_csv(self.zones.as_deref())?;
        let station_max = self.station_max.map(Meters::new);
        let price_range = parse_price_range(self.price_min, self.price_max)?;
        let preset: WeightPreset = self.preset.parse().unwrap(); // Infallible
        let pref_code = self.pref_code.as_deref().map(PrefCode::new).transpose()?;
        let cities = self
            .cities
            .as_deref()
            .map(|csv| {
                csv.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        self.warn_unsupported();

        Ok(OpportunitiesFilters {
            bbox,
            limit,
            offset,
            tls_min,
            risk_max,
            zones,
            station_max,
            price_range,
            preset,
            pref_code,
            cities,
        })
    }

    /// Emit warn-level tracing for query parameters that Phase 4 does not
    /// yet implement. These filters do not fail the request.
    fn warn_unsupported(&self) {
        if self.station_max.is_some() {
            tracing::warn!(
                station_max = ?self.station_max,
                "opportunities: `station_max` filter not implemented in Phase 4",
            );
        }
    }
}

/// Parse a comma-separated list of zone codes.
///
/// An empty token after trimming (e.g. `"商業地域,,住居"`) is treated as
/// a validation error rather than silently dropped, so that client
/// serialization bugs surface as `400 Bad Request` instead of quietly
/// broadening the result set.
fn parse_zones_csv(raw: Option<&str>) -> Result<Vec<ZoneCode>, DomainError> {
    let Some(csv) = raw else {
        return Ok(Vec::new());
    };
    csv.split(',').map(str::trim).map(ZoneCode::parse).collect()
}

/// Parse the `price_min`/`price_max` query pair.
///
/// Rules:
///
/// - `(None, None)` yields `Ok(None)` (no filter).
/// - Missing bounds default to `0` / `i32::MAX` so the resulting
///   [`PricePerSqm`] pair always fits in the database's 32-bit
///   `integer` column. Using `i64::MAX` here would cause the infra
///   layer's `i64 -> i32` conversion to reject the upper bound, which
///   is the silent-clamping bug Copilot flagged.
/// - `price_min > price_max` returns
///   [`DomainError::Validation`] so clients get `400 Bad Request`
///   rather than an empty result set that is indistinguishable from
///   "no matching data".
/// - Out-of-`i32`-range values return
///   [`DomainError::Validation`] since the DB column cannot store them.
fn parse_price_range(
    lo: Option<i64>,
    hi: Option<i64>,
) -> Result<Option<(PricePerSqm, PricePerSqm)>, DomainError> {
    if lo.is_none() && hi.is_none() {
        return Ok(None);
    }

    let lo_raw = lo.unwrap_or(0);
    let hi_raw = hi.unwrap_or(i64::from(i32::MAX));

    if lo_raw > hi_raw {
        return Err(DomainError::Validation(format!(
            "price_min ({lo_raw}) must be <= price_max ({hi_raw})"
        )));
    }
    if lo_raw > i64::from(i32::MAX) {
        return Err(DomainError::Validation(format!(
            "price_min ({lo_raw}) exceeds the maximum supported value {}",
            i32::MAX
        )));
    }
    if hi_raw > i64::from(i32::MAX) {
        return Err(DomainError::Validation(format!(
            "price_max ({hi_raw}) exceeds the maximum supported value {}",
            i32::MAX
        )));
    }

    Ok(Some((PricePerSqm::new(lo_raw)?, PricePerSqm::new(hi_raw)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_query() -> OpportunitiesQuery {
        OpportunitiesQuery {
            bbox: "139.70,35.65,139.80,35.70".to_string(),
            limit: 25,
            offset: 0,
            tls_min: None,
            risk_max: None,
            zones: None,
            station_max: None,
            price_min: None,
            price_max: None,
            preset: "balance".to_string(),
            cities: None,
            pref_code: None,
        }
    }

    #[test]
    fn into_filters_valid_minimal() {
        let filters = valid_query()
            .into_filters()
            .expect("valid query must parse");
        assert_eq!(filters.limit.get(), 25);
        assert_eq!(filters.offset.get(), 0);
        assert!(filters.tls_min.is_none());
        assert!(filters.risk_max.is_none());
        assert!(filters.zones.is_empty());
        assert!(filters.price_range.is_none());
        assert_eq!(filters.preset, WeightPreset::Balance);
    }

    #[test]
    fn into_filters_clamps_excessive_limit() {
        let query = OpportunitiesQuery {
            limit: 200,
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        assert_eq!(filters.limit.get(), 50);
    }

    #[test]
    fn into_filters_rejects_invalid_bbox() {
        let query = OpportunitiesQuery {
            bbox: "nonsense".to_string(),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());
    }

    #[test]
    fn into_filters_rejects_unknown_risk_max() {
        let query = OpportunitiesQuery {
            risk_max: Some("extreme".to_string()),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());
    }

    #[test]
    fn into_filters_parses_zones_csv_with_whitespace() {
        let query = OpportunitiesQuery {
            zones: Some("商業地域, 近隣商業地域 ,第一種住居地域".to_string()),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        assert_eq!(filters.zones.len(), 3);
        assert_eq!(filters.zones[0].as_str(), "商業地域");
        assert_eq!(filters.zones[1].as_str(), "近隣商業地域");
        assert_eq!(filters.zones[2].as_str(), "第一種住居地域");
    }

    #[test]
    fn into_filters_price_range_both_some() {
        let query = OpportunitiesQuery {
            price_min: Some(500_000),
            price_max: Some(2_000_000),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        let (lo, hi) = filters.price_range.unwrap();
        assert_eq!(lo.value(), 500_000);
        assert_eq!(hi.value(), 2_000_000);
    }

    #[test]
    fn into_filters_price_range_only_min() {
        let query = OpportunitiesQuery {
            price_min: Some(500_000),
            price_max: None,
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        let (lo, hi) = filters.price_range.unwrap();
        assert_eq!(lo.value(), 500_000);
        // Upper bound defaults to i32::MAX (not i64::MAX) so it always
        // fits the DB's 32-bit `integer` column.
        assert_eq!(hi.value(), i64::from(i32::MAX));
    }

    #[test]
    fn into_filters_price_range_only_max() {
        let query = OpportunitiesQuery {
            price_min: None,
            price_max: Some(1_000_000),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        let (lo, hi) = filters.price_range.unwrap();
        assert_eq!(lo.value(), 0);
        assert_eq!(hi.value(), 1_000_000);
    }

    #[test]
    fn into_filters_rejects_negative_price() {
        let query = OpportunitiesQuery {
            price_min: Some(-1),
            price_max: Some(100),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());
    }

    #[test]
    fn into_filters_rejects_price_min_greater_than_max() {
        let query = OpportunitiesQuery {
            price_min: Some(2_000_000),
            price_max: Some(500_000),
            ..valid_query()
        };
        let err = query.into_filters().unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("price_min"), "expected price message: {msg}");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn into_filters_rejects_price_above_i32_max() {
        let query = OpportunitiesQuery {
            price_min: None,
            price_max: Some(i64::from(i32::MAX) + 1),
            ..valid_query()
        };
        let err = query.into_filters().unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn into_filters_rejects_empty_zone_token() {
        // Empty token (double comma) must surface as a validation
        // error, not be silently dropped — otherwise client-side
        // serialization bugs quietly broaden the result set.
        let query = OpportunitiesQuery {
            zones: Some("商業地域,,第一種住居地域".to_string()),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());
    }

    #[test]
    fn into_filters_rejects_leading_trailing_empty_zone_token() {
        let query = OpportunitiesQuery {
            zones: Some(",商業地域".to_string()),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());

        let query = OpportunitiesQuery {
            zones: Some("商業地域,".to_string()),
            ..valid_query()
        };
        assert!(query.into_filters().is_err());
    }

    #[test]
    fn into_filters_preset_fallback_to_balance() {
        let query = OpportunitiesQuery {
            preset: "unknown".to_string(),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        assert_eq!(filters.preset, WeightPreset::Balance);
    }

    #[test]
    fn into_filters_preset_investment() {
        let query = OpportunitiesQuery {
            preset: "investment".to_string(),
            ..valid_query()
        };
        assert_eq!(
            query.into_filters().unwrap().preset,
            WeightPreset::Investment
        );
    }

    #[test]
    fn into_filters_preset_disaster_alias() {
        let query = OpportunitiesQuery {
            preset: "disaster".to_string(),
            ..valid_query()
        };
        assert_eq!(
            query.into_filters().unwrap().preset,
            WeightPreset::DisasterFocus
        );
    }

    #[test]
    fn into_filters_tls_min_parsed() {
        let query = OpportunitiesQuery {
            tls_min: Some(75),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        assert_eq!(filters.tls_min.unwrap().value(), 75);
    }

    #[test]
    fn into_filters_risk_max_mid() {
        let query = OpportunitiesQuery {
            risk_max: Some("mid".to_string()),
            ..valid_query()
        };
        let filters = query.into_filters().unwrap();
        assert_eq!(filters.risk_max.unwrap(), RiskLevel::Mid);
    }

    #[test]
    fn into_filters_cities_warns_but_succeeds() {
        let query = OpportunitiesQuery {
            cities: Some("13101,13102".to_string()),
            ..valid_query()
        };
        // Should succeed (cities is just logged)
        assert!(query.into_filters().is_ok());
    }
}
