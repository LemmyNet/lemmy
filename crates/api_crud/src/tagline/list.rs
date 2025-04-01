use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{ListTaglines, ListTaglinesResponse},
};
use lemmy_db_schema::source::tagline::Tagline;
use lemmy_utils::error::LemmyError;

pub async fn list_taglines(
  data: Query<ListTaglines>,
  context: Data<LemmyContext>,
) -> Result<Json<ListTaglinesResponse>, LemmyError> {
  let taglines = Tagline::list(&mut context.pool(), data.page, data.limit).await?;

  Ok(Json(ListTaglinesResponse { taglines }))
}
