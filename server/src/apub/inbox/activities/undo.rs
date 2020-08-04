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
use activitystreams::{activity::*, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
use anyhow::anyhow;
use lemmy_db::{
  comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
  comment_view::CommentView,
  community::{Community, CommunityForm},
  community_view::CommunityView,
  naive_now,
  post::{Post, PostForm, PostLike, PostLikeForm},
  post_view::PostView,
  Crud,
  Likeable,
};

pub async fn receive_undo(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.unwrap();
  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete(undo, client, pool, chat_server).await,
    Some("Remove") => receive_undo_remove(undo, client, pool, chat_server).await,
    Some("Like") => receive_undo_like(undo, client, pool, chat_server).await,
    Some("Dislike") => receive_undo_dislike(undo, client, pool, chat_server).await,
    // TODO: handle undo_dislike?
    _ => receive_unhandled_activity(undo),
  }
}

async fn receive_undo_delete(
  undo: Undo,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(undo.object().to_owned().one().unwrap())?.unwrap();
  let type_ = delete.object().as_single_kind_str().unwrap();
  match type_ {
    "Note" => receive_undo_delete_comment(undo, &delete, client, pool, chat_server).await,
    "Page" => receive_undo_delete_post(undo, &delete, client, pool, chat_server).await,
    "Group" => receive_undo_delete_community(undo, &delete, client, pool, chat_server).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_remove(
  undo: Undo,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(undo.object().to_owned().one().unwrap())?.unwrap();

  let type_ = remove.object().as_single_kind_str().unwrap();
  match type_ {
    "Note" => receive_undo_remove_comment(undo, &remove, client, pool, chat_server).await,
    "Page" => receive_undo_remove_post(undo, &remove, client, pool, chat_server).await,
    "Group" => receive_undo_remove_community(undo, &remove, client, pool, chat_server).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_like(
  undo: Undo,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().unwrap())?.unwrap();

  let type_ = like.object().as_single_kind_str().unwrap();
  match type_ {
    "Note" => receive_undo_like_comment(undo, &like, client, pool, chat_server).await,
    "Page" => receive_undo_like_post(undo, &like, client, pool, chat_server).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_dislike(
  undo: Undo,
  _client: &Client,
  _pool: &DbPool,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().unwrap())?.unwrap();

  let type_ = dislike.object().as_single_kind_str().unwrap();
  Err(anyhow!("Undo Delete type {} not supported", type_).into())
}

async fn receive_undo_delete_comment(
  undo: Undo,
  delete: &Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, client, pool).await?;
  let note = Note::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  let comment_ap_id = CommentForm::from_apub(&note, client, pool)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_comment(&comment_ap_id, client, pool).await?;

  let comment_form = CommentForm {
    content: comment.content.to_owned(),
    parent_id: comment.parent_id,
    post_id: comment.post_id,
    creator_id: comment.creator_id,
    removed: None,
    deleted: Some(false),
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

  announce_if_community_is_local(undo, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_comment(
  undo: Undo,
  remove: &Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, client, pool).await?;
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
    removed: Some(false),
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

  announce_if_community_is_local(undo, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_post(
  undo: Undo,
  delete: &Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, client, pool).await?;
  let page = PageExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

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
    removed: None,
    deleted: Some(false),
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

  announce_if_community_is_local(undo, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_post(
  undo: Undo,
  remove: &Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, client, pool).await?;
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
    removed: Some(false),
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

  announce_if_community_is_local(undo, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_community(
  undo: Undo,
  delete: &Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, client, pool).await?;
  let group = GroupExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

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
    removed: None,
    published: None,
    updated: Some(naive_now()),
    deleted: Some(false),
    nsfw: community.nsfw,
    actor_id: community.actor_id,
    local: community.local,
    private_key: community.private_key,
    public_key: community.public_key,
    last_refreshed_at: None,
    icon: Some(community.icon.to_owned()),
    banner: Some(community.banner.to_owned()),
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

  announce_if_community_is_local(undo, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_community(
  undo: Undo,
  remove: &Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, client, pool).await?;
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
    removed: Some(false),
    published: None,
    updated: Some(naive_now()),
    deleted: None,
    nsfw: community.nsfw,
    actor_id: community.actor_id,
    local: community.local,
    private_key: community.private_key,
    public_key: community.public_key,
    last_refreshed_at: None,
    icon: Some(community.icon.to_owned()),
    banner: Some(community.banner.to_owned()),
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

  announce_if_community_is_local(undo, &mod_, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_like_comment(
  undo: Undo,
  like: &Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(like, client, pool).await?;
  let note = Note::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 0,
  };
  blocking(pool, move |conn| CommentLike::remove(conn, &like_form)).await??;

  // Refetch the view
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
    op: UserOperation::CreateCommentLike,
    comment: res,
    my_id: None,
  });

  announce_if_community_is_local(undo, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_like_post(
  undo: Undo,
  like: &Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(like, client, pool).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
  blocking(pool, move |conn| PostLike::remove(conn, &like_form)).await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(undo, &user, client, pool).await?;
  Ok(HttpResponse::Ok().finish())
}
