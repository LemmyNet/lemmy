use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, CreateCommentLike},
  utils::{blocking, check_community_ban, check_downvotes_enabled, get_local_user_view_from_jwt},
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
  source::{
    comment::{CommentLike, CommentLikeForm},
    comment_reply::CommentReply,
  },
  traits::Likeable,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperation};
use std::convert::TryInto;

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

    // Add parent poster or commenter to recipients
    let comment_reply = blocking(context.pool(), move |conn| {
      CommentReply::read_by_comment(conn, comment_id)
    })
    .await?;
    if let Ok(reply) = comment_reply {
      let recipient_id = reply.recipient_id;
      if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
        LocalUserView::read_person(conn, recipient_id)
      })
      .await?
      {
        recipient_ids.push(local_recipient.local_user.id);
      }
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
      let like = move |conn: &mut _| CommentLike::like(conn, &like_form2);
      blocking(context.pool(), like)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_comment"))?;

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
