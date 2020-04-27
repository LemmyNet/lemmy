pub mod activities;
pub mod community;
pub mod community_inbox;
pub mod fetcher;
pub mod post;
pub mod signatures;
pub mod user;
pub mod user_inbox;

use activitystreams::{
  activity::{Accept, Create, Follow, Update},
  actor::{properties::ApActorProperties, Actor, Group, Person},
  collection::UnorderedCollection,
  context,
  ext::{Ext, Extensible, Extension},
  object::{properties::ObjectProperties, Page},
  public, BaseBox,
};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use failure::Error;
use failure::_core::fmt::Debug;
use http::request::Builder;
use http_signature_normalization::Config;
use isahc::prelude::*;
use log::debug;
use openssl::hash::MessageDigest;
use openssl::sign::{Signer, Verifier};
use openssl::{pkey::PKey, rsa::Rsa};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use url::Url;

use crate::api::site::SearchResponse;
use crate::db::community::{Community, CommunityFollower, CommunityFollowerForm, CommunityForm};
use crate::db::community_view::{CommunityFollowerView, CommunityView};
use crate::db::post::{Post, PostForm};
use crate::db::post_view::PostView;
use crate::db::user::{UserForm, User_};
use crate::db::user_view::UserView;
use crate::db::{Crud, Followable, SearchType};
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::routes::{ChatServerParam, DbPoolParam};
use crate::{convert_datetime, naive_now, Settings};

use activities::accept_follow;
use fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user};
use signatures::verify;
use signatures::{sign, PublicKey, PublicKeyExtension};

type GroupExt = Ext<Ext<Group, ApActorProperties>, PublicKeyExtension>;
type PersonExt = Ext<Ext<Person, ApActorProperties>, PublicKeyExtension>;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

pub enum EndpointType {
  Community,
  User,
  Post,
  Comment,
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
}

pub trait FromApub {
  type ApubType;
  fn from_apub(apub: &Self::ApubType, conn: &PgConnection) -> Result<Self, Error>
  where
    Self: Sized;
}

pub trait ActorType {
  fn actor_id(&self) -> String;

  fn public_key(&self) -> String;

  fn get_inbox_url(&self) -> String {
    format!("{}/inbox", &self.actor_id())
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
