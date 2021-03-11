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
  fetcher::{
    objects::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    user::get_or_fetch_and_upsert_user,
  },
  find_post_or_comment_by_id,
  generate_moderators_url,
  inbox::verify_is_addressed_to_public,
  ActorType,
  PostOrComment,
};
use activitystreams::{
  activity::{
    ActorAndObjectRef,
    Add,
    Announce,
    Create,
    Delete,
    Dislike,
    Like,
    OptTargetRef,
    Remove,
    Undo,
    Update,
  },
  base::AnyBase,
  object::AsObject,
  prelude::*,
};
use anyhow::{anyhow, Context};
use diesel::result::Error::NotFound;
use lemmy_api_structs::blocking;
use lemmy_db_queries::{source::community::CommunityModerator_, ApubObject, Crud, Joinable};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityModerator, CommunityModeratorForm},
    site::Site,
    user::User_,
  },
  DbUrl,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use strum_macros::EnumString;
use url::Url;

#[derive(EnumString)]
enum PageOrNote {
  Page,
  Note,
}

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
  verify_is_addressed_to_public(&create)?;

  let kind = create
    .object()
    .as_single_kind_str()
    .and_then(|s| s.parse().ok());
  match kind {
    Some(PageOrNote::Page) => receive_create_post(create, context, request_counter).await,
    Some(PageOrNote::Note) => receive_create_comment(create, context, request_counter).await,
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
  verify_is_addressed_to_public(&update)?;

  let kind = update
    .object()
    .as_single_kind_str()
    .and_then(|s| s.parse().ok());
  match kind {
    Some(PageOrNote::Page) => receive_update_post(update, context, request_counter).await,
    Some(PageOrNote::Note) => receive_update_comment(update, context, request_counter).await,
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
  verify_is_addressed_to_public(&like)?;

  let object_id = like
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  match fetch_post_or_comment_by_id(&object_id, context, request_counter).await? {
    PostOrComment::Post(post) => receive_like_post(like, *post, context, request_counter).await,
    PostOrComment::Comment(comment) => {
      receive_like_comment(like, *comment, context, request_counter).await
    }
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
  verify_is_addressed_to_public(&dislike)?;

  let object_id = dislike
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  match fetch_post_or_comment_by_id(&object_id, context, request_counter).await? {
    PostOrComment::Post(post) => {
      receive_dislike_post(dislike, *post, context, request_counter).await
    }
    PostOrComment::Comment(comment) => {
      receive_dislike_comment(dislike, *comment, context, request_counter).await
    }
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
  verify_is_addressed_to_public(&delete)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_delete_post(context, *p).await,
    Ok(PostOrComment::Comment(c)) => receive_delete_comment(context, *c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(()),
  }
}

/// A post or comment being removed by a mod/admin
pub(in crate::inbox) async fn receive_remove_for_community(
  context: &LemmyContext,
  remove_any_base: AnyBase,
  announce: Option<Announce>,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let remove = Remove::from_any_base(remove_any_base.to_owned())?.context(location_info!())?;
  let community = extract_community_from_cc(&remove, context).await?;

  verify_mod_activity(&remove, announce, &community, context).await?;
  verify_is_addressed_to_public(&remove)?;
  verify_actor_is_community_mod(&remove, &community, context).await?;

  if remove.target().is_some() {
    let remove_mod = remove
      .object()
      .as_single_xsd_any_uri()
      .context(location_info!())?;
    let remove_mod = get_or_fetch_and_upsert_user(&remove_mod, context, request_counter).await?;
    let form = CommunityModeratorForm {
      community_id: community.id,
      user_id: remove_mod.id,
    };
    blocking(context.pool(), move |conn| {
      CommunityModerator::leave(conn, &form)
    })
    .await??;
    community.send_announce(remove_any_base, context).await?;
    // TODO: send websocket notification about removed mod
    Ok(())
  }
  // Remove a post or comment
  else {
    let object = remove
      .object()
      .to_owned()
      .single_xsd_any_uri()
      .context(location_info!())?;

    match find_post_or_comment_by_id(context, object).await {
      Ok(PostOrComment::Post(p)) => receive_remove_post(context, remove, *p).await,
      Ok(PostOrComment::Comment(c)) => receive_remove_comment(context, remove, *c).await,
      // if we dont have the object, no need to do anything
      Err(_) => Ok(()),
    }
  }
}

#[derive(EnumString)]
enum UndoableActivities {
  Delete,
  Remove,
  Like,
  Dislike,
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
  verify_is_addressed_to_public(&undo)?;

  use UndoableActivities::*;
  match undo
    .object()
    .as_single_kind_str()
    .and_then(|s| s.parse().ok())
  {
    Some(Delete) => receive_undo_delete_for_community(context, undo, expected_domain).await,
    Some(Remove) => receive_undo_remove_for_community(context, undo, expected_domain).await,
    Some(Like) => {
      receive_undo_like_for_community(context, undo, expected_domain, request_counter).await
    }
    Some(Dislike) => {
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
  verify_is_addressed_to_public(&delete)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_undo_delete_post(context, *p).await,
    Ok(PostOrComment::Comment(c)) => receive_undo_delete_comment(context, *c).await,
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
  verify_is_addressed_to_public(&remove)?;

  let object = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  match find_post_or_comment_by_id(context, object).await {
    Ok(PostOrComment::Post(p)) => receive_undo_remove_post(context, *p).await,
    Ok(PostOrComment::Comment(c)) => receive_undo_remove_comment(context, *c).await,
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
  verify_is_addressed_to_public(&like)?;

  let object_id = like
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  match fetch_post_or_comment_by_id(&object_id, context, request_counter).await? {
    PostOrComment::Post(post) => {
      receive_undo_like_post(&like, *post, context, request_counter).await
    }
    PostOrComment::Comment(comment) => {
      receive_undo_like_comment(&like, *comment, context, request_counter).await
    }
  }
}

/// Add a new mod to the community (can only be done by an existing mod).
pub(in crate::inbox) async fn receive_add_for_community(
  context: &LemmyContext,
  add_any_base: AnyBase,
  announce: Option<Announce>,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let add = Add::from_any_base(add_any_base.to_owned())?.context(location_info!())?;
  let community = extract_community_from_cc(&add, context).await?;

  verify_mod_activity(&add, announce, &community, context).await?;
  verify_is_addressed_to_public(&add)?;
  verify_actor_is_community_mod(&add, &community, context).await?;
  verify_add_remove_moderator_target(&add, &community)?;

  let new_mod = add
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  let new_mod = get_or_fetch_and_upsert_user(&new_mod, context, request_counter).await?;

  // If we had to refetch the community while parsing the activity, then the new mod has already
  // been added. Skip it here as it would result in a duplicate key error.
  let new_mod_id = new_mod.id;
  let moderated_communities = blocking(context.pool(), move |conn| {
    CommunityModerator::get_user_moderated_communities(conn, new_mod_id)
  })
  .await??;
  if moderated_communities.contains(&community.id) {
    let form = CommunityModeratorForm {
      community_id: community.id,
      user_id: new_mod.id,
    };
    blocking(context.pool(), move |conn| {
      CommunityModerator::join(conn, &form)
    })
    .await??;
  }
  if community.local {
    community.send_announce(add_any_base, context).await?;
  }
  // TODO: send websocket notification about added mod
  Ok(())
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
  verify_is_addressed_to_public(&dislike)?;

  let object_id = dislike
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  match fetch_post_or_comment_by_id(&object_id, context, request_counter).await? {
    PostOrComment::Post(post) => {
      receive_undo_dislike_post(&dislike, *post, context, request_counter).await
    }
    PostOrComment::Comment(comment) => {
      receive_undo_dislike_comment(&dislike, *comment, context, request_counter).await
    }
  }
}

async fn fetch_post_or_comment_by_id(
  apub_id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<PostOrComment, LemmyError> {
  if let Ok(post) = get_or_fetch_and_insert_post(apub_id, context, request_counter).await {
    return Ok(PostOrComment::Post(Box::new(post)));
  }

  if let Ok(comment) = get_or_fetch_and_insert_comment(apub_id, context, request_counter).await {
    return Ok(PostOrComment::Comment(Box::new(comment)));
  }

  Err(NotFound.into())
}

async fn extract_community_from_cc<T, Kind>(
  activity: &T,
  context: &LemmyContext,
) -> Result<Community, LemmyError>
where
  T: AsObject<Kind>,
{
  let cc = activity
    .cc()
    .map(|c| c.as_many())
    .flatten()
    .context(location_info!())?;
  let community_id = cc
    .first()
    .map(|c| c.as_xsd_any_uri())
    .flatten()
    .context(location_info!())?;
  let community_id: DbUrl = community_id.to_owned().into();
  let community = blocking(&context.pool(), move |conn| {
    Community::read_from_apub_id(&conn, &community_id)
  })
  .await??;
  Ok(community)
}

async fn verify_actor_is_community_mod<T, Kind>(
  activity: &T,
  community: &Community,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + BaseExt<Kind>,
{
  let actor = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?
    .to_owned();
  let actor = blocking(&context.pool(), move |conn| {
    User_::read_from_apub_id(&conn, &actor.into())
  })
  .await??;

  // Note: this will also return true for admins in addition to mods, but as we dont know about
  //       remote admins, it doesnt make any difference.
  let community_id = community.id;
  let actor_id = actor.id;
  let is_mod_or_admin = blocking(context.pool(), move |conn| {
    CommunityView::is_mod_or_admin(conn, actor_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(anyhow!("Not a mod").into());
  }

  Ok(())
}

async fn verify_mod_activity<T, Kind>(
  mod_action: &T,
  announce: Option<Announce>,
  community: &Community,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + OptTargetRef + BaseExt<Kind>,
{
  // Remove was sent by community to user, we just check that it came from the right domain
  if let Some(announce) = announce {
    verify_activity_domains_valid(&announce, &community.actor_id.to_owned().into(), false)?;
  }
  // Remove was sent by a remote mod to community, check that they are actually mod
  else {
    verify_actor_is_community_mod(mod_action, community, context).await?;
  }

  Ok(())
}
fn verify_add_remove_moderator_target<T, Kind>(
  activity: &T,
  community: &Community,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + BaseExt<Kind> + OptTargetRef,
{
  let target = activity
    .target()
    .map(|t| t.as_single_xsd_any_uri())
    .flatten()
    .context(location_info!())?;
  if target != &generate_moderators_url(&community.actor_id)?.into_inner() {
    return Err(anyhow!("Unkown target url").into());
  }
  Ok(())
}
