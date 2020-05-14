use super::*;

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
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  let username = path.into_inner();
  debug!("User {} received activity: {:?}", &username, &input);

  match input {
    UserAcceptedObjects::Accept(a) => receive_accept(&a, &request, &username, &conn),
    UserAcceptedObjects::Create(c) => {
      receive_create_private_message(&c, &request, &conn, chat_server)
    }
    UserAcceptedObjects::Update(u) => {
      receive_update_private_message(&u, &request, &conn, chat_server)
    }
    UserAcceptedObjects::Delete(d) => {
      receive_delete_private_message(&d, &request, &conn, chat_server)
    }
    UserAcceptedObjects::Undo(u) => {
      receive_undo_delete_private_message(&u, &request, &conn, chat_server)
    }
  }
}

/// Handle accepted follows.
fn receive_accept(
  accept: &Accept,
  request: &HttpRequest,
  username: &str,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  let community_uri = accept
    .accept_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let community = get_or_fetch_and_upsert_remote_community(&community_uri, conn)?;
  verify(request, &community.public_key.unwrap())?;

  let user = User_::read_from_name(&conn, username)?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: community.creator_id,
    data: serde_json::to_value(&accept)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  // Now you need to add this to the community follower
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower
  CommunityFollower::follow(&conn, &community_follower_form)?;

  // TODO: make sure that we actually requested a follow
  Ok(HttpResponse::Ok().finish())
}

fn receive_create_private_message(
  create: &Create,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&create)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let private_message = PrivateMessageForm::from_apub(&note, &conn)?;
  let inserted_private_message = PrivateMessage::create(&conn, &private_message)?;

  let message = PrivateMessageView::read(&conn, inserted_private_message.id)?;

  let res = PrivateMessageResponse {
    message: message.to_owned(),
  };

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::CreatePrivateMessage,
    response: res,
    recipient_id: message.recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_update_private_message(
  update: &Update,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&update)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let private_message = PrivateMessageForm::from_apub(&note, &conn)?;
  let private_message_id = PrivateMessage::read_from_apub_id(&conn, &private_message.ap_id)?.id;
  PrivateMessage::update(conn, private_message_id, &private_message)?;

  let message = PrivateMessageView::read(&conn, private_message_id)?;

  let res = PrivateMessageResponse {
    message: message.to_owned(),
  };

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id: message.recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_delete_private_message(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&delete)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let private_message = PrivateMessageForm::from_apub(&note, &conn)?;
  let private_message_id = PrivateMessage::read_from_apub_id(&conn, &private_message.ap_id)?.id;
  let private_message_form = PrivateMessageForm {
    content: private_message.content,
    recipient_id: private_message.recipient_id,
    creator_id: private_message.creator_id,
    deleted: Some(true),
    read: None,
    ap_id: private_message.ap_id,
    local: private_message.local,
    published: None,
    updated: Some(naive_now()),
  };
  PrivateMessage::update(conn, private_message_id, &private_message_form)?;

  let message = PrivateMessageView::read(&conn, private_message_id)?;

  let res = PrivateMessageResponse {
    message: message.to_owned(),
  };

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id: message.recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_undo_delete_private_message(
  undo: &Undo,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&delete)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let private_message = PrivateMessageForm::from_apub(&note, &conn)?;
  let private_message_id = PrivateMessage::read_from_apub_id(&conn, &private_message.ap_id)?.id;
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
  PrivateMessage::update(conn, private_message_id, &private_message_form)?;

  let message = PrivateMessageView::read(&conn, private_message_id)?;

  let res = PrivateMessageResponse {
    message: message.to_owned(),
  };

  chat_server.do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id: message.recipient_id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}
