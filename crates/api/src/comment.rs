use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  check_community_ban,
  check_downvotes_enabled,
  check_person_block,
  comment::*,
  get_local_user_view_from_jwt,
};
use lemmy_apub::{
  activities::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
  PostOrComment,
};
use lemmy_db_queries::{source::comment::Comment_, Likeable, Saveable};
use lemmy_db_schema::{source::comment::*, LocalUserId};
use lemmy_db_views::{comment_view::CommentView, local_user_view::LocalUserView};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperation};
use std::convert::TryInto;

#[async_trait::async_trait(?Send)]
impl Perform for MarkCommentAsRead {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &MarkCommentAsRead = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(&&context.pool.get().await?, comment_id, None)?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the recipient can mark as read
    if local_user_view.person.id != orig_comment.get_recipient_id() {
      return Err(ApiError::err("no_comment_edit_allowed").into());
    }

    // Do the mark as read
    let read = data.read;
    Comment::update_read(&&context.pool.get().await?, comment_id, read)
      .map_err(|_| ApiError::err("couldnt_update_comment"))?;

    // Refetch it
    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = CommentView::read(&&context.pool.get().await?, comment_id, Some(person_id))?;

    let res = CommentResponse {
      comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    };

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SaveComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &SaveComment = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let comment_saved_form = CommentSavedForm {
      comment_id: data.comment_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save_comment = CommentSaved::save(&&context.pool.get().await?, &comment_saved_form);
      if save_comment.is_err() {
        return Err(ApiError::err("couldnt_save_comment").into());
      }
    } else {
      let unsave_comment = CommentSaved::unsave(&&context.pool.get().await?, &comment_saved_form);
      if unsave_comment.is_err() {
        return Err(ApiError::err("couldnt_save_comment").into());
      }
    }

    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = CommentView::read(&&context.pool.get().await?, comment_id, Some(person_id))?;

    Ok(CommentResponse {
      comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentLike {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &CreateCommentLike = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let mut recipient_ids = Vec::<LocalUserId>::new();

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(&&context.pool.get().await?, comment_id, None)?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    check_person_block(
      local_user_view.person.id,
      orig_comment.get_recipient_id(),
      context.pool(),
    )
    .await?;

    // Add parent user to recipients
    let recipient_id = orig_comment.get_recipient_id();
    if let Ok(local_recipient) =
      LocalUserView::read_person(&&context.pool.get().await?, recipient_id)
    {
      recipient_ids.push(local_recipient.local_user.id);
    }

    let like_form = CommentLikeForm {
      comment_id: data.comment_id,
      post_id: orig_comment.post.id,
      person_id: local_user_view.person.id,
      score: data.score,
    };

    // Remove any likes first
    let person_id = local_user_view.person.id;
    CommentLike::remove(&&context.pool.get().await?, person_id, comment_id)?;

    // Only add the like if the score isnt 0
    let comment = orig_comment.comment;
    let object = PostOrComment::Comment(Box::new(comment));
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = CommentLike::like(&&context.pool.get().await?, &like_form2);
      if like.is_err() {
        return Err(ApiError::err("couldnt_like_comment").into());
      }

      Vote::send(
        &object,
        &local_user_view.person,
        orig_comment.community.id,
        like_form.score.try_into()?,
        context,
      )
      .await?;
    } else {
      // API doesn't distinguish between Undo/Like and Undo/Dislike
      UndoVote::send(
        &object,
        &local_user_view.person,
        orig_comment.community.id,
        VoteType::Like,
        context,
      )
      .await?;
    }

    send_comment_ws_message(
      data.comment_id,
      UserOperation::CreateCommentLike,
      websocket_id,
      None,
      Some(local_user_view.person.id),
      recipient_ids,
      context,
    )
    .await
  }
}
