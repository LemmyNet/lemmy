use crate::{apub::ActorType, LemmyError};
use activitystreams_ext::UnparsedExtension;
use activitystreams_new::unparsed::UnparsedMutExt;
use actix_web::{client::ClientRequest, HttpRequest};
use http_signature_normalization_actix::{
  digest::{DigestClient, SignExt},
  Config,
};
use log::debug;
use openssl::{
  hash::MessageDigest,
  pkey::PKey,
  sign::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

lazy_static! {
  static ref HTTP_SIG_CONFIG: Config = Config::new();
}

/// Signs request headers with the given keypair.
pub async fn sign(
  request: ClientRequest,
  actor: &dyn ActorType,
  activity: String,
) -> Result<DigestClient<String>, LemmyError> {
  let signing_key_id = format!("{}#main-key", actor.actor_id()?);
  let private_key = actor.private_key();

  let digest_client = request
    .signature_with_digest(
      HTTP_SIG_CONFIG.clone(),
      signing_key_id,
      Sha256::new(),
      activity,
      move |signing_string| {
        let private_key = PKey::private_key_from_pem(private_key.as_bytes())?;
        let mut signer = Signer::new(MessageDigest::sha256(), &private_key).unwrap();
        signer.update(signing_string.as_bytes()).unwrap();

        Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, LemmyError>
      },
    )
    .await?;

  Ok(digest_client)
}

pub fn verify(request: &HttpRequest, actor: &dyn ActorType) -> Result<(), LemmyError> {
  let verified = HTTP_SIG_CONFIG
    .begin_verify(
      request.method(),
      request.uri().path_and_query(),
      request.headers().clone(),
    )?
    .verify(|signature, signing_string| -> Result<bool, LemmyError> {
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
    Err(format_err!("Invalid signature on request: {}", &request.uri()).into())
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

impl<U> UnparsedExtension<U> for PublicKeyExtension
where
  U: UnparsedMutExt,
{
  type Error = serde_json::Error;

  fn try_from_unparsed(unparsed_mut: &mut U) -> Result<Self, Self::Error> {
    Ok(PublicKeyExtension {
      public_key: unparsed_mut.remove("publicKey")?,
    })
  }

  fn try_into_unparsed(self, unparsed_mut: &mut U) -> Result<(), Self::Error> {
    unparsed_mut.insert("publicKey", self.public_key)?;
    Ok(())
  }
}
