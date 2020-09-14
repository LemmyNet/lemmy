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
    GroupExt,
    PageExt,
  },
  websocket::{
    messages::{SendComment, SendCommunityRoomMessage, SendPost},
    UserOperation,
  },
  LemmyContext,
};
use activitystreams::{activity::Delete, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_api_structs::{
  blocking,
  comment::CommentResponse,
  community::CommunityResponse,
  post::PostResponse,
};
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  community::{Community, CommunityForm},
  community_view::CommunityView,
  naive_now,
  post::{Post, PostForm},
  post_view::PostView,
  Crud,
};
use lemmy_utils::{location_info, LemmyError};

pub async fn receive_delete(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  match delete.object().as_single_kind_str() {
    Some("Page") => receive_delete_post(delete, context).await,
    Some("Note") => receive_delete_comment(delete, context).await,
    Some("Group") => receive_delete_community(delete, context).await,
    _ => receive_unhandled_activity(delete),
  }
}

async fn receive_delete_post(
  delete: Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, context).await?;
  let page = PageExt::from_any_base(delete.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post_ap_id = PostForm::from_apub(&page, context, Some(user.actor_id()?))
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_post(&post_ap_id, context).await?;

  let post_form = PostForm {
    name: post.name.to_owned(),
    url: post.url.to_owned(),
    body: post.body.to_owned(),
    creator_id: post.creator_id.to_owned(),
    community_id: post.community_id,
    removed: None,
    deleted: Some(true),
    nsfw: post.nsfw,
    locked: None,
    stickied: None,
    updated: Some(naive_now()),
    embed_title: post.embed_title,
    embed_description: post.embed_description,
    embed_html: post.embed_html,
    thumbnail_url: post.thumbnail_url,
    ap_id: Some(post.ap_id),
    local: post.local,
    published: None,
  };
  let post_id = post.id;
  blocking(context.pool(), move |conn| {
    Post::update(conn, post_id, &post_form)
  })
  .await??;

  // Refetch the view
  let post_id = post.id;
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

  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_comment(
  delete: Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, context).await?;
  let note = Note::from_any_base(delete.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment_ap_id = CommentForm::from_apub(&note, context, Some(user.actor_id()?))
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_comment(&comment_ap_id, context).await?;

  let comment_form = CommentForm {
    content: comment.content.to_owned(),
    parent_id: comment.parent_id,
    post_id: comment.post_id,
    creator_id: comment.creator_id,
    removed: None,
    deleted: Some(true),
    read: None,
    published: None,
    updated: Some(naive_now()),
    ap_id: Some(comment.ap_id),
    local: comment.local,
  };
  let comment_id = comment.id;
  blocking(context.pool(), move |conn| {
    Comment::update(conn, comment_id, &comment_form)
  })
  .await??;

  // Refetch the view
  let comment_id = comment.id;
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

  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_community(
  delete: Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let group = GroupExt::from_any_base(delete.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_user_from_activity(&delete, context).await?;

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
    deleted: Some(true),
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

  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
