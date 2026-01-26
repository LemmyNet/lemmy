use crate::community::do_follow_community;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::community::{Community, CommunityActions};
use lemmy_db_schema_file::enums::CommunityNotificationsMode;
use lemmy_db_views_community::api::EditCommunityNotifications;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn edit_community_notifications(
  Json(data): Json<EditCommunityNotifications>,
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

  // To get notifications for a remote community, the user needs to follow it over federation.
  // Do this automatically here to avoid confusion.
  if data.mode == CommunityNotificationsMode::AllPostsAndComments
    || data.mode == CommunityNotificationsMode::AllPosts
  {
    let community = Community::read(&mut context.pool(), data.community_id).await?;
    if !community.local {
      do_follow_community(community, &local_user_view.person, true, &context).await?;
    }
  }

  Ok(Json(SuccessResponse::default()))
}
