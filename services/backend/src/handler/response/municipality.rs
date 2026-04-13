//! Response DTOs for `GET /api/v1/municipalities`.

use serde::Serialize;

use crate::domain::municipality::Municipality;

/// Single municipality record returned in the response array.
#[derive(Debug, Serialize)]
pub struct MunicipalityResponse {
    pub city_code: String,
    pub city_name: String,
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
