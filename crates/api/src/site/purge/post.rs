use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeItemResponse, PurgePost},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for PurgePost {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Only let admin purge an item
    is_admin(&local_user_view)?;

    let post_id = data.post_id;

    // Read the post to get the community_id
    let post = Post::read(&mut context.pool(), post_id).await?;

    // Purge image
    if let Some(url) = post.url {
      purge_image_from_pictrs(context.client(), context.settings(), &url)
        .await
        .ok();
    }
    // Purge thumbnail
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(context.client(), context.settings(), &thumbnail_url)
        .await
        .ok();
    }

    let community_id = post.community_id;

    Post::delete(&mut context.pool(), post_id).await?;

    // Mod tables
    let reason = data.reason.clone();
    let form = AdminPurgePostForm {
      admin_person_id: local_user_view.person.id,
      reason,
      community_id,
    };

    AdminPurgePost::create(&mut context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
