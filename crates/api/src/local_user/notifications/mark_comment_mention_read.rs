use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::MarkPersonCommentMentionAsRead,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::person_comment_mention::{PersonCommentMention, PersonCommentMentionUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn mark_comment_mention_as_read(
  data: Json<MarkPersonCommentMentionAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let person_comment_mention_id = data.person_comment_mention_id;
  let read_person_comment_mention =
    PersonCommentMention::read(&mut context.pool(), person_comment_mention_id).await?;

  if local_user_view.person.id != read_person_comment_mention.recipient_id {
    Err(LemmyErrorType::CouldntUpdateComment)?
  }

  let person_comment_mention_id = read_person_comment_mention.id;
  let read = Some(data.read);
  PersonCommentMention::update(
    &mut context.pool(),
    person_comment_mention_id,
    &PersonCommentMentionUpdateForm { read },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  Ok(Json(SuccessResponse::default()))
}
