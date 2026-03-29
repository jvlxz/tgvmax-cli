use crate::error::Result;
use crate::models::{Proposal, SearchParams, Station};

/// Trait abstracting TGVmax availability API access.
///
/// Implement this trait to swap between API backends (MaxJeune portal,
/// SNCF Open Data, mocks for testing, etc).
pub trait TgvmaxClient: Send + Sync {
    /// List all available stations.
    fn list_stations(&self) -> impl std::future::Future<Output = Result<Vec<Station>>> + Send;

    /// Search for available TGVmax trains on a route and date.
    fn search_trains(
        &self,
        params: &SearchParams,
    ) -> impl std::future::Future<Output = Result<Vec<Proposal>>> + Send;
}
