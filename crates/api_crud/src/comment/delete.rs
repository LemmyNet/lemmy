use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, DeleteComment},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_user_action,
};
use lemmy_db_schema::{
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn delete_comment(
  data: Json<DeleteComment>,
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

  // Dont delete it if its already been deleted.
  if orig_comment.comment.deleted == data.deleted {
    Err(LemmyErrorType::CouldntUpdateComment)?
  }

  check_community_user_action(
    &local_user_view.person,
    orig_comment.community.id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can delete
  if local_user_view.person.id != orig_comment.creator.id {
    Err(LemmyErrorType::NoCommentEditAllowed)?
  }

  // Do the delete
  let deleted = data.deleted;
  let updated_comment = Comment::update(
    &mut context.pool(),
    comment_id,
    &CommentUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  let recipient_ids =
    send_local_notifs(vec![], comment_id, &local_user_view.person, false, &context).await?;
  let updated_comment_id = updated_comment.id;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteComment(
      updated_comment,
      local_user_view.person.clone(),
      orig_comment.community,
    ),
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
