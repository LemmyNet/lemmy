use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_liked_combined::{
  ListPersonLiked,
  PersonLikedCombinedView,
  impls::PersonLikedCombinedQuery,
};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_liked(
  Query(data): Query<ListPersonLiked>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<PersonLikedCombinedView>>> {
  let liked = PersonLikedCombinedQuery {
    type_: data.type_,
    like_type: data.like_type,
    page_cursor: data.page_cursor,
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(liked))
}
