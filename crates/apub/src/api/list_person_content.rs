use super::resolve_person_id_from_id_or_username;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListPersonContent, ListPersonContentResponse},
  utils::check_private_instance,
};
use lemmy_db_views::{
  combined::person_content_combined_view::PersonContentCombinedQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_person_content(
  data: Query<ListPersonContent>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListPersonContentResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let person_details_id = resolve_person_id_from_id_or_username(
    &data.person_id,
    &data.username,
    &context,
    &local_user_view,
  )
  .await?;

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;
  let type_ = data.type_;

  let content = PersonContentCombinedQuery {
    creator_id: person_details_id,
    type_,
    page_after,
    page_back,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListPersonContentResponse { content }))
}
