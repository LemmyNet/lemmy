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

pub mod announce;
pub mod create;
pub mod delete;
pub mod dislike;
pub mod like;
pub mod remove;
pub mod undo;
mod undo_comment;
mod undo_post;
pub mod update;

fn receive_unhandled_activity<A>(activity: A) -> Result<HttpResponse, LemmyError>
where
  A: Debug,
{
  debug!("received unhandled activity type: {:?}", activity);
  Ok(HttpResponse::NotImplemented().finish())
}

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
  let community_followers_uri = cc
    .first()
    .context(location_info!())?
    .as_xsd_any_uri()
    .context(location_info!())?;
  // TODO: this is hacky but seems to be the only way to get the community ID
  let community_uri = community_followers_uri
    .to_string()
    .replace("/followers", "");
  let community = get_or_fetch_and_upsert_community(&Url::parse(&community_uri)?, context).await?;

  if community.local {
    community
      .send_announce(activity.into_any_base()?, &user, context)
      .await?;
  }
  Ok(())
}

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
