use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeCommunity, PurgeItemResponse},
  utils::{is_top_admin, local_user_view_from_jwt, purge_image_posts_for_community},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{AdminPurgeCommunity, AdminPurgeCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for PurgeCommunity {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Only let the top admin purge an item
    is_top_admin(context.pool(), local_user_view.person.id).await?;

    let community_id = data.community_id;

    // Read the community to get its images
    let community = Community::read(context.pool(), community_id).await?;

    if let Some(banner) = community.banner {
      purge_image_from_pictrs(context.client(), context.settings(), &banner)
        .await
        .ok();
    }

    if let Some(icon) = community.icon {
      purge_image_from_pictrs(context.client(), context.settings(), &icon)
        .await
        .ok();
    }

    purge_image_posts_for_community(
      community_id,
      context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;

    Community::delete(context.pool(), community_id).await?;

    // Mod tables
    let reason = data.reason.clone();
    let form = AdminPurgeCommunityForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    AdminPurgeCommunity::create(context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
