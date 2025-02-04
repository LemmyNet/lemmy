use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListPersonSaved, ListPersonSavedResponse},
  utils::check_private_instance,
};
use lemmy_db_views::{
  combined::person_saved_combined_view::PersonSavedCombinedQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_utils::error::LemmyResult;

pub async fn list_person_saved(
  data: Query<ListPersonSaved>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPersonSavedResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&Some(local_user_view.clone()), &local_site.local_site)?;

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;
  let type_ = data.type_;

  let saved = PersonSavedCombinedQuery {
    type_,
    page_after,
    page_back,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListPersonSavedResponse { saved }))
}
