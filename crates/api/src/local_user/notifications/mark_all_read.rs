use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{GetRepliesResponse, MarkAllAsRead},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_schema::source::{
  comment::Comment,
  person_mention::PersonMention,
  private_message::PrivateMessage,
};
use lemmy_db_views::comment_view::CommentQueryBuilder;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for MarkAllAsRead {
  type Response = GetRepliesResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &MarkAllAsRead = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let replies = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .my_person_id(person_id)
        .recipient_id(person_id)
        .unread_only(true)
        .page(1)
        .limit(std::i64::MAX)
        .list()
    })
    .await??;

    // TODO: this should probably be a bulk operation
    // Not easy to do as a bulk operation,
    // because recipient_id isn't in the comment table
    for comment_view in &replies {
      let reply_id = comment_view.comment.id;
      let mark_as_read = move |conn: &'_ _| Comment::update_read(conn, reply_id, true);
      blocking(context.pool(), mark_as_read)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;
    }

    // Mark all user mentions as read
    let update_person_mentions =
      move |conn: &'_ _| PersonMention::mark_all_as_read(conn, person_id);
    blocking(context.pool(), update_person_mentions)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Mark all private_messages as read
    let update_pm = move |conn: &'_ _| PrivateMessage::mark_all_as_read(conn, person_id);
    blocking(context.pool(), update_pm)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_private_message"))?;

    Ok(GetRepliesResponse { replies: vec![] })
  }
}
