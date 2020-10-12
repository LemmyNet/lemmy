use crate::{
  activities::receive::{
    announce_if_community_is_local,
    get_actor_as_user,
    receive_unhandled_activity,
    undo_comment::*,
    undo_post::*,
  },
  ActorType,
  FromApub,
  GroupExt,
};
use activitystreams::{
  activity::*,
  base::{AnyBase, AsBase},
  prelude::*,
};
use actix_web::HttpResponse;
use anyhow::{anyhow, Context};
use lemmy_db::{
  community::{Community, CommunityForm},
  community_view::CommunityView,
  naive_now,
  Crud,
};
use lemmy_structs::{blocking, community::CommunityResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};

pub async fn receive_undo(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete(undo, context).await,
    Some("Remove") => receive_undo_remove(undo, context).await,
    Some("Like") => receive_undo_like(undo, context).await,
    Some("Dislike") => receive_undo_dislike(undo, context).await,
    _ => receive_unhandled_activity(undo),
  }
}

fn check_is_undo_valid<T, A>(outer_activity: &Undo, inner_activity: &T) -> Result<(), LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let outer_actor = outer_activity.actor()?;
  let outer_actor_uri = outer_actor
    .as_single_xsd_any_uri()
    .context(location_info!())?;

  let inner_actor = inner_activity.actor()?;
  let inner_actor_uri = inner_actor
    .as_single_xsd_any_uri()
    .context(location_info!())?;

  if outer_actor_uri.domain() != inner_actor_uri.domain() {
    Err(anyhow!("Cant undo receive from a different instance").into())
  } else {
    Ok(())
  }
}

async fn receive_undo_delete(
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &delete)?;
  let type_ = delete
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_delete_comment(undo, &delete, context).await,
    "Page" => receive_undo_delete_post(undo, &delete, context).await,
    "Group" => receive_undo_delete_community(undo, &delete, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_remove(
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &remove)?;

  let type_ = remove
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_remove_comment(undo, &remove, context).await,
    "Page" => receive_undo_remove_post(undo, &remove, context).await,
    "Group" => receive_undo_remove_community(undo, &remove, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_like(undo: Undo, context: &LemmyContext) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &like)?;

  let type_ = like
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_like_comment(undo, &like, context).await,
    "Page" => receive_undo_like_post(undo, &like, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_dislike(
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &dislike)?;

  let type_ = dislike
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_dislike_comment(undo, &dislike, context).await,
    "Page" => receive_undo_dislike_post(undo, &dislike, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_delete_community(
  undo: Undo,
  delete: &Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(delete, context).await?;
  let group = GroupExt::from_any_base(delete.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let community_actor_id = CommunityForm::from_apub(&group, context, Some(user.actor_id()?))
    .await?
    .actor_id
    .context(location_info!())?;

  let community = blocking(context.pool(), move |conn| {
    Community::read_from_actor_id(conn, &community_actor_id)
  })
  .await??;

  let community_form = CommunityForm {
    name: community.name.to_owned(),
    title: community.title.to_owned(),
    description: community.description.to_owned(),
    category_id: community.category_id, // Note: need to keep this due to foreign key constraint
    creator_id: community.creator_id,   // Note: need to keep this due to foreign key constraint
    removed: None,
    published: None,
    updated: Some(naive_now()),
    deleted: Some(false),
    nsfw: community.nsfw,
    actor_id: Some(community.actor_id),
    local: community.local,
    private_key: community.private_key,
    public_key: community.public_key,
    last_refreshed_at: None,
    icon: Some(community.icon.to_owned()),
    banner: Some(community.banner.to_owned()),
  };

  let community_id = community.id;
  blocking(context.pool(), move |conn| {
    Community::update(conn, community_id, &community_form)
  })
  .await??;

  let community_id = community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_community(
  undo: Undo,
  remove: &Remove,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_actor_as_user(remove, context).await?;
  let group = GroupExt::from_any_base(remove.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let community_actor_id = CommunityForm::from_apub(&group, context, Some(mod_.actor_id()?))
    .await?
    .actor_id
    .context(location_info!())?;

  let community = blocking(context.pool(), move |conn| {
    Community::read_from_actor_id(conn, &community_actor_id)
  })
  .await??;

  let community_form = CommunityForm {
    name: community.name.to_owned(),
    title: community.title.to_owned(),
    description: community.description.to_owned(),
    category_id: community.category_id, // Note: need to keep this due to foreign key constraint
    creator_id: community.creator_id,   // Note: need to keep this due to foreign key constraint
    removed: Some(false),
    published: None,
    updated: Some(naive_now()),
    deleted: None,
    nsfw: community.nsfw,
    actor_id: Some(community.actor_id),
    local: community.local,
    private_key: community.private_key,
    public_key: community.public_key,
    last_refreshed_at: None,
    icon: Some(community.icon.to_owned()),
    banner: Some(community.banner.to_owned()),
  };

  let community_id = community.id;
  blocking(context.pool(), move |conn| {
    Community::update(conn, community_id, &community_form)
  })
  .await??;

  let community_id = community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}
