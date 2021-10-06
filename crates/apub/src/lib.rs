pub mod activities;
mod context;
pub mod fetcher;
pub mod http;
pub mod migrations;
pub mod objects;

use crate::fetcher::post_or_comment::PostOrComment;
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{activity_queue::send_activity, traits::ActorType};
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::{
  source::{activity::Activity, person::Person},
  CommunityId,
  DbUrl,
};
use lemmy_db_views_actor::community_person_ban_view::CommunityPersonBanView;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::info;
use serde::Serialize;
use std::net::IpAddr;
use url::{ParseError, Url};

/// Checks if the ID is allowed for sending or receiving.
///
/// In particular, it checks for:
/// - federation being enabled (if its disabled, only local URLs are allowed)
/// - the correct scheme (either http or https)
/// - URL being in the allowlist (if it is active)
/// - URL not being in the blocklist (if it is active)
///
pub(crate) fn check_is_apub_id_valid(
  apub_id: &Url,
  use_strict_allowlist: bool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let domain = apub_id.domain().context(location_info!())?.to_string();
  let local_instance = settings.get_hostname_without_port()?;

  if !settings.federation.enabled {
    return if domain == local_instance {
      Ok(())
    } else {
      Err(
        anyhow!(
          "Trying to connect with {}, but federation is disabled",
          domain
        )
        .into(),
      )
    };
  }

  let host = apub_id.host_str().context(location_info!())?;
  let host_as_ip = host.parse::<IpAddr>();
  if host == "localhost" || host_as_ip.is_ok() {
    return Err(anyhow!("invalid hostname {}: {}", host, apub_id).into());
  }

  if apub_id.scheme() != settings.get_protocol_string() {
    return Err(anyhow!("invalid apub id scheme {}: {}", apub_id.scheme(), apub_id).into());
  }

  // TODO: might be good to put the part above in one method, and below in another
  //       (which only gets called in apub::objects)
  //        -> no that doesnt make sense, we still need the code below for blocklist and strict allowlist
  if let Some(blocked) = settings.to_owned().federation.blocked_instances {
    if blocked.contains(&domain) {
      return Err(anyhow!("{} is in federation blocklist", domain).into());
    }
  }

  if let Some(mut allowed) = settings.to_owned().federation.allowed_instances {
    // Only check allowlist if this is a community, or strict allowlist is enabled.
    let strict_allowlist = settings.to_owned().federation.strict_allowlist;
    if use_strict_allowlist || strict_allowlist {
      // need to allow this explicitly because apub receive might contain objects from our local
      // instance.
      allowed.push(local_instance);

      if !allowed.contains(&domain) {
        return Err(anyhow!("{} not in federation allowlist", domain).into());
      }
    }
  }

  Ok(())
}

#[async_trait::async_trait(?Send)]
pub trait CommunityType {
  fn followers_url(&self) -> Url;
  async fn get_follower_inboxes(
    &self,
    pool: &DbPool,
    settings: &Settings,
  ) -> Result<Vec<Url>, LemmyError>;
}

pub enum EndpointType {
  Community,
  Person,
  Post,
  Comment,
  PrivateMessage,
}

/// Generates an apub endpoint for a given domain, IE xyz.tld
fn generate_apub_endpoint_for_domain(
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

/// Generates the ActivityPub ID for a given object type and ID.
pub fn generate_apub_endpoint(
  endpoint_type: EndpointType,
  name: &str,
  protocol_and_hostname: &str,
) -> Result<DbUrl, ParseError> {
  generate_apub_endpoint_for_domain(endpoint_type, name, protocol_and_hostname)
}

pub fn generate_followers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/followers", actor_id))?.into())
}

pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/inbox", actor_id))?.into())
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

/// Takes in a shortname of the type dessalines@xyz.tld or dessalines (assumed to be local), and outputs the actor id.
/// Used in the API for communities and users.
pub fn build_actor_id_from_shortname(
  endpoint_type: EndpointType,
  short_name: &str,
  settings: &Settings,
) -> Result<DbUrl, ParseError> {
  let split = short_name.split('@').collect::<Vec<&str>>();

  let name = split[0];

  // If there's no @, its local
  let domain = if split.len() == 1 {
    settings.get_protocol_and_hostname()
  } else {
    format!("{}://{}", settings.get_protocol_string(), split[1])
  };

  generate_apub_endpoint_for_domain(endpoint_type, name, &domain)
}

/// Store a sent or received activity in the database, for logging purposes. These records are not
/// persistent.
async fn insert_activity<T>(
  ap_id: &Url,
  activity: T,
  local: bool,
  sensitive: bool,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: Serialize + std::fmt::Debug + Send + 'static,
{
  let ap_id = ap_id.to_owned().into();
  blocking(pool, move |conn| {
    Activity::insert(conn, ap_id, &activity, local, sensitive)
  })
  .await??;
  Ok(())
}

async fn check_community_or_site_ban(
  person: &Person,
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  if person.banned {
    return Err(anyhow!("Person is banned from site").into());
  }
  let person_id = person.id;
  let is_banned =
    move |conn: &'_ _| CommunityPersonBanView::get(conn, person_id, community_id).is_ok();
  if blocking(pool, is_banned).await? {
    return Err(anyhow!("Person is banned from community").into());
  }

  Ok(())
}

pub(crate) async fn send_lemmy_activity<T: Serialize>(
  context: &LemmyContext,
  activity: &T,
  activity_id: &Url,
  actor: &dyn ActorType,
  inboxes: Vec<Url>,
  sensitive: bool,
) -> Result<(), LemmyError> {
  if !context.settings().federation.enabled || inboxes.is_empty() {
    return Ok(());
  }

  info!("Sending activity {}", activity_id.to_string());

  // Don't send anything to ourselves
  // TODO: this should be a debug assert
  let hostname = context.settings().get_hostname_without_port()?;
  let inboxes: Vec<&Url> = inboxes
    .iter()
    .filter(|i| i.domain().expect("valid inbox url") != hostname)
    .collect();

  let serialised_activity = serde_json::to_string(&activity)?;

  insert_activity(
    activity_id,
    serialised_activity.clone(),
    true,
    sensitive,
    context.pool(),
  )
  .await?;

  send_activity(
    serialised_activity,
    actor,
    inboxes,
    context.client(),
    context.activity_queue(),
  )
  .await
}
