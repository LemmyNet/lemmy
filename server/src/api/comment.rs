use crate::{
  api::{claims::Claims, is_mod_or_admin, APIError, Oper, Perform},
  apub::{ApubLikeableType, ApubObjectType},
  blocking,
  websocket::{
    server::{JoinCommunityRoom, SendComment},
    UserOperation,
    WebsocketInfo,
  },
  DbPool,
  LemmyError,
};
use lemmy_db::{
  comment::*,
  comment_view::*,
  community_view::*,
  moderator::*,
  post::*,
  site_view::*,
  user::*,
  user_mention::*,
  Crud,
  Likeable,
  ListingType,
  Saveable,
  SortType,
};
use lemmy_utils::{
  make_apub_endpoint,
  remove_slurs,
  scrape_text_for_mentions,
  send_email,
  settings::Settings,
  EndpointType,
  MentionData,
};
use log::error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  content: String,
  parent_id: Option<i32>,
  pub post_id: i32,
  form_id: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditComment {
  content: String,
  edit_id: i32,
  form_id: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteComment {
  edit_id: i32,
  deleted: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveComment {
  edit_id: i32,
  removed: bool,
  reason: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct MarkCommentAsRead {
  edit_id: i32,
  read: bool,
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
  pub form_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  comment_id: i32,
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateComment> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &CreateComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

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
      ap_id: "http://fake.com".into(),
      local: true,
    };

    // Check for a community ban
    let post_id = data.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check if post is locked, no new comments
    if post.locked {
      return Err(APIError::err("locked").into());
    }

    // Create the comment
    let comment_form2 = comment_form.clone();
    let inserted_comment =
      match blocking(pool, move |conn| Comment::create(&conn, &comment_form2)).await? {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err("couldnt_create_comment").into()),
      };

    // Necessary to update the ap_id
    let inserted_comment_id = inserted_comment.id;
    let updated_comment: Comment = match blocking(pool, move |conn| {
      let apub_id =
        make_apub_endpoint(EndpointType::Comment, &inserted_comment_id.to_string()).to_string();
      Comment::update_ap_id(&conn, inserted_comment_id, apub_id)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_create_comment").into()),
    };

    updated_comment
      .send_create(&user, &self.client, pool)
      .await?;

    // Scan the comment for user mentions, add those rows
    let mentions = scrape_text_for_mentions(&comment_form.content);
    let recipient_ids = send_local_notifs(
      mentions,
      updated_comment.clone(),
      user.clone(),
      post,
      pool,
      true,
    )
    .await?;

    // You like your own comment by default
    let like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: data.post_id,
      user_id,
      score: 1,
    };

    let like = move |conn: &'_ _| CommentLike::like(&conn, &like_form);
    if blocking(pool, like).await?.is_err() {
      return Err(APIError::err("couldnt_like_comment").into());
    }

    updated_comment.send_like(&user, &self.client, pool).await?;

    let comment_view = blocking(pool, move |conn| {
      CommentView::read(&conn, inserted_comment.id, Some(user_id))
    })
    .await??;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
      form_id: data.form_id.to_owned(),
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditComment> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &EditComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let edit_id = data.edit_id;
    let orig_comment =
      blocking(pool, move |conn| CommentView::read(&conn, edit_id, None)).await??;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check for a community ban
    let community_id = orig_comment.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Verify that only the creator can edit
    if user_id != orig_comment.creator_id {
      return Err(APIError::err("no_comment_edit_allowed").into());
    }

    // Do the update
    let content_slurs_removed = remove_slurs(&data.content.to_owned());
    let edit_id = data.edit_id;
    let updated_comment = match blocking(pool, move |conn| {
      Comment::update_content(conn, edit_id, &content_slurs_removed)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    // Send the apub update
    updated_comment
      .send_update(&user, &self.client, pool)
      .await?;

    // Do the mentions / recipients
    let post_id = orig_comment.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let updated_comment_content = updated_comment.content.to_owned();
    let mentions = scrape_text_for_mentions(&updated_comment_content);
    let recipient_ids =
      send_local_notifs(mentions, updated_comment, user, post, pool, false).await?;

    let edit_id = data.edit_id;
    let comment_view = blocking(pool, move |conn| {
      CommentView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
      form_id: data.form_id.to_owned(),
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<DeleteComment> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &DeleteComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let edit_id = data.edit_id;
    let orig_comment =
      blocking(pool, move |conn| CommentView::read(&conn, edit_id, None)).await??;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check for a community ban
    let community_id = orig_comment.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Verify that only the creator can delete
    if user_id != orig_comment.creator_id {
      return Err(APIError::err("no_comment_edit_allowed").into());
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = match blocking(pool, move |conn| {
      Comment::update_deleted(conn, edit_id, deleted)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    // Send the apub message
    if deleted {
      updated_comment
        .send_delete(&user, &self.client, pool)
        .await?;
    } else {
      updated_comment
        .send_undo_delete(&user, &self.client, pool)
        .await?;
    }

    // Refetch it
    let edit_id = data.edit_id;
    let comment_view = blocking(pool, move |conn| {
      CommentView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    // Build the recipients
    let post_id = comment_view.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;
    let mentions = vec![];
    let recipient_ids =
      send_local_notifs(mentions, updated_comment, user, post, pool, false).await?;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
      form_id: None,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendComment {
        op: UserOperation::DeleteComment,
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<RemoveComment> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &RemoveComment = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let edit_id = data.edit_id;
    let orig_comment =
      blocking(pool, move |conn| CommentView::read(&conn, edit_id, None)).await??;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check for a community ban
    let community_id = orig_comment.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Verify that only a mod or admin can remove
    is_mod_or_admin(pool, user_id, community_id).await?;

    // Do the remove
    let removed = data.removed;
    let updated_comment = match blocking(pool, move |conn| {
      Comment::update_removed(conn, edit_id, removed)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    // Mod tables
    let form = ModRemoveCommentForm {
      mod_user_id: user_id,
      comment_id: data.edit_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    blocking(pool, move |conn| ModRemoveComment::create(conn, &form)).await??;

    // Send the apub message
    if removed {
      updated_comment
        .send_remove(&user, &self.client, pool)
        .await?;
    } else {
      updated_comment
        .send_undo_remove(&user, &self.client, pool)
        .await?;
    }

    // Refetch it
    let edit_id = data.edit_id;
    let comment_view = blocking(pool, move |conn| {
      CommentView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    // Build the recipients
    let post_id = comment_view.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;
    let mentions = vec![];
    let recipient_ids =
      send_local_notifs(mentions, updated_comment, user, post, pool, false).await?;

    let mut res = CommentResponse {
      comment: comment_view,
      recipient_ids,
      form_id: None,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendComment {
        op: UserOperation::RemoveComment,
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<MarkCommentAsRead> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &MarkCommentAsRead = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let edit_id = data.edit_id;
    let orig_comment =
      blocking(pool, move |conn| CommentView::read(&conn, edit_id, None)).await??;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check for a community ban
    let community_id = orig_comment.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Verify that only the recipient can mark as read
    // Needs to fetch the parent comment / post to get the recipient
    let parent_id = orig_comment.parent_id;
    match parent_id {
      Some(pid) => {
        let parent_comment =
          blocking(pool, move |conn| CommentView::read(&conn, pid, None)).await??;
        if user_id != parent_comment.creator_id {
          return Err(APIError::err("no_comment_edit_allowed").into());
        }
      }
      None => {
        let parent_post_id = orig_comment.post_id;
        let parent_post = blocking(pool, move |conn| Post::read(conn, parent_post_id)).await??;
        if user_id != parent_post.creator_id {
          return Err(APIError::err("no_comment_edit_allowed").into());
        }
      }
    }

    // Do the mark as read
    let read = data.read;
    match blocking(pool, move |conn| Comment::update_read(conn, edit_id, read)).await? {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    // Refetch it
    let edit_id = data.edit_id;
    let comment_view = blocking(pool, move |conn| {
      CommentView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommentResponse {
      comment: comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    };

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<SaveComment> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
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

    if data.save {
      let save_comment = move |conn: &'_ _| CommentSaved::save(conn, &comment_saved_form);
      if blocking(pool, save_comment).await?.is_err() {
        return Err(APIError::err("couldnt_save_comment").into());
      }
    } else {
      let unsave_comment = move |conn: &'_ _| CommentSaved::unsave(conn, &comment_saved_form);
      if blocking(pool, unsave_comment).await?.is_err() {
        return Err(APIError::err("couldnt_save_comment").into());
      }
    }

    let comment_id = data.comment_id;
    let comment_view = blocking(pool, move |conn| {
      CommentView::read(conn, comment_id, Some(user_id))
    })
    .await??;

    Ok(CommentResponse {
      comment: comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateCommentLike> {
  type Response = CommentResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &CreateCommentLike = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let mut recipient_ids = Vec::new();

    // Don't do a downvote if site has downvotes disabled
    if data.score == -1 {
      let site = blocking(pool, move |conn| SiteView::read(conn)).await??;
      if !site.enable_downvotes {
        return Err(APIError::err("downvotes_disabled").into());
      }
    }

    let comment_id = data.comment_id;
    let orig_comment =
      blocking(pool, move |conn| CommentView::read(&conn, comment_id, None)).await??;

    // Check for a community ban
    let post_id = orig_comment.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let comment_id = data.comment_id;
    let comment = blocking(pool, move |conn| Comment::read(conn, comment_id)).await??;

    // Add to recipient ids
    match comment.parent_id {
      Some(parent_id) => {
        let parent_comment = blocking(pool, move |conn| Comment::read(conn, parent_id)).await??;
        if parent_comment.creator_id != user_id {
          let parent_user = blocking(pool, move |conn| {
            User_::read(conn, parent_comment.creator_id)
          })
          .await??;
          recipient_ids.push(parent_user.id);
        }
      }
      None => {
        recipient_ids.push(post.creator_id);
      }
    }

    let like_form = CommentLikeForm {
      comment_id: data.comment_id,
      post_id,
      user_id,
      score: data.score,
    };

    // Remove any likes first
    let like_form2 = like_form.clone();
    blocking(pool, move |conn| CommentLike::remove(conn, &like_form2)).await??;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = move |conn: &'_ _| CommentLike::like(conn, &like_form2);
      if blocking(pool, like).await?.is_err() {
        return Err(APIError::err("couldnt_like_comment").into());
      }

      if like_form.score == 1 {
        comment.send_like(&user, &self.client, pool).await?;
      } else if like_form.score == -1 {
        comment.send_dislike(&user, &self.client, pool).await?;
      }
    } else {
      comment.send_undo_like(&user, &self.client, pool).await?;
    }

    // Have to refetch the comment to get the current state
    let comment_id = data.comment_id;
    let liked_comment = blocking(pool, move |conn| {
      CommentView::read(conn, comment_id, Some(user_id))
    })
    .await??;

    let mut res = CommentResponse {
      comment: liked_comment,
      recipient_ids,
      form_id: None,
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetComments> {
  type Response = GetCommentsResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommentsResponse, LemmyError> {
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

    let community_id = data.community_id;
    let page = data.page;
    let limit = data.limit;
    let comments = blocking(pool, move |conn| {
      CommentQueryBuilder::create(conn)
        .listing_type(type_)
        .sort(&sort)
        .for_community_id(community_id)
        .my_user_id(user_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?;
    let comments = match comments {
      Ok(comments) => comments,
      Err(_) => return Err(APIError::err("couldnt_get_comments").into()),
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

pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: Comment,
  user: User_,
  post: Post,
  pool: &DbPool,
  do_send_email: bool,
) -> Result<Vec<i32>, LemmyError> {
  let ids = blocking(pool, move |conn| {
    do_send_local_notifs(conn, &mentions, &comment, &user, &post, do_send_email)
  })
  .await?;

  Ok(ids)
}

fn do_send_local_notifs(
  conn: &diesel::PgConnection,
  mentions: &[MentionData],
  comment: &Comment,
  user: &User_,
  post: &Post,
  do_send_email: bool,
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
      // TODO
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
      if do_send_email && mention_user.send_notifications_to_email {
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

            if do_send_email && parent_user.send_notifications_to_email {
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

          if do_send_email && parent_user.send_notifications_to_email {
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
