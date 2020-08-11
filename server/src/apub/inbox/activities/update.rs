use crate::{
  api::{
    comment::{send_local_notifs, CommentResponse},
    post::PostResponse,
  },
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
  blocking,
  routes::ChatServerParam,
  websocket::{
    server::{SendComment, SendPost},
    UserOperation,
  },
  DbPool,
  LemmyError,
};
use activitystreams::{activity::Update, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
use anyhow::Context;
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  post::{Post, PostForm},
  post_view::PostView,
  Crud,
};
use lemmy_utils::{location_info, scrape_text_for_mentions};

pub async fn receive_update(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;

  // ensure that update and actor come from the same instance
  let user = get_user_from_activity(&update, client, pool).await?;
  update.id(user.actor_id()?.domain().context(location_info!())?)?;

  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, client, pool, chat_server).await,
    Some("Note") => receive_update_comment(update, client, pool, chat_server).await,
    _ => receive_unhandled_activity(update),
  }
}

async fn receive_update_post(
  update: Update,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&update, client, pool).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, client, pool, Some(user.actor_id()?)).await?;

  let original_post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  blocking(pool, move |conn| {
    Post::update(conn, original_post_id, &post)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| {
    PostView::read(conn, original_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(update, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_comment(
  update: Update,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_user_from_activity(&update, client, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, Some(user.actor_id()?)).await?;

  let original_comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let updated_comment = blocking(pool, move |conn| {
    Comment::update(conn, original_comment_id, &comment)
  })
  .await??;

  let post_id = updated_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids =
    send_local_notifs(mentions, updated_comment, &user, post, pool, false).await?;

  // Refetch the view
  let comment_view = blocking(pool, move |conn| {
    CommentView::read(conn, original_comment_id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  chat_server.do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    my_id: None,
  });

  announce_if_community_is_local(update, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}
