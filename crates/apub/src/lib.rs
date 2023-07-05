use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::config::{Data, UrlVerifier};
use async_trait::async_trait;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::{Activity, ActivityInsertForm},
    instance::Instance,
    local_site::LocalSite,
  },
  traits::Crud,
  utils::DbPool,
};
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use once_cell::sync::Lazy;
use serde::Serialize;
use url::Url;

pub mod activities;
pub(crate) mod activity_lists;
pub mod api;
pub(crate) mod collections;
pub mod fetcher;
pub mod http;
pub(crate) mod mentions;
pub mod objects;
pub mod protocol;

pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 50;

static CONTEXT: Lazy<Vec<serde_json::Value>> = Lazy::new(|| {
  serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context")
});

#[derive(Clone)]
pub struct VerifyUrlData(pub DbPool);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
  async fn verify(&self, url: &Url) -> Result<(), &'static str> {
    let local_site_data = fetch_local_site_data(&self.0)
      .await
      .expect("read local site data");
    check_apub_id_valid(url, &local_site_data)?;
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
///
/// `use_strict_allowlist` should be true only when parsing a remote community, or when parsing a
/// post/comment in a local community.
#[tracing::instrument(skip(local_site_data))]
fn check_apub_id_valid(apub_id: &Url, local_site_data: &LocalSiteData) -> Result<(), &'static str> {
  let domain = apub_id.domain().expect("apud id has domain").to_string();

  if !local_site_data
    .local_site
    .as_ref()
    .map(|l| l.federation_enabled)
    .unwrap_or(true)
  {
    return Err("Federation disabled");
  }

  if local_site_data
    .blocked_instances
    .iter()
    .any(|i| domain.eq(&i.domain))
  {
    return Err("Domain is blocked");
  }

  // Only check this if there are instances in the allowlist
  if !local_site_data.allowed_instances.is_empty()
    && !local_site_data
      .allowed_instances
      .iter()
      .any(|i| domain.eq(&i.domain))
  {
    return Err("Domain is not in allowlist");
  }

  Ok(())
}

#[derive(Clone)]
pub(crate) struct LocalSiteData {
  local_site: Option<LocalSite>,
  allowed_instances: Vec<Instance>,
  blocked_instances: Vec<Instance>,
}

pub(crate) async fn fetch_local_site_data(
  pool: &DbPool,
) -> Result<LocalSiteData, diesel::result::Error> {
  // LocalSite may be missing
  let local_site = LocalSite::read(pool).await.ok();
  let allowed_instances = Instance::allowlist(pool).await?;
  let blocked_instances = Instance::blocklist(pool).await?;

  Ok(LocalSiteData {
    local_site,
    allowed_instances,
    blocked_instances,
  })
}

#[tracing::instrument(skip(settings, local_site_data))]
pub(crate) fn check_apub_id_valid_with_strictness(
  apub_id: &Url,
  is_strict: bool,
  local_site_data: &LocalSiteData,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let domain = apub_id.domain().expect("apud id has domain").to_string();
  let local_instance = settings
    .get_hostname_without_port()
    .expect("local hostname is valid");
  if domain == local_instance {
    return Ok(());
  }
  check_apub_id_valid(apub_id, local_site_data).map_err(LemmyError::from_message)?;

  // Only check allowlist if this is a community, and there are instances in the allowlist
  if is_strict && !local_site_data.allowed_instances.is_empty() {
    // need to allow this explicitly because apub receive might contain objects from our local
    // instance.
    let mut allowed_and_local = local_site_data
      .allowed_instances
      .iter()
      .map(|i| i.domain.clone())
      .collect::<Vec<String>>();
    let local_instance = settings
      .get_hostname_without_port()
      .expect("local hostname is valid");
    allowed_and_local.push(local_instance);

    let domain = apub_id.domain().expect("apud id has domain").to_string();
    if !allowed_and_local.contains(&domain) {
      return Err(LemmyError::from_message(
        "Federation forbidden by strict allowlist",
      ));
    }
  }
  Ok(())
}

/// Store a sent or received activity in the database.
///
/// Stored activities are served over the HTTP endpoint `GET /activities/{type_}/{id}`. This also
/// ensures that the same activity cannot be received more than once.
#[tracing::instrument(skip(data, activity))]
async fn insert_activity<T>(
  ap_id: &Url,
  activity: &T,
  local: bool,
  sensitive: bool,
  data: &Data<LemmyContext>,
) -> Result<(), LemmyError>
where
  T: Serialize,
{
  let ap_id = ap_id.clone().into();
  let form = ActivityInsertForm {
    ap_id,
    data: serde_json::to_value(activity)?,
    local: Some(local),
    sensitive: Some(sensitive),
    updated: None,
  };
  Activity::create(data.pool(), &form).await?;
  Ok(())
}

#[async_trait::async_trait]
pub trait SendActivity: Sync {
  type Response: Sync + Send;

  async fn send_activity(
    _request: &Self,
    _response: &Self::Response,
    _context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    Ok(())
  }
}
