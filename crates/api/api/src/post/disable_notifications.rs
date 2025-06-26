use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::post::PostActions;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::DisablePostNotifications;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn disable_post_notifications(
  data: Json<DisablePostNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  PostActions::update_notifications_disabled(
    data.post_id,
    local_user_view.person.id,
    data.disable,
    &mut context.pool(),
  )
  .await?;
  Ok(Json(SuccessResponse::default()))
}
