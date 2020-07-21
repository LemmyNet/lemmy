use crate::{
  api::{
    comment::{send_local_notifs, CommentResponse},
    community::CommunityResponse,
    post::PostResponse,
  },
  apub::{
    community::do_announce,
    extensions::signatures::verify,
    fetcher::{
      get_or_fetch_and_insert_remote_comment,
      get_or_fetch_and_insert_remote_post,
      get_or_fetch_and_upsert_remote_community,
      get_or_fetch_and_upsert_remote_user,
    },
    insert_activity,
    ActorType,
    FromApub,
    GroupExt,
    PageExt,
  },
  blocking,
  routes::{ChatServerParam, DbPoolParam},
  websocket::{
    server::{SendComment, SendCommunityRoomMessage, SendPost},
    UserOperation,
  },
  DbPool,
  LemmyError,
};
use activitystreams_new::{
  activity::{ActorAndObjectRef, Announce, Create, Delete, Dislike, Like, Remove, Undo, Update},
  base::{AnyBase, AsBase},
  error::DomainError,
  object::Note,
  prelude::{ExtendsExt, *},
};
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use lemmy_db::{
  comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
  comment_view::CommentView,
  community::{Community, CommunityForm},
  community_view::CommunityView,
  naive_now,
  post::{Post, PostForm, PostLike, PostLikeForm},
  post_view::PostView,
  user::User_,
  Crud,
  Likeable,
};
use lemmy_utils::scrape_text_for_mentions;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SharedAcceptedObjects {
  Create(Box<Create>),
  Update(Box<Update>),
  Like(Box<Like>),
  Dislike(Box<Dislike>),
  Delete(Box<Delete>),
  Undo(Box<Undo>),
  Remove(Box<Remove>),
  Announce(Box<Announce>),
}

impl SharedAcceptedObjects {
  // TODO: these shouldnt be necessary anymore
  // https://git.asonix.dog/asonix/ap-relay/src/branch/main/src/apub.rs
  fn object(&self) -> Option<AnyBase> {
    match self {
      SharedAcceptedObjects::Create(c) => c.object().to_owned().one(),
      SharedAcceptedObjects::Update(u) => u.object().to_owned().one(),
      SharedAcceptedObjects::Like(l) => l.object().to_owned().one(),
      SharedAcceptedObjects::Dislike(d) => d.object().to_owned().one(),
      SharedAcceptedObjects::Delete(d) => d.object().to_owned().one(),
      SharedAcceptedObjects::Undo(d) => d.object().to_owned().one(),
      SharedAcceptedObjects::Remove(r) => r.object().to_owned().one(),
      SharedAcceptedObjects::Announce(a) => a.object().to_owned().one(),
    }
  }
  fn sender(&self) -> Result<&Url, DomainError> {
    let uri = match self {
      SharedAcceptedObjects::Create(c) => c.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Update(u) => u.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Like(l) => l.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Dislike(d) => d.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Delete(d) => d.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Undo(d) => d.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Remove(r) => r.actor()?.as_single_xsd_any_uri(),
      SharedAcceptedObjects::Announce(a) => a.actor()?.as_single_xsd_any_uri(),
    };
    Ok(uri.unwrap())
  }
  fn cc(&self) -> String {
    let cc = match self {
      SharedAcceptedObjects::Create(c) => c.cc().to_owned(),
      SharedAcceptedObjects::Update(u) => u.cc().to_owned(),
      SharedAcceptedObjects::Like(l) => l.cc().to_owned(),
      SharedAcceptedObjects::Dislike(d) => d.cc().to_owned(),
      SharedAcceptedObjects::Delete(d) => d.cc().to_owned(),
      SharedAcceptedObjects::Undo(d) => d.cc().to_owned(),
      SharedAcceptedObjects::Remove(r) => r.cc().to_owned(),
      SharedAcceptedObjects::Announce(a) => a.cc().to_owned(),
    };
    cc.unwrap()
      .clone()
      .many()
      .unwrap()
      .first()
      .unwrap()
      .as_xsd_any_uri()
      .unwrap()
      .to_string()
  }
}

