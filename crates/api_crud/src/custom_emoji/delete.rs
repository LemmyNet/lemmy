use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{DeleteCustomEmoji, DeleteCustomEmojiResponse},
  utils::{has_site_permission, local_user_view_from_jwt},
};
use lemmy_db_schema::{source::custom_emoji::CustomEmoji, SitePermission};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteCustomEmoji {
  type Response = DeleteCustomEmojiResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<DeleteCustomEmojiResponse, LemmyError> {
    let data: &DeleteCustomEmoji = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    has_site_permission(&local_user_view, SitePermission::ModifyCustomEmoji)?;
    CustomEmoji::delete(context.pool(), data.id).await?;
    Ok(DeleteCustomEmojiResponse {
      id: data.id,
      success: true,
    })
  }
}
