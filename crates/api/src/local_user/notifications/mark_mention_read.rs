use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{MarkPersonMentionAsRead, PersonMentionResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::person_mention::{PersonMention, PersonMentionUpdateForm},
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonMentionView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for MarkPersonMentionAsRead {
  type Response = PersonMentionResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<PersonMentionResponse, LemmyError> {
    let data: &MarkPersonMentionAsRead = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let person_mention_id = data.person_mention_id;
    let read_person_mention = PersonMention::read(context.pool(), person_mention_id).await?;

    if local_user_view.person.id != read_person_mention.recipient_id {
      return Err(LemmyError::from_message("couldnt_update_comment"));
    }

    let person_mention_id = read_person_mention.id;
    let read = Some(data.read);
    PersonMention::update(
      context.pool(),
      person_mention_id,
      &PersonMentionUpdateForm { read },
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    let person_mention_id = read_person_mention.id;
    let person_id = local_user_view.person.id;
    let person_mention_view =
      PersonMentionView::read(context.pool(), person_mention_id, Some(person_id)).await?;

    Ok(PersonMentionResponse {
      person_mention_view,
    })
  }
}
