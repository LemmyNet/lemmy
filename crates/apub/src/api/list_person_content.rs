use crate::fetcher::resolve_ap_identifier;
use activitypub_federation::config::Data;
use actix_web::{
  web::{Json, Query},
  HttpRequest,
};
use lemmy_api_utils::{
  context::LemmyContext,
  request::parse_person_id_or_name_from_request,
  utils::check_private_instance,
};
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{source::person::Person, traits::PaginationCursorBuilder};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::api::PersonIdOrName;
use lemmy_db_views_person_content_combined::{
  impls::PersonContentCombinedQuery,
  ListPersonContent,
  ListPersonContentResponse,
  PersonContentCombinedView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_content(
  req: HttpRequest,
  data: Query<ListPersonContent>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListPersonContentResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_id_or_name = parse_person_id_or_name_from_request(&req)?;
  let person_details_id = match person_id_or_name {
    PersonIdOrName::Id(id) => id,
    PersonIdOrName::Name(username) => {
      resolve_ap_identifier::<ApubPerson, Person>(username, &context, &local_user_view, true)
        .await?
        .id
    }
  };

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
    no_limit: None,
  }
  .list(
    &mut context.pool(),
    local_user_view.as_ref(),
    local_instance_id,
  )
  .await?;

  let next_page = content.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = content.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonContentResponse {
    content,
    next_page,
    prev_page,
  }))
}
