pub mod activities;
pub mod comment;
pub mod community;
pub mod community_inbox;
pub mod extensions;
pub mod fetcher;
pub mod post;
pub mod private_message;
pub mod shared_inbox;
pub mod user;
pub mod user_inbox;

use crate::{
  apub::extensions::{
    group_extensions::GroupExtension,
    page_extension::PageExtension,
    signatures::{PublicKey, PublicKeyExtension},
  },
  convert_datetime,
  db::user::User_,
  request::{retry, RecvError},
  routes::webfinger::WebFingerResponse,
  DbPool,
  LemmyError,
  MentionData,
  Settings,
};
use activitystreams::{
  actor::{properties::ApActorProperties, Group, Person},
  object::Page,
};
use activitystreams_ext::{Ext1, Ext2, Ext3};
use activitystreams_new::{activity::Follow, object::Tombstone, prelude::*};
use actix_web::{body::Body, client::Client, HttpResponse};
use chrono::NaiveDateTime;
use log::debug;
use serde::Serialize;
use url::Url;

type GroupExt = Ext3<Group, GroupExtension, ApActorProperties, PublicKeyExtension>;
type PersonExt = Ext2<Person, ApActorProperties, PublicKeyExtension>;
type PageExt = Ext1<Page, PageExtension>;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

pub enum EndpointType {
  Community,
  User,
  Post,
  Comment,
  PrivateMessage,
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
fn create_apub_response<T>(data: &T) -> HttpResponse<Body>
where
  T: Serialize,
{
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(data)
}

fn create_apub_tombstone_response<T>(data: &T) -> HttpResponse<Body>
where
  T: Serialize,
{
  HttpResponse::Gone()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(data)
}

/// Generates the ActivityPub ID for a given object type and ID.
pub fn make_apub_endpoint(endpoint_type: EndpointType, name: &str) -> Url {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::User => "u",
    EndpointType::Post => "post",
    EndpointType::Comment => "comment",
    EndpointType::PrivateMessage => "private_message",
  };

  Url::parse(&format!(
    "{}://{}/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    point,
    name
  ))
  .unwrap()
}

pub fn get_apub_protocol_string() -> &'static str {
  if Settings::get().federation.tls_enabled {
    "https"
  } else {
    "http"
  }
}

// Checks if the ID has a valid format, correct scheme, and is in the allowed instance list.
fn is_apub_id_valid(apub_id: &Url) -> bool {
  debug!("Checking {}", apub_id);
  if apub_id.scheme() != get_apub_protocol_string() {
    debug!("invalid scheme: {:?}", apub_id.scheme());
    return false;
  }

  let allowed_instances: Vec<String> = Settings::get()
    .federation
    .allowed_instances
    .split(',')
    .map(|d| d.to_string())
    .collect();
  match apub_id.domain() {
    Some(d) => {
      let contains = allowed_instances.contains(&d.to_owned());

      if !contains {
        debug!("{} not in {:?}", d, allowed_instances);
      }

      contains
    }
    None => {
      debug!("missing domain");
      false
    }
  }
}

#[async_trait::async_trait(?Send)]
pub trait ToApub {
  type Response;
  async fn to_apub(&self, pool: &DbPool) -> Result<Self::Response, LemmyError>;
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError>;
}

/// Updated is actually the deletion time
fn create_tombstone(
  deleted: bool,
  object_id: &str,
  updated: Option<NaiveDateTime>,
  former_type: String,
) -> Result<Tombstone, LemmyError> {
  if deleted {
    if let Some(updated) = updated {
      let mut tombstone = Tombstone::new();
      tombstone.set_id(object_id.parse()?);
      tombstone.set_former_type(former_type);
      tombstone.set_deleted(convert_datetime(updated).into());
      Ok(tombstone)
    } else {
      Err(format_err!("Cant convert to tombstone because updated time was None.").into())
    }
  } else {
    Err(format_err!("Cant convert object to tombstone if it wasnt deleted").into())
  }
}

#[async_trait::async_trait(?Send)]
pub trait FromApub {
  type ApubType;
  async fn from_apub(
    apub: &Self::ApubType,
    client: &Client,
    pool: &DbPool,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

#[async_trait::async_trait(?Send)]
pub trait ApubObjectType {
  async fn send_create(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_update(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_undo_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub trait ApubLikeableType {
  async fn send_like(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_dislike(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_undo_like(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
}

pub fn get_shared_inbox(actor_id: &str) -> String {
  let url = Url::parse(actor_id).unwrap();
  format!(
    "{}://{}{}/inbox",
    &url.scheme(),
    &url.host_str().unwrap(),
    if let Some(port) = url.port() {
      format!(":{}", port)
    } else {
      "".to_string()
    },
  )
}

#[async_trait::async_trait(?Send)]
pub trait ActorType {
  fn actor_id(&self) -> String;

  fn public_key(&self) -> String;
  fn private_key(&self) -> String;

  // These two have default impls, since currently a community can't follow anything,
  // and a user can't be followed (yet)
  #[allow(unused_variables)]
  async fn send_follow(
    &self,
    follow_actor_id: &str,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_unfollow(
    &self,
    follow_actor_id: &str,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;

  #[allow(unused_variables)]
  async fn send_accept_follow(
    &self,
    follow: &Follow,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;

  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;

  async fn send_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;
  async fn send_undo_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError>;

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<String>, LemmyError>;

  // TODO move these to the db rows
  fn get_inbox_url(&self) -> String {
    format!("{}/inbox", &self.actor_id())
  }

  fn get_shared_inbox_url(&self) -> String {
    get_shared_inbox(&self.actor_id())
  }

  fn get_outbox_url(&self) -> String {
    format!("{}/outbox", &self.actor_id())
  }

  fn get_followers_url(&self) -> String {
    format!("{}/followers", &self.actor_id())
  }

  fn get_following_url(&self) -> String {
    format!("{}/following", &self.actor_id())
  }

  fn get_liked_url(&self) -> String {
    format!("{}/liked", &self.actor_id())
  }

  fn get_public_key_ext(&self) -> PublicKeyExtension {
    PublicKey {
      id: format!("{}#main-key", self.actor_id()),
      owner: self.actor_id(),
      public_key_pem: self.public_key(),
    }
    .to_ext()
  }
}

pub async fn fetch_webfinger_url(
  mention: &MentionData,
  client: &Client,
) -> Result<String, LemmyError> {
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}@{}",
    get_apub_protocol_string(),
    mention.domain,
    mention.name,
    mention.domain
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  let mut response = retry(|| client.get(&fetch_url).send()).await?;

  let res: WebFingerResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  let link = res
    .links
    .iter()
    .find(|l| l.type_.eq(&Some("application/activity+json".to_string())))
    .ok_or_else(|| format_err!("No application/activity+json link found."))?;
  link
    .href
    .to_owned()
    .ok_or_else(|| format_err!("No href found.").into())
}
