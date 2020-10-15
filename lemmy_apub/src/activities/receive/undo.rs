use crate::activities::receive::{
  announce_if_community_is_local,
  find_by_id,
  get_actor_as_user,
  receive_unhandled_activity,
  undo_comment::*,
  undo_post::*,
  verify_activity_domains_valid,
  FindResults,
};
use activitystreams::{activity::*, base::AnyBase, prelude::*};
use actix_web::HttpResponse;
use anyhow::{anyhow, Context};
use lemmy_db::{community::Community, community_view::CommunityView};
use lemmy_structs::{blocking, community::CommunityResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};
use url::Url;

pub async fn receive_undo(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, expected_domain.to_owned(), true)?;

  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete(context, undo, expected_domain).await,
    Some("Remove") => receive_undo_remove(context, undo, expected_domain).await,
    Some("Like") => receive_undo_like(context, undo, expected_domain).await,
    Some("Dislike") => receive_undo_dislike(context, undo, expected_domain).await,
    _ => receive_unhandled_activity(undo),
  }
}

async fn receive_undo_delete(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
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
    Ok(FindResults::Post(p)) => receive_undo_delete_post(context, undo, p).await,
    Ok(FindResults::Comment(c)) => receive_undo_delete_comment(context, undo, c).await,
    Ok(FindResults::Community(c)) => receive_undo_delete_community(context, undo, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_undo_remove(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
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
    Ok(FindResults::Post(p)) => receive_undo_remove_post(context, undo, p).await,
    Ok(FindResults::Comment(c)) => receive_undo_remove_comment(context, undo, c).await,
    Ok(FindResults::Community(c)) => receive_undo_remove_community(context, undo, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_undo_like(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&like, expected_domain, false)?;

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
  context: &LemmyContext,
  undo: Undo,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  verify_activity_domains_valid(&dislike, expected_domain, false)?;

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
  context: &LemmyContext,
  undo: Undo,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, false)
  })
  .await??;

  let community_id = deleted_community.id;
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

  let user = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_community(
  context: &LemmyContext,
  undo: Undo,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, false)
  })
  .await??;

  let community_id = removed_community.id;
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

  let mod_ = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}
