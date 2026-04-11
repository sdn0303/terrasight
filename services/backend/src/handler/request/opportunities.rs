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
use crate::domain::value_object::{BBox, OpportunityLimit, OpportunityOffset, RiskLevel, TlsScore};

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
}

const fn default_limit() -> u32 {
    DEFAULT_OPPORTUNITY_LIMIT
}

fn default_preset() -> String {
    "balance".into()
}

/// Validated, domain-typed filter set passed to
/// [`GetOpportunitiesUsecase`].
#[derive(Debug, Clone)]
pub struct OpportunitiesFilters {
    pub bbox: BBox,
    pub limit: OpportunityLimit,
    pub offset: OpportunityOffset,
    pub tls_min: Option<TlsScore>,
    pub risk_max: Option<RiskLevel>,
    pub zones: Vec<ZoneCode>,
    pub station_max: Option<Meters>,
    pub price_range: Option<(PricePerSqm, PricePerSqm)>,
    pub preset: WeightPreset,
}

impl OpportunitiesQuery {
    /// Validate and convert the raw query into [`OpportunitiesFilters`].
    ///
    /// Returns [`DomainError::Validation`] if `bbox` cannot be parsed,
    /// `risk_max` is not one of `low|mid|high`, `zones` contains an empty
    /// entry after trimming, or the `price_min`/`price_max` pair yields
    /// an invalid [`PricePerSqm`].
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
        let preset = parse_preset(&self.preset);

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
        })
    }

    /// Emit warn-level tracing for query parameters that Phase 4 does not
    /// yet implement. These filters do not fail the request.
    fn warn_unsupported(&self) {
        if self.cities.is_some() {
            tracing::warn!(
                cities = ?self.cities,
                "opportunities: `cities` filter not implemented in Phase 4",
            );
        }
        if self.station_max.is_some() {
            tracing::warn!(
                station_max = ?self.station_max,
                "opportunities: `station_max` filter not implemented in Phase 4",
            );
        }
    }
}

fn parse_zones_csv(raw: Option<&str>) -> Result<Vec<ZoneCode>, DomainError> {
    match raw {
        None => Ok(Vec::new()),
        Some(csv) => csv
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ZoneCode::parse)
            .collect(),
    }
}

fn parse_price_range(
    lo: Option<i64>,
    hi: Option<i64>,
) -> Result<Option<(PricePerSqm, PricePerSqm)>, DomainError> {
    match (lo, hi) {
        (None, None) => Ok(None),
        (lo, hi) => {
            let min = PricePerSqm::new(lo.unwrap_or(0))?;
            let max = PricePerSqm::new(hi.unwrap_or(i64::MAX))?;
            Ok(Some((min, max)))
        }
    }
}

fn parse_preset(raw: &str) -> WeightPreset {
    match raw {
        "investment" => WeightPreset::Investment,
        "residential" => WeightPreset::Residential,
        "disaster" | "disaster_focus" => WeightPreset::DisasterFocus,
        _ => WeightPreset::Balance,
    }
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
        assert_eq!(hi.value(), i64::MAX);
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
