pub mod activities;
pub mod community;
pub mod community_inbox;
pub mod fetcher;
pub mod post;
pub mod signatures;
pub mod user;
pub mod user_inbox;

use activitystreams::{
  context, public, BaseBox,
  actor::{
    Actor,
    Person,
    Group,
    properties::ApActorProperties, 
  },
  activity::{Accept, Create, Follow, Update},
  object::{
    Page,
    properties::ObjectProperties,
  },
  ext::{
    Ext,
    Extensible,
    Extension,
  },
  collection::{
    UnorderedCollection, 
    OrderedCollection,
  },
};
use actix_web::body::Body;
use actix_web::{web, Result, HttpRequest, HttpResponse};
use actix_web::web::Path;
use url::Url;
use failure::Error;
use failure::_core::fmt::Debug;
use log::debug;
use isahc::prelude::*;
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use http::request::Builder;
use http_signature_normalization::Config;
use openssl::hash::MessageDigest;
use openssl::sign::{Signer, Verifier};
use openssl::{pkey::PKey, rsa::Rsa};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::routes::{DbPoolParam, ChatServerParam};
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::{convert_datetime, naive_now, Settings};
use crate::db::community::{Community, CommunityForm, CommunityFollower, CommunityFollowerForm};
use crate::db::community_view::{CommunityFollowerView, CommunityView};
use crate::db::post::{Post, PostForm};
use crate::db::post_view::PostView;
use crate::db::user::{UserForm, User_};
use crate::db::user_view::UserView;
// TODO check on unpooled connection
use crate::db::{Crud, Followable, SearchType, establish_unpooled_connection};
use crate::api::site::SearchResponse;

use signatures::{PublicKey, PublicKeyExtension, sign};
use activities::accept_follow;
use signatures::verify;
use fetcher::{fetch_remote_object, get_or_fetch_and_upsert_remote_user, get_or_fetch_and_upsert_remote_community};

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
