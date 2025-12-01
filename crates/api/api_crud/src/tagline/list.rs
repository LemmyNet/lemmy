use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::tagline::Tagline;
use lemmy_db_views_site::api::ListTaglines;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyError;

pub async fn list_taglines(
  Query(data): Query<ListTaglines>,
  context: Data<LemmyContext>,
) -> Result<Json<PagedResponse<Tagline>>, LemmyError> {
  let taglines = Tagline::list(&mut context.pool(), data.page_cursor, data.limit).await?;

  Ok(Json(taglines))
}
