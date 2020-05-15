use super::*;

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
    (SharedAcceptedObjects::Delete(d), Some("Page")) => {
      receive_delete_post(&d, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Remove(r), Some("Page")) => {
      receive_remove_post(&r, &request, &conn, chat_server)
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
    (SharedAcceptedObjects::Delete(d), Some("Note")) => {
      receive_delete_comment(&d, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Remove(r), Some("Note")) => {
      receive_remove_comment(&r, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Delete(d), Some("Group")) => {
      receive_delete_community(&d, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Remove(r), Some("Group")) => {
      receive_remove_community(&r, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Undo(u), Some("Delete")) => {
      receive_undo_delete(&u, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Undo(u), Some("Remove")) => {
      receive_undo_remove(&u, &request, &conn, chat_server)
    }
    (SharedAcceptedObjects::Undo(u), Some("Like")) => {
      receive_undo_like(&u, &request, &conn, chat_server)
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
    .into_concrete::<PageExt>()?;

  let user_uri = create
    .create_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &create, false)?;

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
    .into_concrete::<Note>()?;

  let user_uri = create
    .create_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &create, false)?;

  let comment = CommentForm::from_apub(&note, &conn)?;
  let inserted_comment = Comment::create(conn, &comment)?;
  let post = Post::read(&conn, inserted_comment.post_id)?;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
  let recipient_ids = send_local_notifs(&conn, &mentions, &inserted_comment, &user, &post);

  // Refetch the view
  let comment_view = CommentView::read(&conn, inserted_comment.id, None)?;

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
    .into_concrete::<PageExt>()?;

  let user_uri = update
    .update_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &update, false)?;

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
    .into_concrete::<PageExt>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &like, false)?;

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
    .into_concrete::<PageExt>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &dislike, false)?;

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
    .into_concrete::<Note>()?;

  let user_uri = update
    .update_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &update, false)?;

  let comment = CommentForm::from_apub(&note, &conn)?;
  let comment_id = Comment::read_from_apub_id(conn, &comment.ap_id)?.id;
  let updated_comment = Comment::update(conn, comment_id, &comment)?;
  let post = Post::read(&conn, updated_comment.post_id)?;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(&conn, &mentions, &updated_comment, &user, &post);

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment_id, None)?;

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
    .into_concrete::<Note>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &like, false)?;

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
    .into_concrete::<Note>()?;

  let user_uri = dislike
    .dislike_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &dislike, false)?;

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

