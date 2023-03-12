use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{GetPrivateMessages, PrivateMessagesResponse},
  utils::get_local_user_view_from_jwt,
};
use lemmy_db_views::private_message_view::PrivateMessageQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPrivateMessages {
  type Response = PrivateMessagesResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessagesResponse, LemmyError> {
    let data: &GetPrivateMessages = self;
    let local_user_view =
      get_local_user_view_from_jwt(data.auth.as_ref(), context.pool(), context.secret()).await?;
    let person_id = local_user_view.person.id;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let mut messages = PrivateMessageQuery::builder()
      .pool(context.pool())
      .recipient_id(person_id)
      .page(page)
      .limit(limit)
      .unread_only(unread_only)
      .build()
      .list()
      .await?;

    // Messages sent by ourselves should be marked as read. The `read` column in database is only
    // for the recipient, and shouldnt be exposed to sender.
    messages.iter_mut().for_each(|pmv| {
      if pmv.creator.id == person_id {
        pmv.private_message.read = true
      }
    });

    Ok(PrivateMessagesResponse {
      private_messages: messages,
    })
  }
}
