use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::community::CommunityActions;
use lemmy_db_views_community::api::UpdateCommunityNotifications;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn update_community_notifications(
  data: Json<UpdateCommunityNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  CommunityActions::update_notification_state(
    data.community_id,
    local_user_view.person.id,
    data.mode,
    &mut context.pool(),
  )
  .await?;
  Ok(Json(SuccessResponse::default()))
}
