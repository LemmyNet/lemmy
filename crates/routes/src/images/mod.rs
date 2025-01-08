use actix_web::web::*;
use lemmy_api_common::{context::LemmyContext, image::DeleteImageParams, SuccessResponse};
use lemmy_db_schema::source::images::LocalImage;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;
use utils::PICTRS_CLIENT;

pub mod download;
pub mod upload;
mod utils;

pub async fn delete_community_icon(
  data: Json<DeleteImageParams>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  todo!()
}

// TODO: get rid of delete tokens and allow deletion by admin or uploader
pub async fn delete_image(
  data: Json<DeleteImageParams>,
  context: Data<LemmyContext>,
  // require login
  _local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!(
    "{}image/delete/{}/{}",
    pictrs_config.url, &data.token, &data.filename
  );

  PICTRS_CLIENT.delete(url).send().await?.error_for_status()?;

  LocalImage::delete_by_alias(&mut context.pool(), &data.filename).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn pictrs_health(context: Data<LemmyContext>) -> LemmyResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!("{}healthz", pictrs_config.url);

  PICTRS_CLIENT.get(url).send().await?.error_for_status()?;

  Ok(Json(SuccessResponse::default()))
}
