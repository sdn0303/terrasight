//! Municipality lookup domain types.
//!
//! Municipality records are used by the `/api/v1/municipalities` endpoint to
//! populate the city filter dropdown in the Terrasight frontend. They are
//! derived from the `municipalities` table, which is populated from JIS X 0402
//! reference data during the data pipeline setup.

/// A Japanese municipality (市区町村) identified by its JIS X 0402 code.
///
/// Used as a lightweight lookup record — not an aggregate root. Business
/// rules about valid code ranges are enforced by
/// [`CityCode`](crate::domain::value_object::CityCode).
#[derive(Debug, Clone)]
pub struct Municipality {
    /// JIS X 0402 5-digit municipality code (e.g. `"13101"` for 千代田区).
    pub city_code: String,
    /// Human-readable municipality name in Japanese (e.g. `"千代田区"`).
    pub city_name: String,
    /// 2-digit prefecture code for the parent prefecture (e.g. `"13"`).
    pub pref_code: String,
}
