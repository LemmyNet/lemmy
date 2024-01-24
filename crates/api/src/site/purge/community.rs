use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgeCommunity,
  utils::{is_admin, purge_image_posts_for_community},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{AdminPurgeCommunity, AdminPurgeCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn purge_community(
  data: Json<PurgeCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the community to get its images
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  if let Some(banner) = &community.banner {
    purge_image_from_pictrs(banner, &context).await.ok();
  }

  if let Some(icon) = &community.icon {
    purge_image_from_pictrs(icon, &context).await.ok();
  }

  purge_image_posts_for_community(data.community_id, &context).await?;

  Community::delete(&mut context.pool(), data.community_id).await?;

  // Mod tables
  let form = AdminPurgeCommunityForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
  };
  AdminPurgeCommunity::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity(
      local_user_view.person.clone(),
      community,
      data.reason.clone(),
      true,
    ),
    &context,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
