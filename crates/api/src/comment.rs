use std::convert::TryInto;

use actix_web::web::Data;

use lemmy_api_common::{
  blocking,
  check_community_ban,
  check_downvotes_enabled,
  comment::*,
  get_local_user_view_from_jwt,
};
use lemmy_apub::{
  fetcher::post_or_comment::PostOrComment,
  protocol::activities::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::comment::*,
  traits::{Likeable, Saveable},
};
use lemmy_db_views::{comment_view::CommentView, local_user_view::LocalUserView};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperation};

use crate::Perform;

#[async_trait::async_trait(?Send)]
impl Perform for MarkCommentAsRead {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &MarkCommentAsRead = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    // Verify that only the recipient can mark as read
    if local_user_view.person.id != orig_comment.get_recipient_id() {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    // Do the mark as read
    let read = data.read;
    blocking(context.pool(), move |conn| {
      Comment::update_read(conn, comment_id, read)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_update_comment"))?;

    // Refetch it
    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

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

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &SaveComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_saved_form = CommentSavedForm {
      comment_id: data.comment_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save_comment = move |conn: &'_ _| CommentSaved::save(conn, &comment_saved_form);
      blocking(context.pool(), save_comment)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_save_comment"))?;
    } else {
      let unsave_comment = move |conn: &'_ _| CommentSaved::unsave(conn, &comment_saved_form);
      blocking(context.pool(), unsave_comment)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_save_comment"))?;
    }

    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

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

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &CreateCommentLike = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let mut recipient_ids = Vec::<LocalUserId>::new();

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Add parent user to recipients
    let recipient_id = orig_comment.get_recipient_id();
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
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
    blocking(context.pool(), move |conn| {
      CommentLike::remove(conn, person_id, comment_id)
    })
    .await??;

    // Only add the like if the score isnt 0
    let comment = orig_comment.comment;
    let object = PostOrComment::Comment(Box::new(comment.into()));
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = move |conn: &'_ _| CommentLike::like(conn, &like_form2);
      blocking(context.pool(), like)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_like_comment"))?;

      Vote::send(
        &object,
        &local_user_view.person.clone().into(),
        orig_comment.community.id,
        like_form.score.try_into()?,
        context,
      )
      .await?;
    } else {
      // API doesn't distinguish between Undo/Like and Undo/Dislike
      UndoVote::send(
        &object,
        &local_user_view.person.clone().into(),
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
