pub mod activities;
pub mod community;
pub mod community_inbox;
pub mod fetcher;
pub mod post;
pub mod signatures;
pub mod user;
pub mod user_inbox;
use crate::apub::signatures::PublicKeyExtension;
use crate::Settings;
use activitystreams::actor::{properties::ApActorProperties, Group, Person};
use activitystreams::ext::Ext;
use actix_web::body::Body;
use actix_web::HttpResponse;
use openssl::{pkey::PKey, rsa::Rsa};
use serde::ser::Serialize;
use url::Url;

type GroupExt = Ext<Ext<Group, ApActorProperties>, PublicKeyExtension>;
type PersonExt = Ext<Ext<Person, ApActorProperties>, PublicKeyExtension>;

static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

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
    EndpointType::Post => "p",
    // TODO I have to change this else my update advanced_migrations crashes the
    // server if a comment exists.
    EndpointType::Comment => "comment",
  };

  Url::parse(&format!(
    "{}://{}/federation/{}/{}",
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

/// Generate the asymmetric keypair for ActivityPub HTTP signatures.
pub fn gen_keypair_str() -> (String, String) {
  let rsa = Rsa::generate(2048).expect("sign::gen_keypair: key generation error");
  let pkey = PKey::from_rsa(rsa).expect("sign::gen_keypair: parsing error");
  let public_key = pkey
    .public_key_to_pem()
    .expect("sign::gen_keypair: public key encoding error");
  let private_key = pkey
    .private_key_to_pem_pkcs8()
    .expect("sign::gen_keypair: private key encoding error");
  (vec_bytes_to_str(public_key), vec_bytes_to_str(private_key))
}

fn vec_bytes_to_str(bytes: Vec<u8>) -> String {
  String::from_utf8_lossy(&bytes).into_owned()
}
