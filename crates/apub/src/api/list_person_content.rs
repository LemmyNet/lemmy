use super::resolve_person_id_from_id_or_username;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_content_combined::{
  impls::PersonContentCombinedQuery,
  ListPersonContent,
  ListPersonContentResponse,
  PersonContentCombinedView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_content(
  data: Query<ListPersonContent>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListPersonContentResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_details_id = resolve_person_id_from_id_or_username(
    &data.person_id,
    &data.username,
    &context,
    &local_user_view,
  )
  .await?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonContentCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let content = PersonContentCombinedQuery {
    creator_id: person_details_id,
    type_: data.type_,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool(), &local_user_view, local_instance_id)
  .await?;

  let next_page = content.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = content.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonContentResponse {
    content,
    next_page,
    prev_page,
  }))
}
