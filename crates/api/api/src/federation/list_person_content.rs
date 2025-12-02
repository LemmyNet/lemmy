use crate::federation::fetcher::resolve_person_identifier;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_content_combined::{
  ListPersonContent,
  impls::PersonContentCombinedQuery,
};
use lemmy_db_views_post_comment_combined::PostCommentCombinedView;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_content(
  Query(data): Query<ListPersonContent>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<PostCommentCombinedView>>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_details_id =
    resolve_person_identifier(data.person_id, &data.username, &context, &local_user_view).await?;

  let res = PersonContentCombinedQuery {
    creator_id: person_details_id,
    type_: data.type_,
    page_cursor: data.page_cursor,
    limit: data.limit,
    no_limit: None,
  }
  .list(
    &mut context.pool(),
    local_user_view.as_ref(),
    local_instance_id,
  )
  .await?;

  Ok(Json(res))
}
