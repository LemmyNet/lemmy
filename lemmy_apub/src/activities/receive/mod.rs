use crate::{
  fetcher::{get_or_fetch_and_upsert_community, get_or_fetch_and_upsert_user},
  ActorType,
};
use activitystreams::{
  activity::{ActorAndObjectRef, ActorAndObjectRefExt},
  base::{AsBase, BaseExt, Extends, ExtendsExt},
  error::DomainError,
  object::{AsObject, ObjectExt},
};
use actix_web::HttpResponse;
use anyhow::Context;
use diesel::result::Error::NotFound;
use lemmy_db::{comment::Comment, community::Community, post::Post, user::User_};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::Serialize;
use std::fmt::Debug;
use url::Url;

pub(crate) mod comment;
pub(crate) mod comment_undo;
pub(crate) mod community;
pub(crate) mod post;
pub(crate) mod post_undo;

/// Return HTTP 501 for unsupported activities in inbox.
pub(crate) fn receive_unhandled_activity<A>(activity: A) -> Result<HttpResponse, LemmyError>
where
  A: Debug,
{
  debug!("received unhandled activity type: {:?}", activity);
  Ok(HttpResponse::NotImplemented().finish())
}

/// Reads the destination community from the activity's `cc` field. If this refers to a local
/// community, the activity is announced to all community followers.
async fn announce_if_community_is_local<T, Kind>(
  activity: T,
  user: &User_,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind>,
  T: Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let cc = activity.cc().context(location_info!())?;
  let cc = cc.as_many().context(location_info!())?;
  let community_uri = cc
    .first()
    .context(location_info!())?
    .as_xsd_any_uri()
    .context(location_info!())?;
  let community = get_or_fetch_and_upsert_community(&community_uri, context).await?;

  if community.local {
    community
      .send_announce(activity.into_any_base()?, &user, context)
      .await?;
  }
  Ok(())
}

/// Reads the actor field of an activity and returns the corresponding `User_`.
pub(crate) async fn get_actor_as_user<T, A>(
  activity: &T,
  context: &LemmyContext,
) -> Result<User_, LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let actor = activity.actor()?;
  let user_uri = actor.as_single_xsd_any_uri().context(location_info!())?;
  get_or_fetch_and_upsert_user(&user_uri, context).await
}

pub(crate) enum FindResults {
  Comment(Comment),
  Community(Community),
  Post(Post),
}

/// Tries to find a community, post or comment in the local database, without any network requests.
/// This is used to handle deletions and removals, because in case we dont have the object, we can
/// simply ignore the activity.
pub(crate) async fn find_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<FindResults, LemmyError> {
  let ap_id = apub_id.to_string();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_actor_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = community {
    return Ok(FindResults::Community(c));
  }

  let ap_id = apub_id.to_string();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(p) = post {
    return Ok(FindResults::Post(p));
  }

  let ap_id = apub_id.to_string();
  let comment = blocking(context.pool(), move |conn| {
    Comment::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = comment {
    return Ok(FindResults::Comment(c));
  }

  return Err(NotFound.into());
}

/// Ensure that the ID of an incoming activity comes from the same domain as the actor. Optionally
/// also checks the ID of the inner object.
///
/// The reason that this starts with the actor ID is that it was already confirmed as correct by the
/// HTTP signature.
pub(crate) fn verify_activity_domains_valid<T, Kind>(
  activity: &T,
  actor_id: Url,
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
