use actix_web::HttpRequest;
use anyhow::anyhow;
use http_signature_normalization_actix::Config as ConfigActix;
use http_signature_normalization_reqwest::prelude::{Config, SignExt};
use once_cell::sync::Lazy;
use openssl::{
  hash::MessageDigest,
  pkey::PKey,
  rsa::Rsa,
  sign::{Signer, Verifier},
};
use reqwest::Request;
use reqwest_middleware::RequestBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Error, ErrorKind};
use tracing::debug;
use url::Url;

static CONFIG2: Lazy<ConfigActix> = Lazy::new(ConfigActix::new);
static HTTP_SIG_CONFIG: Lazy<Config> = Lazy::new(Config::new);

#[derive(Debug)]
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
  let key_to_string = |key| match String::from_utf8(key) {
    Ok(s) => Ok(s),
    Err(e) => Err(Error::new(
      ErrorKind::Other,
      format!("Failed converting key to string: {}", e),
    )),
  };
  Ok(Keypair {
    private_key: key_to_string(private_key)?,
    public_key: key_to_string(public_key)?,
  })
}

/// Creates an HTTP post request to `inbox_url`, with the given `client` and `headers`, and
/// `activity` as request body. The request is signed with `private_key` and then sent.
pub(crate) async fn sign_request(
  request_builder: RequestBuilder,
  activity: String,
  public_key: PublicKey,
  private_key: String,
) -> Result<Request, anyhow::Error> {
  request_builder
    .signature_with_digest(
      HTTP_SIG_CONFIG.clone(),
      public_key.key_id,
      Sha256::new(),
      activity,
      move |signing_string| {
        let private_key = PKey::private_key_from_pem(private_key.as_bytes())?;
        let mut signer = Signer::new(MessageDigest::sha256(), &private_key)?;
        signer.update(signing_string.as_bytes())?;

        Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, anyhow::Error>
      },
    )
    .await
}

/// Verifies the HTTP signature on an incoming inbox request.
pub fn verify_signature(request: &HttpRequest, public_key: &str) -> Result<(), anyhow::Error> {
  let verified = CONFIG2
    .begin_verify(
      request.method(),
      request.uri().path_and_query(),
      request.headers().clone(),
    )?
    .verify(|signature, signing_string| -> Result<bool, anyhow::Error> {
      debug!(
        "Verifying with key {}, message {}",
        &public_key, &signing_string
      );
      let public_key = PKey::public_key_from_pem(public_key.as_bytes())?;
      let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)?;
      verifier.update(signing_string.as_bytes())?;
      Ok(verifier.verify(&base64::decode(signature)?)?)
    })?;

  if verified {
    debug!("verified signature for {}", &request.uri());
    Ok(())
  } else {
    Err(anyhow!("Invalid signature on request: {}", &request.uri()))
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
  pub(crate) key_id: String,
  pub(crate) actor_id: Url,
  pub public_key_pem: String,
}

impl PublicKey {
  /// Create public key with default id, for actors that only have a single keypair
  pub fn new_main_key(actor_id: Url, public_key_pem: String) -> Self {
    let key_id = format!("{}#main-key", &actor_id);
    PublicKey::new(key_id, actor_id, public_key_pem)
  }

  /// Create public key with custom key id. Use this method if there are multiple keypairs per actor
  pub fn new(key_id: String, actor_id: Url, public_key_pem: String) -> Self {
    PublicKey {
      key_id,
      actor_id,
      public_key_pem,
    }
  }
}
