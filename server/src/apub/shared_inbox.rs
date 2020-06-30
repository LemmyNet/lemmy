use crate::{
  api::{
    comment::{send_local_notifs, CommentResponse},
    community::CommunityResponse,
    post::PostResponse,
  },
  apub::{
    extensions::signatures::verify,
    fetcher::{
      get_or_fetch_and_insert_remote_comment,
      get_or_fetch_and_insert_remote_post,
      get_or_fetch_and_upsert_remote_community,
      get_or_fetch_and_upsert_remote_user,
    },
    FromApub,
    GroupExt,
    PageExt,
  },
  blocking,
  db::{
    activity::insert_activity,
    comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
    comment_view::CommentView,
    community::{Community, CommunityForm},
    community_view::CommunityView,
    post::{Post, PostForm, PostLike, PostLikeForm},
    post_view::PostView,
    Crud,
    Likeable,
  },
  naive_now,
  routes::{ChatServerParam, DbPoolParam},
  scrape_text_for_mentions,
  websocket::{
    server::{SendComment, SendCommunityRoomMessage, SendPost},
    UserOperation,
  },
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{Announce, Create, Delete, Dislike, Like, Remove, Undo, Update},
  object::Note,
  Activity,
  Base,
  BaseBox,
};
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug)]
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
  fn object(&self) -> Option<&BaseBox> {
    match self {
      SharedAcceptedObjects::Create(c) => c.create_props.get_object_base_box(),
      SharedAcceptedObjects::Update(u) => u.update_props.get_object_base_box(),
      SharedAcceptedObjects::Like(l) => l.like_props.get_object_base_box(),
      SharedAcceptedObjects::Dislike(d) => d.dislike_props.get_object_base_box(),
      SharedAcceptedObjects::Delete(d) => d.delete_props.get_object_base_box(),
      SharedAcceptedObjects::Undo(d) => d.undo_props.get_object_base_box(),
      SharedAcceptedObjects::Remove(r) => r.remove_props.get_object_base_box(),
      SharedAcceptedObjects::Announce(a) => a.announce_props.get_object_base_box(),
    }
  }
  fn sender(&self) -> String {
    let uri = match self {
      SharedAcceptedObjects::Create(c) => c.create_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Update(u) => u.update_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Like(l) => l.like_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Dislike(d) => d.dislike_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Delete(d) => d.delete_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Undo(d) => d.undo_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Remove(r) => r.remove_props.get_actor_xsd_any_uri(),
      SharedAcceptedObjects::Announce(a) => a.announce_props.get_actor_xsd_any_uri(),
    };
    uri.unwrap().clone().to_string()
  }
  fn cc(&self) -> String {
    // TODO: there is probably an easier way to do this
    let oprops = match self {
      SharedAcceptedObjects::Create(c) => &c.object_props,
      SharedAcceptedObjects::Update(u) => &u.object_props,
      SharedAcceptedObjects::Like(l) => &l.object_props,
      SharedAcceptedObjects::Dislike(d) => &d.object_props,
      SharedAcceptedObjects::Delete(d) => &d.object_props,
      SharedAcceptedObjects::Undo(d) => &d.object_props,
      SharedAcceptedObjects::Remove(r) => &r.object_props,
      SharedAcceptedObjects::Announce(a) => &a.object_props,
    };
    oprops
      .get_many_cc_xsd_any_uris()
      .unwrap()
      .next()
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

  let object = activity.object().cloned().unwrap();
  let sender = &activity.sender();
  let cc = &activity.cc();
  // TODO: this is hacky, we should probably send the community id directly somehow
  let to = cc.replace("/followers", "");

  // TODO: this is ugly
  match get_or_fetch_and_upsert_remote_user(&sender.to_string(), &client, pool).await {
    Ok(u) => verify(&request, &u)?,
    Err(_) => {
      let c = get_or_fetch_and_upsert_remote_community(&sender.to_string(), &client, pool).await?;
      verify(&request, &c)?;
    }
  }

  match (activity, object.kind()) {
    (SharedAcceptedObjects::Create(c), Some("Page")) => {
      receive_create_post((*c).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Create>(*c, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Update(u), Some("Page")) => {
      receive_update_post((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Update>(*u, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Like(l), Some("Page")) => {
      receive_like_post((*l).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Like>(*l, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Dislike(d), Some("Page")) => {
      receive_dislike_post((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Dislike>(*d, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Page")) => {
      receive_delete_post((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Delete>(*d, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Page")) => {
      receive_remove_post((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Remove>(*r, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Create(c), Some("Note")) => {
      receive_create_comment((*c).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Create>(*c, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Update(u), Some("Note")) => {
      receive_update_comment((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Update>(*u, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Like(l), Some("Note")) => {
      receive_like_comment((*l).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Like>(*l, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Dislike(d), Some("Note")) => {
      receive_dislike_comment((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Dislike>(*d, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Note")) => {
      receive_delete_comment((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Delete>(*d, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Note")) => {
      receive_remove_comment((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Remove>(*r, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Delete(d), Some("Group")) => {
      receive_delete_community((*d).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Delete>(*d, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Remove(r), Some("Group")) => {
      receive_remove_community((*r).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Remove>(*r, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Delete")) => {
      receive_undo_delete((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Undo>(*u, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Remove")) => {
      receive_undo_remove((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Undo>(*u, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Undo(u), Some("Like")) => {
      receive_undo_like((*u).clone(), client, pool, chat_server).await?;
      announce_activity_if_valid::<Undo>(*u, &to, sender, client, pool).await
    }
    (SharedAcceptedObjects::Announce(a), _) => receive_announce(a, client, pool, chat_server).await,
    (a, _) => receive_unhandled_activity(a),
  }
}

// TODO: should pass in sender as ActorType, but thats a bit tricky in shared_inbox()
async fn announce_activity_if_valid<A>(
  activity: A,
  community_uri: &str,
  sender: &str,
  client: &Client,
  pool: &DbPool,
) -> Result<HttpResponse, LemmyError>
where
  A: Activity + Base + Serialize + Debug,
{
  let community_uri = community_uri.to_owned();
  let community = blocking(pool, move |conn| {
    Community::read_from_actor_id(conn, &community_uri)
  })
  .await??;

  if community.local {
    let sending_user = get_or_fetch_and_upsert_remote_user(sender, client, pool).await?;

    Community::do_announce(activity, &community, &sending_user, client, pool).await
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
  let object = announce
    .announce_props
    .get_object_base_box()
    .unwrap()
    .to_owned();
  // TODO: too much copy paste
  match object.kind() {
    Some("Create") => {
      let create = object.into_concrete::<Create>()?;
      let inner_object = create.create_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_create_post(create, client, pool, chat_server).await,
        Some("Note") => receive_create_comment(create, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Update") => {
      let update = object.into_concrete::<Update>()?;
      let inner_object = update.update_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_update_post(update, client, pool, chat_server).await,
        Some("Note") => receive_update_comment(update, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Like") => {
      let like = object.into_concrete::<Like>()?;
      let inner_object = like.like_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_like_post(like, client, pool, chat_server).await,
        Some("Note") => receive_like_comment(like, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Dislike") => {
      let dislike = object.into_concrete::<Dislike>()?;
      let inner_object = dislike.dislike_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_dislike_post(dislike, client, pool, chat_server).await,
        Some("Note") => receive_dislike_comment(dislike, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Delete") => {
      let delete = object.into_concrete::<Delete>()?;
      let inner_object = delete.delete_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_delete_post(delete, client, pool, chat_server).await,
        Some("Note") => receive_delete_comment(delete, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Remove") => {
      let remove = object.into_concrete::<Remove>()?;
      let inner_object = remove.remove_props.get_object_base_box().unwrap();
      match inner_object.kind() {
        Some("Page") => receive_remove_post(remove, client, pool, chat_server).await,
        Some("Note") => receive_remove_comment(remove, client, pool, chat_server).await,
        _ => receive_unhandled_activity(announce),
      }
    }
    Some("Undo") => {
      let undo = object.into_concrete::<Undo>()?;
      let inner_object = undo.undo_props.get_object_base_box().unwrap();
      match inner_object.kind() {
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

async fn receive_create_post(
  create: Create,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let page = create
    .create_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user_uri = create
    .create_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, create, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool).await?;

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
  let note = create
    .create_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = create
    .create_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, create, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let inserted_comment = blocking(pool, move |conn| Comment::create(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
  let recipient_ids =
    send_local_notifs(mentions, inserted_comment.clone(), user, post, pool).await?;

  // Refetch the view
  let comment_view = blocking(pool, move |conn| {
    CommentView::read(conn, inserted_comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
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
  let page = update
    .update_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user_uri = update
    .update_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, update, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.ap_id, client, pool)
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
  let page = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, like, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.ap_id, client, pool)
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
  let page = dislike
    .dislike_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, dislike, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.ap_id, client, pool)
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
  let note = update
    .update_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = update
    .update_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, update, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.ap_id, client, pool)
    .await?
    .id;

  let updated_comment = blocking(pool, move |conn| {
    Comment::update(conn, comment_id, &comment)
  })
  .await??;

  let post_id = updated_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(mentions, updated_comment, user, post, pool).await?;

  // Refetch the view
  let comment_view =
    blocking(pool, move |conn| CommentView::read(conn, comment_id, None)).await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
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
  let note = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, like, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.ap_id, client, pool)
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
  let note = dislike
    .dislike_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, dislike, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.ap_id, client, pool)
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
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let group = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<GroupExt>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let group = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<GroupExt>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_post(
  delete: Delete,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let page = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool).await?.ap_id;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let page = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool).await?.ap_id;

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
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let note = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool).await?.ap_id;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let note = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool).await?.ap_id;

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
  let delete = undo
    .undo_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Delete>()?;

  let type_ = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .kind()
    .unwrap();

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
  let remove = undo
    .undo_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Remove>()?;

  let type_ = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .kind()
    .unwrap();

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
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let note = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool).await?.ap_id;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let note = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

  let comment_ap_id = CommentForm::from_apub(&note, client, pool).await?.ap_id;

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
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let page = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool).await?.ap_id;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let page = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

  let post_ap_id = PostForm::from_apub(&page, client, pool).await?.ap_id;

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
  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let group = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<GroupExt>()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, delete, false, pool).await?;

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
  let mod_uri = remove
    .remove_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let group = remove
    .remove_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<GroupExt>()?;

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, client, pool).await?;

  insert_activity(mod_.id, remove, false, pool).await?;

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
  let like = undo
    .undo_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Like>()?;

  let type_ = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .kind()
    .unwrap();

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
  let note = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, like, false, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_remote_comment(&comment.ap_id, client, pool)
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
  let page = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<PageExt>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;

  insert_activity(user.id, like, false, pool).await?;

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_remote_post(&post.ap_id, client, pool)
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
