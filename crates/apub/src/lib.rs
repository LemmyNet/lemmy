use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{
  core::signatures::PublicKey,
  traits::{Actor, ApubObject},
  InstanceSettings,
  LocalInstance,
  UrlVerifier,
};
use anyhow::Context;
use async_trait::async_trait;
use diesel::PgConnection;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{activity::Activity, instance::Instance, local_site::LocalSite},
  utils::DbPool,
};
use lemmy_utils::{error::LemmyError, location_info, settings::structs::Settings};
use lemmy_websocket::LemmyContext;
use once_cell::sync::{Lazy, OnceCell};
use url::{ParseError, Url};

pub mod activities;
pub(crate) mod activity_lists;
pub(crate) mod collections;
pub mod fetcher;
pub mod http;
pub(crate) mod mentions;
pub mod objects;
pub mod protocol;

static CONTEXT: Lazy<Vec<serde_json::Value>> = Lazy::new(|| {
  serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context")
});

// TODO: store this in context? but its only used in this crate, no need to expose it elsewhere
// TODO this singleton needs to be redone to account for live data.
fn local_instance(context: &LemmyContext) -> &'static LocalInstance {
  static LOCAL_INSTANCE: OnceCell<LocalInstance> = OnceCell::new();
  LOCAL_INSTANCE.get_or_init(|| {
    let conn = &mut context
      .pool()
      .get()
      .expect("getting connection for LOCAL_INSTANCE init");
    // Local site may be missing
    let local_site = &LocalSite::read(conn);
    let worker_count = local_site
      .as_ref()
      .map(|l| l.federation_worker_count)
      .unwrap_or(64) as u64;
    let http_fetch_retry_limit = local_site
      .as_ref()
      .map(|l| l.federation_http_fetch_retry_limit)
      .unwrap_or(25);
    let federation_debug = local_site
      .as_ref()
      .map(|l| l.federation_debug)
      .unwrap_or(true);

    let settings = InstanceSettings::builder()
      .http_fetch_retry_limit(http_fetch_retry_limit)
      .worker_count(worker_count)
      .debug(federation_debug)
      .http_signature_compat(true)
      .url_verifier(Box::new(VerifyUrlData(context.clone())))
      .build()
      .expect("configure federation");
    LocalInstance::new(
      context.settings().hostname.to_owned(),
      context.client().clone(),
      settings,
    )
  })
}

#[derive(Clone)]
struct VerifyUrlData(LemmyContext);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
  async fn verify(&self, url: &Url) -> Result<(), &'static str> {
    let local_site_data = blocking(self.0.pool(), fetch_local_site_data)
      .await
      .expect("read local site data")
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

pub(crate) fn fetch_local_site_data(
  conn: &mut PgConnection,
) -> Result<LocalSiteData, diesel::result::Error> {
  // LocalSite may be missing
  let local_site = LocalSite::read(conn).ok();
  let allowed = Instance::allowlist(conn)?;
  let blocked = Instance::blocklist(conn)?;

  // These can return empty vectors, so convert them to options
  let allowed_instances = (!allowed.is_empty()).then(|| allowed);
  let blocked_instances = (!blocked.is_empty()).then(|| blocked);

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
    // Only check allowlist if this is a community, or strict allowlist is enabled.
    let strict_allowlist = local_site_data
      .local_site
      .as_ref()
      .map(|l| l.federation_strict_allowlist)
      .unwrap_or(true);
    if is_strict || strict_allowlist {
      // need to allow this explicitly because apub receive might contain objects from our local
      // instance.
      let mut allowed_and_local = allowed.to_owned();
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

pub enum EndpointType {
  Community,
  Person,
  Post,
  Comment,
  PrivateMessage,
}

/// Generates an apub endpoint for a given domain, IE xyz.tld
pub fn generate_local_apub_endpoint(
  endpoint_type: EndpointType,
  name: &str,
  domain: &str,
) -> Result<DbUrl, ParseError> {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::Person => "u",
    EndpointType::Post => "post",
    EndpointType::Comment => "comment",
    EndpointType::PrivateMessage => "private_message",
  };

  Ok(Url::parse(&format!("{}/{}/{}", domain, point, name))?.into())
}

pub fn generate_followers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/followers", actor_id))?.into())
}

pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/inbox", actor_id))?.into())
}

pub fn generate_site_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  let mut actor_id: Url = actor_id.clone().into();
  actor_id.set_path("site_inbox");
  Ok(actor_id.into())
}

pub fn generate_shared_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  let actor_id: Url = actor_id.clone().into();
  let url = format!(
    "{}://{}{}/inbox",
    &actor_id.scheme(),
    &actor_id.host_str().context(location_info!())?,
    if let Some(port) = actor_id.port() {
      format!(":{}", port)
    } else {
      "".to_string()
    },
  );
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/outbox", actor_id))?.into())
}

fn generate_moderators_url(community_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  Ok(Url::parse(&format!("{}/moderators", community_id))?.into())
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
  let ap_id = ap_id.to_owned().into();
  Ok(
    blocking(pool, move |conn| {
      Activity::insert(conn, ap_id, activity, local, Some(sensitive))
    })
    .await??,
  )
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
