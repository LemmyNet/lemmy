use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{
  config::{Data, UrlVerifier},
  error::Error as ActivityPubError,
};
use async_trait::async_trait;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{activity::ReceivedActivity, instance::Instance, local_site::LocalSite},
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use moka::future::Cache;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use url::Url;

pub mod activities;
pub mod activity_lists;
pub mod api;
pub(crate) mod collections;
pub mod fetcher;
pub mod http;
pub(crate) mod mentions;
pub mod objects;
pub mod protocol;

pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 50;
/// All incoming and outgoing federation actions read the blocklist/allowlist and slur filters
/// multiple times. This causes a huge number of database reads if we hit the db directly. So we
/// cache these values for a short time, which will already make a huge difference and ensures that
/// changes take effect quickly.
const BLOCKLIST_CACHE_DURATION: Duration = Duration::from_secs(60);

/// Only include a basic context to save space and bandwidth. The main context is hosted statically
/// on join-lemmy.org. Include activitystreams explicitly for better compat, but this could
/// theoretically also be moved.
pub static FEDERATION_CONTEXT: Lazy<Value> = Lazy::new(|| {
  Value::Array(vec![
    Value::String("https://join-lemmy.org/context.json".to_string()),
    Value::String("https://www.w3.org/ns/activitystreams".to_string()),
  ])
});

#[derive(Clone)]
pub struct VerifyUrlData(pub ActualDbPool);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
  async fn verify(&self, url: &Url) -> Result<(), ActivityPubError> {
    let local_site_data = local_site_data_cached(&mut (&self.0).into())
      .await
      .expect("read local site data");
    check_apub_id_valid(url, &local_site_data).map_err(|err| match err {
      LemmyError {
        error_type: LemmyErrorType::FederationDisabled,
        ..
      } => ActivityPubError::Other("Federation disabled".into()),
      LemmyError {
        error_type: LemmyErrorType::DomainBlocked(domain),
        ..
      } => ActivityPubError::Other(format!("Domain {domain:?} is blocked")),
      LemmyError {
        error_type: LemmyErrorType::DomainNotInAllowList(domain),
        ..
      } => ActivityPubError::Other(format!("Domain {domain:?} is not in allowlist")),
      _ => ActivityPubError::Other("Failed validating apub id".into()),
    })?;
    Ok(())
  }
}

/// Checks if the ID is allowed for sending or receiving.
///
/// In particular, it checks for:
/// - federation being enabled (if its disabled, only local URLs are allowed)
/// - the correct scheme (either http or https)
/// - URL being in the allowlist (if it is active)
/// - URL not being in the blocklist (if it is active)
#[tracing::instrument(skip(local_site_data))]
fn check_apub_id_valid(apub_id: &Url, local_site_data: &LocalSiteData) -> Result<(), LemmyError> {
  let domain = apub_id.domain().expect("apud id has domain").to_string();

  if !local_site_data
    .local_site
    .as_ref()
    .map(|l| l.federation_enabled)
    .unwrap_or(true)
  {
    Err(LemmyErrorType::FederationDisabled)?
  }

  if local_site_data
    .blocked_instances
    .iter()
    .any(|i| domain.to_lowercase().eq(&i.domain.to_lowercase()))
  {
    Err(LemmyErrorType::DomainBlocked(domain.clone()))?
  }

  // Only check this if there are instances in the allowlist
  if !local_site_data.allowed_instances.is_empty()
    && !local_site_data
      .allowed_instances
      .iter()
      .any(|i| domain.to_lowercase().eq(&i.domain.to_lowercase()))
  {
    Err(LemmyErrorType::DomainNotInAllowList(domain))?
  }

  Ok(())
}

#[derive(Clone)]
pub(crate) struct LocalSiteData {
  local_site: Option<LocalSite>,
  allowed_instances: Vec<Instance>,
  blocked_instances: Vec<Instance>,
}

pub(crate) async fn local_site_data_cached(
  pool: &mut DbPool<'_>,
) -> LemmyResult<Arc<LocalSiteData>> {
  static CACHE: Lazy<Cache<(), Arc<LocalSiteData>>> = Lazy::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(BLOCKLIST_CACHE_DURATION)
      .build()
  });
  Ok(
    CACHE
      .try_get_with((), async {
        let (local_site, allowed_instances, blocked_instances) =
          lemmy_db_schema::try_join_with_pool!(pool => (
            // LocalSite may be missing
            |pool| async {
              Ok(LocalSite::read(pool).await.ok())
            },
            Instance::allowlist,
            Instance::blocklist
          ))?;

        Ok::<_, diesel::result::Error>(Arc::new(LocalSiteData {
          local_site,
          allowed_instances,
          blocked_instances,
        }))
      })
      .await?,
  )
}

pub(crate) async fn check_apub_id_valid_with_strictness(
  apub_id: &Url,
  is_strict: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let domain = apub_id.domain().expect("apud id has domain").to_string();
  let local_instance = context
    .settings()
    .get_hostname_without_port()
    .expect("local hostname is valid");
  if domain == local_instance {
    return Ok(());
  }

  let local_site_data = local_site_data_cached(&mut context.pool()).await?;
  check_apub_id_valid(apub_id, &local_site_data)?;

  // Only check allowlist if this is a community, and there are instances in the allowlist
  if is_strict && !local_site_data.allowed_instances.is_empty() {
    // need to allow this explicitly because apub receive might contain objects from our local
    // instance.
    let mut allowed_and_local = local_site_data
      .allowed_instances
      .iter()
      .map(|i| i.domain.clone())
      .collect::<Vec<String>>();
    let local_instance = context
      .settings()
      .get_hostname_without_port()
      .expect("local hostname is valid");
    allowed_and_local.push(local_instance);

    let domain = apub_id.domain().expect("apud id has domain").to_string();
    if !allowed_and_local.contains(&domain) {
      Err(LemmyErrorType::FederationDisabledByStrictAllowList)?
    }
  }
  Ok(())
}

/// Store received activities in the database.
///
/// This ensures that the same activity doesnt get received and processed more than once, which
/// would be a waste of resources.
#[tracing::instrument(skip(data))]
async fn insert_received_activity(
  ap_id: &Url,
  data: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  ReceivedActivity::create(&mut data.pool(), &ap_id.clone().into()).await?;
  Ok(())
}
