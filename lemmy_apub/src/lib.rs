//! The Lemmy apub crate
#![deny(missing_docs)]
#[macro_use]
extern crate lazy_static;

/// The activities
pub mod activities;
/// The activity queue
pub mod activity_queue;
/// The apub extensions
pub mod extensions;
/// The apub fetcher
pub mod fetcher;
/// HTTP endpoints
pub mod http;
/// The apub inbox
pub mod inbox;
/// The apub objects
pub mod objects;

use crate::extensions::{
  group_extensions::GroupExtension,
  page_extension::PageExtension,
  signatures::{PublicKey, PublicKeyExtension},
};
use activitystreams::{
  activity::Follow,
  actor::{ApActor, Group, Person},
  base::AnyBase,
  object::{Page, Tombstone},
};
use activitystreams_ext::{Ext1, Ext2};
use anyhow::{anyhow, Context};
use lemmy_db::{activity::Activity, user::User_, DbPool};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use std::net::IpAddr;
use url::{ParseError, Url};

/// Activitystreams type for community
type GroupExt = Ext2<ApActor<Group>, GroupExtension, PublicKeyExtension>;
/// Activitystreams type for user
type PersonExt = Ext1<ApActor<Person>, PublicKeyExtension>;
/// Activitystreams type for post
type PageExt = Ext1<Page, PageExtension>;

/// The apub json content type
pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

/// Checks if the ID is allowed for sending or receiving.
///
/// In particular, it checks for:
/// - federation being enabled (if its disabled, only local URLs are allowed)
/// - the correct scheme (either http or https)
/// - URL being in the allowlist (if it is active)
/// - URL not being in the blocklist (if it is active)
///
/// Note that only one of allowlist and blacklist can be enabled, not both.
fn check_is_apub_id_valid(apub_id: &Url) -> Result<(), LemmyError> {
  let settings = Settings::get();
  let domain = apub_id.domain().context(location_info!())?.to_string();
  let local_instance = settings
    .hostname
    .split(':')
    .collect::<Vec<&str>>()
    .first()
    .context(location_info!())?
    .to_string();

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
    return Err(anyhow!("invalid hostname: {:?}", host).into());
  }

  if apub_id.scheme() != Settings::get().get_protocol_string() {
    return Err(anyhow!("invalid apub id scheme: {:?}", apub_id.scheme()).into());
  }

  let mut allowed_instances = Settings::get().get_allowed_instances();
  let blocked_instances = Settings::get().get_blocked_instances();
  if allowed_instances.is_empty() && blocked_instances.is_empty() {
    Ok(())
  } else if !allowed_instances.is_empty() {
    // need to allow this explicitly because apub receive might contain objects from our local
    // instance. split is needed to remove the port in our federation test setup.
    allowed_instances.push(local_instance);

    if allowed_instances.contains(&domain) {
      Ok(())
    } else {
      Err(anyhow!("{} not in federation allowlist", domain).into())
    }
  } else if !blocked_instances.is_empty() {
    if blocked_instances.contains(&domain) {
      Err(anyhow!("{} is in federation blocklist", domain).into())
    } else {
      Ok(())
    }
  } else {
    panic!("Invalid config, both allowed_instances and blocked_instances are specified");
  }
}

/// Trait for converting an object or actor into the respective ActivityPub type.
#[async_trait::async_trait(?Send)]
pub trait ToApub {
  /// The apub type
  type ApubType;
  /// Converts to an apub object
  async fn to_apub(&self, pool: &DbPool) -> Result<Self::ApubType, LemmyError>;
  /// Creates a tombstone from a lemmy object
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError>;
}

/// Trait for converting from an apub object into a lemmy object
#[async_trait::async_trait(?Send)]
pub trait FromApub {
  /// The apub type
  type ApubType;
  /// Converts an object from ActivityPub type to Lemmy internal type.
  ///
  /// * `apub` The object to read from
  /// * `context` LemmyContext which holds DB pool, HTTP client etc
  /// * `expected_domain` If present, ensure that the domains of this and of the apub object ID are
  ///                     identical
  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    expected_domain: Option<Url>,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

/// Common functions for ActivityPub objects, which are implemented by most (but not all) objects
/// and actors in Lemmy.
#[async_trait::async_trait(?Send)]
pub trait ApubObjectType {
  /// Sends a create
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an update
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends a delete
  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an undo delete
  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;
  /// Sends a remove
  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an undo remove
  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
}

#[async_trait::async_trait(?Send)]
/// A trait for a likeable apub type
pub trait ApubLikeableType {
  /// Sends a like
  async fn send_like(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends a dislike
  async fn send_dislike(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an undo dislike
  async fn send_undo_like(&self, creator: &User_, context: &LemmyContext)
    -> Result<(), LemmyError>;
}

/// Common methods provided by ActivityPub actors (community and user). Not all methods are
/// implemented by all actors.
#[async_trait::async_trait(?Send)]
pub trait ActorType {
  /// The actor id as a string
  fn actor_id_str(&self) -> String;

  // TODO: every actor should have a public key, so this shouldnt be an option (needs to be fixed in db)
  /// The public key
  fn public_key(&self) -> Option<String>;
  /// The private key
  fn private_key(&self) -> Option<String>;

  /// Sends a follow
  async fn send_follow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  /// Sends an unfollow
  async fn send_unfollow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  /// Sends an accept follow
  async fn send_accept_follow(
    &self,
    follow: Follow,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  /// Sends a delete
  async fn send_delete(&self, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an undo delete
  async fn send_undo_delete(&self, context: &LemmyContext) -> Result<(), LemmyError>;

  /// Sends a remove
  async fn send_remove(&self, context: &LemmyContext) -> Result<(), LemmyError>;
  /// Sends an undo remove
  async fn send_undo_remove(&self, context: &LemmyContext) -> Result<(), LemmyError>;

  /// Sends an announce
  async fn send_announce(
    &self,
    activity: AnyBase,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError>;

  /// Gets the actor id as a URL
  fn actor_id(&self) -> Result<Url, ParseError> {
    Url::parse(&self.actor_id_str())
  }

  // TODO move these to the db rows
  /// Gets the inbox as a URL
  fn get_inbox_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/inbox", &self.actor_id_str()))
  }

  /// Gets the shared inbox as a URL
  fn get_shared_inbox_url(&self) -> Result<Url, LemmyError> {
    let actor_id = self.actor_id()?;
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
    Ok(Url::parse(&url)?)
  }

  /// Gets the outbox as a URL
  fn get_outbox_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/outbox", &self.actor_id_str()))
  }

  /// Gets the folloers as a URL
  fn get_followers_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/followers", &self.actor_id_str()))
  }

  /// Gets the public key
  fn get_public_key_ext(&self) -> Result<PublicKeyExtension, LemmyError> {
    Ok(
      PublicKey {
        id: format!("{}#main-key", self.actor_id_str()),
        owner: self.actor_id_str(),
        public_key_pem: self.public_key().context(location_info!())?,
      }
      .to_ext(),
    )
  }
}

/// Store a sent or received activity in the database, for logging purposes. These records are not
/// persistent.
pub async fn insert_activity<T>(
  ap_id: &Url,
  activity: T,
  local: bool,
  sensitive: bool,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: Serialize + std::fmt::Debug + Send + 'static,
{
  let ap_id = ap_id.to_string();
  blocking(pool, move |conn| {
    Activity::insert(conn, ap_id, &activity, local, sensitive)
  })
  .await??;
  Ok(())
}
