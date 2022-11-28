use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeCommunity, PurgeItemResponse},
  utils::{get_local_user_view_from_jwt, is_admin, purge_image_posts_for_community},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{AdminPurgeCommunity, AdminPurgeCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for PurgeCommunity {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

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
