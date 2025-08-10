use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{newtypes::CommunityId, source::community::CommunityActions};
use lemmy_db_views_community::api::UpdateCommunityNotifications;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn update_community_notifications(
  community_id: Path<CommunityId>,
  data: Json<UpdateCommunityNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community_id = community_id.into_inner();
  CommunityActions::update_notification_state(
    community_id,
    local_user_view.person.id,
    data.mode,
    &mut context.pool(),
  )
  .await?;
  Ok(Json(SuccessResponse::default()))
}
