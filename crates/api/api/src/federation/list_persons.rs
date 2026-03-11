use crate::federation::fetcher::resolve_person_identifier;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{api::ListPersons, impls::PersonQuery};
use lemmy_db_views_person_content_combined::{
  ListPersonContent,
  impls::PersonContentCombinedQuery,
};
use lemmy_db_views_post_comment_combined::PostCommentCombinedView;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_persons(
  Query(data): Query<ListPersons>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<PersonView>>> {
  let SiteView {
    site, local_site, ..
  } = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let res = PersonQuery {
    local_user: local_user_view.map(|l| l.local_user).as_ref(),
    sort: data.sort,
    listing_type: data.type_,
    search_term: data.search_term,
    search_title_only: data.search_title_only,
    limit: data.limit,
    page_cursor: data.page_cursor,
  }
  .list(&site, &mut context.pool())
  .await?;

  Ok(Json(res))
}
