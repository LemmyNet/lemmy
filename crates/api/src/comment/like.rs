use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_comment_response,
  comment::{CommentResponse, CreateCommentLike},
  context::LemmyContext,
  utils::{check_community_ban, check_downvotes_enabled, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::{
    comment::{CommentLike, CommentLikeForm},
    comment_reply::CommentReply,
    local_site::LocalSite,
  },
  traits::Likeable,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentLike {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &CreateCommentLike = self;
    let local_site = LocalSite::read(context.pool()).await?;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let mut recipient_ids = Vec::<LocalUserId>::new();

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, &local_site)?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(context.pool(), comment_id, None).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Add parent poster or commenter to recipients
    let comment_reply = CommentReply::read_by_comment(context.pool(), comment_id).await;
    if let Ok(reply) = comment_reply {
      let recipient_id = reply.recipient_id;
      if let Ok(local_recipient) = LocalUserView::read_person(context.pool(), recipient_id).await {
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

    CommentLike::remove(context.pool(), person_id, comment_id).await?;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      CommentLike::like(context.pool(), &like_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_comment"))?;
    }

    build_comment_response(
      context,
      comment_id,
      Some(local_user_view),
      None,
      recipient_ids,
    )
    .await
  }
}
