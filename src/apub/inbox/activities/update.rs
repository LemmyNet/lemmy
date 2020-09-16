use crate::{
  apub::{
    fetcher::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    inbox::shared_inbox::{
      announce_if_community_is_local,
      get_user_from_activity,
      receive_unhandled_activity,
    },
    ActorType,
    FromApub,
    PageExt,
  },
  LemmyContext,
};
use activitystreams::{activity::Update, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  post::{Post, PostForm},
  post_view::PostView,
  Crud,
};
use lemmy_structs::{
  blocking,
  comment::CommentResponse,
  post::PostResponse,
  send_local_notifs,
  websocket::{SendComment, SendPost, UserOperation},
};
use lemmy_utils::{location_info, utils::scrape_text_for_mentions, LemmyError};

pub async fn receive_update(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;

  // ensure that update and actor come from the same instance
  let user = get_user_from_activity(&update, context).await?;
  update.id(user.actor_id()?.domain().context(location_info!())?)?;

  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, context).await,
    Some("Note") => receive_update_comment(update, context).await,
    _ => receive_unhandled_activity(update),
  }
}

async fn receive_update_post(
  update: Update,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&update, context).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  let original_post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  blocking(context.pool(), move |conn| {
    Post::update(conn, original_post_id, &post)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, original_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(update, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_comment(
  update: Update,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_user_from_activity(&update, context).await?;

  let comment = CommentForm::from_apub(&note, context, Some(user.actor_id()?)).await?;

  let original_comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
    .await?
    .id;

  let updated_comment = blocking(context.pool(), move |conn| {
    Comment::update(conn, original_comment_id, &comment)
  })
  .await??;

  let post_id = updated_comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    updated_comment,
    &user,
    post,
    context.pool(),
    false,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, original_comment_id, None)
  })
  .await??;

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

  announce_if_community_is_local(update, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
