// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::fetcher::post_or_comment::PostOrComment;
use anyhow::{anyhow, Context};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{newtypes::DbUrl, source::activity::Activity, utils::DbPool};
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use serde::{Deserialize, Deserializer};
use std::net::IpAddr;
use url::{ParseError, Url};

pub mod activities;
pub(crate) mod activity_lists;
pub(crate) mod collections;
mod context;
pub mod fetcher;
pub mod http;
pub(crate) mod mentions;
pub mod objects;
pub mod protocol;

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
#[tracing::instrument(skip(settings))]
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
      let err = anyhow!(
        "Trying to connect with {}, but federation is disabled",
        domain
      );
      Err(LemmyError::from_error_message(err, "federation_disabled"))
    };
  }

  let host = apub_id.host_str().context(location_info!())?;
  let host_as_ip = host.parse::<IpAddr>();
  if host == "localhost" || host_as_ip.is_ok() {
    let err = anyhow!("invalid hostname {}: {}", host, apub_id);
    return Err(LemmyError::from_error_message(err, "invalid_hostname"));
  }

  if apub_id.scheme() != settings.get_protocol_string() {
    let err = anyhow!("invalid apub id scheme {}: {}", apub_id.scheme(), apub_id);
    return Err(LemmyError::from_error_message(err, "invalid_scheme"));
  }

  // TODO: might be good to put the part above in one method, and below in another
  //       (which only gets called in apub::objects)
  //        -> no that doesnt make sense, we still need the code below for blocklist and strict allowlist
  if let Some(blocked) = settings.to_owned().federation.blocked_instances {
    if blocked.contains(&domain) {
      let err = anyhow!("{} is in federation blocklist", domain);
      return Err(LemmyError::from_error_message(err, "federation_blocked"));
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
        let err = anyhow!("{} not in federation allowlist", domain);
        return Err(LemmyError::from_error_message(
          err,
          "federation_not_allowed",
        ));
      }
    }
  }

  Ok(())
}

pub(crate) fn deserialize_one_or_many<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
  T: Deserialize<'de>,
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
  }

  let result: OneOrMany<T> = Deserialize::deserialize(deserializer)?;
  Ok(match result {
    OneOrMany::Many(list) => list,
    OneOrMany::One(value) => vec![value],
  })
}

pub(crate) fn deserialize_one<'de, T, D>(deserializer: D) -> Result<[T; 1], D::Error>
where
  T: Deserialize<'de>,
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum MaybeArray<T> {
    Simple(T),
    Array([T; 1]),
  }

  let result: MaybeArray<T> = Deserialize::deserialize(deserializer)?;
  Ok(match result {
    MaybeArray::Simple(value) => [value],
    MaybeArray::Array(value) => value,
  })
}

pub(crate) fn deserialize_skip_error<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: Deserialize<'de> + Default,
  D: Deserializer<'de>,
{
  let result = Deserialize::deserialize(deserializer);
  Ok(match result {
    Ok(o) => o,
    Err(_) => Default::default(),
  })
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
      Activity::insert(conn, ap_id, activity, local, sensitive)
    })
    .await??,
  )
}
