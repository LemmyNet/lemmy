use crate::{
  activities::receive::{
    announce_if_community_is_local,
    get_actor_as_user,
    receive_unhandled_activity,
    verify_activity_domains_valid,
  },
  ActorType,
  FromApub,
  PageExt,
};
use activitystreams::{activity::Create, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  post::{Post, PostForm},
  post_view::PostView,
};
use lemmy_structs::{blocking, comment::CommentResponse, post::PostResponse, send_local_notifs};
use lemmy_utils::{location_info, utils::scrape_text_for_mentions, LemmyError};
use lemmy_websocket::{
  messages::{SendComment, SendPost},
  LemmyContext,
  UserOperation,
};
use url::Url;

pub async fn receive_create(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, expected_domain, true)?;

  match create.object().as_single_kind_str() {
    Some("Page") => receive_create_post(create, context).await,
    Some("Note") => receive_create_comment(create, context).await,
    _ => receive_unhandled_activity(create),
  }
}

async fn receive_create_post(
  create: Create,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&create, context).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  // Using an upsert, since likes (which fetch the post), sometimes come in before the create
  // resulting in double posts.
  let inserted_post = blocking(context.pool(), move |conn| Post::upsert(conn, &post)).await??;

  // Refetch the view
  let inserted_post_id = inserted_post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, inserted_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_comment(
  create: Create,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&create, context).await?;
  let note = Note::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, Some(user.actor_id()?)).await?;

  let inserted_comment =
    blocking(context.pool(), move |conn| Comment::upsert(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    inserted_comment.clone(),
    &user,
    post,
    context.pool(),
    true,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, inserted_comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
