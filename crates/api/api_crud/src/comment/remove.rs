use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  newtypes::CommentId,
  source::{
    comment::{Comment, CommentUpdateForm},
    comment_report::CommentReport,
    local_user::LocalUser,
    mod_log::moderator::{ModRemoveComment, ModRemoveCommentForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views_comment::{
  api::{CommentResponse, RemoveComment},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn remove_comment(
  comment_id: Path<CommentId>,
  data: Json<RemoveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_id = comment_id.into_inner();
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

  // Don't allow removing or restoring comment which was deleted by user, as it would reveal
  // the comment text in mod log.
  if orig_comment.comment.deleted {
    return Err(LemmyErrorType::CouldntUpdate.into());
  }

  // Do the remove
  let removed = data.removed;
  let updated_comment = Comment::update(
    &mut context.pool(),
    comment_id,
    &CommentUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  CommentReport::resolve_all_for_object(&mut context.pool(), comment_id, local_user_view.person.id)
    .await?;

  // Mod tables
  let form = ModRemoveCommentForm {
    mod_person_id: local_user_view.person.id,
    comment_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemoveComment::create(&mut context.pool(), &form).await?;

  let updated_comment_id = updated_comment.id;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: updated_comment,
      moderator: local_user_view.person.clone(),
      community: orig_comment.community,
      reason: data.reason.clone(),
    },
    &context,
  )?;

  Ok(Json(
    build_comment_response(
      &context,
      updated_comment_id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
