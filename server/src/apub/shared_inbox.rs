use super::*;

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug)]
pub enum SharedAcceptedObjects {
  Create(Create),
  Update(Update),
}

/// Handler for all incoming activities to user inboxes.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<SharedAcceptedObjects>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let conn = &db.get().unwrap();

  let json = serde_json::to_string(&input)?;
  debug!("Shared inbox received activity: {:?}", &json);

  match input {
    SharedAcceptedObjects::Create(c) => handle_create(&c, &request, &conn, chat_server),
    SharedAcceptedObjects::Update(u) => handle_update(&u, &request, &conn, chat_server),
  }
}

/// Handle create activities and insert them in the database.
fn handle_create(
  create: &Create,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let base_box = create.create_props.get_object_base_box().unwrap();

  if base_box.is_kind(PageType) {
    let page = create
      .create_props
      .get_object_base_box()
      .to_owned()
      .unwrap()
      .to_owned()
      .to_concrete::<Page>()?;
    receive_create_post(&create, &page, &request, &conn, chat_server)?;
  } else if base_box.is_kind(NoteType) {
    let note = create
      .create_props
      .get_object_base_box()
      .to_owned()
      .unwrap()
      .to_owned()
      .to_concrete::<Note>()?;
    receive_create_comment(&create, &note, &request, &conn, chat_server)?;
  } else {
    return Err(format_err!("Unknown base box type"));
  }

  Ok(HttpResponse::Ok().finish())
}

fn receive_create_post(
  create: &Create,
  page: &Page,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<(), Error> {
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

  let post = PostForm::from_apub(&page, &conn)?;
  let inserted_post = Post::create(conn, &post)?;

  // Refetch the view
  let post_view = PostView::read(&conn, inserted_post.id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    my_id: None,
  });

  Ok(())
}

fn receive_create_comment(
  create: &Create,
  note: &Note,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<(), Error> {
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

  let comment = CommentForm::from_apub(&note, &conn)?;
  let inserted_comment = Comment::create(conn, &comment)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, inserted_comment.id, None)?;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
  };

  chat_server.do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    my_id: None,
  });

  Ok(())
}

/// Handle create activities and insert them in the database.
fn handle_update(
  update: &Update,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let base_box = update.update_props.get_object_base_box().unwrap();

  if base_box.is_kind(PageType) {
    let page = update
      .update_props
      .get_object_base_box()
      .to_owned()
      .unwrap()
      .to_owned()
      .to_concrete::<Page>()?;

    receive_update_post(&update, &page, &request, &conn, chat_server)?;
  } else if base_box.is_kind(NoteType) {
    let note = update
      .update_props
      .get_object_base_box()
      .to_owned()
      .unwrap()
      .to_owned()
      .to_concrete::<Note>()?;
    receive_update_comment(&update, &note, &request, &conn, chat_server)?;
  } else {
    return Err(format_err!("Unknown base box type"));
  }

  Ok(HttpResponse::Ok().finish())
}

fn receive_update_post(
  update: &Update,
  page: &Page,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<(), Error> {
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

  let post = PostForm::from_apub(&page, conn)?;
  let post_id = Post::read_from_apub_id(conn, &post.ap_id)?.id;
  Post::update(conn, post_id, &post)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post_id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(())
}

fn receive_update_comment(
  update: &Update,
  note: &Note,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<(), Error> {
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

  let comment = CommentForm::from_apub(&note, &conn)?;
  let comment_id = Comment::read_from_apub_id(conn, &comment.ap_id)?.id;
  Comment::update(conn, comment_id, &comment)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment_id, None)?;

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

  Ok(())
}