fn receive_delete_community(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let community_actor_id = CommunityForm::from_apub(&group, &conn)?.actor_id;
  let community = Community::read_from_actor_id(conn, &community_actor_id)?;

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

  Community::update(&conn, community.id, &community_form)?;

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

fn receive_remove_community(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let community_actor_id = CommunityForm::from_apub(&group, &conn)?.actor_id;
  let community = Community::read_from_actor_id(conn, &community_actor_id)?;

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

  Community::update(&conn, community.id, &community_form)?;

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

fn receive_delete_post(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let post_ap_id = PostForm::from_apub(&page, conn)?.ap_id;
  let post = Post::read_from_apub_id(conn, &post_ap_id)?;

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
  Post::update(&conn, post.id, &post_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post.id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_remove_post(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let post_ap_id = PostForm::from_apub(&page, conn)?.ap_id;
  let post = Post::read_from_apub_id(conn, &post_ap_id)?;

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
  Post::update(&conn, post.id, &post_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post.id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_delete_comment(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let comment_ap_id = CommentForm::from_apub(&note, &conn)?.ap_id;
  let comment = Comment::read_from_apub_id(conn, &comment_ap_id)?;
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
  Comment::update(&conn, comment.id, &comment_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment.id, None)?;

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

fn receive_remove_comment(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let comment_ap_id = CommentForm::from_apub(&note, &conn)?.ap_id;
  let comment = Comment::read_from_apub_id(conn, &comment_ap_id)?;
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
  Comment::update(&conn, comment.id, &comment_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment.id, None)?;

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

fn receive_undo_delete(
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

  let type_ = delete
    .delete_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .kind()
    .unwrap();

  match type_ {
    "Note" => receive_undo_delete_comment(&delete, &request, &conn, chat_server),
    "Page" => receive_undo_delete_post(&delete, &request, &conn, chat_server),
    "Group" => receive_undo_delete_community(&delete, &request, &conn, chat_server),
    d => Err(format_err!("Undo Delete type {} not supported", d)),
  }
}

fn receive_undo_remove(
  undo: &Undo,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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
    "Note" => receive_undo_remove_comment(&remove, &request, &conn, chat_server),
    "Page" => receive_undo_remove_post(&remove, &request, &conn, chat_server),
    "Group" => receive_undo_remove_community(&remove, &request, &conn, chat_server),
    d => Err(format_err!("Undo Delete type {} not supported", d)),
  }
}

fn receive_undo_delete_comment(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let comment_ap_id = CommentForm::from_apub(&note, &conn)?.ap_id;
  let comment = Comment::read_from_apub_id(conn, &comment_ap_id)?;
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
  Comment::update(&conn, comment.id, &comment_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment.id, None)?;

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

fn receive_undo_remove_comment(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let comment_ap_id = CommentForm::from_apub(&note, &conn)?.ap_id;
  let comment = Comment::read_from_apub_id(conn, &comment_ap_id)?;
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
  Comment::update(&conn, comment.id, &comment_form)?;

  // Refetch the view
  let comment_view = CommentView::read(&conn, comment.id, None)?;

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

fn receive_undo_delete_post(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let post_ap_id = PostForm::from_apub(&page, conn)?.ap_id;
  let post = Post::read_from_apub_id(conn, &post_ap_id)?;

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
  Post::update(&conn, post.id, &post_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post.id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_undo_remove_post(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let post_ap_id = PostForm::from_apub(&page, conn)?.ap_id;
  let post = Post::read_from_apub_id(conn, &post_ap_id)?;

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
  Post::update(&conn, post.id, &post_form)?;

  // Refetch the view
  let post_view = PostView::read(&conn, post.id, None)?;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

fn receive_undo_delete_community(
  delete: &Delete,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &delete, false)?;

  let community_actor_id = CommunityForm::from_apub(&group, &conn)?.actor_id;
  let community = Community::read_from_actor_id(conn, &community_actor_id)?;

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

  Community::update(&conn, community.id, &community_form)?;

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

fn receive_undo_remove_community(
  remove: &Remove,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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

  let mod_ = get_or_fetch_and_upsert_remote_user(&mod_uri, &conn)?;
  verify(request, &mod_)?;

  insert_activity(&conn, mod_.id, &remove, false)?;

  let community_actor_id = CommunityForm::from_apub(&group, &conn)?.actor_id;
  let community = Community::read_from_actor_id(conn, &community_actor_id)?;

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

  Community::update(&conn, community.id, &community_form)?;

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

fn receive_undo_like(
  undo: &Undo,
  request: &HttpRequest,
  conn: &PgConnection,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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
    "Note" => receive_undo_like_comment(&like, &request, &conn, chat_server),
    "Page" => receive_undo_like_post(&like, &request, &conn, chat_server),
    d => Err(format_err!("Undo Delete type {} not supported", d)),
  }
}

fn receive_undo_like_comment(
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
    .into_concrete::<Note>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &like, false)?;

  let comment = CommentForm::from_apub(&note, &conn)?;
  let comment_id = Comment::read_from_apub_id(conn, &comment.ap_id)?.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 0,
  };
  CommentLike::remove(&conn, &like_form)?;

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

fn receive_undo_like_post(
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
    .into_concrete::<PageExt>()?;

  let user_uri = like.like_props.get_actor_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(request, &user)?;

  insert_activity(&conn, user.id, &like, false)?;

  let post = PostForm::from_apub(&page, conn)?;
  let post_id = Post::read_from_apub_id(conn, &post.ap_id)?.id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
  PostLike::remove(&conn, &like_form)?;

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
