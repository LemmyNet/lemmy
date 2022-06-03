use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, MarkCommentAsRead},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_schema::source::comment::Comment;
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for MarkCommentAsRead {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &MarkCommentAsRead = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    // Verify that only the recipient can mark as read
    if local_user_view.person.id != orig_comment.get_recipient_id() {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    // Do the mark as read
    let read = data.read;
    blocking(context.pool(), move |conn| {
      Comment::update_read(conn, comment_id, read)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Refetch it
    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

    let res = CommentResponse {
      comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    };

    Ok(res)
  }
}
