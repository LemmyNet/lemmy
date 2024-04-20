use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{MarkPersonMentionAsRead, PersonMentionResponse},
};
use lemmy_db_schema::{
  source::person_mention::{PersonMention, PersonMentionUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonMentionView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn mark_person_mention_as_read(
  data: Json<MarkPersonMentionAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PersonMentionResponse>> {
  let person_mention_id = data.person_mention_id;
  let read_person_mention = PersonMention::read(&mut context.pool(), person_mention_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPersonMention)?;

  if local_user_view.person.id != read_person_mention.recipient_id {
    Err(LemmyErrorType::CouldntUpdateComment)?
  }

  let person_mention_id = read_person_mention.id;
  let read = Some(data.read);
  PersonMention::update(
    &mut context.pool(),
    person_mention_id,
    &PersonMentionUpdateForm { read },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  let person_mention_id = read_person_mention.id;
  let person_id = local_user_view.person.id;
  let person_mention_view =
    PersonMentionView::read(&mut context.pool(), person_mention_id, Some(person_id))
      .await?
      .ok_or(LemmyErrorType::CouldntFindPersonMention)?;

  Ok(Json(PersonMentionResponse {
    person_mention_view,
  }))
}
