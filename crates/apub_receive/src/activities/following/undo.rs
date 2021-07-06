use crate::activities::{following::follow::FollowCommunity, LemmyActivity};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::community::get_or_fetch_and_upsert_community};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::{
  community::{CommunityFollower, CommunityFollowerForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollowCommunity {
  to: Url,
  object: LemmyActivity<FollowCommunity>,
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<UndoFollowCommunity> {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.to, &self.inner.object.inner.object)?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.verify(context).await
  }

  async fn receive(
    &self,
    actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.to, context, request_counter).await?;

    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: false,
    };

    // This will fail if they aren't a follower, but ignore the error.
    blocking(context.pool(), move |conn| {
      CommunityFollower::unfollow(conn, &community_follower_form).ok()
    })
    .await?;
    Ok(())
  }
}