/// Handler for all incoming activities to user inboxes.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<SharedAcceptedObjects>,
  client: web::Data<Client>,
  pool: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  let pool = &pool;
  let client = &client;

  let json = serde_json::to_string(&activity)?;
  debug!("Shared inbox received activity: {}", json);

  let sender = &activity.sender()?.clone();
  let cc = activity.to_owned().cc();
  // TODO: this is hacky, we should probably send the community id directly somehow
  let to = cc.replace("/followers", "");

  // TODO: this is ugly
  match get_or_fetch_and_upsert_remote_user(sender, &client, pool).await {
    Ok(u) => verify(&request, &u)?,
    Err(_) => {
      let c = get_or_fetch_and_upsert_remote_community(sender, &client, pool).await?;
      verify(&request, &c)?;
    }
  }

  let object = activity.object().unwrap();
  match (activity, object.kind_str()) {
    (SharedAcceptedObjects::Create(c), Some("Page")) => {
      receive_create_post((*c).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(c.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Update(u), Some("Page")) => {
      receive_update_post((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(u.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Like(l), Some("Page")) => {
      receive_like_post((*l).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(l.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Dislike(d), Some("Page")) => {
      receive_dislike_post((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(d.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Page")) => {
      receive_delete_post((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(d.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Page")) => {
      receive_remove_post((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(r.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Create(c), Some("Note")) => {
      receive_create_comment((*c).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(c.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Update(u), Some("Note")) => {
      receive_update_comment((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(u.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Like(l), Some("Note")) => {
      receive_like_comment((*l).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(l.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Dislike(d), Some("Note")) => {
      receive_dislike_comment((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(d.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Note")) => {
      receive_delete_comment((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(d.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Note")) => {
      receive_remove_comment((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(r.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Group")) => {
      receive_delete_community((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(d.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Group")) => {
      receive_remove_community((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(r.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Delete")) => {
      receive_undo_delete((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(u.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Remove")) => {
      receive_undo_remove((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(u.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Like")) => {
      receive_undo_like((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid(u.into_any_base()?, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Announce(a), _) => receive_announce(a, client, pool, chat_server).await,
    (a, _) => receive_unhandled_activity(a),
  }
}

// TODO: should pass in sender as ActorType, but thats a bit tricky in shared_inbox()
async fn announce_activity_if_valid(
  activity: AnyBase,
  community_uri: &str,
  sender: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<HttpResponse, LemmyError> {
  let community_uri = community_uri.to_owned();
  let community = blocking(pool, move |conn| {
    Community::read_from_actor_id(conn, &community_uri)
  })
  .await??;

  if community.local {
    let sending_user = get_or_fetch_and_upsert_remote_user(sender, client, pool).await?;

    do_announce(activity, &community, &sending_user, client, pool).await
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}

async fn receive_announce(
  announce: Box<Announce>,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let object = announce.to_owned().object().clone().one().unwrap();
  // TODO: too much copy paste
  match object.kind_str() {
    Some("Create") => {
      let create = Create::from_any_base(object)?.unwrap();
      match create.object().as_single_kind_str() {
        Some("Page") => receive_create_post(create, client, pool, chat_server).await,
        Some("Note") => receive_create_comment(create, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Update") => {
      let update = Update::from_any_base(object)?.unwrap();
      match update.object().as_single_kind_str() {
        Some("Page") => receive_update_post(update, client, pool, chat_server).await,
        Some("Note") => receive_update_comment(update, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Like") => {
      let like = Like::from_any_base(object)?.unwrap();
      match like.object().as_single_kind_str() {
        Some("Page") => receive_like_post(like, client, pool, chat_server).await,
        Some("Note") => receive_like_comment(like, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Dislike") => {
      let dislike = Dislike::from_any_base(object)?.unwrap();
      match dislike.object().as_single_kind_str() {
        Some("Page") => receive_dislike_post(dislike, client, pool, chat_server).await,
        Some("Note") => receive_dislike_comment(dislike, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Delete") => {
      let delete = Delete::from_any_base(object)?.unwrap();
      match delete.object().as_single_kind_str() {
        Some("Page") => receive_delete_post(delete, client, pool, chat_server).await,
        Some("Note") => receive_delete_comment(delete, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Remove") => {
      let remove = Remove::from_any_base(object)?.unwrap();
      match remove.object().as_single_kind_str() {
        Some("Page") => receive_remove_post(remove, client, pool, chat_server).await,
        Some("Note") => receive_remove_comment(remove, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Undo") => {
      let undo = Undo::from_any_base(object)?.unwrap();
      match undo.object().as_single_kind_str() {
        Some("Delete") => receive_undo_delete(undo, client, pool, chat_server).await,
        Some("Remove") => receive_undo_remove(undo, client, pool, chat_server).await,
        Some("Like") => receive_undo_like(undo, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    _ => receive_unhandled_activity(announce),
  }
}

fn receive_unhandled_activity<A>(activity: A) -> Result<HttpResponse, LemmyError>
where
  A: Debug,
{
  debug!("received unhandled activity type: {:?}", activity);
  Ok(HttpResponse::NotImplemented().finish())
}

async fn get_user_from_activity<T, A>(
  activity: &T,
  client: &Client,
  pool: &DbPool,
) -> Result<User_, LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let actor = activity.actor()?;
  let user_uri = actor.as_single_xsd_any_uri().unwrap();
  get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await
}

async fn receive_create_post(
  create: Create,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, client, pool).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, create, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool, &user.actor_id()?).await?;

  let inserted_post = blocking(pool, move |conn| Post::create(conn, &post)).await??;

  // Refetch the view
  let inserted_post_id = inserted_post.id;
  let post_view = blocking(pool, move |conn| {
    PostView::read(conn, inserted_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_comment(
  create: Create,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, client, pool).await?;
  let note = Note::from_any_base(create.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, create, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, &user.actor_id()?).await?;

  let inserted_comment = blocking(pool, move |conn| Comment::create(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
  let recipient_ids =
    send_local_notifs(mentions, inserted_comment.clone(), user, post, pool, true).await?;

  // Refetch the view
  let comment_view = blocking(pool, move |conn| {
    CommentView::read(conn, inserted_comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  chat_server.do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_post(
  update: Update,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&update, client, pool).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, update, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool, &user.actor_id()?).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  blocking(pool, move |conn| Post::update(conn, post_id, &post)).await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_like_post(
  like: Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&like, client, pool).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, like, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool, &user.actor_id()?).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
  blocking(pool, move |conn| {
    PostLike::remove(conn, &like_form)?;
    PostLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_dislike_post(
  dislike: Dislike,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&dislike, client, pool).await?;
  let page = PageExt::from_any_base(dislike.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, dislike, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool, &user.actor_id()?).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: -1,
  };
  blocking(pool, move |conn| {
    PostLike::remove(conn, &like_form)?;
    PostLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_comment(
  update: Update,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&update, client, pool).await?;

  insert_activity(user.id, update, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, &user.actor_id()?).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let updated_comment = blocking(pool, move |conn| {
    Comment::update(conn, comment_id, &comment)
  })
  .await??;

  let post_id = updated_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(mentions, updated_comment, user, post, pool, false).await?;

  // Refetch the view
  let comment_view =
    blocking(pool, move |conn| CommentView::read(conn, comment_id, None)).await??;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_like_comment(
  like: Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&like, client, pool).await?;

  insert_activity(user.id, like, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, &user.actor_id()?).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 1,
  };
  blocking(pool, move |conn| {
    CommentLike::remove(conn, &like_form)?;
    CommentLike::like(conn, &like_form)
  })
  .await??;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_dislike_comment(
  dislike: Dislike,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(dislike.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&dislike, client, pool).await?;

  insert_activity(user.id, dislike, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, &user.actor_id()?).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: -1,
  };
  blocking(pool, move |conn| {
    CommentLike::remove(conn, &like_form)?;
    CommentLike::like(conn, &like_form)
  })
  .await??;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_community(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let group = GroupExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&delete, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

  let community_actor_id = CommunityForm::from_apub(&group, client, pool, &user.actor_id()?)
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
    deleted: Some(true),
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

  insert_activity(mod_.id, remove, false, pool).await?;

  let community_actor_id = CommunityForm::from_apub(&group, client, pool, &mod_.actor_id()?)
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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_post(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, client, pool).await?;
  let page = PageExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, delete, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool, &user.actor_id()?)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_remote_post(&post_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_remove_post(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let page = PageExt::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(mod_.id, remove, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool, &mod_.actor_id()?)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_remote_post(&post_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_comment(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, client, pool).await?;
  let note = Note::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, delete, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool, &user.actor_id()?)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_remote_comment(&comment_ap_id, client, pool).await?;

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

  insert_activity(mod_.id, remove, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool, &mod_.actor_id()?)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_remote_comment(&comment_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
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
    "Note" => receive_undo_delete_comment(delete, client, pool, chat_server).await,
    "Page" => receive_undo_delete_post(delete, client, pool, chat_server).await,
    "Group" => receive_undo_delete_community(delete, client, pool, chat_server).await,
    d => Err(format_err!("Undo Delete type {} not supported", d).into()),
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
    "Note" => receive_undo_remove_comment(remove, client, pool, chat_server).await,
    "Page" => receive_undo_remove_post(remove, client, pool, chat_server).await,
    "Group" => receive_undo_remove_community(remove, client, pool, chat_server).await,
    d => Err(format_err!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_delete_comment(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, client, pool).await?;
  let note = Note::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, delete, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool, &user.actor_id()?)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_remote_comment(&comment_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_comment(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let note = Note::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(mod_.id, remove, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool, &mod_.actor_id()?)
    .await?
    .get_ap_id()?;

  let comment = get_or_fetch_and_insert_remote_comment(&comment_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_post(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, client, pool).await?;
  let page = PageExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, delete, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool, &user.actor_id()?)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_remote_post(&post_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_post(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let page = PageExt::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(mod_.id, remove, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool, &mod_.actor_id()?)
    .await?
    .get_ap_id()?;

  let post = get_or_fetch_and_insert_remote_post(&post_ap_id, client, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_community(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&delete, client, pool).await?;
  let group = GroupExt::from_any_base(delete.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, delete, false, pool).await?;

  let community_actor_id = CommunityForm::from_apub(&group, client, pool, &user.actor_id()?)
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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_remove_community(
  remove: Remove,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let mod_ = get_user_from_activity(&remove, client, pool).await?;
  let group = GroupExt::from_any_base(remove.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(mod_.id, remove, false, pool).await?;

  let community_actor_id = CommunityForm::from_apub(&group, client, pool, &mod_.actor_id()?)
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

  Ok(HttpResponse::Ok().finish())
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
    "Note" => receive_undo_like_comment(like, client, pool, chat_server).await,
    "Page" => receive_undo_like_post(like, client, pool, chat_server).await,
    d => Err(format_err!("Undo Delete type {} not supported", d).into()),
  }
}

async fn receive_undo_like_comment(
  like: Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&like, client, pool).await?;
  let note = Note::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, like, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool, &user.actor_id()?).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.get_ap_id()?, client, pool)
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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_like_post(
  like: Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&like, client, pool).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  insert_activity(user.id, like, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool, &user.actor_id()?).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.get_ap_id()?, client, pool)
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

  Ok(HttpResponse::Ok().finish())
}
