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
  LemmyContext,
};
use activitystreams::{
  activity::*,
  base::{AnyBase, AsBase},
  object::Note,
  prelude::*,
};
use actix_web::HttpResponse;
use anyhow::{anyhow, Context};
use lemmy_db::{
  comment::{Comment, CommentForm, CommentLike},
  comment_view::CommentView,
  community::{Community, CommunityForm},
  community_view::CommunityView,
  naive_now,
  post::{Post, PostForm, PostLike},
  post_view::PostView,
  Crud,
  Likeable,
};
use lemmy_structs::{
  blocking,
  comment::CommentResponse,
  community::CommunityResponse,
  post::PostResponse,
  websocket::{SendComment, SendCommunityRoomMessage, SendPost, UserOperation},
};
use lemmy_utils::{location_info, LemmyError};

pub async fn receive_undo(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  match undo.object().as_single_kind_str() {
    Some("Delete") => receive_undo_delete(undo, context).await,
    Some("Remove") => receive_undo_remove(undo, context).await,
    Some("Like") => receive_undo_like(undo, context).await,
    Some("Dislike") => receive_undo_dislike(undo, context).await,
    _ => receive_unhandled_activity(undo),
  }
}

fn check_is_undo_valid<T, A>(outer_activity: &Undo, inner_activity: &T) -> Result<(), LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let outer_actor = outer_activity.actor()?;
  let outer_actor_uri = outer_actor
    .as_single_xsd_any_uri()
    .context(location_info!())?;

  let inner_actor = inner_activity.actor()?;
  let inner_actor_uri = inner_actor
    .as_single_xsd_any_uri()
    .context(location_info!())?;

  if outer_actor_uri.domain() != inner_actor_uri.domain() {
    Err(anyhow!("Cant undo activities from a different instance").into())
  } else {
    Ok(())
  }
}

async fn receive_undo_delete(
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &delete)?;
  let type_ = delete
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_delete_comment(undo, &delete, context).await,
    "Page" => receive_undo_delete_post(undo, &delete, context).await,
    "Group" => receive_undo_delete_community(undo, &delete, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_remove(
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let remove = Remove::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &remove)?;

  let type_ = remove
    .object()
    .as_single_kind_str()
    .context(location_info!())?;
  match type_ {
    "Note" => receive_undo_remove_comment(undo, &remove, context).await,
    "Page" => receive_undo_remove_post(undo, &remove, context).await,
    "Group" => receive_undo_remove_community(undo, &remove, context).await,
    d => Err(anyhow!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_like(undo: Undo, context: &LemmyContext) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &like)?;

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
  undo: Undo,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let dislike = Dislike::from_any_base(undo.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  check_is_undo_valid(&undo, &dislike)?;

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

async fn receive_undo_delete_comment(
  undo: Undo,
  delete: &Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, context).await?;
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
    deleted: Some(false),
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_comment(
  undo: Undo,
  remove: &Remove,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, context).await?;
  let note = Note::from_any_base(remove.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment_ap_id = CommentForm::from_apub(&note, context, None)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_comment(&comment_ap_id, context).await?;

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

  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_post(
  undo: Undo,
  delete: &Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, context).await?;
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
    deleted: Some(false),
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_post(
  undo: Undo,
  remove: &Remove,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, context).await?;
  let page = PageExt::from_any_base(remove.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post_ap_id = PostForm::from_apub(&page, context, None)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_post(&post_ap_id, context).await?;

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

  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_community(
  undo: Undo,
  delete: &Delete,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(delete, context).await?;
  let group = GroupExt::from_any_base(delete.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

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
    deleted: Some(false),
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_community(
  undo: Undo,
  remove: &Remove,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(remove, context).await?;
  let group = GroupExt::from_any_base(remove.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let community_actor_id = CommunityForm::from_apub(&group, context, Some(mod_.actor_id()?))
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
    removed: Some(false),
    published: None,
    updated: Some(naive_now()),
    deleted: None,
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

  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_like_comment(
  undo: Undo,
  like: &Like,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(like, context).await?;
  let note = Note::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
    .await?
    .id;

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_like_post(
  undo: Undo,
  like: &Like,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(like, context).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_dislike_comment(
  undo: Undo,
  dislike: &Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(dislike, context).await?;
  let note = Note::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
    .await?
    .id;

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_dislike_post(
  undo: Undo,
  dislike: &Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(dislike, context).await?;
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

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}
