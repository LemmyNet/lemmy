use serde::{Deserialize, Serialize};

use crate::db::comment;
use crate::db::comment_view;
use crate::db::community_view;
use crate::db::moderator;
use crate::db::post;
use crate::db::user;
use crate::db::user_view;
use crate::db::{
    Crud,
    Likeable,
    Saveable,
    establish_connection,
};


#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  content: String,
  parent_id: Option<i32>,
  edit_id: Option<i32>,
  pub post_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditComment {
  content: String,
  parent_id: Option<i32>,
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
  op: String,
  pub comment: comment_view::CommentView,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  comment_id: i32,
  pub post_id: i32,
  score: i16,
  auth: String,
}

impl super::Perform<CommentResponse> for super::Oper<CreateComment> {
  fn perform(&self) -> Result<CommentResponse, super::Error> {
    let data: &CreateComment = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(super::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = post::Post::read(&conn, data.post_id)?;
    if community_view::CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(super::APIError::err(&self.op, "community_ban"))?;
    }

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(super::APIError::err(&self.op, "site_ban"))?;
    }

    let content_slurs_removed = crate::remove_slurs(&data.content.to_owned());

    let comment_form = comment::CommentForm {
      content: content_slurs_removed,
      parent_id: data.parent_id.to_owned(),
      post_id: data.post_id,
      creator_id: user_id,
      removed: None,
      deleted: None,
      read: None,
      updated: None,
    };

    let inserted_comment = match comment::Comment::create(&conn, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_create_comment"))?,
    };

    // You like your own comment by default
    let like_form = comment::CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: data.post_id,
      user_id: user_id,
      score: 1,
    };

    let _inserted_like = match comment::CommentLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_like_comment"))?,
    };

    let comment_view = comment_view::CommentView::read(&conn, inserted_comment.id, Some(user_id))?;

    Ok(CommentResponse {
      op: self.op.to_string(),
      comment: comment_view,
    })
  }
}

impl super::Perform<CommentResponse> for super::Oper<EditComment> {
  fn perform(&self) -> Result<CommentResponse, super::Error> {
    let data: &EditComment = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(super::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let orig_comment = comment_view::CommentView::read(&conn, data.edit_id, None)?;

    // You are allowed to mark the comment as read even if you're banned.
    if data.read.is_none() {
      // Verify its the creator or a mod, or an admin
      let mut editors: Vec<i32> = vec![data.creator_id];
      editors.append(
        &mut community_view::CommunityModeratorView::for_community(&conn, orig_comment.community_id)?
          .into_iter()
          .map(|m| m.user_id)
          .collect(),
      );
      editors.append(&mut user_view::UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());

      if !editors.contains(&user_id) {
        return Err(super::APIError::err(&self.op, "no_comment_edit_allowed"))?;
      }

      // Check for a community ban
      if community_view::CommunityUserBanView::get(&conn, user_id, orig_comment.community_id).is_ok() {
        return Err(super::APIError::err(&self.op, "community_ban"))?;
      }

      // Check for a site ban
      if user_view::UserView::read(&conn, user_id)?.banned {
        return Err(super::APIError::err(&self.op, "site_ban"))?;
      }
    }

    let content_slurs_removed = crate::remove_slurs(&data.content.to_owned());

    let comment_form = comment::CommentForm {
      content: content_slurs_removed,
      parent_id: data.parent_id,
      post_id: data.post_id,
      creator_id: data.creator_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
      read: data.read.to_owned(),
      updated: if data.read.is_some() {
        orig_comment.updated
      } else {
        Some(crate::naive_now())
      },
    };

    let _updated_comment = match comment::Comment::update(&conn, data.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_update_comment"))?,
    };

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let form = moderator::ModRemoveCommentForm {
        mod_user_id: user_id,
        comment_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
      };
      moderator::ModRemoveComment::create(&conn, &form)?;
    }

    let comment_view = comment_view::CommentView::read(&conn, data.edit_id, Some(user_id))?;

    Ok(CommentResponse {
      op: self.op.to_string(),
      comment: comment_view,
    })
  }
}

impl super::Perform<CommentResponse> for super::Oper<SaveComment> {
  fn perform(&self) -> Result<CommentResponse, super::Error> {
    let data: &SaveComment = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(super::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let comment_saved_form = comment::CommentSavedForm {
      comment_id: data.comment_id,
      user_id: user_id,
    };

    if data.save {
      match comment::CommentSaved::save(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_save_comment"))?,
      };
    } else {
      match comment::CommentSaved::unsave(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_save_comment"))?,
      };
    }

    let comment_view = comment_view::CommentView::read(&conn, data.comment_id, Some(user_id))?;

    Ok(CommentResponse {
      op: self.op.to_string(),
      comment: comment_view,
    })
  }
}

impl super::Perform<CommentResponse> for super::Oper<CreateCommentLike> {
  fn perform(&self) -> Result<CommentResponse, super::Error> {
    let data: &CreateCommentLike = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(super::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = post::Post::read(&conn, data.post_id)?;
    if community_view::CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(super::APIError::err(&self.op, "community_ban"))?;
    }

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(super::APIError::err(&self.op, "site_ban"))?;
    }

    let like_form = comment::CommentLikeForm {
      comment_id: data.comment_id,
      post_id: data.post_id,
      user_id: user_id,
      score: data.score,
    };

    // Remove any likes first
    comment::CommentLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    let do_add = &like_form.score != &0 && (&like_form.score == &1 || &like_form.score == &-1);
    if do_add {
      let _inserted_like = match comment::CommentLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => return Err(super::APIError::err(&self.op, "couldnt_like_comment"))?,
      };
    }

    // Have to refetch the comment to get the current state
    let liked_comment = comment_view::CommentView::read(&conn, data.comment_id, Some(user_id))?;

    Ok(CommentResponse {
      op: self.op.to_string(),
      comment: liked_comment,
    })
  }
}
