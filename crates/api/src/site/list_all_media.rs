use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse},
  utils::is_admin,
};
use lemmy_db_schema::source::images::LocalImage;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_all_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ListMediaResponse>, LemmyError> {
  // Only let admins view all media
  is_admin(&local_user_view)?;

  let page = data.page;
  let limit = data.limit;
  let images = LocalImage::get_all(&mut context.pool(), page, limit).await?;
  Ok(Json(ListMediaResponse { images }))
}
