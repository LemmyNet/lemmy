use crate::{activities_new::post::send_websocket_message, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::DeleteType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::objects::get_or_fetch_and_insert_post};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::source::post::Post_;
use lemmy_db_schema::source::post::Post;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePost {
  actor: Url,
  to: PublicUrl,
  object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: DeleteType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<DeletePost> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.actor, &self.inner.object)?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<DeletePost> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let post = get_or_fetch_and_insert_post(&self.inner.object, context, request_counter).await?;

    let deleted_post = blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, post.id, true)
    })
    .await??;

    send_websocket_message(deleted_post.id, UserOperationCrud::EditPost, context).await
  }
}
