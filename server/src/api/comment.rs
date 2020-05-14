use super::*;

#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  content: String,
  parent_id: Option<i32>,
  edit_id: Option<i32>, // TODO this isn't used
  pub post_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditComment {
  content: String,
  parent_id: Option<i32>, // TODO why are the parent_id, creator_id, post_id, etc fields required? They aren't going to change
  edit_id: i32,
  creator_id: i32,
  pub post_id: i32,
  removed: Option<bool>,
  deleted: Option<bool>,
  reason: Option<String>,
  read: Option<bool>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveComment {
  comment_id: i32,
  save: bool,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentResponse {
  pub comment: CommentView,
  pub recipient_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  comment_id: i32,
  pub post_id: i32,
  score: i16,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetComments {
  type_: String,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  pub community_id: Option<i32>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommentsResponse {
  comments: Vec<CommentView>,
}

impl Perform for Oper<CreateComment> {
  type Response = CommentResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, Error> {
    let data: &CreateComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    // Check for a community ban
    let post = Post::read(&conn, data.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = User_::read(&conn, user_id)?;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: data.parent_id.to_owned(),
      post_id: data.post_id,
      creator_id: user_id,
      removed: None,
      deleted: None,
      read: None,
      published: None,
      updated: None,
      ap_id: "changeme".into(),
      local: true,
    };

    let inserted_comment = match Comment::create(&conn, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_create_comment").into()),
    };

    let updated_comment = match Comment::update_ap_id(&conn, inserted_comment.id) {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_create_comment").into()),
    };

    updated_comment.send_create(&user, &conn)?;

    // Scan the comment for user mentions, add those rows
    let mentions = scrape_text_for_mentions(&comment_form.content);
    let recipient_ids = send_local_notifs(&conn, &mentions, &updated_comment, &user, &post);

    // You like your own comment by default
    let like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: data.post_id,
      user_id,
      score: 1,
    };

    let _inserted_like = match CommentLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => return Err(APIError::err("couldnt_like_comment").into()),
    };

    updated_comment.send_like(&user, &conn)?;

    let comment_view = CommentView::read(&conn, inserted_comment.id, Some(user_id))?;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendComment {
        op: UserOperation::CreateComment,
        comment: res.clone(),
        my_id: ws.id,
      });

      // strip out the recipient_ids, so that
      // users don't get double notifs
      res.recipient_ids = Vec::new();
    }

    Ok(res)
  }
}

impl Perform for Oper<EditComment> {
  type Response = CommentResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, Error> {
    let data: &EditComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    let user = User_::read(&conn, user_id)?;

    let orig_comment = CommentView::read(&conn, data.edit_id, None)?;

    // You are allowed to mark the comment as read even if you're banned.
    if data.read.is_none() {
      // Verify its the creator or a mod, or an admin
      let mut editors: Vec<i32> = vec![data.creator_id];
      editors.append(
        &mut CommunityModeratorView::for_community(&conn, orig_comment.community_id)?
          .into_iter()
          .map(|m| m.user_id)
          .collect(),
      );
      editors.append(&mut UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());

      if !editors.contains(&user_id) {
        return Err(APIError::err("no_comment_edit_allowed").into());
      }

      // Check for a community ban
      if CommunityUserBanView::get(&conn, user_id, orig_comment.community_id).is_ok() {
        return Err(APIError::err("community_ban").into());
      }

      // Check for a site ban
      if user.banned {
        return Err(APIError::err("site_ban").into());
      }
    }

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let read_comment = Comment::read(&conn, data.edit_id)?;

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: data.parent_id,
      post_id: data.post_id,
      creator_id: data.creator_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
      read: data.read.to_owned(),
      published: None,
      updated: if data.read.is_some() {
        orig_comment.updated
      } else {
        Some(naive_now())
      },
      ap_id: read_comment.ap_id,
      local: read_comment.local,
    };

