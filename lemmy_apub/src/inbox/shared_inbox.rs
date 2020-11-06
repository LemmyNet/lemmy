use crate::{
  activities::receive::{
    comment::{
      receive_create_comment,
      receive_delete_comment,
      receive_dislike_comment,
      receive_like_comment,
      receive_remove_comment,
      receive_update_comment,
    },
    comment_undo::{
      receive_undo_delete_comment,
      receive_undo_dislike_comment,
      receive_undo_like_comment,
      receive_undo_remove_comment,
    },
    community::{
      receive_delete_community,
      receive_remove_community,
      receive_undo_delete_community,
      receive_undo_remove_community,
    },
    find_by_id,
    post::{
      receive_create_post,
      receive_delete_post,
      receive_dislike_post,
      receive_like_post,
      receive_remove_post,
      receive_update_post,
    },
    post_undo::{
      receive_undo_delete_post,
      receive_undo_dislike_post,
      receive_undo_like_post,
      receive_undo_remove_post,
    },
    receive_unhandled_activity,
    verify_activity_domains_valid,
    FindResults,
  },
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  inbox::{get_activity_id, is_activity_already_known},
  insert_activity,
  ActorType,
};
use activitystreams::{
  activity::{ActorAndObject, Announce, Create, Delete, Dislike, Like, Remove, Undo, Update},
  base::AnyBase,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use lemmy_db::{site::Site, Crud};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

/// Allowed activity types for shared inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Undo,
  Remove,
  Announce,
}

// TODO: this isnt entirely correct, cause some of these receive are not ActorAndObject,
//       but it might still work due to the anybase conversion
pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming requests to shared inbox.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  let actor_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  debug!(
    "Shared inbox received activity {:?} from {}",
    &activity.id_unchecked(),
    &actor_id
  );

  check_is_apub_id_valid(&actor_id)?;

  let request_counter = &mut 0;
  let actor = get_or_fetch_and_upsert_actor(&actor_id, &context, request_counter).await?;
  verify_signature(&request, actor.as_ref())?;

  let activity_id = get_activity_id(&activity, &actor_id)?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Announce => {
      receive_announce(&context, any_base, actor.as_ref(), request_counter).await
    }
    ValidTypes::Create => receive_create(&context, any_base, actor_id, request_counter).await,
    ValidTypes::Update => receive_update(&context, any_base, actor_id, request_counter).await,
    ValidTypes::Like => receive_like(&context, any_base, actor_id, request_counter).await,
    ValidTypes::Dislike => receive_dislike(&context, any_base, actor_id, request_counter).await,
    ValidTypes::Remove => receive_remove(&context, any_base, actor_id).await,
    ValidTypes::Delete => receive_delete(&context, any_base, actor_id, request_counter).await,
    ValidTypes::Undo => receive_undo(&context, any_base, actor_id, request_counter).await,
  };

  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;
  res
}

/// Takes an announce and passes the inner activity to the appropriate handler.
async fn receive_announce(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&announce, actor.actor_id()?, false)?;

  let kind = announce.object().as_single_kind_str();
  let object = announce
    .object()
    .to_owned()
    .one()
    .context(location_info!())?;

  let inner_id = object.id().context(location_info!())?.to_owned();
  check_is_apub_id_valid(&inner_id)?;
  if is_activity_already_known(context.pool(), &inner_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  match kind {
    Some("Create") => receive_create(context, object, inner_id, request_counter).await,
    Some("Update") => receive_update(context, object, inner_id, request_counter).await,
    Some("Like") => receive_like(context, object, inner_id, request_counter).await,
    Some("Dislike") => receive_dislike(context, object, inner_id, request_counter).await,
    Some("Delete") => receive_delete(context, object, inner_id, request_counter).await,
    Some("Remove") => receive_remove(context, object, inner_id).await,
    Some("Undo") => receive_undo(context, object, inner_id, request_counter).await,
    _ => receive_unhandled_activity(announce),
  }
}

async fn receive_create(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, expected_domain, true)?;

  match create.object().as_single_kind_str() {
    Some("Page") => receive_create_post(create, context, request_counter).await,
    Some("Note") => receive_create_comment(create, context, request_counter).await,
    _ => receive_unhandled_activity(create),
  }
}

