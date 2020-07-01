use crate::{
  api::user::PrivateMessageResponse,
  apub::{
    extensions::signatures::verify,
    fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user},
    FromApub,
  },
  blocking,
  db::{
    activity::insert_activity,
    community::{CommunityFollower, CommunityFollowerForm},
    private_message::{PrivateMessage, PrivateMessageForm},
    private_message_view::PrivateMessageView,
    user::User_,
    Crud,
    Followable,
  },
  naive_now,
  routes::{ChatServerParam, DbPoolParam},
  websocket::{server::SendUserRoomMessage, UserOperation},
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{Accept, Create, Delete, Undo, Update},
  object::Note,
};
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use log::debug;
use serde::Deserialize;
use std::fmt::Debug;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum UserAcceptedObjects {
  Accept(Box<Accept>),
  Create(Box<Create>),
  Update(Box<Update>),
  Delete(Box<Delete>),
  Undo(Box<Undo>),
}

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  request: HttpRequest,
  input: web::Json<UserAcceptedObjects>,
  path: web::Path<String>,
  client: web::Data<Client>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let username = path.into_inner();
  debug!("User {} received activity: {:?}", &username, &input);

  match input {
    UserAcceptedObjects::Accept(a) => receive_accept(*a, &request, &username, &client, &db).await,
    UserAcceptedObjects::Create(c) => {
      receive_create_private_message(*c, &request, &client, &db, chat_server).await
    }
    UserAcceptedObjects::Update(u) => {
      receive_update_private_message(*u, &request, &client, &db, chat_server).await
    }
    UserAcceptedObjects::Delete(d) => {
      receive_delete_private_message(*d, &request, &client, &db, chat_server).await
    }
    UserAcceptedObjects::Undo(u) => {
      receive_undo_delete_private_message(*u, &request, &client, &db, chat_server).await
    }
  }
}

/// Handle accepted follows.
async fn receive_accept(
  accept: Accept,
  request: &HttpRequest,
  username: &str,
  client: &Client,
  pool: &DbPool,
) -> Result<HttpResponse, LemmyError> {
  let community_uri = accept
    .accept_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let community = get_or_fetch_and_upsert_remote_community(&community_uri, client, pool).await?;
  verify(request, &community)?;

  let username = username.to_owned();
  let user = blocking(pool, move |conn| User_::read_from_name(conn, &username)).await??;

  insert_activity(community.creator_id, accept, false, pool).await?;

  // Now you need to add this to the community follower
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower
  blocking(pool, move |conn| {
    CommunityFollower::follow(conn, &community_follower_form)
  })
  .await??;

  // TODO: make sure that we actually requested a follow
  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_private_message(
  create: Create,
  request: &HttpRequest,
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
  verify(request, &user)?;

  insert_activity(user.id, create, false, pool).await?;

  let private_message = PrivateMessageForm::from_apub(&note, client, pool).await?;

  let inserted_private_message = blocking(pool, move |conn| {
    PrivateMessage::create(conn, &private_message)
  })
  .await??;

  let message = blocking(pool, move |conn| {
    PrivateMessageView::read(conn, inserted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::CreatePrivateMessage,
    response: res,
    recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_private_message(
  update: Update,
  request: &HttpRequest,
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
  verify(request, &user)?;

  insert_activity(user.id, update, false, pool).await?;

  let private_message_form = PrivateMessageForm::from_apub(&note, client, pool).await?;

  let private_message_ap_id = private_message_form.ap_id.clone();
  let private_message = blocking(pool, move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id)
  })
  .await??;

  let private_message_id = private_message.id;
  blocking(pool, move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let private_message_id = private_message.id;
  let message = blocking(pool, move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_private_message(
  delete: Delete,
  request: &HttpRequest,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;
  verify(request, &user)?;

  insert_activity(user.id, delete, false, pool).await?;

  let private_message_form = PrivateMessageForm::from_apub(&note, client, pool).await?;

  let private_message_ap_id = private_message_form.ap_id;
  let private_message = blocking(pool, move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id)
  })
  .await??;

  let private_message_form = PrivateMessageForm {
    content: private_message_form.content,
    recipient_id: private_message.recipient_id,
    creator_id: private_message.creator_id,
    deleted: Some(true),
    read: None,
    ap_id: private_message.ap_id,
    local: private_message.local,
    published: None,
    updated: Some(naive_now()),
  };

  let private_message_id = private_message.id;
  blocking(pool, move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let private_message_id = private_message.id;
  let message = blocking(pool, move |conn| {
    PrivateMessageView::read(&conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_private_message(
  undo: Undo,
  request: &HttpRequest,
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

  let note = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Note>()?;

  let user_uri = delete
    .delete_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool).await?;
  verify(request, &user)?;

  insert_activity(user.id, delete, false, pool).await?;

  let private_message = PrivateMessageForm::from_apub(&note, client, pool).await?;

  let private_message_ap_id = private_message.ap_id.clone();
  let private_message_id = blocking(pool, move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id).map(|pm| pm.id)
  })
  .await??;

  let private_message_form = PrivateMessageForm {
    content: private_message.content,
    recipient_id: private_message.recipient_id,
    creator_id: private_message.creator_id,
    deleted: Some(false),
    read: None,
    ap_id: private_message.ap_id,
    local: private_message.local,
    published: None,
    updated: Some(naive_now()),
  };

  blocking(pool, move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let message = blocking(pool, move |conn| {
    PrivateMessageView::read(&conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}
