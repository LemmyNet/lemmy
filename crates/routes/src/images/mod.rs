use actix_web::web::*;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub mod delete;
pub mod download;
pub mod upload;
mod utils;

pub async fn pictrs_health(context: Data<LemmyContext>) -> LemmyResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!("{}healthz", pictrs_config.url);

  context
    .pictrs_client()
    .get(url)
    .send()
    .await?
    .error_for_status()?;

  Ok(Json(SuccessResponse::default()))
}
