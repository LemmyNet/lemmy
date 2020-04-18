use activitystreams::{actor::Actor, ext::Extension};
use failure::Error;
use http::request::Builder;
use http_signature_normalization::Config;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::{pkey::PKey, rsa::Rsa};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Keypair {
  pub private_key: String,
  pub public_key: String,
}

/// Generate the asymmetric keypair for ActivityPub HTTP signatures.
pub fn generate_actor_keypair() -> Keypair {
  let rsa = Rsa::generate(2048).expect("sign::gen_keypair: key generation error");
  let pkey = PKey::from_rsa(rsa).expect("sign::gen_keypair: parsing error");
  let public_key = pkey
    .public_key_to_pem()
    .expect("sign::gen_keypair: public key encoding error");
  let private_key = pkey
    .private_key_to_pem_pkcs8()
    .expect("sign::gen_keypair: private key encoding error");
  Keypair {
    private_key: String::from_utf8_lossy(&private_key).into_owned(),
    public_key: String::from_utf8_lossy(&public_key).into_owned(),
  }
}

/// Signs request headers with the given keypair.
pub fn sign(request: &Builder, keypair: &Keypair, sender_id: &str) -> Result<String, Error> {
  let signing_key_id = format!("{}#main-key", sender_id);
  let config = Config::new();

  let headers = request
    .headers_ref()
    .unwrap()
    .iter()
    .map(|h| -> Result<(String, String), Error> {
      Ok((h.0.as_str().to_owned(), h.1.to_str()?.to_owned()))
    })
    .collect::<Result<BTreeMap<String, String>, Error>>()?;

  let signature_header_value = config
    .begin_sign(
      request.method_ref().unwrap().as_str(),
      request
        .uri_ref()
        .unwrap()
        .path_and_query()
        .unwrap()
        .as_str(),
      headers,
    )
    .sign(signing_key_id, |signing_string| {
      let private_key = PKey::private_key_from_pem(keypair.private_key.as_bytes())?;
      let mut signer = Signer::new(MessageDigest::sha256(), &private_key).unwrap();
      signer.update(signing_string.as_bytes()).unwrap();
      Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, Error>
    })?
    .signature_header();

  Ok(signature_header_value)
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

impl<T> Extension<T> for PublicKeyExtension where T: Actor {}
