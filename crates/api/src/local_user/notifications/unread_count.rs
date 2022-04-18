use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{GetUnreadCount, GetUnreadCountResponse},
};
use lemmy_db_views::{comment_view::CommentView, private_message_view::PrivateMessageView};
use lemmy_db_views_actor::person_mention_view::PersonMentionView;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetUnreadCount {
  type Response = GetUnreadCountResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;

    let replies = blocking(context.pool(), move |conn| {
      CommentView::get_unread_replies(conn, person_id)
    })
    .await??;

    let mentions = blocking(context.pool(), move |conn| {
      PersonMentionView::get_unread_mentions(conn, person_id)
    })
    .await??;

    let private_messages = blocking(context.pool(), move |conn| {
      PrivateMessageView::get_unread_messages(conn, person_id)
    })
    .await??;

    let res = Self::Response {
      replies,
      mentions,
      private_messages,
    };

    Ok(res)
  }
}
