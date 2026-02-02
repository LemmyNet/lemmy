use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, remove_or_restore_comment_thread},
};
use lemmy_db_schema::source::{comment_report::CommentReport, local_user::LocalUser};
use lemmy_db_views_comment::{
  CommentView,
  api::{CommentResponse, RemoveCommentWithChildren},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn remove_comment_with_children(
  Json(data): Json<RemoveCommentWithChildren>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  check_community_mod_action(
    &local_user_view,
    &orig_comment.community,
    false,
    &mut context.pool(),
  )
  .await?;

  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    orig_comment.community.id,
    local_user_view.person.id,
    vec![orig_comment.creator.id],
  )
  .await?;

  let updated_comments = remove_or_restore_comment_thread(
    &orig_comment.comment,
    local_user_view.person.id,
    data.removed,
    &data.reason,
    &context,
  )
  .await?;

  let orig_comment_id = orig_comment.comment.id;
  let updated_comment = updated_comments
    .iter()
    .find(|c| c.id == orig_comment_id)
    .ok_or(LemmyErrorType::CouldntUpdate)?;

  CommentReport::resolve_all_for_thread(
    &mut context.pool(),
    &orig_comment.comment.path,
    local_user_view.person.id,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: updated_comment.clone(),
      moderator: local_user_view.person.clone(),
      community: orig_comment.community.clone(),
      reason: data.reason.clone(),
      with_replies: data.removed.into(),
    },
    &context,
  )?;

  build_comment_response(
    &context,
    comment_id,
    local_user_view.into(),
    local_instance_id,
  )
  .await
  .map(Json)
}
