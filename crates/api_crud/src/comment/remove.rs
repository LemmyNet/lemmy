use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, RemoveComment},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_ban, is_mod_or_admin},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    moderator::{ModRemoveComment, ModRemoveCommentForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn remove_comment(
  data: Json<RemoveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CommentResponse>, LemmyError> {
  let comment_id = data.comment_id;
  let orig_comment = CommentView::read(&mut context.pool(), comment_id, None).await?;

  check_community_ban(
    local_user_view.person.id,
    orig_comment.community.id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only a mod or admin can remove
  is_mod_or_admin(
    &mut context.pool(),
    local_user_view.person.id,
    orig_comment.community.id,
  )
  .await?;

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

  // Mod tables
  let form = ModRemoveCommentForm {
    mod_person_id: local_user_view.person.id,
    comment_id: data.comment_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemoveComment::create(&mut context.pool(), &form).await?;

  let post_id = updated_comment.post_id;
  let post = Post::read(&mut context.pool(), post_id).await?;
  let recipient_ids = send_local_notifs(
    vec![],
    &updated_comment,
    &local_user_view.person.clone(),
    &post,
    false,
    &context,
  )
  .await?;
  let updated_comment_id = updated_comment.id;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment(
      updated_comment,
      local_user_view.person.clone(),
      orig_comment.community,
      data.reason.clone(),
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
