use crate::community::do_follow_community;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostActions},
  },
  traits::Crud,
};
use lemmy_db_schema_file::enums::PostNotificationsMode;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::UpdatePostNotifications;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn update_post_notifications(
  data: Json<UpdatePostNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  PostActions::update_notification_state(
    data.post_id,
    local_user_view.person.id,
    data.mode,
    &mut context.pool(),
  )
  .await?;
  let post = Post::read(&mut context.pool(), data.post_id).await?;

  // To get notifications for a remote community, the user needs to follow it over federation.
  // Do this automatically here to avoid confusion.
  if data.mode == PostNotificationsMode::AllComments {
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    if !community.local {
      do_follow_community(community, &local_user_view.person, true, &context).await?;
    }
  }
  Ok(Json(SuccessResponse::default()))
}
