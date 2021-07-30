use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{GetPrivateMessages, PrivateMessagesResponse},
};
use lemmy_db_queries::DeleteableOrRemoveable;
use lemmy_db_views::private_message_view::PrivateMessageQueryBuilder;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPrivateMessages {
  type Response = PrivateMessagesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessagesResponse, LemmyError> {
    let data: &GetPrivateMessages = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;
    let person_id = local_user_view.person.id;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let mut messages = blocking(context.pool(), move |conn| {
      PrivateMessageQueryBuilder::create(conn, person_id)
        .page(page)
        .limit(limit)
        .unread_only(unread_only)
        .list()
    })
    .await??;

    // Blank out deleted or removed info
    for pmv in messages
      .iter_mut()
      .filter(|pmv| pmv.private_message.deleted)
    {
      pmv.private_message = pmv
        .to_owned()
        .private_message
        .blank_out_deleted_or_removed_info();
    }

    Ok(PrivateMessagesResponse {
      private_messages: messages,
    })
  }
}
