use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListPersonSaved, ListPersonSavedResponse},
  utils::check_private_instance,
};
use lemmy_db_schema::traits::PageCursorBuilder;
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

  let saved = PersonSavedCombinedQuery {
    type_: data.type_,
    page_cursor: data.page_cursor.clone(),
    page_back: data.page_back,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = saved.last().map(PageCursorBuilder::cursor);

  Ok(Json(ListPersonSavedResponse { saved, next_page }))
}
