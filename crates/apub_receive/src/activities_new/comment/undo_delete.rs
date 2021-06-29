use crate::{
  activities_new::comment::{delete::DeleteComment, send_websocket_message},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::objects::get_or_fetch_and_insert_comment};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::source::comment::Comment_;
use lemmy_db_schema::source::comment::Comment;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeleteComment {
  actor: Url,
  to: PublicUrl,
  object: Activity<DeleteComment>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UndoDeleteComment> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.actor, &self.inner.object.inner.actor)?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    self.inner.object.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UndoDeleteComment> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment =
      get_or_fetch_and_insert_comment(&self.inner.object.inner.object, context, request_counter)
        .await?;

    let deleted_comment = blocking(context.pool(), move |conn| {
      Comment::update_deleted(conn, comment.id, false)
    })
    .await??;

    send_websocket_message(
      deleted_comment.id,
      vec![],
      UserOperationCrud::EditComment,
      context,
    )
    .await
  }
}
