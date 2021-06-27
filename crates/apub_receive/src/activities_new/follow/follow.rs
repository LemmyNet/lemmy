use crate::inbox::new_inbox_routing::Activity;
use activitystreams::{
  activity::{kind::FollowType, Follow},
  base::{AnyBase, ExtendsExt},
};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  CommunityType,
};
use lemmy_apub_lib::{verify_domains_match, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::community::{CommunityFollower, CommunityFollowerForm};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowCommunity {
  actor: Url,
  to: Url,
  pub(in crate::activities_new::follow) object: Url,
  #[serde(rename = "type")]
  kind: FollowType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<FollowCommunity> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.to, &self.inner.object)?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<FollowCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.object, context, request_counter).await?;
    let person =
      get_or_fetch_and_upsert_person(&self.inner.actor, context, request_counter).await?;
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: person.id,
      pending: false,
    };

    // This will fail if they're already a follower, but ignore the error.
    blocking(&context.pool(), move |conn| {
      CommunityFollower::follow(&conn, &community_follower_form).ok()
    })
    .await?;

    // TODO: avoid the conversion and pass our own follow struct directly
    let anybase = AnyBase::from_arbitrary_json(serde_json::to_string(self)?)?;
    let anybase = Follow::from_any_base(anybase)?.context(location_info!())?;
    community.send_accept_follow(anybase, context).await
  }
}
