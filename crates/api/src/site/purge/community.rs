use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeCommunity, PurgeItemResponse},
  utils::{is_admin, purge_image_posts_for_community, sanitize_html_opt},
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
) -> Result<Json<PurgeItemResponse>, LemmyError> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let community_id = data.community_id;

  // Read the community to get its images
  let community = Community::read(&mut context.pool(), community_id).await?;

  if let Some(banner) = community.banner {
    purge_image_from_pictrs(&banner, &context).await.ok();
  }

  if let Some(icon) = community.icon {
    purge_image_from_pictrs(&icon, &context).await.ok();
  }

  purge_image_posts_for_community(community_id, &context).await?;

  Community::delete(&mut context.pool(), community_id).await?;

  // Mod tables
  let reason = sanitize_html_opt(&data.reason);
  let form = AdminPurgeCommunityForm {
    admin_person_id: local_user_view.person.id,
    reason,
  };

  AdminPurgeCommunity::create(&mut context.pool(), &form).await?;

  Ok(Json(PurgeItemResponse { success: true }))
}
