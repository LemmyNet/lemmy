use crate::{
  api::{comment::CommentResponse, post::PostResponse},
  apub::{
    fetcher::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    inbox::shared_inbox::{
      announce_if_community_is_local,
      get_user_from_activity,
      receive_unhandled_activity,
    },
    FromApub,
    PageExt,
  },
  blocking,
  websocket::{
    server::{SendComment, SendPost},
    UserOperation,
  },
  LemmyContext,
  LemmyError,
};
use activitystreams::{activity::Dislike, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  comment::{CommentForm, CommentLike, CommentLikeForm},
  comment_view::CommentView,
  post::{PostForm, PostLike, PostLikeForm},
  post_view::PostView,
  Likeable,
};
use lemmy_utils::location_info;

pub async fn receive_dislike(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(activity)?.context(location_info!())?;
  match dislike.object().as_single_kind_str() {
    Some("Page") => receive_dislike_post(dislike, context).await,
    Some("Note") => receive_dislike_comment(dislike, context).await,
    _ => receive_unhandled_activity(dislike),
  }
}

async fn receive_dislike_post(
  dislike: Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&dislike, context).await?;
  let page = PageExt::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: -1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)?;
    PostLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(dislike, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_dislike_comment(
  dislike: Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;
  let user = get_user_from_activity(&dislike, context).await?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: -1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)?;
    CommentLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
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
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(dislike, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
