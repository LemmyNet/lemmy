pub mod activities;
pub mod community;
pub mod fetcher;
pub mod inbox;
pub mod post;
pub mod signatures;
pub mod user;
use crate::apub::signatures::PublicKeyExtension;
use crate::Settings;
use activitystreams::actor::{properties::ApActorProperties, Group, Person};
use activitystreams::ext::Ext;
use actix_web::body::Body;
use actix_web::HttpResponse;
use openssl::{pkey::PKey, rsa::Rsa};
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

pub struct Instance {
  domain: String,
}

fn create_apub_response<T>(json: &T) -> HttpResponse<Body>
where
  T: serde::ser::Serialize,
{
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(json)
}

// TODO: we will probably need to change apub endpoint urls so that html and activity+json content
//       types are handled at the same endpoint, so that you can copy the url into mastodon search
//       and have it fetch the object.
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

pub fn gen_keypair() -> (Vec<u8>, Vec<u8>) {
  let rsa = Rsa::generate(2048).expect("sign::gen_keypair: key generation error");
  let pkey = PKey::from_rsa(rsa).expect("sign::gen_keypair: parsing error");
  (
    pkey
      .public_key_to_pem()
      .expect("sign::gen_keypair: public key encoding error"),
    pkey
      .private_key_to_pem_pkcs8()
      .expect("sign::gen_keypair: private key encoding error"),
  )
}

pub fn gen_keypair_str() -> (String, String) {
  let (public_key, private_key) = gen_keypair();
  (vec_bytes_to_str(public_key), vec_bytes_to_str(private_key))
}

fn vec_bytes_to_str(bytes: Vec<u8>) -> String {
  String::from_utf8_lossy(&bytes).into_owned()
}

pub fn get_following_instances() -> Vec<Instance> {
  Settings::get()
    .federation
    .followed_instances
    .split(',')
    .map(|i| Instance {
      domain: i.to_string(),
    })
    .collect()
}
