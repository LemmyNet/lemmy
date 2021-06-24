use activitystreams::{
  activity::kind::{AcceptType, FollowType},
};
use lemmy_api_common::blocking;
use lemmy_apub::fetcher::{
  community::get_or_fetch_and_upsert_community,
  person::get_or_fetch_and_upsert_person,
};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::community::CommunityFollower;
use lemmy_utils::{LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;
use lemmy_apub_lib::{ReceiveActivity, verify_domains_match};
use crate::inbox::new_inbox_routing::Activity;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
  actor: Url,
  to: Url,
  object: Url,
  #[serde(rename = "type")]
  kind: FollowType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<Follow> {
  async fn receive(
    &self,
    _context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    println!("receive follow");
    todo!()
  }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
  actor: Url,
  to: Url,
  object: Activity<Follow>,
  #[serde(rename = "type")]
  kind: AcceptType,
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<Accept> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // TODO: move check for id.domain == actor.domain to library and do it automatically
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    let follow = &self.inner.object;
    verify_domains_match(&follow.inner.actor, &follow.id_unchecked())?;

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
