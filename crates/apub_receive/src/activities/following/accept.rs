use crate::activities::{following::follow::FollowCommunity, LemmyActivity};
use activitystreams::activity::kind::AcceptType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::person::get_or_fetch_and_upsert_person};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::community::{Community, CommunityFollower};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  to: Url,
  object: LemmyActivity<FollowCommunity>,
  #[serde(rename = "type")]
  kind: AcceptType,
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<AcceptFollowCommunity> {
  type Actor = Community;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.verify(context).await
  }

  async fn receive(
    &self,
    actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person = get_or_fetch_and_upsert_person(&self.inner.to, context, request_counter).await?;
    // This will throw an error if no follow was requested
    blocking(&context.pool(), move |conn| {
      CommunityFollower::follow_accepted(conn, actor.id, person.id)
    })
    .await??;

    Ok(())
  }
}
