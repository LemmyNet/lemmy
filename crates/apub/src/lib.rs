use activitypub_federation::{config::UrlVerifier, error::Error as ActivityPubError};
use async_trait::async_trait;
use lemmy_apub_objects::utils::functions::{check_apub_id_valid, local_site_data_cached};
use lemmy_db_schema::utils::ActualDbPool;
use lemmy_utils::error::{FederationError, LemmyError, LemmyErrorType};
use url::Url;

pub mod activities;
pub mod activity_lists;
pub mod api;
pub mod collections;
pub mod fetcher;
pub mod http;
pub mod protocol;

/// Maximum number of outgoing HTTP requests to fetch a single object. Needs to be high enough
/// to fetch a new community with posts, moderators and featured posts.
pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 100;

#[derive(Clone)]
pub struct VerifyUrlData(pub ActualDbPool);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
  async fn verify(&self, url: &Url) -> Result<(), ActivityPubError> {
    let local_site_data = local_site_data_cached(&mut (&self.0).into())
      .await
      .map_err(|e| ActivityPubError::Other(format!("Cant read local site data: {e}")))?;

    use FederationError::*;
    check_apub_id_valid(url, &local_site_data).map_err(|err| match err {
      LemmyError {
        error_type:
          LemmyErrorType::FederationError {
            error: Some(FederationDisabled),
          },
        ..
      } => ActivityPubError::Other("Federation disabled".into()),
      LemmyError {
        error_type:
          LemmyErrorType::FederationError {
            error: Some(DomainBlocked(domain)),
          },
        ..
      } => ActivityPubError::Other(format!("Domain {domain:?} is blocked")),
      LemmyError {
        error_type:
          LemmyErrorType::FederationError {
            error: Some(DomainNotInAllowList(domain)),
          },
        ..
      } => ActivityPubError::Other(format!("Domain {domain:?} is not in allowlist")),
      _ => ActivityPubError::Other("Failed validating apub id".into()),
    })?;
    Ok(())
  }
}
