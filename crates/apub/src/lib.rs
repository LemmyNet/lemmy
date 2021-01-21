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
  object::{ApObject, Note, Page},
};
use activitystreams_ext::{Ext1, Ext2};
use anyhow::{anyhow, Context};
use diesel::NotFound;
use lemmy_db_queries::{source::activity::Activity_, ApubObject, DbPool};
use lemmy_db_schema::source::{
  activity::Activity,
  comment::Comment,
  community::Community,
  post::Post,
  private_message::PrivateMessage,
  user::User_,
};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use std::net::IpAddr;
use url::{ParseError, Url};

/// Activitystreams type for community
type GroupExt = Ext2<ApActor<ApObject<Group>>, GroupExtension, PublicKeyExtension>;
/// Activitystreams type for user
type PersonExt = Ext1<ApActor<ApObject<Person>>, PublicKeyExtension>;
/// Activitystreams type for post
type PageExt = Ext1<ApObject<Page>, PageExtension>;
type NoteExt = ApObject<Note>;

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

/// Common functions for ActivityPub objects, which are implemented by most (but not all) objects
/// and actors in Lemmy.
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

/// Common methods provided by ActivityPub actors (community and user). Not all methods are
/// implemented by all actors.
#[async_trait::async_trait(?Send)]
pub trait ActorType {
  fn actor_id_str(&self) -> String;

  // TODO: every actor should have a public key, so this shouldnt be an option (needs to be fixed in db)
  fn public_key(&self) -> Option<String>;
  fn private_key(&self) -> Option<String>;

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

  async fn send_accept_follow(
    &self,
    follow: Follow,
    context: &LemmyContext,
  ) -> Result<(), LemmyError>;

  async fn send_delete(&self, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_delete(&self, context: &LemmyContext) -> Result<(), LemmyError>;

  async fn send_remove(&self, context: &LemmyContext) -> Result<(), LemmyError>;
  async fn send_undo_remove(&self, context: &LemmyContext) -> Result<(), LemmyError>;

  async fn send_announce(
    &self,
    activity: AnyBase,
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

/// Store a sent or received activity in the database, for logging purposes. These records are not
/// persistent.
pub(crate) async fn insert_activity<T>(
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

pub(crate) enum PostOrComment {
  Comment(Comment),
  Post(Post),
}

/// Tries to find a post or comment in the local database, without any network requests.
/// This is used to handle deletions and removals, because in case we dont have the object, we can
/// simply ignore the activity.
pub(crate) async fn find_post_or_comment_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<PostOrComment, LemmyError> {
  let ap_id = apub_id.to_string();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(p) = post {
    return Ok(PostOrComment::Post(p));
  }

  let ap_id = apub_id.to_string();
  let comment = blocking(context.pool(), move |conn| {
    Comment::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = comment {
    return Ok(PostOrComment::Comment(c));
  }

  Err(NotFound.into())
}

pub(crate) enum Object {
  Comment(Comment),
  Post(Post),
  Community(Community),
  User(User_),
  PrivateMessage(PrivateMessage),
}

pub(crate) async fn find_object_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<Object, LemmyError> {
  if let Ok(pc) = find_post_or_comment_by_id(context, apub_id.to_owned()).await {
    return Ok(match pc {
      PostOrComment::Post(p) => Object::Post(p),
      PostOrComment::Comment(c) => Object::Comment(c),
    });
  }

  let ap_id = apub_id.to_string();
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(u) = user {
    return Ok(Object::User(u));
  }

  let ap_id = apub_id.to_string();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = community {
    return Ok(Object::Community(c));
  }

  let ap_id = apub_id.to_string();
  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(pm) = private_message {
    return Ok(Object::PrivateMessage(pm));
  }

  Err(NotFound.into())
}
