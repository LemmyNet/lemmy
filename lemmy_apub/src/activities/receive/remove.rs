use crate::activities::receive::{find_by_id, verify_activity_domains_valid, FindResults};
use activitystreams::{activity::Remove, base::AnyBase, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  comment::Comment,
  comment_view::CommentView,
  community::Community,
  community_view::CommunityView,
  post::Post,
  post_view::PostView,
};
use lemmy_structs::{
  blocking,
  comment::CommentResponse,
  community::CommunityResponse,
  post::PostResponse,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{
  messages::{SendComment, SendCommunityRoomMessage, SendPost},
  LemmyContext,
  UserOperation,
};
use url::Url;

pub async fn receive_remove(
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

async fn receive_remove_post(
  context: &LemmyContext,
  _remove: Remove,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
  let removed_post = blocking(context.pool(), move |conn| {
    Post::update_removed(conn, post.id, true)
  })
  .await??;

  // Refetch the view
  let post_id = removed_post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };
  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_remove_comment(
  context: &LemmyContext,
  _remove: Remove,
  comment: Comment,
) -> Result<HttpResponse, LemmyError> {
  let removed_comment = blocking(context.pool(), move |conn| {
    Comment::update_removed(conn, comment.id, true)
  })
  .await??;

  // Refetch the view
  let comment_id = removed_comment.id;
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };
  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_remove_community(
  context: &LemmyContext,
  _remove: Remove,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, true)
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

  Ok(HttpResponse::Ok().finish())
}
