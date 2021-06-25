use crate::{
  activities_new::{post::send_websocket_message, verify_mod_action},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::RemoveType;
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
pub struct RemovePost {
  actor: Url,
  to: PublicUrl,
  object: Url,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: RemoveType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<RemovePost> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_mod_action(self.inner.actor.clone(), self.inner.cc.clone(), context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<RemovePost> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // TODO: check that actor is instance mod if community is local (same for DeleteComment)
    let post = get_or_fetch_and_insert_post(&self.inner.object, context, request_counter).await?;

    let removed_post = blocking(context.pool(), move |conn| {
      Post::update_removed(conn, post.id, true)
    })
    .await??;

    send_websocket_message(removed_post.id, UserOperationCrud::EditPost, context).await
  }
}
