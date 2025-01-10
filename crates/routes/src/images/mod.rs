use actix_web::web::*;
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_utils::error::LemmyResult;
use reqwest_middleware::ClientWithMiddleware;

pub mod delete;
pub mod download;
pub mod upload;
mod utils;

pub async fn pictrs_health(
  client: Data<ClientWithMiddleware>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!("{}healthz", pictrs_config.url);

  client.get(url).send().await?.error_for_status()?;

  Ok(Json(SuccessResponse::default()))
}
