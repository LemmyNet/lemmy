#[macro_use]
extern crate lazy_static;

pub mod activities;
pub mod activity_queue;
pub mod extensions;
pub mod fetcher;
pub mod http;
pub mod inbox;
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
use lemmy_db::{activity::do_insert_activity, user::User_, DbPool};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use url::{ParseError, Url};

type GroupExt = Ext2<ApActor<Group>, GroupExtension, PublicKeyExtension>;
type PersonExt = Ext1<ApActor<Person>, PublicKeyExtension>;
type PageExt = Ext1<Page, PageExtension>;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

// Checks if the ID has a valid format, correct scheme, and is in the allowed instance list.
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

#[async_trait::async_trait(?Send)]
pub trait ToApub {
  type Response;
  async fn to_apub(&self, pool: &DbPool) -> Result<Self::Response, LemmyError>;
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub trait FromApub {
  type ApubType;
  /// Converts an object from ActivityPub type to Lemmy internal type.
  ///
  /// * `apub` The object to read from
  /// * `context` LemmyContext which holds DB pool, HTTP client etc
  /// * `expected_domain` If present, ensure that the apub object comes from the same domain as
  ///                     this URL
  ///
  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    expected_domain: Option<Url>,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

#[async_trait::async_trait(?Send)]
pub trait ApubObjectType {
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;
  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub trait ApubLikeableType {
  async fn send_like(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_dislike(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_like(&self, creator: &User_, context: &LemmyContext)
    -> Result<(), LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub trait ActorType {
  fn actor_id_str(&self) -> String;

  // TODO: every actor should have a public key, so this shouldnt be an option (needs to be fixed in db)
  fn public_key(&self) -> Option<String>;
  fn private_key(&self) -> Option<String>;

  /// numeric id in the database, used for insert_activity
  fn user_id(&self) -> i32;

  // These two have default impls, since currently a community can't follow anything,
  // and a user can't be followed (yet)
  #[allow(unused_variables)]
  async fn send_follow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;
  async fn send_unfollow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  #[allow(unused_variables)]
  async fn send_accept_follow(
    &self,
    follow: Follow,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError>;

  async fn send_announce(
    &self,
    activity: AnyBase,
    sender: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError>;

  fn actor_id(&self) -> Result<Url, ParseError> {
    Url::parse(&self.actor_id_str())
  }

  // TODO move these to the db rows
  fn get_inbox_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/inbox", &self.actor_id_str()))
  }

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

  fn get_outbox_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/outbox", &self.actor_id_str()))
  }

  fn get_followers_url(&self) -> Result<Url, ParseError> {
    Url::parse(&format!("{}/followers", &self.actor_id_str()))
  }

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

pub async fn insert_activity<T>(
  user_id: i32,
  data: T,
  local: bool,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: Serialize + std::fmt::Debug + Send + 'static,
{
  blocking(pool, move |conn| {
    do_insert_activity(conn, user_id, &data, local)
  })
  .await??;
  Ok(())
}
