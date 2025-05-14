use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_saved_combined::{
  impls::PersonSavedCombinedQuery,
  ListPersonSaved,
  ListPersonSavedResponse,
  PersonSavedCombinedView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_saved(
  data: Query<ListPersonSaved>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPersonSavedResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&Some(local_user_view.clone()), &local_site.local_site)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonSavedCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let saved = PersonSavedCombinedQuery {
    type_: data.type_,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = saved.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = saved.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonSavedResponse {
    saved,
    next_page,
    prev_page,
  }))
}
