use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{ApproveComment, CommentResponse},
  context::LemmyContext,
  utils::{get_local_user_view_from_jwt, is_admin},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::source::review_comment::ReviewComment;
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for ApproveComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &ApproveComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    is_admin(&local_user_view)?;

    let person_id = local_user_view.person.id;
    let review = ReviewComment::approve(context.pool(), data.review_id, person_id)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;

    let comment_view = CommentView::read(context.pool(), review.comment_id, None).await?;

    let res = CommentResponse {
      comment_view,
      recipient_ids: vec![],
      form_id: None,
    };

    context
      .chat_server()
      .send_all_message(UserOperationCrud::CreateComment, &res, websocket_id)
      .await?;

    Ok(res)
  }
}
