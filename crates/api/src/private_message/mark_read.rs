use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{MarkPrivateMessageAsRead, PrivateMessageResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for MarkPrivateMessageAsRead {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &MarkPrivateMessageAsRead = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message =
      PrivateMessage::read(&mut context.pool(), private_message_id).await?;
    if local_user_view.person.id != orig_private_message.recipient_id {
      return Err(LemmyErrorType::CouldntUpdatePrivateMessage)?;
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let read = data.read;
    PrivateMessage::update(
      &mut context.pool(),
      private_message_id,
      &PrivateMessageUpdateForm::builder().read(Some(read)).build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

    let view = PrivateMessageView::read(&mut context.pool(), private_message_id).await?;
    Ok(PrivateMessageResponse {
      private_message_view: view,
    })
  }
}
