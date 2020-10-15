use crate::activities::receive::{
  announce_if_community_is_local,
  find_by_id,
  get_actor_as_user,
  verify_activity_domains_valid,
  FindResults,
};
use activitystreams::{activity::Delete, base::AnyBase, prelude::*};
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

pub async fn receive_delete(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;

  let object = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_by_id(context, object).await {
    Ok(FindResults::Post(p)) => receive_delete_post(context, delete, p).await,
    Ok(FindResults::Comment(c)) => receive_delete_comment(context, delete, c).await,
    Ok(FindResults::Community(c)) => receive_delete_community(context, delete, c).await,
    // if we dont have the object, no need to do anything
    Err(_) => Ok(HttpResponse::Ok().finish()),
  }
}

async fn receive_delete_post(
  context: &LemmyContext,
  delete: Delete,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
  let deleted_post = blocking(context.pool(), move |conn| {
    Post::update_deleted(conn, post.id, true)
  })
  .await??;

  // Refetch the view
  let post_id = deleted_post.id;
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

  let user = get_actor_as_user(&delete, context).await?;
  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_comment(
  context: &LemmyContext,
  delete: Delete,
  comment: Comment,
) -> Result<HttpResponse, LemmyError> {
  let deleted_comment = blocking(context.pool(), move |conn| {
    Comment::update_deleted(conn, comment.id, true)
  })
  .await??;

  // Refetch the view
  let comment_id = deleted_comment.id;
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

  let user = get_actor_as_user(&delete, context).await?;
  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_community(
  context: &LemmyContext,
  delete: Delete,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, true)
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

  let user = get_actor_as_user(&delete, context).await?;
  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
