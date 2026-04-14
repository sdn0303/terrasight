//! [`MunicipalityRepository`] trait — municipality lookup data by prefecture.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::municipality::Municipality;
use crate::domain::value_object::PrefCode;

/// Repository for municipality lookup data.
///
/// Provides the list of [`Municipality`] records for a given prefecture,
/// used by the `/api/v1/municipalities` endpoint.
///
/// Implemented by `PgMunicipalityRepository` in the `infra` layer.
#[async_trait]
pub trait MunicipalityRepository: Send + Sync {
    /// Fetch all municipalities for the given prefecture.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_municipalities(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<Municipality>, DomainError>;
}
