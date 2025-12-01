use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_saved_combined::{
  ListPersonSaved,
  PersonSavedCombinedView,
  impls::PersonSavedCombinedQuery,
};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_saved(
  Query(data): Query<ListPersonSaved>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<PersonSavedCombinedView>>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&Some(local_user_view.clone()), &local_site.local_site)?;

  let saved = PersonSavedCombinedQuery {
    type_: data.type_,
    page_cursor: data.page_cursor,
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(saved))
}
