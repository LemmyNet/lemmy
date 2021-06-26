use crate::{
  activities_new::community::send_websocket_message,
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::RemoveType;
use lemmy_api_common::blocking;
use lemmy_apub::check_is_apub_id_valid;
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::{source::community::Community_, ApubObject};
use lemmy_db_schema::source::community::Community;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveCommunity {
  actor: Url,
  to: PublicUrl,
  pub(in crate::activities_new::community) object: Url,
  cc: Url,
  #[serde(rename = "type")]
  kind: RemoveType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<RemoveCommunity> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_domains_match(&self.inner.actor, &self.inner.object)?;
    verify_domains_match(&self.inner.actor, &self.inner.cc)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<RemoveCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let object = self.inner.object.clone();
    // only search in local database, there is no reason to fetch something thats deleted
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &object.into())
    })
    .await??;
    let removed_community = blocking(context.pool(), move |conn| {
      Community::update_removed(conn, community.id, true)
    })
    .await??;

    send_websocket_message(
      removed_community.id,
      UserOperationCrud::RemoveCommunity,
      context,
    )
    .await
  }
}
