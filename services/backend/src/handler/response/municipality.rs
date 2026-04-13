//! Response DTOs for `GET /api/v1/municipalities`.

use serde::Serialize;

use crate::domain::municipality::Municipality;

/// Single municipality record returned in the `GET /api/v1/municipalities` response array.
#[derive(Debug, Serialize)]
pub struct MunicipalityResponse {
    /// 5-digit municipality code (e.g. `"13101"` for Chiyoda-ku).
    pub city_code: String,
    /// Municipality name in Japanese (e.g. `"千代田区"`).
    pub city_name: String,
    /// 2-digit parent prefecture code (e.g. `"13"` for Tokyo).
    pub pref_code: String,
}

impl From<Municipality> for MunicipalityResponse {
    fn from(m: Municipality) -> Self {
        Self {
            city_code: m.city_code,
            city_name: m.city_name,
            pref_code: m.pref_code,
        }
    }
}
