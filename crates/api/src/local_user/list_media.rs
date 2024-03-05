use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse},
};
use lemmy_db_schema::source::images::LocalImage;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ListMediaResponse>, LemmyError> {
  let page = data.page;
  let limit = data.limit;
  let images = LocalImage::get_all_paged_by_local_user_id(
    &mut context.pool(),
    local_user_view.local_user.id,
    page,
    limit,
  )
  .await?;
  Ok(Json(ListMediaResponse { images }))
}
