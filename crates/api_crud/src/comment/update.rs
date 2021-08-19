use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  comment::*,
  get_local_user_view_from_jwt,
  send_local_notifs,
};
use lemmy_apub::activities::{
  comment::create_or_update::CreateOrUpdateComment,
  CreateOrUpdateType,
};
use lemmy_db_queries::source::comment::Comment_;
use lemmy_db_schema::source::comment::*;
use lemmy_db_views::comment_view::CommentView;
use lemmy_utils::{
  utils::{remove_slurs, scrape_text_for_mentions},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &EditComment = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    // TODO is this necessary? It should really only need to check on create
    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can edit
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(ApiError::err("no_comment_edit_allowed").into());
    }

    // Do the update
    let content_slurs_removed = remove_slurs(&data.content.to_owned());
    let comment_id = data.comment_id;
    let updated_comment = blocking(context.pool(), move |conn| {
      Comment::update_content(conn, comment_id, &content_slurs_removed)
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_update_comment"))?;

    // Send the apub update
    CreateOrUpdateComment::send(
      &updated_comment,
      &local_user_view.person,
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    // Do the mentions / recipients
    let updated_comment_content = updated_comment.content.to_owned();
    let mentions = scrape_text_for_mentions(&updated_comment_content);
    let recipient_ids = send_local_notifs(
      mentions,
      updated_comment,
      local_user_view.person.clone(),
      orig_comment.post,
      context.pool(),
      false,
    )
    .await?;

    send_comment_ws_message(
      data.comment_id,
      UserOperationCrud::EditComment,
      websocket_id,
      data.form_id.to_owned(),
      None,
      recipient_ids,
      context,
    )
    .await
  }
}
