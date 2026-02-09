use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, remove_or_restore_comment_thread},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    comment_report::CommentReport,
    local_user::LocalUser,
    modlog::{Modlog, ModlogInsertForm},
  },
  traits::Reportable,
};
use lemmy_db_views_comment::{
  CommentView,
  api::{CommentResponse, RemoveComment},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn remove_comment(
  Json(data): Json<RemoveComment>,
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

  let updated_comment = if let Some(remove_children) = data.remove_children {
    let updated_comments = remove_or_restore_comment_thread(
      &orig_comment.comment,
      local_user_view.person.id,
      remove_children,
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

    updated_comment.clone()
  } else {
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

    CommentReport::resolve_all_for_object(
      &mut context.pool(),
      comment_id,
      local_user_view.person.id,
    )
    .await?;

    // Mod tables
    let form = ModlogInsertForm::mod_remove_comment(
      local_user_view.person.id,
      &orig_comment.comment,
      removed,
      &data.reason,
    );
    let actions = Modlog::create(&mut context.pool(), &[form]).await?;
    notify_mod_action(actions, context.app_data());

    updated_comment
  };

  let updated_comment_id = updated_comment.id;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: updated_comment,
      moderator: local_user_view.person.clone(),
      community: orig_comment.community,
      reason: data.reason.clone(),
      with_replies: data.remove_children.unwrap_or_default(),
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
