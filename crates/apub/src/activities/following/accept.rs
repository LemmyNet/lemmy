use crate::{
  activities::{
    following::follow::FollowCommunity,
    generate_activity_id,
    verify_activity,
    verify_community,
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  ActorType,
};
use activitystreams::{
  activity::kind::AcceptType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_urls_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::{ApubObject, Followable};
use lemmy_db_schema::source::{
  community::{Community, CommunityFollower},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  actor: Url,
  to: Url,
  object: FollowCommunity,
  #[serde(rename = "type")]
  kind: AcceptType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl AcceptFollowCommunity {
  pub async fn send(follow: FollowCommunity, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = follow.object.clone();
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &community_id.into())
    })
    .await??;
    let person_id = follow.actor().clone();
    let person = blocking(context.pool(), move |conn| {
      Person::read_from_apub_id(conn, &person_id.into())
    })
    .await??;

    let accept = AcceptFollowCommunity {
      actor: community.actor_id(),
      to: person.actor_id(),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(AcceptType::Accept)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let inbox = vec![person.inbox_url.into()];
    send_activity_new(context, &accept, &accept.id, &community, inbox, true).await
  }
}
/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ActivityHandler for AcceptFollowCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_urls_match(&self.to, self.object.actor())?;
    verify_urls_match(&self.actor, &self.object.to)?;
    verify_community(&self.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = get_or_fetch_and_upsert_community(&self.actor, context, request_counter).await?;
    let to = get_or_fetch_and_upsert_person(&self.to, context, request_counter).await?;
    // This will throw an error if no follow was requested
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow_accepted(conn, actor.id, to.id)
    })
    .await??;

    Ok(())
  }
}
