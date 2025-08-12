use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{newtypes::PostId, source::post::PostActions};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::UpdatePostNotifications;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn update_post_notifications(
  post_id: Path<PostId>,
  data: Json<UpdatePostNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  PostActions::update_notification_state(
    post_id.into_inner(),
    local_user_view.person.id,
    data.mode,
    &mut context.pool(),
  )
  .await?;
  Ok(Json(SuccessResponse::default()))
}
