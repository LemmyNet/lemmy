pub mod activities;
pub(crate) mod activity_lists;
pub(crate) mod collections;
mod context;
pub mod fetcher;
pub mod http;
pub mod migrations;
pub mod objects;
pub mod protocol;

#[macro_use]
extern crate lazy_static;

use crate::fetcher::post_or_comment::PostOrComment;
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_apub_lib::webfinger::{webfinger_resolve_actor, WebfingerType};
use lemmy_db_schema::{newtypes::DbUrl, source::activity::Activity, DbPool};
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
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
/// `use_strict_allowlist` should be true only when parsing a remote community, or when parsing a
/// post/comment in a local community.
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
pub async fn get_actor_id_from_name(
  webfinger_type: WebfingerType,
  short_name: &str,
  context: &LemmyContext,
) -> Result<DbUrl, LemmyError> {
  let split = short_name.split('@').collect::<Vec<&str>>();

  let name = split[0];

  // If there's no @, its local
  if split.len() == 1 {
    let domain = context.settings().get_protocol_and_hostname();
    let endpoint_type = match webfinger_type {
      WebfingerType::Person => EndpointType::Person,
      WebfingerType::Group => EndpointType::Community,
    };
    Ok(generate_local_apub_endpoint(endpoint_type, name, &domain)?)
  } else {
    let protocol = context.settings().get_protocol_string();
    Ok(
      webfinger_resolve_actor(name, split[1], webfinger_type, context.client(), protocol)
        .await?
        .into(),
    )
  }
}

/// Store a sent or received activity in the database, for logging purposes. These records are not
/// persistent.
async fn insert_activity<T>(
  ap_id: &Url,
  activity: &T,
  local: bool,
  sensitive: bool,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: Serialize + std::fmt::Debug + Send + 'static,
{
  let data = serde_json::to_value(activity)?;
  let ap_id = ap_id.to_owned().into();
  blocking(pool, move |conn| {
    Activity::insert(conn, ap_id, data, local, sensitive)
  })
  .await??;
  Ok(())
}
