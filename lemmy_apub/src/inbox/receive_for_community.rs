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
  },
  inbox::is_addressed_to_public,
};
use activitystreams::{
  activity::{Create, Delete, Dislike, Like, Remove, Undo, Update},
  base::AnyBase,
  prelude::*,
};
use anyhow::Context;
use diesel::result::Error::NotFound;
use lemmy_db::{comment::Comment, post::Post, site::Site, Crud};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

/// This file is for post/comment activities received by the community, and for post/comment
///       activities announced by the community and received by the user.

/// A post or comment being created
pub(in crate::inbox) async fn receive_create_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, &expected_domain, true)?;
  is_addressed_to_public(&create)?;

  match create.object().as_single_kind_str() {
    Some("Page") => receive_create_post(create, context, request_counter).await,
    Some("Note") => receive_create_comment(create, context, request_counter).await,
    _ => receive_unhandled_activity(create),
  }
}

/// A post or comment being edited
pub(in crate::inbox) async fn receive_update_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&update, &expected_domain, true)?;
  is_addressed_to_public(&update)?;

  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, context, request_counter).await,
    Some("Note") => receive_update_comment(update, context, request_counter).await,
    _ => receive_unhandled_activity(update),
  }
}

/// A post or comment being upvoted
pub(in crate::inbox) async fn receive_like_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let like = Like::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&like, &expected_domain, false)?;
  is_addressed_to_public(&like)?;

  match like.object().as_single_kind_str() {
    Some("Page") => receive_like_post(like, context, request_counter).await,
    Some("Note") => receive_like_comment(like, context, request_counter).await,
    _ => receive_unhandled_activity(like),
  }
}

/// A post or comment being downvoted
pub(in crate::inbox) async fn receive_dislike_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let enable_downvotes = blocking(context.pool(), move |conn| {
    Site::read(conn, 1).map(|s| s.enable_downvotes)
  })
  .await??;
  if !enable_downvotes {
    return Ok(());
  }

  let dislike = Dislike::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&dislike, &expected_domain, false)?;
  is_addressed_to_public(&dislike)?;

  match dislike.object().as_single_kind_str() {
    Some("Page") => receive_dislike_post(dislike, context, request_counter).await,
    Some("Note") => receive_dislike_comment(dislike, context, request_counter).await,
    _ => receive_unhandled_activity(dislike),
  }
}

/// A post or comment being deleted by its creator
pub(in crate::inbox) async fn receive_delete_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, &expected_domain, true)?;
  is_addressed_to_public(&delete)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_delete_post(context, p).await,
    Ok(PostOrComment::Comment(c)) => receive_delete_comment(context, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(()),
  }
}

/// A post or comment being removed by a mod/admin
pub(in crate::inbox) async fn receive_remove_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let remove = Remove::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&remove, &expected_domain, false)?;
  is_addressed_to_public(&remove)?;

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

  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_remove_post(context, remove, p).await,
    Ok(PostOrComment::Comment(c)) => receive_remove_comment(context, remove, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(()),
  }
}

/// A post/comment action being reverted (either a delete, remove, upvote or downvote)
pub(in crate::inbox) async fn receive_undo_for_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, &expected_domain.to_owned(), true)?;
  is_addressed_to_public(&undo)?;

  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete_for_community(context, undo, expected_domain).await,
    Some("Remove") => receive_undo_remove_for_community(context, undo, expected_domain).await,
    Some("Like") => {
      receive_undo_like_for_community(context, undo, expected_domain, request_counter).await
    }
    Some("Dislike") => {
      receive_undo_dislike_for_community(context, undo, expected_domain, request_counter).await
    }
    _ => receive_unhandled_activity(undo),
  }
}

/// A post or comment deletion being reverted
pub(in crate::inbox) async fn receive_undo_delete_for_community(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let delete = Delete::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&delete, &expected_domain, true)?;
  is_addressed_to_public(&delete)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_undo_delete_post(context, p).await,
    Ok(PostOrComment::Comment(c)) => receive_undo_delete_comment(context, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(()),
  }
}

/// A post or comment removal being reverted
pub(in crate::inbox) async fn receive_undo_remove_for_community(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let remove = Remove::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&remove, &expected_domain, false)?;
  is_addressed_to_public(&remove)?;

  let object = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_undo_remove_post(context, p).await,
    Ok(PostOrComment::Comment(c)) => receive_undo_remove_comment(context, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(()),
  }
}

/// A post or comment upvote being reverted
pub(in crate::inbox) async fn receive_undo_like_for_community(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&like, &expected_domain, false)?;
  is_addressed_to_public(&like)?;

  let type_ = like
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_like_comment(&like, context, request_counter).await,
    "Page" => receive_undo_like_post(&like, context, request_counter).await,
    _ => receive_unhandled_activity(like),
  }
}

/// A post or comment downvote being reverted
pub(in crate::inbox) async fn receive_undo_dislike_for_community(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&dislike, &expected_domain, false)?;
  is_addressed_to_public(&dislike)?;

  let type_ = dislike
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_dislike_comment(&dislike, context, request_counter).await,
    "Page" => receive_undo_dislike_post(&dislike, context, request_counter).await,
    _ => receive_unhandled_activity(dislike),
  }
}

enum PostOrComment {
  Comment(Comment),
  Post(Post),
}

/// Tries to find a post or comment in the local database, without any network requests.
/// This is used to handle deletions and removals, because in case we dont have the object, we can
/// simply ignore the activity.
async fn find_post_or_comment_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<PostOrComment, LemmyError> {
  let ap_id = apub_id.to_string();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(p) = post {
    return Ok(PostOrComment::Post(p));
  }

  let ap_id = apub_id.to_string();
  let comment = blocking(context.pool(), move |conn| {
    Comment::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = comment {
    return Ok(PostOrComment::Comment(c));
  }

  return Err(NotFound.into());
}
