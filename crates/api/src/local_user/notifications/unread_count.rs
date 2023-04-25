use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetUnreadCount, GetUnreadCountResponse},
  sensitive::Sensitive,
  utils::local_user_view_from_jwt_new,
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_db_views_actor::structs::{CommentReplyView, PersonMentionView};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for GetUnreadCount {
  type Response = GetUnreadCountResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let _data = self;
    let local_user_view = local_user_view_from_jwt_new(auth, context).await?;

    let person_id = local_user_view.person.id;

    let replies = CommentReplyView::get_unread_replies(context.pool(), person_id).await?;

    let mentions = PersonMentionView::get_unread_mentions(context.pool(), person_id).await?;

    let private_messages =
      PrivateMessageView::get_unread_messages(context.pool(), person_id).await?;

    let res = Self::Response {
      replies,
      mentions,
      private_messages,
    };

    Ok(res)
  }
}
