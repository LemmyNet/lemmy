use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, RemoveComment},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, check_is_higher_mod_or_admin},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    comment_report::CommentReport,
    moderator::{ModRemoveComment, ModRemoveCommentForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn remove_comment(
  data: Json<RemoveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_id = data.comment_id;
  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindComment)?;

  check_community_mod_action(
    &local_user_view.person,
    orig_comment.community.id,
    false,
    &mut context.pool(),
  )
  .await?;

  check_is_higher_mod_or_admin(
    &mut context.pool(),
    &local_user_view,
    orig_comment.community.id,
    &[orig_comment.creator.id],
  )
  .await?;

  // Don't allow removing or restoring comment which was deleted by user, as it would reveal
  // the comment text in mod log.
  if orig_comment.comment.deleted {
    return Err(LemmyErrorType::CouldntUpdateComment.into());
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
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  CommentReport::resolve_all_for_object(&mut context.pool(), comment_id, local_user_view.person.id)
    .await?;

  // Mod tables
  let form = ModRemoveCommentForm {
    mod_person_id: local_user_view.person.id,
    comment_id: data.comment_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemoveComment::create(&mut context.pool(), &form).await?;

  let recipient_ids =
    send_local_notifs(vec![], comment_id, &local_user_view.person, false, &context).await?;
  let updated_comment_id = updated_comment.id;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: updated_comment,
      moderator: local_user_view.person.clone(),
      community: orig_comment.community,
      reason: data.reason.clone(),
    },
    &context,
  )
  .await?;

  Ok(Json(
    build_comment_response(
      &context,
      updated_comment_id,
      Some(local_user_view),
      recipient_ids,
    )
    .await?,
  ))
}
