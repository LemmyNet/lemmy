use crate::APUB_JSON_CONTENT_TYPE;
use activitystreams::chrono::Utc;
use actix_web::HttpRequest;
use anyhow::anyhow;
use http::{header::HeaderName, HeaderMap, HeaderValue};
use http_signature_normalization_actix::Config as ConfigActix;
use http_signature_normalization_reqwest::prelude::{Config, SignExt};
use lemmy_utils::LemmyError;
use log::debug;
use openssl::{
  hash::MessageDigest,
  pkey::PKey,
  sign::{Signer, Verifier},
};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use url::Url;

lazy_static! {
  static ref CONFIG2: ConfigActix = ConfigActix::new();
  static ref HTTP_SIG_CONFIG: Config = Config::new().mastodon_compat();
}

/// Creates an HTTP post request to `inbox_url`, with the given `client` and `headers`, and
/// `activity` as request body. The request is signed with `private_key` and then sent.
pub async fn sign_and_send(
  client: &Client,
  inbox_url: &Url,
  activity: String,
  actor_id: &Url,
  private_key: String,
) -> Result<Response, LemmyError> {
  let signing_key_id = format!("{}#main-key", actor_id);

  let mut headers = HeaderMap::new();
  let mut host = inbox_url.domain().expect("read inbox domain").to_string();
  if let Some(port) = inbox_url.port() {
    host = format!("{}:{}", host, port);
  }
  headers.insert(
    HeaderName::from_str("Content-Type")?,
    HeaderValue::from_str(APUB_JSON_CONTENT_TYPE)?,
  );
  headers.insert(HeaderName::from_str("Host")?, HeaderValue::from_str(&host)?);
  // Need to use legacy timezone because mastodon and doesnt understand any new standards
  let date = Utc::now().to_rfc2822().replace("+0000", "GMT");
  headers.insert(HeaderName::from_str("Date")?, HeaderValue::from_str(&date)?);

  let response = client
    .post(&inbox_url.to_string())
    .headers(headers)
    .signature_with_digest(
      HTTP_SIG_CONFIG.clone(),
      signing_key_id,
      Sha256::new(),
      activity,
      move |signing_string| {
        let private_key = PKey::private_key_from_pem(private_key.as_bytes())?;
        let mut signer = Signer::new(MessageDigest::sha256(), &private_key)?;
        signer.update(signing_string.as_bytes())?;

        Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, LemmyError>
      },
    )
    .await?;

  Ok(response)
}

/// Verifies the HTTP signature on an incoming inbox request.
pub fn verify_signature(request: &HttpRequest, public_key: &str) -> Result<(), LemmyError> {
  let verified = CONFIG2
    .begin_verify(
      request.method(),
      request.uri().path_and_query(),
      request.headers().clone(),
    )?
    .verify(|signature, signing_string| -> Result<bool, LemmyError> {
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
    Err(anyhow!("Invalid signature on request: {}", &request.uri()).into())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
  pub(crate) id: String,
  pub(crate) owner: Box<Url>,
  pub public_key_pem: String,
}
