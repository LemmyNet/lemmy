use crate::{
  core::{object_id::ObjectId, signatures::verify_signature},
  data::Data,
  traits::{ActivityHandler, ApubObject},
  utils::verify_domains_match,
  Error,
  LocalInstance,
};
use actix_web::{HttpRequest, HttpResponse};
use serde::de::DeserializeOwned;
use tracing::log::debug;

pub trait ActorPublicKey {
  /// Returns the actor's public key for verification of HTTP signatures
  fn public_key(&self) -> &str;
}

/// Receive an activity and perform some basic checks, including HTTP signature verification.
pub async fn receive_activity<Activity, Actor, Datatype, E>(
  request: HttpRequest,
  activity: Activity,
  local_instance: &LocalInstance,
  data: &Data<Datatype>,
) -> Result<HttpResponse, E>
where
  Activity: ActivityHandler<DataType = Datatype, Error = E> + DeserializeOwned + Send + 'static,
  Actor: ApubObject<DataType = Datatype, Error = E> + ActorPublicKey + Send + 'static,
  for<'de2> <Actor as ApubObject>::ApubType: serde::Deserialize<'de2>,
  E: From<anyhow::Error> + From<Error>,
{
  verify_domains_match(activity.id(), activity.actor())?;
  if local_instance.is_local_url(activity.id()) {
    return Err(Error::UrlVerificationError("Activity was sent from local instance").into());
  }

  (local_instance.settings.verify_url_function)(activity.id())
    .map_err(Error::UrlVerificationError)?;

  let request_counter = &mut 0;
  let actor = ObjectId::<Actor>::new(activity.actor().clone())
    .dereference::<E>(data, local_instance, request_counter)
    .await?;
  verify_signature(&request, actor.public_key())?;

  debug!("Verifying activity {}", activity.id().to_string());
  activity.verify(data, request_counter).await?;

  debug!("Receiving activity {}", activity.id().to_string());
  activity.receive(data, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}
