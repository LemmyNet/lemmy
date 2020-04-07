pub mod community;
pub mod post;
pub mod puller;
pub mod user;
use crate::Settings;
use openssl::{pkey::PKey, rsa::Rsa};

use activitystreams::actor::{properties::ApActorProperties, Group};
use activitystreams::ext::Ext;
use actix_web::body::Body;
use actix_web::HttpResponse;
use url::Url;

type GroupExt = Ext<Group, ApActorProperties>;

fn create_apub_response<T>(json: &T) -> HttpResponse<Body>
where
  T: serde::ser::Serialize,
{
  HttpResponse::Ok()
    .content_type("application/activity+json")
    .json(json)
}

pub enum EndpointType {
  Community,
  User,
  Post,
  Comment,
}

// TODO: we will probably need to change apub endpoint urls so that html and activity+json content
//       types are handled at the same endpoint, so that you can copy the url into mastodon search
//       and have it fetch the object.
pub fn make_apub_endpoint(endpoint_type: EndpointType, name: &str) -> Url {
  let point = match endpoint_type {
    EndpointType::Community => "community",
    EndpointType::User => "user",
    EndpointType::Post => "post",
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

/// If community is on local instance, don't include the @instance part. This is only for displaying
/// to the user and should never be used otherwise.
pub fn format_community_name(name: &str, instance: &str) -> String {
  if instance == Settings::get().hostname {
    format!("!{}", name)
  } else {
    format!("!{}@{}", name, instance)
  }
}

pub fn get_following_instances() -> Vec<&'static str> {
  Settings::get()
    .federation
    .followed_instances
    .split(',')
    .collect()
}

/// Returns a tuple of (username, domain) from an identifier like "main@dev.lemmy.ml"
fn split_identifier(identifier: &str) -> (String, String) {
  let x: Vec<&str> = identifier.split('@').collect();
  (x[0].replace("!", ""), x[1].to_string())
}

fn get_remote_community_uri(identifier: &str) -> String {
  let (name, domain) = split_identifier(identifier);
  format!("http://{}/federation/c/{}", domain, name)
}