    let updated_comment = match Comment::update(&conn, data.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    if let Some(deleted) = data.deleted.to_owned() {
      if deleted {
        updated_comment.send_delete(&user, &conn)?;
      } else {
        updated_comment.send_undo_delete(&user, &conn)?;
      }
    } else if let Some(removed) = data.removed.to_owned() {
      if removed {
        updated_comment.send_remove(&user, &conn)?;
      } else {
        updated_comment.send_undo_remove(&user, &conn)?;
      }
    } else {
      updated_comment.send_update(&user, &conn)?;
    }

    let post = Post::read(&conn, data.post_id)?;

    let mentions = scrape_text_for_mentions(&comment_form.content);
    let recipient_ids = send_local_notifs(&conn, &mentions, &updated_comment, &user, &post);

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let form = ModRemoveCommentForm {
        mod_user_id: user_id,
        comment_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
      };
      ModRemoveComment::create(&conn, &form)?;
    }

    let comment_view = CommentView::read(&conn, data.edit_id, Some(user_id))?;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendComment {
        op: UserOperation::EditComment,
        comment: res.clone(),
        my_id: ws.id,
      });

      // strip out the recipient_ids, so that
      // users don't get double notifs
      res.recipient_ids = Vec::new();
    }

    Ok(res)
  }
}

impl Perform for Oper<SaveComment> {
  type Response = CommentResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, Error> {
    let data: &SaveComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let comment_saved_form = CommentSavedForm {
      comment_id: data.comment_id,
      user_id,
    };

    let conn = pool.get()?;

    if data.save {
      match CommentSaved::save(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err("couldnt_save_comment").into()),
      };
    } else {
      match CommentSaved::unsave(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err("couldnt_save_comment").into()),
      };
    }

    let comment_view = CommentView::read(&conn, data.comment_id, Some(user_id))?;

    Ok(CommentResponse {
      comment: comment_view,
      recipient_ids: Vec::new(),
    })
  }
}

impl Perform for Oper<CreateCommentLike> {
  type Response = CommentResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, Error> {
    let data: &CreateCommentLike = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let mut recipient_ids = Vec::new();

    let conn = pool.get()?;

    // Don't do a downvote if site has downvotes disabled
    if data.score == -1 {
      let site = SiteView::read(&conn)?;
      if !site.enable_downvotes {
        return Err(APIError::err("downvotes_disabled").into());
      }
    }

    // Check for a community ban
    let post = Post::read(&conn, data.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = User_::read(&conn, user_id)?;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let comment = Comment::read(&conn, data.comment_id)?;

    // Add to recipient ids
    match comment.parent_id {
      Some(parent_id) => {
        let parent_comment = Comment::read(&conn, parent_id)?;
        if parent_comment.creator_id != user_id {
          let parent_user = User_::read(&conn, parent_comment.creator_id)?;
          recipient_ids.push(parent_user.id);
        }
      }
      None => {
        recipient_ids.push(post.creator_id);
      }
    }

    let like_form = CommentLikeForm {
      comment_id: data.comment_id,
      post_id: data.post_id,
      user_id,
      score: data.score,
    };

    // Remove any likes first
    CommentLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let _inserted_like = match CommentLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => return Err(APIError::err("couldnt_like_comment").into()),
      };

      if like_form.score == 1 {
        comment.send_like(&user, &conn)?;
      } else if like_form.score == -1 {
        comment.send_dislike(&user, &conn)?;
      }
    } else {
      comment.send_undo_like(&user, &conn)?;
    }

    // Have to refetch the comment to get the current state
    let liked_comment = CommentView::read(&conn, data.comment_id, Some(user_id))?;

    let mut res = CommentResponse {
      comment: liked_comment,
      recipient_ids,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendComment {
        op: UserOperation::CreateCommentLike,
        comment: res.clone(),
        my_id: ws.id,
      });

      // strip out the recipient_ids, so that
      // users don't get double notifs
      res.recipient_ids = Vec::new();
    }

    Ok(res)
  }
}

