use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::MarkPersonPostMentionAsRead,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::person_post_mention::{PersonPostMention, PersonPostMentionUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

pub async fn mark_post_mention_as_read(
  data: Json<MarkPersonPostMentionAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let person_post_mention_id = data.person_post_mention_id;
  let read_person_post_mention =
    PersonPostMention::read(&mut context.pool(), person_post_mention_id).await?;

  if local_user_view.person.id != read_person_post_mention.recipient_id {
    Err(LemmyErrorType::CouldntUpdatePost)?
  }

  let person_post_mention_id = read_person_post_mention.id;
  let read = Some(data.read);
  PersonPostMention::update(
    &mut context.pool(),
    person_post_mention_id,
    &PersonPostMentionUpdateForm { read },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  Ok(Json(SuccessResponse::default()))
}
