use crate::{
  activities_new::community::{remove::RemoveCommunity, send_websocket_message},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::RemoveType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::community::get_or_fetch_and_upsert_community};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::source::community::Community_;
use lemmy_db_schema::source::community::Community;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemoveCommunity {
  actor: Url,
  to: PublicUrl,
  object: Activity<RemoveCommunity>,
  cc: Url,
  #[serde(rename = "type")]
  kind: RemoveType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UndoRemoveCommunity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_domains_match(&self.inner.actor, &self.inner.object.inner.object)?;
    verify_domains_match(&self.inner.actor, &self.inner.cc)?;
    self.inner.object.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UndoRemoveCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community_id = self.inner.object.inner.object.clone();
    let community =
      get_or_fetch_and_upsert_community(&community_id, context, request_counter).await?;

    let restored_community = blocking(context.pool(), move |conn| {
      Community::update_removed(conn, community.id, false)
    })
    .await??;

    send_websocket_message(
      restored_community.id,
      UserOperationCrud::EditCommunity,
      context,
    )
    .await
  }
}
