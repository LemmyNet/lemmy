use super::*;

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug)]
pub enum SharedAcceptedObjects {
  Create(Create),
  Update(Update),
  Like(Like),
  Dislike(Dislike),
  Delete(Delete),
}

impl SharedAcceptedObjects {
  fn object(&self) -> Option<&BaseBox> {
    match self {
      SharedAcceptedObjects::Create(c) => c.create_props.get_object_base_box(),
      SharedAcceptedObjects::Update(u) => u.update_props.get_object_base_box(),
      SharedAcceptedObjects::Like(l) => l.like_props.get_object_base_box(),
      SharedAcceptedObjects::Dislike(d) => d.dislike_props.get_object_base_box(),
      SharedAcceptedObjects::Delete(d) => d.delete_props.get_object_base_box(),
    }
  }
}

/// Handler for all incoming activities to user inboxes.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<SharedAcceptedObjects>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let activity = input.into_inner();
  let conn = &db.get().unwrap();

  let json = serde_json::to_string(&activity)?;
  debug!("Shared inbox received activity: {}", json);

  let object = activity.object().cloned().unwrap();

  match (activity, object.kind()) {
    (SharedAcceptedObjects::Create(c), Some("Page")) => {
      receive_create_post(&c, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Update(u), Some("Page")) => {
      receive_update_post(&u, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Like(l), Some("Page")) => {
      receive_like_post(&l, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Dislike(d), Some("Page")) => {
      receive_dislike_post(&d, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Create(c), Some("Note")) => {
      receive_create_comment(&c, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Update(u), Some("Note")) => {
      receive_update_comment(&u, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Like(l), Some("Note")) => {
      receive_like_comment(&l, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Dislike(d), Some("Note")) => {
      receive_dislike_comment(&d, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Delete(d), Some("Tombstone")) => {
      receive_delete(&d, &request, &conn, chat_server)
    }
    _ => Err(format_err!("Unknown incoming activity type.")),
  }
}

fn receive_create_post(
  create: &Create,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let page = create
    .create_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;

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

  Ok(HttpResponse::Ok().finish())
}

fn receive_create_comment(
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
    .to_concrete::<Note>()?;

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

  Ok(HttpResponse::Ok().finish())
}

fn receive_update_post(
  update: &Update,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let page = update
    .update_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;

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

  Ok(HttpResponse::Ok().finish())
}

fn receive_like_post(
  like: &Like,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let page = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&like)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let post = PostForm::from_apub(&page, conn)?;
  let post_id = Post::read_from_apub_id(conn, &post.ap_id)?.id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
  PostLike::remove(&conn, &like_form)?;
  PostLike::like(&conn, &like_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post_id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_dislike_post(
  dislike: &Dislike,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let page = dislike
    .dislike_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&dislike)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let post = PostForm::from_apub(&page, conn)?;
  let post_id = Post::read_from_apub_id(conn, &post.ap_id)?.id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: -1,
  };
  PostLike::remove(&conn, &like_form)?;
  PostLike::like(&conn, &like_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post_id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_update_comment(
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
    .to_concrete::<Note>()?;

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

  Ok(HttpResponse::Ok().finish())
}

fn receive_like_comment(
  like: &Like,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let note = like
    .like_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Note>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&like)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let comment = CommentForm::from_apub(&note, &conn)?;
  let comment_id = Comment::read_from_apub_id(conn, &comment.ap_id)?.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 1,
  };
  CommentLike::remove(&conn, &like_form)?;
  CommentLike::like(&conn, &like_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment_id, None)?;

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

fn receive_dislike_comment(
  dislike: &Dislike,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let note = dislike
    .dislike_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Note>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&dislike)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let comment = CommentForm::from_apub(&note, &conn)?;
  let comment_id = Comment::read_from_apub_id(conn, &comment.ap_id)?.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: -1,
  };
  CommentLike::remove(&conn, &like_form)?;
  CommentLike::like(&conn, &like_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment_id, None)?;

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

fn receive_delete(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let tombstone = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Tombstone>()?;
  // TODO: not sure how to handle formerType (should be a string)
  // https://www.w3.org/TR/activitystreams-vocabulary/#dfn-formertype
  let former_type: &str = tombstone.tombstone_props.get_former_type_object_box().unwrap().to_concrete::<String>();
  match former_type {
    "Group" => {},
    d => return Err(format_err!("Delete type {} not supported", d)),
  }
  let community_apub_id = tombstone.object_props.get_id().unwrap().to_string();

  let community = Community::read_from_actor_id(conn, &community_apub_id)?;
  verify(request, &community.public_key.clone().unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: community.creator_id,
    data: serde_json::to_value(&delete)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let community_form = CommunityForm {
    name: "".to_string(),
    title: "".to_string(),
    description: None,
    category_id: community.category_id, // Note: need to keep this due to foreign key constraint
    creator_id: community.creator_id,   // Note: need to keep this due to foreign key constraint
    removed: None,
    published: None,
    updated: None,
    deleted: Some(true),
    nsfw: false,
    actor_id: community.actor_id,
    local: false,
    private_key: None,
    public_key: community.public_key,
    last_refreshed_at: Some(community.last_refreshed_at),
  };

  Community::update(conn, community.id, &community_form)?;

  let res = CommunityResponse {
    community: CommunityView::read(&conn, community.id, None)?,
  };

  chat_server.do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id: community.id,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}
