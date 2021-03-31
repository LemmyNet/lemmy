use activitystreams::{
  activity::{ActorAndObjectRef, ActorAndObjectRefExt},
  base::{AsBase, BaseExt},
  error::DomainError,
};
use anyhow::{anyhow, Context};
use lemmy_apub::fetcher::person::get_or_fetch_and_upsert_person;
use lemmy_db_schema::source::person::Person;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use std::fmt::Debug;
use url::Url;

pub(crate) mod comment;
pub(crate) mod comment_undo;
pub(crate) mod community;
pub(crate) mod post;
pub(crate) mod post_undo;
pub(crate) mod private_message;

/// Return HTTP 501 for unsupported activities in inbox.
pub(crate) fn receive_unhandled_activity<A>(activity: A) -> Result<(), LemmyError>
where
  A: Debug,
{
  debug!("received unhandled activity type: {:?}", activity);
  Err(anyhow!("Activity not supported").into())
}

/// Reads the actor field of an activity and returns the corresponding `Person`.
pub(crate) async fn get_actor_as_person<T, A>(
  activity: &T,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Person, LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let actor = activity.actor()?;
  let person_uri = actor.as_single_xsd_any_uri().context(location_info!())?;
  get_or_fetch_and_upsert_person(&person_uri, context, request_counter).await
}

/// Ensure that the ID of an incoming activity comes from the same domain as the actor. Optionally
/// also checks the ID of the inner object.
///
/// The reason that this starts with the actor ID is that it was already confirmed as correct by the
/// HTTP signature.
pub(crate) fn verify_activity_domains_valid<T, Kind>(
  activity: &T,
  actor_id: &Url,
  object_domain_must_match: bool,
) -> Result<(), LemmyError>
where
  T: AsBase<Kind> + ActorAndObjectRef,
{
  let expected_domain = actor_id.domain().context(location_info!())?;

  activity.id(expected_domain)?;

  let object_id = match activity.object().to_owned().single_xsd_any_uri() {
    // object is just an ID
    Some(id) => id,
    // object is something like an activity, a comment or a post
    None => activity
      .object()
      .to_owned()
      .one()
      .context(location_info!())?
      .id()
      .context(location_info!())?
      .to_owned(),
  };

  if object_domain_must_match && object_id.domain() != Some(expected_domain) {
    return Err(DomainError.into());
  }

  Ok(())
}