impl Perform for Oper<GetComments> {
  type Response = GetCommentsResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommentsResponse, Error> {
    let data: &GetComments = &self.data;

    let user_claims: Option<Claims> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => Some(claims.claims),
        Err(_e) => None,
      },
      None => None,
    };

    let user_id = match &user_claims {
      Some(claims) => Some(claims.id),
      None => None,
    };

    let type_ = ListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let conn = pool.get()?;

    let comments = match CommentQueryBuilder::create(&conn)
      .listing_type(type_)
      .sort(&sort)
      .for_community_id(data.community_id)
      .my_user_id(user_id)
      .page(data.page)
      .limit(data.limit)
      .list()
    {
      Ok(comments) => comments,
      Err(_e) => return Err(APIError::err("couldnt_get_comments").into()),
    };

    if let Some(ws) = websocket_info {
      // You don't need to join the specific community room, bc this is already handled by
      // GetCommunity
      if data.community_id.is_none() {
        if let Some(id) = ws.id {
          // 0 is the "all" community
          ws.chatserver.do_send(JoinCommunityRoom {
            community_id: 0,
            id,
          });
        }
      }
    }

    Ok(GetCommentsResponse { comments })
  }
}

pub fn send_local_notifs(
  conn: &PgConnection,
  mentions: &[MentionData],
  comment: &Comment,
  user: &User_,
  post: &Post,
) -> Vec<i32> {
  let mut recipient_ids = Vec::new();
  let hostname = &format!("https://{}", Settings::get().hostname);

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local() && m.name.ne(&user.name))
    .collect::<Vec<&MentionData>>()
  {
    if let Ok(mention_user) = User_::read_from_name(&conn, &mention.name) {
      // At some point, make it so you can't tag the parent creator either
      // This can cause two notifications, one for reply and the other for mention
      recipient_ids.push(mention_user.id);

      let user_mention_form = UserMentionForm {
        recipient_id: mention_user.id,
        comment_id: comment.id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      match UserMention::create(&conn, &user_mention_form) {
        Ok(_mention) => (),
        Err(_e) => error!("{}", &_e),
      };

      // Send an email to those users that have notifications on
      if mention_user.send_notifications_to_email {
        if let Some(mention_email) = mention_user.email {
          let subject = &format!("{} - Mentioned by {}", Settings::get().hostname, user.name,);
          let html = &format!(
            "<h1>User Mention</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
            user.name, comment.content, hostname
          );
          match send_email(subject, &mention_email, &mention_user.name, html) {
            Ok(_o) => _o,
            Err(e) => error!("{}", e),
          };
        }
      }
    }
  }

  // Send notifs to the parent commenter / poster
  match comment.parent_id {
    Some(parent_id) => {
      if let Ok(parent_comment) = Comment::read(&conn, parent_id) {
        if parent_comment.creator_id != user.id {
          if let Ok(parent_user) = User_::read(&conn, parent_comment.creator_id) {
            recipient_ids.push(parent_user.id);

            if parent_user.send_notifications_to_email {
              if let Some(comment_reply_email) = parent_user.email {
                let subject = &format!("{} - Reply from {}", Settings::get().hostname, user.name,);
                let html = &format!(
                  "<h1>Comment Reply</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
                  user.name, comment.content, hostname
                );
                match send_email(subject, &comment_reply_email, &parent_user.name, html) {
                  Ok(_o) => _o,
                  Err(e) => error!("{}", e),
                };
              }
            }
          }
        }
      }
    }
    // Its a post
    None => {
      if post.creator_id != user.id {
        if let Ok(parent_user) = User_::read(&conn, post.creator_id) {
          recipient_ids.push(parent_user.id);

          if parent_user.send_notifications_to_email {
            if let Some(post_reply_email) = parent_user.email {
              let subject = &format!("{} - Reply from {}", Settings::get().hostname, user.name,);
              let html = &format!(
                "<h1>Post Reply</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
                user.name, comment.content, hostname
              );
              match send_email(subject, &post_reply_email, &parent_user.name, html) {
                Ok(_o) => _o,
                Err(e) => error!("{}", e),
              };
            }
          }
        }
      }
    }
  };
  recipient_ids
}
