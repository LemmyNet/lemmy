use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{EditPrivateMessage, PrivateMessageResponse},
  utils::{local_site_to_slur_regex, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::{
  error::LemmyError,
  utils::{slurs::remove_slurs, validation::is_valid_body_field},
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPrivateMessage {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &EditPrivateMessage = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = PrivateMessage::read(context.pool(), private_message_id).await?;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(LemmyError::from_message("no_private_message_edit_allowed"));
    }

    // Doing the update
    let content_slurs_removed = remove_slurs(&data.content, &local_site_to_slur_regex(&local_site));
    is_valid_body_field(&Some(content_slurs_removed.clone()))?;

    let private_message_id = data.private_message_id;
    PrivateMessage::update(
      context.pool(),
      private_message_id,
      &PrivateMessageUpdateForm::builder()
        .content(Some(content_slurs_removed))
        .updated(Some(Some(naive_now())))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_private_message"))?;

    let view = PrivateMessageView::read(context.pool(), private_message_id).await?;

    Ok(PrivateMessageResponse {
      private_message_view: view,
    })
  }
}
