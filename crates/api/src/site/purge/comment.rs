use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    local_user::LocalUser,
    mod_log::admin::{AdminPurgeComment, AdminPurgeCommentForm},
  },
  traits::Crud,
};
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_db_views_comment::{api::PurgeComment, CommentView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn purge_comment(
  data: Json<PurgeComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;

  // Read the comment to get the post_id and community
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![comment_view.creator.id],
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
  )?;

  Ok(Json(SuccessResponse::default()))
}
