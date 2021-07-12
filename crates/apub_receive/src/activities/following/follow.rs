use crate::activities::{verify_activity, verify_person};
use activitystreams::{
  activity::{kind::FollowType, Follow},
  base::{AnyBase, ExtendsExt},
};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  CommunityType,
};
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::community::{CommunityFollower, CommunityFollowerForm};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowCommunity {
  pub(in crate::activities::following) to: Url,
  pub(in crate::activities::following) object: Url,
  #[serde(rename = "type")]
  kind: FollowType,
  #[serde(flatten)]
  pub(in crate::activities::following) common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for FollowCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_urls_match(&self.to, &self.object)?;
    verify_person(&self.common.actor, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor =
      get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
    let community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await?;
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: false,
    };

    // This will fail if they're already a follower, but ignore the error.
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    // TODO: avoid the conversion and pass our own follow struct directly
    let anybase = AnyBase::from_arbitrary_json(serde_json::to_string(self)?)?;
    let anybase = Follow::from_any_base(anybase)?.context(location_info!())?;
    community.send_accept_follow(anybase, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
