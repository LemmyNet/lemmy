use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{GetReplies, GetRepliesResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_views_actor::comment_reply_view::CommentReplyQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetReplies {
  type Response = GetRepliesResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &GetReplies = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let sort = data.sort;
    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let person_id = Some(local_user_view.person.id);
    let show_bot_accounts = Some(local_user_view.local_user.show_bot_accounts);

    let replies = blocking(context.pool(), move |conn| {
      CommentReplyQuery::builder()
        .conn(conn)
        .recipient_id(person_id)
        .my_person_id(person_id)
        .sort(sort)
        .unread_only(unread_only)
        .show_bot_accounts(show_bot_accounts)
        .page(page)
        .limit(limit)
        .build()
        .list()
    })
    .await??;

    Ok(GetRepliesResponse { replies })
  }
}
