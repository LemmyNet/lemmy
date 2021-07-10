use lemmy_api_common::{blocking, post::PostResponse};
use lemmy_db_schema::PostId;
use lemmy_db_views::post_view::PostView;
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendPost, LemmyContext};

pub mod create;
pub mod delete;
pub mod remove;
pub mod undo_delete;
pub mod undo_remove;
pub mod update;

pub(crate) async fn send_websocket_message<
  OP: ToString + Send + lemmy_websocket::OperationType + 'static,
>(
  post_id: PostId,
  op: OP,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op,
    post: res,
    websocket_id: None,
  });

  Ok(())
}
