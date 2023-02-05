use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{
  core::signatures::PublicKey,
  traits::{Actor, ApubObject},
  InstanceSettings,
  LocalInstance,
  UrlVerifier,
};
use async_trait::async_trait;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{activity::Activity, instance::Instance, local_site::LocalSite},
  utils::DbPool,
};
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
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

const FEDERATION_HTTP_FETCH_LIMIT: i32 = 25;

static CONTEXT: Lazy<Vec<serde_json::Value>> = Lazy::new(|| {
  serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context")
});

// TODO: store this in context? but its only used in this crate, no need to expose it elsewhere
// TODO this singleton needs to be redone to account for live data.
async fn local_instance(context: &LemmyContext) -> &'static LocalInstance {
  static LOCAL_INSTANCE: OnceCell<LocalInstance> = OnceCell::const_new();
  LOCAL_INSTANCE
    .get_or_init(|| async {
      // Local site may be missing
      let local_site = &LocalSite::read(context.pool()).await;
      let worker_count = local_site
        .as_ref()
        .map(|l| l.federation_worker_count)
        .unwrap_or(64) as u64;

      let settings = InstanceSettings::builder()
        .http_fetch_retry_limit(FEDERATION_HTTP_FETCH_LIMIT)
        .worker_count(worker_count)
        .debug(cfg!(debug_assertions))
        .http_signature_compat(true)
        .url_verifier(Box::new(VerifyUrlData(context.clone())))
        .build()
        .expect("configure federation");
      LocalInstance::new(
        context.settings().hostname.clone(),
        context.client().clone(),
        settings,
      )
    })
    .await
}

#[derive(Clone)]
struct VerifyUrlData(LemmyContext);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
  async fn verify(&self, url: &Url) -> Result<(), &'static str> {
    let local_site_data = fetch_local_site_data(self.0.pool())
      .await
      .expect("read local site data");
    check_apub_id_valid(url, &local_site_data, self.0.settings())
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
#[tracing::instrument(skip(settings, local_site_data))]
fn check_apub_id_valid(
  apub_id: &Url,
  local_site_data: &LocalSiteData,
  settings: &Settings,
) -> Result<(), &'static str> {
  let domain = apub_id.domain().expect("apud id has domain").to_string();
  let local_instance = settings
    .get_hostname_without_port()
    .expect("local hostname is valid");
  if domain == local_instance {
    return Ok(());
  }

  if !local_site_data
    .local_site
    .as_ref()
    .map(|l| l.federation_enabled)
    .unwrap_or(true)
  {
    return Err("Federation disabled");
  }

  if apub_id.scheme() != settings.get_protocol_string() {
    return Err("Invalid protocol scheme");
  }

  if let Some(blocked) = local_site_data.blocked_instances.as_ref() {
    if blocked.contains(&domain) {
      return Err("Domain is blocked");
    }
  }

  if let Some(allowed) = local_site_data.allowed_instances.as_ref() {
    if !allowed.contains(&domain) {
      return Err("Domain is not in allowlist");
    }
  }

  Ok(())
}

#[derive(Clone)]
pub(crate) struct LocalSiteData {
  local_site: Option<LocalSite>,
  allowed_instances: Option<Vec<String>>,
  blocked_instances: Option<Vec<String>>,
}

pub(crate) async fn fetch_local_site_data(
  pool: &DbPool,
) -> Result<LocalSiteData, diesel::result::Error> {
  // LocalSite may be missing
  let local_site = LocalSite::read(pool).await.ok();
  let allowed = Instance::allowlist(pool).await?;
  let blocked = Instance::blocklist(pool).await?;

  // These can return empty vectors, so convert them to options
  let allowed_instances = (!allowed.is_empty()).then_some(allowed);
  let blocked_instances = (!blocked.is_empty()).then_some(blocked);

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
  check_apub_id_valid(apub_id, local_site_data, settings).map_err(LemmyError::from_message)?;
  let domain = apub_id.domain().expect("apud id has domain").to_string();
  let local_instance = settings
    .get_hostname_without_port()
    .expect("local hostname is valid");
  if domain == local_instance {
    return Ok(());
  }

  if let Some(allowed) = local_site_data.allowed_instances.as_ref() {
    // Only check allowlist if this is a community
    if is_strict {
      // need to allow this explicitly because apub receive might contain objects from our local
      // instance.
      let mut allowed_and_local = allowed.clone();
      allowed_and_local.push(local_instance);

      if !allowed_and_local.contains(&domain) {
        return Err(LemmyError::from_message(
          "Federation forbidden by strict allowlist",
        ));
      }
    }
  }
  Ok(())
}

/// Store a sent or received activity in the database, for logging purposes. These records are not
/// persistent.
#[tracing::instrument(skip(pool))]
async fn insert_activity(
  ap_id: &Url,
  activity: serde_json::Value,
  local: bool,
  sensitive: bool,
  pool: &DbPool,
) -> Result<bool, LemmyError> {
  let ap_id = ap_id.clone().into();
  Ok(Activity::insert(pool, ap_id, activity, local, Some(sensitive)).await?)
}

/// Common methods provided by ActivityPub actors (community and person). Not all methods are
/// implemented by all actors.
pub trait ActorType: Actor + ApubObject {
  fn actor_id(&self) -> Url;

  fn private_key(&self) -> Option<String>;

  fn get_public_key(&self) -> PublicKey {
    PublicKey::new_main_key(self.actor_id(), self.public_key().to_string())
  }
}

#[async_trait::async_trait(?Send)]
pub trait SendActivity {
  type Response;

  async fn send_activity(
    _request: &Self,
    _response: &Self::Response,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    Ok(())
  }
}
