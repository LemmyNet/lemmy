use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgeComment,
  utils::{check_is_higher_admin, is_admin},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    moderator::{AdminPurgeComment, AdminPurgeCommentForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn purge_comment(
  data: Json<PurgeComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let comment_id = data.comment_id;

  // Read the comment to get the post_id and community
  let comment_view = CommentView::read(&mut context.pool(), comment_id, None)
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;

  // Also check that you're a higher admin
  check_is_higher_admin(
    &mut context.pool(),
    &local_user_view,
    &[comment_view.creator.id],
  )
  .await?;

  let post_id = comment_view.comment.post_id;

  // TODO read comments for pictrs images and purge them

  Comment::delete(&mut context.pool(), comment_id).await?;

  // Mod tables
  let form = AdminPurgeCommentForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    post_id,
  };
  AdminPurgeComment::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: comment_view.comment,
      moderator: local_user_view.person.clone(),
      community: comment_view.community,
      reason: data.reason.clone(),
    },
    &context,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