async fn receive_update(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&update, expected_domain, true)?;

  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, context, request_counter).await,
    Some("Note") => receive_update_comment(update, context, request_counter).await,
    _ => receive_unhandled_activity(update),
  }
}

async fn receive_like(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&like, expected_domain, false)?;

  match like.object().as_single_kind_str() {
    Some("Page") => receive_like_post(like, context, request_counter).await,
    Some("Note") => receive_like_comment(like, context, request_counter).await,
    _ => receive_unhandled_activity(like),
  }
}

async fn receive_dislike(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let enable_downvotes = blocking(context.pool(), move |conn| {
    Site::read(conn, 1).map(|s| s.enable_downvotes)
  })
  .await??;
  if !enable_downvotes {
    return Ok(HttpResponse::Ok().finish());
  }

  let dislike = Dislike::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&dislike, expected_domain, false)?;

  match dislike.object().as_single_kind_str() {
    Some("Page") => receive_dislike_post(dislike, context, request_counter).await,
    Some("Note") => receive_dislike_comment(dislike, context, request_counter).await,
    _ => receive_unhandled_activity(dislike),
  }
}

pub async fn receive_delete(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_by_id(context, object).await {
    Ok(FindResults::Post(p)) => receive_delete_post(context, delete, p, request_counter).await,
    Ok(FindResults::Comment(c)) => {
      receive_delete_comment(context, delete, c, request_counter).await
    }
    Ok(FindResults::Community(c)) => {
      receive_delete_community(context, delete, c, request_counter).await
    }
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_remove(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&remove, expected_domain, false)?;

  let cc = remove
    .cc()
    .map(|c| c.as_many())
    .flatten()
    .context(location_info!())?;
  let community_id = cc
    .first()
    .map(|c| c.as_xsd_any_uri())
    .flatten()
    .context(location_info!())?;

  let object = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  // Ensure that remove activity comes from the same domain as the community
  remove.id(community_id.domain().context(location_info!())?)?;

  match find_by_id(context, object).await {
    Ok(FindResults::Post(p)) => receive_remove_post(context, remove, p).await,
    Ok(FindResults::Comment(c)) => receive_remove_comment(context, remove, c).await,
    Ok(FindResults::Community(c)) => receive_remove_community(context, remove, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_undo(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, expected_domain.to_owned(), true)?;

  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete(context, undo, expected_domain, request_counter).await,
    Some("Remove") => receive_undo_remove(context, undo, expected_domain, request_counter).await,
    Some("Like") => receive_undo_like(context, undo, expected_domain, request_counter).await,
    Some("Dislike") => receive_undo_dislike(context, undo, expected_domain, request_counter).await,
    _ => receive_unhandled_activity(undo),
  }
}

async fn receive_undo_delete(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_by_id(context, object).await {
    Ok(FindResults::Post(p)) => receive_undo_delete_post(context, undo, p, request_counter).await,
    Ok(FindResults::Comment(c)) => {
      receive_undo_delete_comment(context, undo, c, request_counter).await
    }
    Ok(FindResults::Community(c)) => {
      receive_undo_delete_community(context, undo, c, request_counter).await
    }
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_undo_remove(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&remove, expected_domain, false)?;

  let object = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_by_id(context, object).await {
    Ok(FindResults::Post(p)) => receive_undo_remove_post(context, undo, p, request_counter).await,
    Ok(FindResults::Comment(c)) => {
      receive_undo_remove_comment(context, undo, c, request_counter).await
    }
    Ok(FindResults::Community(c)) => {
      receive_undo_remove_community(context, undo, c, request_counter).await
    }
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_undo_like(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&like, expected_domain, false)?;

  let type_ = like
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_like_comment(undo, &like, context, request_counter).await,
    "Page" => receive_undo_like_post(undo, &like, context, request_counter).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_dislike(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&dislike, expected_domain, false)?;

  let type_ = dislike
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_dislike_comment(undo, &dislike, context, request_counter).await,
    "Page" => receive_undo_dislike_post(undo, &dislike, context, request_counter).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}
