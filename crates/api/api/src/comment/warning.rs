use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_mod_action,
  utils::{check_comment_deleted_or_removed, check_community_mod_action},
};
use lemmy_db_schema::source::modlog::{Modlog, ModlogInsertForm};
use lemmy_db_views_comment::{
  CommentView,
  api::{CommentResponse, CreateCommentWarning},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

/// Creates a warning against a comment and notifies the user
pub async fn create_comment_warning(
  Json(data): Json<CreateCommentWarning>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;

  let orig_comment =
    CommentView::read(&mut context.pool(), comment_id, None, local_instance_id).await?;

  check_community_mod_action(
    &local_user_view,
    &orig_comment.community,
    false,
    &mut context.pool(),
  )
  .await?;

  // Don't allow creating warnings for removed / deleted comments
  check_comment_deleted_or_removed(&orig_comment.comment)?;

  let form = ModlogInsertForm::mod_create_comment_warning(
    local_user_view.person.id,
    &orig_comment.comment,
    &data.reason,
  );

  let action = Modlog::create(&mut context.pool(), &[form]).await?;

  notify_mod_action(action, &context);

  // TODO federate activity

  Ok(Json(CommentResponse {
    comment_view: orig_comment,
  }))
}
