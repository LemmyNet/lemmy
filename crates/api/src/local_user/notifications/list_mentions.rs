use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonMentions, GetPersonMentionsResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_views_actor::person_mention_view::PersonMentionQuery;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_mentions(
  data: Query<GetPersonMentions>,
  context: Data<LemmyContext>,
) -> Result<Json<GetPersonMentionsResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let unread_only = data.unread_only.unwrap_or_default();
  let person_id = Some(local_user_view.person.id);
  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;

  let mentions = PersonMentionQuery {
    recipient_id: person_id,
    my_person_id: person_id,
    sort,
    unread_only,
    show_bot_accounts,
    page,
    limit,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(GetPersonMentionsResponse { mentions }))
}
