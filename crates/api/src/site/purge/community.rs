use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeCommunity, PurgeItemResponse},
  utils::{is_admin, local_user_view_from_jwt, purge_image_posts_for_community, sanitize_html_opt},
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

    // Only let admin purge an item
    is_admin(&local_user_view)?;

    let community_id = data.community_id;

    // Read the community to get its images
    let community = Community::read(&mut context.pool(), community_id).await?;

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
      &mut context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;

    Community::delete(&mut context.pool(), community_id).await?;

    // Mod tables
    let reason = sanitize_html_opt(&data.reason);
    let form = AdminPurgeCommunityForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    AdminPurgeCommunity::create(&mut context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
