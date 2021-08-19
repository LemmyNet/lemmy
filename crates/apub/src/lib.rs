#[macro_use]
extern crate lazy_static;

pub mod activities;
pub mod activity_queue;
pub mod extensions;
pub mod fetcher;
pub mod http;
pub mod migrations;
pub mod objects;

use crate::extensions::signatures::PublicKey;
use anyhow::{anyhow, Context};
use diesel::NotFound;
use lemmy_api_common::blocking;
use lemmy_db_queries::{source::activity::Activity_, ApubObject, DbPool};
use lemmy_db_schema::{
  source::{
    activity::Activity,
    comment::Comment,
    community::Community,
    person::{Person as DbPerson, Person},
    post::Post,
    private_message::PrivateMessage,
  },
  CommunityId,
  DbUrl,
};
use lemmy_db_views_actor::community_person_ban_view::CommunityPersonBanView;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use std::net::IpAddr;
use url::{ParseError, Url};

static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

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
) -> Result<(), LemmyError> {
  let settings = Settings::get();
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

  if apub_id.scheme() != Settings::get().get_protocol_string() {
    return Err(anyhow!("invalid apub id scheme {}: {}", apub_id.scheme(), apub_id).into());
  }

  // TODO: might be good to put the part above in one method, and below in another
  //       (which only gets called in apub::objects)
  //        -> no that doesnt make sense, we still need the code below for blocklist and strict allowlist
  if let Some(blocked) = Settings::get().federation.blocked_instances {
    if blocked.contains(&domain) {
      return Err(anyhow!("{} is in federation blocklist", domain).into());
    }
  }

  if let Some(mut allowed) = Settings::get().federation.allowed_instances {
    // Only check allowlist if this is a community, or strict allowlist is enabled.
    let strict_allowlist = Settings::get().federation.strict_allowlist;
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

/// Common methods provided by ActivityPub actors (community and person). Not all methods are
/// implemented by all actors.
trait ActorType {
  fn is_local(&self) -> bool;
  fn actor_id(&self) -> Url;
  fn name(&self) -> String;

  // TODO: every actor should have a public key, so this shouldnt be an option (needs to be fixed in db)
  fn public_key(&self) -> Option<String>;
  fn private_key(&self) -> Option<String>;

  fn get_shared_inbox_or_inbox_url(&self) -> Url;

  /// Outbox URL is not generally used by Lemmy, so it can be generated on the fly (but only for
  /// local actors).
  fn get_outbox_url(&self) -> Result<Url, LemmyError> {
    /* TODO
    if !self.is_local() {
      return Err(anyhow!("get_outbox_url() called for remote actor").into());
    }
    */
    Ok(Url::parse(&format!("{}/outbox", &self.actor_id()))?)
  }

  fn get_public_key(&self) -> Result<PublicKey, LemmyError> {
    Ok(PublicKey {
      id: format!("{}#main-key", self.actor_id()),
      owner: self.actor_id(),
      public_key_pem: self.public_key().context(location_info!())?,
    })
  }
}

#[async_trait::async_trait(?Send)]
pub trait CommunityType {
  fn followers_url(&self) -> Url;
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError>;
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
) -> Result<DbUrl, ParseError> {
  generate_apub_endpoint_for_domain(
    endpoint_type,
    name,
    &Settings::get().get_protocol_and_hostname(),
  )
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

fn generate_moderators_url(community_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  Ok(Url::parse(&format!("{}/moderators", community_id))?.into())
}

/// Takes in a shortname of the type dessalines@xyz.tld or dessalines (assumed to be local), and outputs the actor id.
/// Used in the API for communities and users.
pub fn build_actor_id_from_shortname(
  endpoint_type: EndpointType,
  short_name: &str,
) -> Result<DbUrl, ParseError> {
  let split = short_name.split('@').collect::<Vec<&str>>();

  let name = split[0];

  // If there's no @, its local
  let domain = if split.len() == 1 {
    Settings::get().get_protocol_and_hostname()
  } else {
    format!("{}://{}", Settings::get().get_protocol_string(), split[1])
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

pub enum PostOrComment {
  Comment(Box<Comment>),
  Post(Box<Post>),
}

impl PostOrComment {
  pub(crate) fn ap_id(&self) -> Url {
    match self {
      PostOrComment::Post(p) => p.ap_id.clone(),
      PostOrComment::Comment(c) => c.ap_id.clone(),
    }
    .into()
  }
}

/// Tries to find a post or comment in the local database, without any network requests.
/// This is used to handle deletions and removals, because in case we dont have the object, we can
/// simply ignore the activity.
pub(crate) async fn find_post_or_comment_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<PostOrComment, LemmyError> {
  let ap_id = apub_id.clone();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(p) = post {
    return Ok(PostOrComment::Post(Box::new(p)));
  }

  let ap_id = apub_id.clone();
  let comment = blocking(context.pool(), move |conn| {
    Comment::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(c) = comment {
    return Ok(PostOrComment::Comment(Box::new(c)));
  }

  Err(NotFound.into())
}

#[derive(Debug)]
enum Object {
  Comment(Box<Comment>),
  Post(Box<Post>),
  Community(Box<Community>),
  Person(Box<DbPerson>),
  PrivateMessage(Box<PrivateMessage>),
}

async fn find_object_by_id(context: &LemmyContext, apub_id: Url) -> Result<Object, LemmyError> {
  let ap_id = apub_id.clone();
  if let Ok(pc) = find_post_or_comment_by_id(context, ap_id.to_owned()).await {
    return Ok(match pc {
      PostOrComment::Post(p) => Object::Post(Box::new(*p)),
      PostOrComment::Comment(c) => Object::Comment(Box::new(*c)),
    });
  }

  let ap_id = apub_id.clone();
  let person = blocking(context.pool(), move |conn| {
    DbPerson::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(u) = person {
    return Ok(Object::Person(Box::new(u)));
  }

  let ap_id = apub_id.clone();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(c) = community {
    return Ok(Object::Community(Box::new(c)));
  }

  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &apub_id.into())
  })
  .await?;
  if let Ok(pm) = private_message {
    return Ok(Object::PrivateMessage(Box::new(pm)));
  }

  Err(NotFound.into())
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
