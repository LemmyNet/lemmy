use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse}, utils::is_admin,
};
use lemmy_db_views::structs::{LocalImageView, LocalUserView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListMediaResponse>> {
  let page = data.page;
  let limit = data.limit;

  if local_user_view.person.id != data.person_id {
    is_admin(&local_user_view)?;
  }

  let images = LocalImageView::get_all_paged_by_local_user_id(
    &mut context.pool(),
    data.person_id,
    local_user_view.local_user.id,
    page,
    limit,
  )
  .await?;
  Ok(Json(ListMediaResponse { images }))
}
