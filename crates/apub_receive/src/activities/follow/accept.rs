use crate::{activities::follow::follow::FollowCommunity, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::AcceptType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
};
use lemmy_apub_lib::{verify_domains_match, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::community::CommunityFollower;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  actor: Url,
  to: Url,
  object: Activity<FollowCommunity>,
  #[serde(rename = "type")]
  kind: AcceptType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<AcceptFollowCommunity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    self.inner.object.verify(context).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<AcceptFollowCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.actor, context, request_counter).await?;
    let person = get_or_fetch_and_upsert_person(&self.inner.to, context, request_counter).await?;
    // This will throw an error if no follow was requested
    blocking(&context.pool(), move |conn| {
      CommunityFollower::follow_accepted(conn, community.id, person.id)
    })
    .await??;

    Ok(())
  }
}
