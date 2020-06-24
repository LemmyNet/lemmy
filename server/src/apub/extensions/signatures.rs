use crate::apub::ActorType;
use activitystreams::ext::Extension;
use actix_web::HttpRequest;
use attohttpc::RequestBuilder;
use failure::Error;
use http_signature_normalization::Config;
use log::debug;
use openssl::{
  hash::MessageDigest,
  pkey::PKey,
  rsa::Rsa,
  sign::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

lazy_static! {
  static ref HTTP_SIG_CONFIG: Config = Config::new();
}

pub struct Keypair {
  pub private_key: String,
  pub public_key: String,
}

/// Generate the asymmetric keypair for ActivityPub HTTP signatures.
pub fn generate_actor_keypair() -> Result<Keypair, Error> {
  let rsa = Rsa::generate(2048)?;
  let pkey = PKey::from_rsa(rsa)?;
  let public_key = pkey.public_key_to_pem()?;
  let private_key = pkey.private_key_to_pem_pkcs8()?;
  Ok(Keypair {
    private_key: String::from_utf8(private_key)?,
    public_key: String::from_utf8(public_key)?,
  })
}

// TODO is it possible to create this signature, with just the url and actor?
/// Signs request headers with the given keypair.
pub fn sign(request: &mut RequestBuilder, actor: &dyn ActorType) -> Result<String, Error> {
  let signing_key_id = format!("{}#main-key", actor.actor_id());

  let headers = request
    .inspect()
    .headers()
    .iter()
    .map(|h| -> Result<(String, String), Error> {
      Ok((h.0.as_str().to_owned(), h.1.to_str()?.to_owned()))
    })
    .collect::<Result<BTreeMap<String, String>, Error>>()?;

  let mut path_and_query = request.inspect().url().path().to_owned();
  if let Some(query) = request.inspect().url().query() {
    path_and_query.push_str(query);
  }

  let signature_header_value = HTTP_SIG_CONFIG
    .begin_sign(
      request.inspect().method().as_str(),
      &path_and_query,
      headers,
    )?
    .sign(signing_key_id, |signing_string| {
      let private_key = PKey::private_key_from_pem(actor.private_key().as_bytes())?;
      let mut signer = Signer::new(MessageDigest::sha256(), &private_key).unwrap();
      signer.update(signing_string.as_bytes()).unwrap();
      Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, Error>
    })?
    .signature_header();

  Ok(signature_header_value)
}

pub fn verify(request: &HttpRequest, actor: &dyn ActorType) -> Result<(), Error> {
  let headers = request
    .headers()
    .iter()
    .map(|h| -> Result<(String, String), Error> {
      Ok((h.0.as_str().to_owned(), h.1.to_str()?.to_owned()))
    })
    .collect::<Result<BTreeMap<String, String>, Error>>()?;

  let verified = HTTP_SIG_CONFIG
    .begin_verify(
      request.method().as_str(),
      request.uri().path_and_query().unwrap().as_str(),
      headers,
    )?
    .verify(|signature, signing_string| -> Result<bool, Error> {
      debug!(
        "Verifying with key {}, message {}",
        &actor.public_key(),
        &signing_string
      );
      let public_key = PKey::public_key_from_pem(actor.public_key().as_bytes())?;
      let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).unwrap();
      verifier.update(&signing_string.as_bytes()).unwrap();
      Ok(verifier.verify(&base64::decode(signature)?)?)
    })?;

  if verified {
    debug!("verified signature for {}", &request.uri());
    Ok(())
  } else {
    Err(format_err!(
      "Invalid signature on request: {}",
      &request.uri()
    ))
  }
}

// The following is taken from here:
// https://docs.rs/activitystreams/0.5.0-alpha.17/activitystreams/ext/index.html

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
  pub id: String,
  pub owner: String,
  pub public_key_pem: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyExtension {
  pub public_key: PublicKey,
}

impl PublicKey {
  pub fn to_ext(&self) -> PublicKeyExtension {
    PublicKeyExtension {
      public_key: self.to_owned(),
    }
  }
}

impl<T> Extension<T> for PublicKeyExtension where T: activitystreams::Actor {}
