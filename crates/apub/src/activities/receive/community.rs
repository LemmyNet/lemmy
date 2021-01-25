use crate::{activities::receive::verify_activity_domains_valid, inbox::is_addressed_to_public};
use activitystreams::{
  activity::{ActorAndObjectRefExt, Delete, Remove, Undo},
  base::{AnyBase, ExtendsExt},
};
use anyhow::Context;
use lemmy_db_queries::{source::community::Community_, ApubObject};
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_structs::{blocking, community::CommunityResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};
use url::Url;

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
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_remove_community(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let remove = Remove::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&remove, expected_domain, true)?;
  is_addressed_to_public(&remove)?;

  let community_uri = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &community_uri.into())
  })
  .await??;

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
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_delete_community(
  context: &LemmyContext,
  undo: Undo,
  community: Community,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  is_addressed_to_public(&undo)?;
  let inner = undo.object().to_owned().one().context(location_info!())?;
  let delete = Delete::from_any_base(inner)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;
  is_addressed_to_public(&delete)?;

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
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_remove_community(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  is_addressed_to_public(&undo)?;

  let inner = undo.object().to_owned().one().context(location_info!())?;
  let remove = Remove::from_any_base(inner)?.context(location_info!())?;
  verify_activity_domains_valid(&remove, &expected_domain, true)?;
  is_addressed_to_public(&remove)?;

  let community_uri = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &community_uri.into())
  })
  .await??;

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
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}
