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
use crate::api::community::CommunityResponse;
use crate::websocket::server::SendCommunityRoomMessage;
use activitystreams::object::kind::{NoteType, PageType};
use activitystreams::{
  activity::{Accept, Create, Delete, Dislike, Follow, Like, Remove, Undo, Update},
  actor::{kind::GroupType, properties::ApActorProperties, Group, Person},
  collection::UnorderedCollection,
  context,
  endpoint::EndpointProperties,
  ext::{Ext, Extensible},
  link::Mention,
  object::{properties::ObjectProperties, Note, Page, Tombstone},
  public, BaseBox,
};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use failure::Error;
use failure::_core::fmt::Debug;
use isahc::prelude::*;
use itertools::Itertools;
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::api::comment::{send_local_notifs, CommentResponse};
use crate::api::post::PostResponse;
use crate::api::site::SearchResponse;
use crate::api::user::PrivateMessageResponse;
use crate::db::comment::{Comment, CommentForm, CommentLike, CommentLikeForm};
use crate::db::comment_view::CommentView;
use crate::db::community::{
  Community, CommunityFollower, CommunityFollowerForm, CommunityForm, CommunityModerator,
  CommunityModeratorForm,
};
use crate::db::community_view::{CommunityFollowerView, CommunityModeratorView, CommunityView};
use crate::db::post::{Post, PostForm, PostLike, PostLikeForm};
use crate::db::post_view::PostView;
use crate::db::private_message::{PrivateMessage, PrivateMessageForm};
use crate::db::private_message_view::PrivateMessageView;
use crate::db::user::{UserForm, User_};
use crate::db::user_view::UserView;
use crate::db::{activity, Crud, Followable, Joinable, Likeable, SearchType};
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::routes::webfinger::WebFingerResponse;
use crate::routes::{ChatServerParam, DbPoolParam};
use crate::websocket::{
  server::{SendComment, SendPost, SendUserRoomMessage},
  UserOperation,
};
use crate::{convert_datetime, naive_now, scrape_text_for_mentions, MentionData, Settings};

use crate::apub::extensions::group_extensions::GroupExtension;
use crate::apub::extensions::page_extension::PageExtension;
use activities::{populate_object_props, send_activity};
use chrono::NaiveDateTime;
use extensions::signatures::verify;
use extensions::signatures::{sign, PublicKey, PublicKeyExtension};
use fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user};

type GroupExt = Ext<Ext<Ext<Group, GroupExtension>, ApActorProperties>, PublicKeyExtension>;
type PersonExt = Ext<Ext<Person, ApActorProperties>, PublicKeyExtension>;
type PageExt = Ext<Page, PageExtension>;

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

/// Generates the ActivityPub ID for a given object type and name.
///
/// TODO: we will probably need to change apub endpoint urls so that html and activity+json content
///       types are handled at the same endpoint, so that you can copy the url into mastodon search
///       and have it fetch the object.
pub fn make_apub_endpoint(endpoint_type: EndpointType, name: &str) -> Url {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::User => "u",
    EndpointType::Post => "post",
    // TODO I have to change this else my update advanced_migrations crashes the
    // server if a comment exists.
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

// Checks if the ID has a valid format, correct scheme, and is in the whitelist.
fn is_apub_id_valid(apub_id: &Url) -> bool {
  if apub_id.scheme() != get_apub_protocol_string() {
    return false;
  }

  let whitelist: Vec<String> = Settings::get()
    .federation
    .instance_whitelist
    .split(',')
    .map(|d| d.to_string())
    .collect();
  match apub_id.domain() {
    Some(d) => whitelist.contains(&d.to_owned()),
    None => false,
  }
}

// TODO Not sure good names for these
pub trait ToApub {
  type Response;
  fn to_apub(&self, conn: &PgConnection) -> Result<Self::Response, Error>;
  fn to_tombstone(&self) -> Result<Tombstone, Error>;
}

/// Updated is actually the deletion time
fn create_tombstone(
  deleted: bool,
  object_id: &str,
  updated: Option<NaiveDateTime>,
  former_type: String,
) -> Result<Tombstone, Error> {
  if deleted {
    if let Some(updated) = updated {
      let mut tombstone = Tombstone::default();
      tombstone.object_props.set_id(object_id)?;
      tombstone
        .tombstone_props
        .set_former_type_xsd_string(former_type)?
        .set_deleted(convert_datetime(updated))?;
      Ok(tombstone)
    } else {
      Err(format_err!(
        "Cant convert to tombstone because updated time was None."
      ))
    }
  } else {
    Err(format_err!(
      "Cant convert object to tombstone if it wasnt deleted"
    ))
  }
}

pub trait FromApub {
  type ApubType;
  fn from_apub(apub: &Self::ApubType, conn: &PgConnection) -> Result<Self, Error>
  where
    Self: Sized;
}

pub trait ApubObjectType {
  fn send_create(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_update(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error>;
}

pub trait ApubLikeableType {
  fn send_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_dislike(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_undo_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
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

pub trait ActorType {
  fn actor_id(&self) -> String;

  fn public_key(&self) -> String;

  // These two have default impls, since currently a community can't follow anything,
  // and a user can't be followed (yet)
  #[allow(unused_variables)]
  fn send_follow(&self, follow_actor_id: &str, conn: &PgConnection) -> Result<(), Error>;
  fn send_unfollow(&self, follow_actor_id: &str, conn: &PgConnection) -> Result<(), Error>;

  #[allow(unused_variables)]
  fn send_accept_follow(&self, follow: &Follow, conn: &PgConnection) -> Result<(), Error>;

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error>;

  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error>;
  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error>;

  /// For a given community, returns the inboxes of all followers.
  fn get_follower_inboxes(&self, conn: &PgConnection) -> Result<Vec<String>, Error>;

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

pub fn fetch_webfinger_url(mention: &MentionData) -> Result<String, Error> {
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}@{}",
    get_apub_protocol_string(),
    mention.domain,
    mention.name,
    mention.domain
  );
  debug!("Fetching webfinger url: {}", &fetch_url);
  let text = isahc::get(&fetch_url)?.text()?;
  let res: WebFingerResponse = serde_json::from_str(&text)?;
  let link = res
    .links
    .iter()
    .find(|l| l.r#type.eq(&Some("application/activity+json".to_string())))
    .ok_or_else(|| format_err!("No application/activity+json link found."))?;
  link
    .href
    .to_owned()
    .ok_or_else(|| format_err!("No href found."))
}
