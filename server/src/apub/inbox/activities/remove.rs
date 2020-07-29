use crate::{
  api::{comment::CommentResponse, community::CommunityResponse, post::PostResponse},
  apub::{
    fetcher::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    inbox::shared_inbox::{
      announce_if_community_is_local,
      get_user_from_activity,
      receive_unhandled_activity,
    },
    FromApub,
    GroupExt,
    PageExt,
  },
  blocking,
  routes::ChatServerParam,
  websocket::{
    server::{SendComment, SendCommunityRoomMessage, SendPost},
    UserOperation,
  },
  DbPool,
  LemmyError,
};
use activitystreams_new::{activity::Remove, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
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

pub async fn receive_remove(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(activity)?.unwrap();
  match remove.object().as_single_kind_str() {
    Some("Page") => receive_remove_post(remove, client, pool, chat_server).await,
    Some("Note") => receive_remove_comment(remove, client, pool, chat_server).await,
    Some("Group") => receive_remove_community(remove, client, pool, chat_server).await,
    _ => receive_unhandled_activity(remove),
  }
}

async fn receive_remove_post(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let page = PageExt::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  let post_ap_id = PostForm::from_apub(&page, client, pool)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_post(&post_ap_id, client, pool).await?;

  let post_form = PostForm {
    name: post.name.to_owned(),
    url: post.url.to_owned(),
    body: post.body.to_owned(),
    creator_id: post.creator_id.to_owned(),
    community_id: post.community_id,
    removed: Some(true),
    deleted: None,
    nsfw: post.nsfw,
    locked: None,
    stickied: None,
    updated: Some(naive_now()),
    embed_title: post.embed_title,
    embed_description: post.embed_description,
    embed_html: post.embed_html,
    thumbnail_url: post.thumbnail_url,
    ap_id: post.ap_id,
    local: post.local,
    published: None,
  };
  let post_id = post.id;
  blocking(pool, move |conn| Post::update(conn, post_id, &post_form)).await??;

  // Refetch the view
  let post_id = post.id;
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(remove, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_remove_comment(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let note = Note::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  let comment_ap_id = CommentForm::from_apub(&note, client, pool)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_comment(&comment_ap_id, client, pool).await?;

  let comment_form = CommentForm {
    content: comment.content.to_owned(),
    parent_id: comment.parent_id,
    post_id: comment.post_id,
    creator_id: comment.creator_id,
    removed: Some(true),
    deleted: None,
    read: None,
    published: None,
    updated: Some(naive_now()),
    ap_id: comment.ap_id,
    local: comment.local,
  };
  let comment_id = comment.id;
  blocking(pool, move |conn| {
    Comment::update(conn, comment_id, &comment_form)
  })
  .await??;

  // Refetch the view
  let comment_id = comment.id;
  let comment_view =
    blocking(pool, move |conn| CommentView::read(conn, comment_id, None)).await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
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

  announce_if_community_is_local(remove, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_remove_community(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let group = GroupExt::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  let community_actor_id = CommunityForm::from_apub(&group, client, pool)
    .await?
    .actor_id;

  let community = blocking(pool, move |conn| {
    Community::read_from_actor_id(conn, &community_actor_id)
  })
  .await??;

  let community_form = CommunityForm {
    name: community.name.to_owned(),
    title: community.title.to_owned(),
    description: community.description.to_owned(),
    category_id: community.category_id, // Note: need to keep this due to foreign key constraint
    creator_id: community.creator_id,   // Note: need to keep this due to foreign key constraint
    removed: Some(true),
    published: None,
    updated: Some(naive_now()),
    deleted: None,
    nsfw: community.nsfw,
    actor_id: community.actor_id,
    local: community.local,
    private_key: community.private_key,
    public_key: community.public_key,
    last_refreshed_at: None,
  };

  let community_id = community.id;
  blocking(pool, move |conn| {
    Community::update(conn, community_id, &community_form)
  })
  .await??;

  let community_id = community.id;
  let res = CommunityResponse {
    community: blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;

  chat_server.do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    my_id: None,
  });

  announce_if_community_is_local(remove, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}
