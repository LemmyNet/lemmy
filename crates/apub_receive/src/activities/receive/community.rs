use crate::{
  activities::receive::get_actor_as_person,
  inbox::receive_for_community::verify_actor_is_community_mod,
};
use activitystreams::{
  activity::{ActorAndObjectRefExt, Delete, Undo, Update},
  base::ExtendsExt,
};
use anyhow::{anyhow, Context};
use lemmy_api_common::{blocking, community::CommunityResponse};
use lemmy_apub::{
  get_community_from_to_or_cc,
  objects::FromApubToForm,
  ActorType,
  CommunityType,
  GroupExt,
};
use lemmy_db_queries::{source::community::Community_, Crud};
use lemmy_db_schema::source::{
  community::{Community, CommunityForm},
  person::Person,
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperationCrud};

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
pub(crate) async fn receive_remote_mod_update_community(
  update: Update,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let community = get_community_from_to_or_cc(&update, context, request_counter).await?;
  verify_actor_is_community_mod(&update, &community, context).await?;
  let group = GroupExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let updated_community = CommunityForm::from_apub(
    &group,
    context,
    community.actor_id(),
    request_counter,
    false,
  )
  .await?;
  let cf = CommunityForm {
    name: updated_community.name,
    title: updated_community.title,
    description: updated_community.description,
    nsfw: updated_community.nsfw,
    // TODO: icon and banner would be hosted on the other instance, ideally we would copy it to ours
    icon: updated_community.icon,
    banner: updated_community.banner,
    ..CommunityForm::default()
  };
  blocking(context.pool(), move |conn| {
    Community::update(conn, community.id, &cf)
  })
  .await??;

  Ok(())
}

pub(crate) async fn receive_remote_mod_delete_community(
  delete: Delete,
  community: Community,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  verify_actor_is_community_mod(&delete, &community, context).await?;
  let actor = get_actor_as_person(&delete, context, request_counter).await?;
  verify_is_remote_community_creator(&actor, &community, context).await?;
  let community_id = community.id;
  blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community_id, true)
  })
  .await??;
  community.send_delete(actor, context).await
}

pub(crate) async fn receive_delete_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, true)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_remove_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, true)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_remote_mod_undo_delete_community(
  undo: Undo,
  community: Community,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  verify_actor_is_community_mod(&undo, &community, context).await?;
  let actor = get_actor_as_person(&undo, context, request_counter).await?;
  verify_is_remote_community_creator(&actor, &community, context).await?;
  let community_id = community.id;
  blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community_id, false)
  })
  .await??;
  community.send_undo_delete(actor, context).await
}

pub(crate) async fn receive_undo_delete_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, false)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_remove_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, false)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

/// Checks if the remote user is creator of the local community. This can only happen if a community
/// is created by a local user, and then transferred to a remote user.
async fn verify_is_remote_community_creator(
  user: &Person,
  community: &Community,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community_id = community.id;
  let community_mods = blocking(context.pool(), move |conn| {
    CommunityModeratorView::for_community(conn, community_id)
  })
  .await??;

  if user.id != community_mods[0].moderator.id {
    Err(anyhow!("Actor is not community creator").into())
  } else {
    Ok(())
  }
}
